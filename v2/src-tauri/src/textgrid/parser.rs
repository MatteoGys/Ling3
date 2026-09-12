use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Interval {
    pub xmin: f64,
    pub xmax: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Tier {
    pub index: usize,
    pub name: String,
    pub intervals: Vec<Interval>,
}

pub fn read_text_auto(path: &Path) -> Result<String, String> {
    let raw = fs::read(path).map_err(|e| e.to_string())?;

    if raw.starts_with(&[0xFF, 0xFE]) {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(&raw[2..]);
        return Ok(text.into_owned());
    }
    if raw.starts_with(&[0xFE, 0xFF]) {
        let (text, _, _) = encoding_rs::UTF_16BE.decode(&raw[2..]);
        return Ok(text.into_owned());
    }
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (text, _, _) = encoding_rs::UTF_8.decode(&raw[3..]);
        return Ok(text.into_owned());
    }

    match String::from_utf8(raw.clone()) {
        Ok(text) => Ok(text),
        Err(_) => {
            let (text, _, _) = encoding_rs::UTF_16LE.decode(&raw);
            Ok(text.into_owned())
        }
    }
}

pub fn parse_textgrid(path: &Path) -> Result<Vec<Tier>, String> {
    let content = read_text_auto(path)?;
    let mut tiers: Vec<Tier> = Vec::new();
    let mut current_tier_idx: Option<usize> = None;
    let mut in_interval = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if line.starts_with("item [") && !line.starts_with("item []") {
            if let Some(id_str) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                if let Ok(id) = id_str.trim().parse::<usize>() {
                    tiers.push(Tier { index: id, name: String::new(), intervals: Vec::new() });
                    current_tier_idx = Some(tiers.len() - 1);
                    in_interval = false;
                }
            }
        } else if line.starts_with("name =") && !in_interval && current_tier_idx.is_some() {
            let name = line.splitn(2, '=').nth(1).unwrap_or("").trim().trim_matches('"');
            tiers[current_tier_idx.unwrap()].name = name.to_string();
        } else if line.starts_with("intervals [") && current_tier_idx.is_some() {
            in_interval = true;
            let idx = current_tier_idx.unwrap();
            tiers[idx].intervals.push(Interval { xmin: 0.0, xmax: 0.0, text: String::new() });
        } else if in_interval && current_tier_idx.is_some() {
            let idx = current_tier_idx.unwrap();
            if let Some(interval) = tiers[idx].intervals.last_mut() {
                if let Some(v) = line.strip_prefix("xmin =") {
                    interval.xmin = v.trim().parse().unwrap_or(0.0);
                } else if let Some(v) = line.strip_prefix("xmax =") {
                    interval.xmax = v.trim().parse().unwrap_or(0.0);
                } else if let Some(v) = line.strip_prefix("text =") {
                    interval.text = v.trim().trim_matches('"').to_string();
                }
            }
        }
    }

    Ok(tiers)
}
