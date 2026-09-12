//! cooccurrences.rs — Remplace stats_view.py (moteur, hors UI).
//! Charge les tokens de chaque tier dans l'ordre (avec des bornes `None`
//! entre fichiers différents), puis recherche un pivot et compte les mots
//! de contexte à une ou plusieurs distances données.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use serde::Serialize;

use super::parser;

/// Token ordonné d'un tier ; `None` = frontière entre deux fichiers
/// différents (empêche un contexte de chevaucher deux fichiers).
pub type TokenSlot = Option<String>;

#[derive(Debug, Clone, Default)]
pub struct StatsBaseData {
    pub tiers_raw_tokens: HashMap<String, Vec<TokenSlot>>,
    pub tier_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsLoadSummary {
    pub tiers: Vec<String>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c == ' ' || c == '\'')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

pub fn load(paths: &[String]) -> (StatsBaseData, StatsLoadSummary) {
    let mut tiers_raw_tokens: HashMap<String, Vec<TokenSlot>> = HashMap::new();
    let mut tier_order: Vec<String> = Vec::new();
    let mut missing_files = Vec::new();
    let mut error_files = Vec::new();
    let mut processed_count = 0usize;

    for element in paths {
        let path = Path::new(element);
        if element.is_empty() || !path.exists() {
            missing_files.push(if element.is_empty() { "(chemin vide)".to_string() } else { element.clone() });
            continue;
        }

        match parser::parse_textgrid(path) {
            Ok(file_tiers) => {
                let mut tiers_seen_this_file: HashSet<String> = HashSet::new();

                for tier in file_tiers {
                    let name = if tier.name.is_empty() { "Unknown".to_string() } else { tier.name };

                    if !tiers_raw_tokens.contains_key(&name) {
                        tiers_raw_tokens.insert(name.clone(), Vec::new());
                        tier_order.push(name.clone());
                    } else if !tiers_seen_this_file.contains(&name) {
                        // Ce tier existe déjà via un AUTRE fichier : borne de
                        // séparation pour ne pas mélanger deux fichiers dans
                        // une même fenêtre de contexte.
                        tiers_raw_tokens.get_mut(&name).unwrap().push(None);
                    }
                    tiers_seen_this_file.insert(name.clone());

                    let bucket = tiers_raw_tokens.get_mut(&name).unwrap();
                    for interval in tier.intervals {
                        if interval.text.trim().is_empty() {
                            continue;
                        }
                        for tok in tokenize(&interval.text) {
                            bucket.push(Some(tok));
                        }
                    }
                }
                processed_count += 1;
            }
            Err(e) => {
                let label = path.file_name().and_then(|s| s.to_str()).unwrap_or(element);
                error_files.push(format!("{} ({})", label, e));
            }
        }
    }

    let summary = StatsLoadSummary {
        tiers: tier_order.clone(),
        missing_files,
        error_files,
        processed_count,
    };

    (StatsBaseData { tiers_raw_tokens, tier_order }, summary)
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsRow {
    pub display: String,
    pub global_count: u32,
    pub tier_counts: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatsAnalysisResult {
    pub rows: Vec<StatsRow>,
    pub total_pivot_global: u32,
    pub total_pivot_tier: HashMap<String, u32>,
}

/// True si une borne de fichier (None) se trouve entre les index a et b
/// inclus (bornes remises dans l'ordre si besoin, et resserrées aux limites
/// du vecteur).
fn has_file_boundary(tokens: &[TokenSlot], a: i64, b: i64) -> bool {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let lo = lo.max(0) as usize;
    let hi = (hi.min(tokens.len() as i64 - 1)).max(0) as usize;
    if tokens.is_empty() {
        return false;
    }
    tokens[lo..=hi.min(tokens.len() - 1)].iter().any(|t| t.is_none())
}

pub fn analyze(
    base: &StatsBaseData,
    pivot_raw: &str,
    distances: &[i64],
    direction_suivant: bool,
    active_tiers: &[String],
) -> Result<StatsAnalysisResult, String> {
    let pivot_tokens: Vec<String> = pivot_raw
        .trim()
        .to_lowercase()
        .split(|c: char| c == ' ' || c == '\'')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if pivot_tokens.is_empty() {
        return Err("Veuillez entrer un pivot valide.".to_string());
    }

    let p_len = pivot_tokens.len() as i64;
    let pivot_display = pivot_tokens.join(" ");

    // display_str -> compte, dans l'ordre de première rencontre (pour
    // reproduire l'ordre stable de Counter.most_common() en cas d'égalité).
    let mut order: Vec<String> = Vec::new();
    let mut counts_global: HashMap<String, u32> = HashMap::new();
    let mut counts_tier: HashMap<String, HashMap<String, u32>> = HashMap::new();
    let mut total_pivot_global: u32 = 0;
    let mut total_pivot_tier: HashMap<String, u32> = HashMap::new();

    for tier in active_tiers {
        total_pivot_tier.entry(tier.clone()).or_insert(0);
        counts_tier.entry(tier.clone()).or_insert_with(HashMap::new);

        let Some(tokens) = base.tiers_raw_tokens.get(tier) else { continue };
        let n = tokens.len() as i64;

        let mut i: i64 = 0;
        while i <= n - p_len {
            let window = &tokens[i as usize..(i + p_len) as usize];
            if window.iter().any(|t| t.is_none()) {
                i += 1;
                continue;
            }
            let window_words: Vec<&str> = window.iter().map(|t| t.as_deref().unwrap()).collect();
            let matches = window_words.len() == pivot_tokens.len()
                && window_words.iter().zip(pivot_tokens.iter()).all(|(a, b)| *a == b);

            if matches {
                *total_pivot_tier.get_mut(tier).unwrap() += 1;
                total_pivot_global += 1;

                for &d in distances {
                    let (target_idx, span) = if direction_suivant {
                        let target_idx = i + p_len + d - 1;
                        (target_idx, (i + p_len, target_idx))
                    } else {
                        let target_idx = i - d;
                        (target_idx, (target_idx, i - 1))
                    };

                    if target_idx >= 0 && target_idx < n && !has_file_boundary(tokens, span.0, span.1) {
                        let target_word = tokens[target_idx as usize].as_deref().unwrap_or("");

                        let underscores = if d == 1 {
                            " ".to_string()
                        } else {
                            format!(" {} ", vec!["_"; (d - 1).max(0) as usize].join(" "))
                        };

                        let display = if direction_suivant {
                            format!("{}{}{}", pivot_display, underscores, target_word)
                        } else {
                            format!("{}{}{}", target_word, underscores, pivot_display)
                        }
                        .replace("  ", " ");

                        if !counts_global.contains_key(&display) {
                            order.push(display.clone());
                        }
                        *counts_global.entry(display.clone()).or_insert(0) += 1;
                        *counts_tier.get_mut(tier).unwrap().entry(display).or_insert(0) += 1;
                    }
                }
            }
            i += 1;
        }
    }

    // Tri par compte global décroissant, égalités départagées par ordre de
    // première rencontre (comme Counter.most_common()).
    let mut rows: Vec<StatsRow> = order
        .into_iter()
        .map(|display| {
            let global_count = *counts_global.get(&display).unwrap_or(&0);
            let mut tier_counts = HashMap::new();
            for tier in active_tiers {
                let c = counts_tier.get(tier).and_then(|m| m.get(&display)).copied().unwrap_or(0);
                tier_counts.insert(tier.clone(), c);
            }
            StatsRow { display, global_count, tier_counts }
        })
        .collect();

    rows.sort_by(|a, b| b.global_count.cmp(&a.global_count));
    // sort_by est stable en Rust : l'ordre initial (première rencontre) est
    // préservé pour les égalités, comme Counter.most_common().

    Ok(StatsAnalysisResult { rows, total_pivot_global, total_pivot_tier })
}
