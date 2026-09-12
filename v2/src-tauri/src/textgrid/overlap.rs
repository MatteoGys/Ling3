use std::path::Path;
use serde::Serialize;
use super::parser::{self, Tier};

#[derive(Debug, Clone, Serialize)]
pub struct OverlapError {
    pub source_tier: String,
    pub timecode: f64,
    pub interval_idx: usize,
    pub text: String,
    pub error_tiers: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileOverlapResult {
    pub file: String,
    pub errors: Vec<OverlapError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverlapReport {
    pub results: Vec<FileOverlapResult>,
    pub missing_files: Vec<String>,
    pub error_files: Vec<String>,
    pub processed_count: usize,
    pub total_errors: usize,
}

pub fn analyze_files(paths: &[String]) -> OverlapReport {
    let mut results = Vec::new();
    let mut missing_files = Vec::new();
    let mut error_files = Vec::new();
    let mut processed_count = 0usize;
    let mut total_errors = 0usize;

    for element in paths {
        let path = Path::new(element);
        if element.is_empty() || !path.exists() {
            missing_files.push(if element.is_empty() { "(chemin vide)".to_string() } else { element.clone() });
            continue;
        }

        match parser::parse_textgrid(path) {
            Ok(tiers) => {
                let file_label = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(element)
                    .to_string();
                let errors = detect_overlaps(&tiers, &file_label);
                total_errors += errors.len();
                processed_count += 1;
                results.push(FileOverlapResult { file: file_label, errors });
            }
            Err(e) => error_files.push(format!(
                "{} ({})",
                path.file_name().and_then(|s| s.to_str()).unwrap_or(element),
                e
            )),
        }
    }

    OverlapReport { results, missing_files, error_files, processed_count, total_errors }
}

fn detect_overlaps(tiers: &[Tier], file_label: &str) -> Vec<OverlapError> {
    let mut errors = Vec::new();

    // Indices triés par xmin pour chaque tier : permet la recherche dichotomique
    // du seul intervalle candidat pouvant chevaucher un instant xmax donné,
    // au lieu d'un parcours linéaire de tous les intervalles (comme en Python).
    let sorted_idx: Vec<Vec<usize>> = tiers
        .iter()
        .map(|tier| {
            let mut idxs: Vec<usize> = (0..tier.intervals.len()).collect();
            idxs.sort_by(|&a, &b| {
                tier.intervals[a]
                    .xmin
                    .partial_cmp(&tier.intervals[b].xmin)
                    .unwrap()
            });
            idxs
        })
        .collect();

    for (tier_pos, tier) in tiers.iter().enumerate() {
        for (idx_int, interval) in tier.intervals.iter().enumerate() {
            let xmax = interval.xmax;
            let mut error_tiers = Vec::new();

            for (target_pos, target_tier) in tiers.iter().enumerate() {
                if target_pos == tier_pos {
                    continue;
                }
                let idxs = &sorted_idx[target_pos];
                // Dernier intervalle dont xmin <= xmax.
                let pos = idxs.partition_point(|&i| target_tier.intervals[i].xmin <= xmax);
                if pos == 0 {
                    continue;
                }
                let cand = &target_tier.intervals[idxs[pos - 1]];
                if cand.xmin < xmax && xmax < cand.xmax && !cand.text.is_empty() {
                    error_tiers.push(format!("Tier {}", target_tier.index));
                }
            }

            if !error_tiers.is_empty() {
                errors.push(OverlapError {
                    source_tier: format!("Tier {} {}", tier.index, file_label),
                    timecode: xmax,
                    interval_idx: idx_int + 1,
                    text: interval.text.clone(),
                    error_tiers: error_tiers.join(", "),
                });
            }
        }
    }

    errors
}
