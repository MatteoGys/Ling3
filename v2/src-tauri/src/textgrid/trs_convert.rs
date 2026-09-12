use std::collections::HashMap;
use std::fs;
use regex::Regex;

struct Utterance {
    start: f64,
    text: String,
    end: f64,
}

struct Interval {
    start: f64,
    end: f64,
    text: String,
}

/// Traduction directe de _parse_and_convert_trs (elle-même une traduction
/// d'un script Perl historique). Convertit un fichier .trs(.xml) Transcriber
/// en fichier .TextGrid Praat, une tier par locuteur, silences comblés.
#[allow(unused_assignments)]
pub fn convert_trs_to_textgrid(trs_path: &str, out_path: &str) -> Result<(), String> {
    let raw = fs::read(trs_path).map_err(|e| e.to_string())?;
    // Équivalent de open(..., encoding="utf-8", errors="ignore") : on retombe
    // sur une lecture tolérante si le fichier n'est pas de l'UTF-8 strict.
    let content = String::from_utf8(raw.clone())
        .unwrap_or_else(|_| String::from_utf8_lossy(&raw).into_owned());

    let re_spk = Regex::new(r#"<Speaker.+?id="(.+?)".+?name="(.+?)""#).map_err(|e| e.to_string())?;
    let re_sec = Regex::new(r#"<Section.+?startTime="(.+?)".+?endTime="(.+?)""#).map_err(|e| e.to_string())?;
    let re_turn = Regex::new(r#"<Turn.+?speaker="(.+?)".+?startTime="(.+?)".+?endTime="(.+?)""#).map_err(|e| e.to_string())?;
    let re_sync = Regex::new(r#"<Sync time="(.+?)""#).map_err(|e| e.to_string())?;
    let re_who = Regex::new(r#"<Who nb="(\d+?)""#).map_err(|e| e.to_string())?;

    let mut speaker_order: Vec<String> = Vec::new();
    let mut speaker_names: HashMap<String, String> = HashMap::new();
    let mut tiers: HashMap<String, Vec<Interval>> = HashMap::new();
    let mut utterances: HashMap<String, Vec<Utterance>> = HashMap::new();

    let mut start_time = 100000.0_f64;
    let mut end_time = 0.0_f64;

    let mut in_turn = false;
    let mut utt_needs_end = false;
    let mut turn_end = 0.0_f64;
    let mut active_spkrs = String::new();
    let mut cur_spkr = String::new();
    let mut turn_spkrs: Vec<String> = Vec::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if let Some(caps) = re_spk.captures(line) {
            let id = caps[1].to_string();
            let name = caps[2].to_string();
            if !speaker_names.contains_key(&id) {
                speaker_order.push(id.clone());
            }
            speaker_names.insert(id.clone(), name);
            tiers.insert(id.clone(), Vec::new());
            utterances.insert(id, Vec::new());
            continue;
        }

        if let Some(caps) = re_sec.captures(line) {
            let s: f64 = caps[1].parse().map_err(|_| format!("startTime invalide : {}", &caps[1]))?;
            let e: f64 = caps[2].parse().map_err(|_| format!("endTime invalide : {}", &caps[2]))?;
            start_time = start_time.min(s);
            end_time = end_time.max(e);
            continue;
        }

        if let Some(caps) = re_turn.captures(line) {
            in_turn = true;
            active_spkrs = caps[1].to_string();
            turn_end = caps[3].parse().map_err(|_| format!("endTime de Turn invalide : {}", &caps[3]))?;
            turn_spkrs = active_spkrs.split_whitespace().map(|s| s.to_string()).collect();
            cur_spkr = turn_spkrs.first().cloned().unwrap_or_default();
            continue;
        }

        if line.contains("</Turn>") {
            if utt_needs_end {
                utt_needs_end = false;
                for s in &turn_spkrs {
                    if let Some(list) = utterances.get_mut(s) {
                        if let Some(last) = list.last_mut() {
                            last.end = turn_end;
                        }
                    }
                }
            }
            in_turn = false;
            continue;
        }

        if !in_turn {
            continue;
        }

        if let Some(caps) = re_sync.captures(line) {
            let t_val: f64 = caps[1].parse().map_err(|_| format!("time de Sync invalide : {}", &caps[1]))?;
            if utt_needs_end {
                utt_needs_end = false;
                for s in &turn_spkrs {
                    if let Some(list) = utterances.get_mut(s) {
                        if let Some(last) = list.last_mut() {
                            last.end = t_val;
                        }
                    }
                }
            }
            for s in &turn_spkrs {
                utterances
                    .entry(s.clone())
                    .or_default()
                    .push(Utterance { start: t_val, text: String::new(), end: 0.0 });
            }
            utt_needs_end = true;
            continue;
        }

        if let Some(caps) = re_who.captures(line) {
            let nb: usize = caps[1].parse().map_err(|_| format!("nb de Who invalide : {}", &caps[1]))?;
            if nb >= 1 {
                let idx = nb - 1;
                if idx < turn_spkrs.len() {
                    cur_spkr = turn_spkrs[idx].clone();
                }
            }
            continue;
        }

        if !line.starts_with('<') {
            let text = line.replace('"', "'");
            if cur_spkr.is_empty() {
                cur_spkr = active_spkrs.clone();
            }
            if !text.is_empty() {
                if let Some(list) = utterances.get_mut(&cur_spkr) {
                    if let Some(last) = list.last_mut() {
                        last.text.push_str(&text);
                        last.text.push(' ');
                    }
                }
            }
        }
    }

    // Comblement par des silences (ordre d'apparition des locuteurs dans le fichier).
    for id in &speaker_order {
        let mut last_time = start_time;
        let mut new_tier = Vec::new();
        if let Some(utts) = utterances.get(id) {
            for utt in utts {
                if last_time < utt.start {
                    new_tier.push(Interval { start: last_time, end: utt.start, text: String::new() });
                }
                new_tier.push(Interval { start: utt.start, end: utt.end, text: utt.text.trim().to_string() });
                last_time = utt.end;
            }
        }
        if last_time < end_time {
            new_tier.push(Interval { start: last_time, end: end_time, text: String::new() });
        }
        tiers.insert(id.clone(), new_tier);
    }

    // Écriture du TextGrid (locuteurs triés par id, comme sorted(speakers.keys()) en Python).
    let mut sorted_ids = speaker_order.clone();
    sorted_ids.sort();

    let mut out = String::new();
    out.push_str("File type = \"ooTextFile\"\nObject class = \"TextGrid\"\n\n");
    out.push_str(&format!(
        "xmin = {:?}\nxmax = {:?}\ntiers? <exists>\nsize = {}\nitem []:\n",
        start_time, end_time, speaker_order.len()
    ));

    let empty_vec: Vec<Interval> = Vec::new();
    for (i, id) in sorted_ids.iter().enumerate() {
        let name = speaker_names.get(id).cloned().unwrap_or_default();
        let intervals = tiers.get(id).unwrap_or(&empty_vec);
        out.push_str(&format!("    item [{}]:\n        class = \"IntervalTier\"\n", i + 1));
        out.push_str(&format!(
            "        name = \"{}\"\n        xmin = {:?}\n        xmax = {:?}\n",
            name, start_time, end_time
        ));
        out.push_str(&format!("        intervals: size = {}\n", intervals.len()));
        for (j, interval) in intervals.iter().enumerate() {
            out.push_str(&format!("        intervals [{}]:\n            xmin = {:?}\n", j + 1, interval.start));
            out.push_str(&format!("            xmax = {:?}\n            text = \"{}\"\n", interval.end, interval.text));
        }
    }

    fs::write(out_path, out).map_err(|e| e.to_string())
}
