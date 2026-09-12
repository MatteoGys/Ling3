//! concordance.rs — Remplace concordancier_view.py (moteur d'extraction,
//! hors UI et hors drag&drop).

use std::collections::HashMap;
use std::path::Path;
use regex::Regex;
use serde::Serialize;

use super::parser;

#[derive(Debug, Clone, Serialize)]
pub struct ConcordanceReport {
    /// Chaque ligne est déjà ordonnée selon `active_columns`.
    pub rows: Vec<Vec<String>>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
    pub total_occurrences: usize,
}

pub fn generate(
    paths: &[String],
    pivots: &[String],
    nb_left: i64,
    nb_right: i64,
    active_columns: &[String],
) -> Result<ConcordanceReport, String> {
    let compiled: Vec<Regex> = pivots
        .iter()
        .map(|p| Regex::new(&format!(r"(?i)\b{}\b", regex::escape(p))))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut missing_files = Vec::new();
    let mut error_files = Vec::new();
    let mut processed_count = 0usize;
    let mut occurrence_id: u64 = 1;
    // Compteur GLOBAL (pas remis à zéro par fichier), comme `tier_counter`
    // en Python : sert uniquement à l'affichage (Tier Numéro / Tier Lettre).
    let mut tier_counter: usize = 0;

    for element in paths {
        let path = Path::new(element);
        if element.is_empty() || !path.exists() {
            missing_files.push(if element.is_empty() { "(chemin vide)".to_string() } else { element.clone() });
            continue;
        }

        let file_tiers = match parser::parse_textgrid(path) {
            Ok(t) => t,
            Err(e) => {
                let label = path.file_name().and_then(|s| s.to_str()).unwrap_or(element);
                error_files.push(format!("{} ({})", label, e));
                continue;
            }
        };

        for tier in &file_tiers {
            tier_counter += 1;
            let tier_number = tier_counter;
            let tier_letter = if tier_number <= 26 {
                ((b'A' + (tier_number - 1) as u8) as char).to_string()
            } else {
                tier_number.to_string()
            };

            // flat_words + interval_word_start : propres à CE tier, DANS CE
            // FICHIER (le contexte ne traverse jamais deux fichiers, contrairement
            // à Statistiques).
            let mut flat_words: Vec<&str> = Vec::new();
            let mut interval_word_start: Vec<usize> = Vec::new();
            for interval in &tier.intervals {
                interval_word_start.push(flat_words.len());
                flat_words.extend(interval.text.split_whitespace());
            }
            interval_word_start.push(flat_words.len()); // sentinelle finale

            let is_empty: Vec<bool> = tier.intervals.iter().map(|iv| iv.text.trim().is_empty()).collect();

            for (idx, interval) in tier.intervals.iter().enumerate() {
                if is_empty[idx] {
                    continue;
                }
                let txt = &interval.text;

                for pattern in &compiled {
                    for m in pattern.find_iter(txt) {
                        let ip_empty = if idx == 0 { true } else { is_empty[idx - 1] };
                        let in_empty = if idx == tier.intervals.len() - 1 { true } else { is_empty[idx + 1] };

                        let pos = match (ip_empty, in_empty) {
                            (true, true) => "Isolé",
                            (true, false) => "Initial",
                            (false, true) => "Final",
                            (false, false) => "Médian",
                        };

                        let texte_gauche = &txt[..m.start()];
                        let texte_droit = &txt[m.end()..];
                        let mots_gauche_locaux: Vec<&str> = texte_gauche.split_whitespace().collect();
                        let mots_droit_locaux: Vec<&str> = texte_droit.split_whitespace().collect();

                        let mots_gauche: Vec<&str> = if nb_left > 0 {
                            let nb_left = nb_left as usize;
                            if mots_gauche_locaux.len() >= nb_left {
                                mots_gauche_locaux[mots_gauche_locaux.len() - nb_left..].to_vec()
                            } else {
                                let manquants = nb_left - mots_gauche_locaux.len();
                                let debut_global = interval_word_start[idx];
                                let start = debut_global.saturating_sub(manquants);
                                let mut prefixe: Vec<&str> = flat_words[start..debut_global].to_vec();
                                prefixe.extend(mots_gauche_locaux.iter());
                                prefixe
                            }
                        } else {
                            Vec::new()
                        };

                        let mots_droit: Vec<&str> = if nb_right > 0 {
                            let nb_right = nb_right as usize;
                            if mots_droit_locaux.len() >= nb_right {
                                mots_droit_locaux[..nb_right].to_vec()
                            } else {
                                let manquants = nb_right - mots_droit_locaux.len();
                                let fin_global = interval_word_start[idx + 1];
                                let end = (fin_global + manquants).min(flat_words.len());
                                let mut suffixe: Vec<&str> = mots_droit_locaux.clone();
                                suffixe.extend(flat_words[fin_global..end].iter());
                                suffixe
                            }
                        } else {
                            Vec::new()
                        };

                        let ctx_gauche = mots_gauche.join(" ");
                        let ctx_droit = mots_droit.join(" ");

                        let mut row_data: HashMap<&str, String> = HashMap::new();
                        row_data.insert("ID", occurrence_id.to_string());
                        row_data.insert("Tier Nom", tier.name.clone());
                        row_data.insert("Tier Numéro", tier_number.to_string());
                        row_data.insert("Tier Lettre", tier_letter.clone());
                        row_data.insert("x_min", format!("{:.4}", interval.xmin));
                        row_data.insert("x_max", format!("{:.4}", interval.xmax));
                        row_data.insert("durée_occurence", format!("{:.4}", interval.xmax - interval.xmin));
                        row_data.insert("contexte gauche", ctx_gauche);
                        row_data.insert("pivot", m.as_str().to_string());
                        row_data.insert("contexte droit", ctx_droit);
                        row_data.insert("POS_PAUSES_VIDES", pos.to_string());

                        let final_row: Vec<String> = active_columns
                            .iter()
                            .map(|col| row_data.get(col.as_str()).cloned().unwrap_or_default())
                            .collect();

                        rows.push(final_row);
                        occurrence_id += 1;
                    }
                }
            }
        }
        processed_count += 1;
    }

    let total_occurrences = rows.len();
    Ok(ConcordanceReport { rows, missing_files, error_files, processed_count, total_occurrences })
}
