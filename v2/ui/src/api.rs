//! api.rs — Pont typé vers les commandes Tauri (remplace les appels directs
//! à SettingsManager/backend qu'on avait côté Python : ici tout passe par
//! IPC via `invoke`).
//!
//! Non compilé dans le sandbox utilisé pour cette conversion (pas d'accès
//! à un environnement wasm32 ici) — écrit à partir de la documentation et
//! du code source vérifiés de Tauri v2 / tauri-plugin-dialog. Lancez
//! `trunk build` chez vous en premier pour confirmer.

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn invoke_raw(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Appelle une commande Tauri par son nom, avec des arguments sérialisables,
/// et désérialise la réponse dans le type attendu.
///
/// ⚠️ Nos propres commandes (settings::*, commands::*) utilisent
/// `#[tauri::command(rename_all = "snake_case")]` côté Rust : les clés de
/// `args` doivent donc être en snake_case (ex. `{ "merge_silences": true }`).
/// Le plugin dialog natif (`plugin:dialog|open`), lui, garde son propre
/// format documenté par Tauri (voir `pick_files` plus bas) — ce n'est pas
/// une de nos commandes, la règle snake_case ne s'y applique pas.
pub async fn invoke<A: Serialize, T: DeserializeOwned>(cmd: &str, args: &A) -> Result<T, String> {
    let args_js = serde_wasm_bindgen::to_value(args).map_err(|e| e.to_string())?;
    let result = invoke_raw(cmd, args_js)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

/// Args vide : sérialise en `{}`, pour les commandes qui ne prennent que
/// l'état géré par Tauri (ex. `get_current_data`, `analyze_overlaps`).
#[derive(Serialize)]
pub struct Empty {}

// ── Types miroirs des structs Serde côté src-tauri ──────────────────────
// À garder synchronisés avec config::ProfileData et textgrid::overlap::*.
// (Une prochaine amélioration possible : un crate `shared/` commun aux deux
// côtés pour ne plus dupliquer ces définitions.)

#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct ProfileData {
    pub textgrid_path_show: String,
    pub textgrid_path_files: Vec<String>,
    pub excel_path: String,
    pub separator: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OverlapError {
    pub source_tier: String,
    pub timecode: f64,
    pub interval_idx: usize,
    pub text: String,
    pub error_tiers: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FileOverlapResult {
    pub file: String,
    pub errors: Vec<OverlapError>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OverlapReport {
    pub results: Vec<FileOverlapResult>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
    pub total_errors: usize,
}

// ── Arguments des commandes settings::* / commands::* (snake_case) ──────

#[derive(Serialize)]
pub struct PathArg {
    pub path: String,
}

#[derive(Serialize)]
pub struct PathsArg {
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub struct SeparatorArg {
    pub separator: String,
}

#[derive(Serialize)]
pub struct TextModArgs {
    pub find: String,
    pub replace: String,
    pub merge_silences: bool,
}

#[derive(Serialize)]
pub struct PivotArgs {
    pub pivots_raw: String,
}

#[derive(Serialize)]
pub struct TrsPathsArg {
    pub trs_paths: Vec<String>,
}

// ── Tokenisation ⬅ textgrid::tokenisation::* côté src-tauri ─────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct TokenisationData {
    pub tiers: Vec<String>,
    pub global_counts: std::collections::HashMap<String, u32>,
    pub tier_counts: std::collections::HashMap<String, std::collections::HashMap<String, u32>>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
}

// ── Statistiques ⬅ textgrid::stats::* côté src-tauri ────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatsLoadSummary {
    pub tiers: Vec<String>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatsRow {
    pub display: String,
    pub global_count: u32,
    pub tier_counts: std::collections::HashMap<String, u32>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StatsAnalysisResult {
    pub rows: Vec<StatsRow>,
    pub total_pivot_global: u32,
    pub total_pivot_tier: std::collections::HashMap<String, u32>,
}

#[derive(Serialize)]
pub struct StatsAnalysisArgs {
    pub pivot: String,
    pub distances_raw: String,
    pub direction_suivant: bool,
    pub active_tiers: Vec<String>,
}

// ── Export Excel générique ⬅ commands::export_table_excel ───────────────

#[derive(Serialize)]
pub struct ExportTableArgs {
    pub sheet_title: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub default_filename: String,
}

// ── Concordancier ⬅ commands::generate_concordance_excel ────────────────

#[derive(Serialize)]
pub struct ConcordanceArgs {
    pub pivots_raw: String,
    pub nb_left: i64,
    pub nb_right: i64,
    pub active_columns: Vec<String>,
}

// ── Sélecteurs de fichiers/dossiers natifs (plugin dialog) ──────────────
// Format vérifié dans le code source officiel de tauri-plugin-dialog
// (plugins-workspace/plugins/dialog/guest-js/index.ts) :
// `invoke('plugin:dialog|open', { options })`.

#[derive(Serialize)]
struct DialogOpenArgs {
    options: DialogOpenOptions,
}

#[derive(Serialize, Default)]
struct DialogOpenOptions {
    directory: bool,
    multiple: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<DialogFilter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Serialize)]
struct DialogFilter {
    name: String,
    extensions: Vec<String>,
}

/// Ouvre le sélecteur natif pour choisir un ou plusieurs fichiers.
/// Retourne la liste des chemins choisis (vide si l'utilisateur annule).
pub async fn pick_files(title: &str, extensions: &[&str], multiple: bool) -> Result<Vec<String>, String> {
    let options = DialogOpenOptions {
        directory: false,
        multiple,
        filters: Some(vec![DialogFilter {
            name: "Fichiers".to_string(),
            extensions: extensions.iter().map(|e| e.to_string()).collect(),
        }]),
        title: Some(title.to_string()),
    };
    let result: Option<serde_json::Value> =
        invoke("plugin:dialog|open", &DialogOpenArgs { options }).await?;
    Ok(parse_paths(result))
}

fn parse_paths(value: Option<serde_json::Value>) -> Vec<String> {
    match value {
        None => vec![],
        Some(serde_json::Value::String(s)) => vec![s],
        Some(serde_json::Value::Array(arr)) => arr
            .into_iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}