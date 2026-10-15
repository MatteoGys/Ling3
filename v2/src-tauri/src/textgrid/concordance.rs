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

struct ContextWords<'a> {
    words: Vec<&'a str>,
    marker_at: Option<usize>,
}

fn collect_left<'a>(interval_words: &[Vec<&'a str>], idx: usize, manquants: usize) -> ContextWords<'a> {
    let mut collected: Vec<&str> = Vec::new();
    let mut near_count: Option<usize> = None;
    let mut k = idx;
    while collected.len() < manquants && k > 0 {
        k -= 1;
        if interval_words[k].is_empty() {
            if near_count.is_none() {
                near_count = Some(collected.len());
            }
            continue;
        }
        let mut combined: Vec<&str> = interval_words[k].clone();
        combined.extend(collected);
        collected = combined;
    }

    let mut marker_at = near_count.map(|n| collected.len() - n);
    if collected.len() > manquants {
        let cut = collected.len() - manquants;
        collected = collected[cut..].to_vec();
        if let Some(m) = marker_at {
            marker_at = Some(m.saturating_sub(cut));
        }
    }
    ContextWords { words: collected, marker_at }
}

fn collect_right<'a>(interval_words: &[Vec<&'a str>], idx: usize, manquants: usize) -> ContextWords<'a> {
    let mut collected: Vec<&str> = Vec::new();
    let mut near_count: Option<usize> = None;
    let mut k = idx;
    while collected.len() < manquants && k + 1 < interval_words.len() {
        k += 1;
        if interval_words[k].is_empty() {
            if near_count.is_none() {
                near_count = Some(collected.len());
            }
            continue;
        }
        collected.extend(interval_words[k].iter().copied());
    }

    let marker_at = near_count;
    if collected.len() > manquants {
        collected.truncate(manquants);
    }
    ContextWords { words: collected, marker_at }
}

fn join_with_marker(words: &[&str], marker_at: Option<usize>) -> String {
    match marker_at {
        None => words.join(" "),
        Some(pos) => {
            let pos = pos.min(words.len());
            let mut parts: Vec<&str> = Vec::with_capacity(words.len() + 1);
            parts.extend(&words[..pos]);
            parts.push("|_|");
            parts.extend(&words[pos..]);
            parts.join(" ")
        }
    }
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

            let interval_words: Vec<Vec<&str>> =
                tier.intervals.iter().map(|iv| iv.text.split_whitespace().collect()).collect();

            for (idx, interval) in tier.intervals.iter().enumerate() {
                if interval_words[idx].is_empty() {
                    continue;
                }
                let txt = &interval.text;

                for pattern in &compiled {
                    for m in pattern.find_iter(txt) {
                        let ip_empty = if idx == 0 { true } else { interval_words[idx - 1].is_empty() };
                        let in_empty = if idx == tier.intervals.len() - 1 { true } else { interval_words[idx + 1].is_empty() };

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

                        let ctx_gauche = if nb_left > 0 {
                            let nb_left = nb_left as usize;
                            if mots_gauche_locaux.len() >= nb_left {
                                mots_gauche_locaux[mots_gauche_locaux.len() - nb_left..].join(" ")
                            } else {
                                let manquants = nb_left - mots_gauche_locaux.len();
                                let prefixe = collect_left(&interval_words, idx, manquants);
                                let mut all: Vec<&str> = prefixe.words.clone();
                                all.extend(mots_gauche_locaux.iter().copied());
                                join_with_marker(&all, prefixe.marker_at)
                            }
                        } else {
                            String::new()
                        };

                        let ctx_droit = if nb_right > 0 {
                            let nb_right = nb_right as usize;
                            if mots_droit_locaux.len() >= nb_right {
                                mots_droit_locaux[..nb_right].join(" ")
                            } else {
                                let manquants = nb_right - mots_droit_locaux.len();
                                let suffixe = collect_right(&interval_words, idx, manquants);
                                let mut all: Vec<&str> = mots_droit_locaux.clone();
                                let local_len = all.len();
                                all.extend(suffixe.words.iter().copied());
                                let marker_at = suffixe.marker_at.map(|m| local_len + m);
                                join_with_marker(&all, marker_at)
                            }
                        } else {
                            String::new()
                        };

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
