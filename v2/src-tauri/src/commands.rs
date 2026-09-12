//! commands.rs — Commandes Tauri exposées au frontend, orchestrant
//! settings + textgrid. C'est ici que vivent les anciens `process_textgrid`
//! (chevauchement_view.py) et `_execute_transformations` /
//! `_execute_pivot_extraction` (modification_TextGrid_view.py), débarrassés
//! de tout code Flet.

use std::path::Path;
use std::sync::Mutex;
use tauri::State;

use crate::excel;
use crate::settings::{self, SettingsState};
use crate::textgrid::{concordance, editor, overlap, cooccurrences, tokenisation, trs_convert};

/// ⬅ chevauchement_view.py :: process_textgrid
#[tauri::command(rename_all = "snake_case")]
pub fn analyze_overlaps(state: State<'_, SettingsState>) -> Result<overlap::OverlapReport, String> {
    let data = settings::get_current_data(state);
    if data.textgrid_path_files.is_empty() {
        return Err("Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres.".into());
    }
    Ok(overlap::analyze_files(&data.textgrid_path_files))
}

/// ⬅ modification_TextGrid_view.py :: _execute_transformations
///
/// Remarque : le script Python original interrompt tout le traitement dès le
/// premier fichier manquant (`return` immédiat) alors que les vues
/// Chevauchements et Concordancier ignorent le fichier fautif et continuent
/// avec les suivants. On reproduit ici le comportement d'origine (arrêt),
/// à harmoniser plus tard si vous préférez le comportement "on continue".
#[tauri::command(rename_all = "snake_case")]
pub fn apply_text_modifications(
    state: State<'_, SettingsState>,
    find: String,
    replace: String,
    merge_silences: bool,
) -> Result<String, String> {
    let data = settings::get_current_data(state);
    if data.textgrid_path_files.is_empty() {
        return Ok(String::new());
    }

    let find = find.trim().to_string();
    let replace = replace.trim().to_string();
    let mut total_replaced = 0usize;
    let mut total_merged = 0usize;

    for path in &data.textgrid_path_files {
        if path.is_empty() || !Path::new(path).exists() {
            // Comportement fidèle à l'original : on arrête tout ici.
            break;
        }

        let mut tg = editor::parse_textgrid_editable(path)?;

        if !find.is_empty() {
            total_replaced += editor::replace_text(&mut tg.tiers, &find, &replace);
        }
        if merge_silences {
            total_merged += editor::merge_empty_intervals(&mut tg.tiers);
        }

        editor::write_textgrid(path, &tg)?;
    }

    let mut summary = String::from("Fichier mis à jour. ");
    if !find.is_empty() {
        summary.push_str(&format!("{} texte(s) modifié(s). ", total_replaced));
    }
    if merge_silences {
        summary.push_str(&format!("{} intervalle(s) fusionné(s).", total_merged));
    }
    Ok(summary)
}

/// ⬅ modification_TextGrid_view.py :: _execute_pivot_extraction
#[tauri::command(rename_all = "snake_case")]
pub fn apply_pivot_extraction(
    state: State<'_, SettingsState>,
    pivots_raw: String,
) -> Result<String, String> {
    let data = settings::get_current_data(state);
    if data.textgrid_path_files.is_empty() {
        return Ok(String::new());
    }

    let separator = data.separator.clone();
    let pivots_raw = pivots_raw.trim().to_string();
    if pivots_raw.is_empty() {
        return Err("Veuillez définir au moins un pivot.".into());
    }
    let pivots: Vec<String> = pivots_raw
        .split(separator.as_str())
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if pivots.is_empty() {
        return Err("Veuillez définir au moins un pivot.".into());
    }

    let mut total_pivots = 0usize;

    for path in &data.textgrid_path_files {
        let mut tg = editor::parse_textgrid_editable(path)?;

        total_pivots += editor::extract_pivots(&mut tg.tiers, &pivots);
        editor::merge_empty_intervals(&mut tg.tiers); // Fusion obligatoire, comme en Python.

        let p = Path::new(path);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let out_path = p.with_file_name(format!("{}_pivot.TextGrid", stem));
        editor::write_textgrid(out_path.to_str().unwrap_or_default(), &tg)?;
    }

    Ok(format!(
        "Création(s) réussie(s) ! {} pivot(s) isolé(s). Fichier(s) X_pivot.TextGrid",
        total_pivots
    ))
}

/// ⬅ modification_TextGrid_view.py :: _convert_trs_to_tg
///
/// Retourne un résultat par fichier (succès ou message d'erreur), dans le
/// même ordre que `trs_paths`, pour que le frontend affiche un statut par
/// fichier plutôt qu'un seul message global.
#[tauri::command(rename_all = "snake_case")]
pub fn convert_trs_files(trs_paths: Vec<String>) -> Vec<Result<String, String>> {
    trs_paths
        .iter()
        .map(|element| {
            if element.is_empty() || !Path::new(element).exists() {
                return Err(format!("Fichier introuvable : {}", element));
            }

            // Suppression de la double extension si présente (ex: fichier.trs.xml -> fichier).
            let stem = Path::new(element)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let base_name = stem.strip_suffix(".trs").unwrap_or(stem);

            let desktop = dirs::desktop_dir()
                .ok_or_else(|| "Impossible de localiser le Bureau.".to_string())?;
            let out_path = desktop.join(format!("{}.TextGrid", base_name));

            trs_convert::convert_trs_to_textgrid(element, out_path.to_str().unwrap_or_default())
                .map_err(|e| format!("Erreur lors de la conversion : {}", e))?;

            Ok(format!(
                "Conversion réussie ! Enregistré sur le bureau : {}.TextGrid",
                base_name
            ))
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════
// Tokenisation ⬅ token_view.py
// ═══════════════════════════════════════════════════════════════════════

/// ⬅ token_view.py :: _load_data
/// Le filtrage/tri/bascule de tiers reste côté frontend (comme en Python,
/// tout était recalculé en mémoire depuis `self.global_counts`/`tiers_data`
/// sans nouvel appel réseau) : cette commande ne fait que parser + agréger.
#[tauri::command(rename_all = "snake_case")]
pub fn load_tokenisation_data(state: State<'_, SettingsState>) -> Result<tokenisation::TokenisationData, String> {
    let data = settings::get_current_data(state);
    if data.textgrid_path_files.is_empty() {
        return Err("Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres.".into());
    }
    Ok(tokenisation::load(&data.textgrid_path_files))
}

// ═══════════════════════════════════════════════════════════════════════
// Cooccurrences ⬅ stats_view.py
// ═══════════════════════════════════════════════════════════════════════

/// État géré par Tauri : les tokens ordonnés restent en mémoire côté
/// natif entre le chargement et chaque relance d'analyse (équivalent de
/// `self.tiers_raw_tokens` conservé sur l'instance de la vue en Python) —
/// évite de re-parser tous les fichiers à chaque clic sur "Analyser".
#[derive(Default)]
pub struct StatsState(pub Mutex<Option<cooccurrences::StatsBaseData>>);

/// ⬅ stats_view.py :: _load_base_data
#[tauri::command(rename_all = "snake_case")]
pub fn load_stats_data(
    settings_state: State<'_, SettingsState>,
    stats_state: State<'_, StatsState>,
) -> Result<cooccurrences::StatsLoadSummary, String> {
    let data = settings::get_current_data(settings_state);
    if data.textgrid_path_files.is_empty() {
        return Err("Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres.".into());
    }
    let (base, summary) = cooccurrences::load(&data.textgrid_path_files);
    *stats_state.0.lock().unwrap() = Some(base);
    Ok(summary)
}

/// ⬅ stats_view.py :: _run_analysis
#[tauri::command(rename_all = "snake_case")]
pub fn run_stats_analysis(
    stats_state: State<'_, StatsState>,
    pivot: String,
    distances_raw: String,
    direction_suivant: bool,
    active_tiers: Vec<String>,
) -> Result<cooccurrences::StatsAnalysisResult, String> {
    let guard = stats_state.0.lock().unwrap();
    let base = guard
        .as_ref()
        .ok_or_else(|| "Chargez d'abord les données (bouton Rafraîchir).".to_string())?;

    let distances: Vec<i64> = distances_raw
        .replace(' ', "")
        .split(',')
        .filter_map(|d| d.parse::<i64>().ok())
        .collect();
    let distances = if distances.is_empty() { vec![1] } else { distances };

    cooccurrences::analyze(base, &pivot, &distances, direction_suivant, &active_tiers)
}

// ═══════════════════════════════════════════════════════════════════════
// Export Excel générique ⬅ commun à token_view.py et stats_view.py
// (les deux exports Python sont quasi identiques : ouvrir/créer une feuille
// portant le bon nom, écrire en-têtes + lignes, sauvegarder).
// ═══════════════════════════════════════════════════════════════════════

#[tauri::command(rename_all = "snake_case")]
pub fn export_table_excel(
    state: State<'_, SettingsState>,
    sheet_title: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    default_filename: String,
) -> Result<String, String> {
    if rows.is_empty() {
        return Err("Aucune donnée à exporter.".into());
    }

    let data = settings::get_current_data(state);
    let excel_path = data.excel_path.trim().to_string();
    let export_path = if excel_path.is_empty() { default_filename } else { excel_path };

    let (mut book, real_sheet_name) = excel::get_or_create_sheet(&export_path, &sheet_title)?;
    excel::write_table(&mut book, &real_sheet_name, &headers, &rows)?;
    excel::save(&book, &export_path)?;

    Ok(format!(
        "Données écrites avec succès dans la feuille \"{}\" de :\n{}",
        real_sheet_name, export_path
    ))
}

// ═══════════════════════════════════════════════════════════════════════
// Concordancier ⬅ concordancier_view.py
// (le drag & drop des colonnes reste côté frontend ; ici uniquement le
// moteur d'extraction + l'écriture Excel, fusionnés comme dans
// `_generate_excel`, qui fait tout en une seule opération sans aperçu.)
// ═══════════════════════════════════════════════════════════════════════

#[tauri::command(rename_all = "snake_case")]
pub fn generate_concordance_excel(
    state: State<'_, SettingsState>,
    pivots_raw: String,
    nb_left: i64,
    nb_right: i64,
    active_columns: Vec<String>,
) -> Result<String, String> {
    let data = settings::get_current_data(state);
    if data.textgrid_path_files.is_empty() {
        return Err("Aucun fichier TextGrid configuré. Veuillez vérifier vos paramètres.".into());
    }

    let separator = data.separator.clone();
    let pivots: Vec<String> = pivots_raw
        .split(separator.as_str())
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if pivots.is_empty() {
        return Err("Veuillez indiquer au moins un pivot.".into());
    }

    // Note : le fallback Python ("tokens_export.xlsx") semble être un copier-
    // coller de token_view.py plutôt qu'un vrai choix pour Concordancier ;
    // reproduit tel quel par fidélité — dites-moi si vous préférez le
    // renommer en "concordancier_export.xlsx".
    let excel_path = data.excel_path.trim().to_string();
    let export_path = if excel_path.is_empty() { "tokens_export.xlsx".to_string() } else { excel_path };

    let report = concordance::generate(&data.textgrid_path_files, &pivots, nb_left, nb_right, &active_columns)?;

    let (mut book, real_sheet_name) = excel::get_or_create_sheet(&export_path, "Concordancier")?;
    excel::write_table(&mut book, &real_sheet_name, &active_columns, &report.rows)?;
    excel::save(&book, &export_path)?;

    let mut parts = vec![
        format!("{} fichier(s) analysé(s)", report.processed_count),
        format!("{} occurrence(s)", report.total_occurrences),
    ];
    if !report.missing_files.is_empty() {
        parts.push(format!("{} introuvable(s)", report.missing_files.len()));
    }
    if !report.error_files.is_empty() {
        parts.push(format!("{} en erreur", report.error_files.len()));
    }

    Ok(format!(
        "{}. Exporté dans la feuille \"{}\" de : {}",
        parts.join(" · "),
        real_sheet_name,
        export_path
    ))
}
