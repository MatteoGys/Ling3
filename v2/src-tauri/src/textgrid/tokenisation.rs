//! tokenisation.rs — Remplace token_view.py (moteur, hors UI).
//! Analyse tous les fichiers TextGrid configurés, compte les occurrences de
//! chaque token, par tier ET au global (somme des tiers).

use std::collections::HashMap;
use std::path::Path;
use serde::Serialize;

use super::parser;

#[derive(Debug, Clone, Serialize)]
pub struct TokenisationData {
    /// Noms des tiers rencontrés, dans l'ordre de première apparition.
    pub tiers: Vec<String>,
    /// mot -> compte total (somme des comptes de tous les tiers, comme en Python).
    pub global_counts: HashMap<String, u32>,
    /// tier -> mot -> compte.
    pub tier_counts: HashMap<String, HashMap<String, u32>>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
}

/// Découpe un texte d'intervalle en tokens minuscules, comme
/// `re.split(r"[ ']+", txt)` en Python.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c == ' ' || c == '\'')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase())
        .collect()
}

pub fn load(paths: &[String]) -> TokenisationData {
    let mut tiers_order: Vec<String> = Vec::new();
    let mut tier_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
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
                for tier in file_tiers {
                    let name = if tier.name.is_empty() { "Unknown".to_string() } else { tier.name };
                    let entry = tier_counts.entry(name.clone()).or_insert_with(|| {
                        tiers_order.push(name.clone());
                        HashMap::new()
                    });
                    for interval in tier.intervals {
                        if interval.text.trim().is_empty() {
                            continue;
                        }
                        for tok in tokenize(&interval.text) {
                            *entry.entry(tok).or_insert(0) += 1;
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

    // Agrégation globale — somme des comptes de tous les tiers (comme
    // `for tier_counts in self.tiers_data.values(): self.global_counts.update(tier_counts)`,
    // qui ADDITIONNE, ne fait pas juste l'union).
    let mut global_counts: HashMap<String, u32> = HashMap::new();
    for counts in tier_counts.values() {
        for (word, count) in counts {
            *global_counts.entry(word.clone()).or_insert(0) += count;
        }
    }

    TokenisationData {
        tiers: tiers_order,
        global_counts,
        tier_counts,
        missing_files,
        error_files,
        processed_count,
    }
}
