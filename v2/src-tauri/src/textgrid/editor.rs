use std::collections::HashMap;
use std::fs;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct EditableInterval {
    pub indent: String,
    pub props: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EditableTier {
    pub indent: String,
    pub props: Vec<String>,
    pub intervals: Vec<EditableInterval>,
}

#[derive(Debug, Clone)]
pub struct EditableTextGrid {
    pub header: Vec<String>,
    pub tiers: Vec<EditableTier>,
}

fn decode_with_fallback(raw: &[u8]) -> String {
    if let Some(stripped) = raw.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        if let Ok(s) = std::str::from_utf8(stripped) {
            return s.to_string();
        }
    }
    if let Some(stripped) = raw.strip_prefix(&[0xFF, 0xFE]) {
        let (s, _, _) = encoding_rs::UTF_16LE.decode(stripped);
        return s.into_owned();
    }
    if let Some(stripped) = raw.strip_prefix(&[0xFE, 0xFF]) {
        let (s, _, _) = encoding_rs::UTF_16BE.decode(stripped);
        return s.into_owned();
    }
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }
    // Repli latin-1 : chaque octet correspond exactement à un point de code
    // Unicode 0-255, donc ce décodage ne peut jamais échouer (comme en Python).
    raw.iter().map(|&b| b as char).collect()
}

pub fn parse_textgrid_editable(path: &str) -> Result<EditableTextGrid, String> {
    let raw = fs::read(path).map_err(|e| e.to_string())?;
    let content = decode_with_fallback(&raw);

    let tier_idx_re = Regex::new(r"item\s*\[\s*(\d+)\s*\]\s*:").unwrap();
    let interval_idx_re = Regex::new(r"intervals\s*\[\s*(\d+)\s*\]\s*:").unwrap();

    let mut header = Vec::new();
    let mut tiers: Vec<EditableTier> = Vec::new();
    let mut header_done = false;

    for line in content.lines() {
        let stripped = line.trim();

        if !header_done {
            header.push(line.to_string());
            if stripped.starts_with("item []") {
                header_done = true;
            }
            continue;
        }

        if tier_idx_re.is_match(stripped) {
            let indent = line.split("item").next().unwrap_or("").to_string();
            tiers.push(EditableTier { indent, props: Vec::new(), intervals: Vec::new() });
            continue;
        }

        if let Some(tier) = tiers.last_mut() {
            if interval_idx_re.is_match(stripped) {
                let indent = line.split("intervals").next().unwrap_or("").to_string();
                tier.intervals.push(EditableInterval { indent, props: HashMap::new() });
                continue;
            }

            if let Some(interval) = tier.intervals.last_mut() {
                if let Some((k, v)) = stripped.split_once('=') {
                    interval.props.insert(k.trim().to_string(), v.trim().to_string());
                }
            } else if !stripped.contains("intervals: size") {
                tier.props.push(line.to_string());
            }
        }
    }

    Ok(EditableTextGrid { header, tiers })
}

fn text_value(interval: &EditableInterval) -> String {
    interval
        .props
        .get("text")
        .cloned()
        .unwrap_or_else(|| "\"\"".to_string())
        .trim_matches('"')
        .trim()
        .to_string()
}

/// Fusionne les intervalles vides consécutifs (silences). Retourne le nombre de fusions.
pub fn merge_empty_intervals(tiers: &mut [EditableTier]) -> usize {
    let mut merged_count = 0;
    for tier in tiers.iter_mut() {
        let old = std::mem::take(&mut tier.intervals);
        let mut cleaned: Vec<EditableInterval> = Vec::new();

        for item in old {
            if cleaned.is_empty() {
                cleaned.push(item);
                continue;
            }
            let last_txt = text_value(cleaned.last().unwrap());
            let curr_txt = text_value(&item);

            if last_txt.is_empty() && curr_txt.is_empty() {
                let new_xmax = item.props.get("xmax").cloned().unwrap_or_default();
                if let Some(last) = cleaned.last_mut() {
                    last.props.insert("xmax".to_string(), new_xmax);
                }
                merged_count += 1;
            } else {
                cleaned.push(item);
            }
        }
        tier.intervals = cleaned;
    }
    merged_count
}

/// Remplace `find` par `replace` dans le texte de chaque intervalle. Retourne le nombre d'occurrences remplacées.
pub fn replace_text(tiers: &mut [EditableTier], find: &str, replace: &str) -> usize {
    let mut count = 0;
    if find.is_empty() {
        return 0;
    }
    for tier in tiers.iter_mut() {
        for interval in tier.intervals.iter_mut() {
            if let Some(raw) = interval.props.get("text").cloned() {
                let pure = raw.trim_matches('"');
                if pure.contains(find) {
                    count += pure.matches(find).count();
                    let replaced = pure.replace(find, replace);
                    interval.props.insert("text".to_string(), format!("\"{}\"", replaced));
                }
            }
        }
    }
    count
}

/// Ne garde que les mots pivots (le premier trouvé) dans chaque intervalle, vide les autres.
/// Retourne le nombre d'intervalles où un pivot a été isolé.
pub fn extract_pivots(tiers: &mut [EditableTier], pivots: &[String]) -> usize {
    let mut count = 0;
    for tier in tiers.iter_mut() {
        for interval in tier.intervals.iter_mut() {
            if let Some(raw) = interval.props.get("text").cloned() {
                let pure = raw.trim_matches('"');
                if let Some(found) = pivots.iter().find(|p| pure.contains(p.as_str())) {
                    interval.props.insert("text".to_string(), format!("\"{}\"", found));
                    count += 1;
                } else {
                    interval.props.insert("text".to_string(), "\"\"".to_string());
                }
            }
        }
    }
    count
}

pub fn write_textgrid(path: &str, tg: &EditableTextGrid) -> Result<(), String> {
    let mut out: Vec<String> = Vec::new();
    out.extend(tg.header.iter().cloned());

    for (i, tier) in tg.tiers.iter().enumerate() {
        out.push(format!("{}item [{}]:", tier.indent, i + 1));
        out.extend(tier.props.iter().cloned());

        let size_indent = format!("{}    ", tier.indent);
        out.push(format!("{}intervals: size = {}", size_indent, tier.intervals.len()));

        for (j, interval) in tier.intervals.iter().enumerate() {
            out.push(format!("{}intervals [{}]:", interval.indent, j + 1));
            let sub_indent = format!("{}    ", interval.indent);
            out.push(format!(
                "{}xmin = {}",
                sub_indent,
                interval.props.get("xmin").cloned().unwrap_or_default()
            ));
            out.push(format!(
                "{}xmax = {}",
                sub_indent,
                interval.props.get("xmax").cloned().unwrap_or_default()
            ));
            out.push(format!(
                "{}text = {}",
                sub_indent,
                interval.props.get("text").cloned().unwrap_or_else(|| "\"\"".to_string())
            ));
        }
    }

    fs::write(path, out.join("\n")).map_err(|e| e.to_string())
}
