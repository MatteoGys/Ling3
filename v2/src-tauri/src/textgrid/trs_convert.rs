//! trs_convert.rs — Conversion d'un fichier Transcriber (.trs / .trs.xml)
//! en TextGrid Praat : une tier par locuteur, silences comblés, avec
//! vérification et correction des bornes (logique réutilisable dans
//! tg_check.rs).
//!
//! Étapes, dans l'ordre :
//!   1. lecture du TRS ........ `parse_trs()`  → données brutes ; erreurs précises (n° de ligne)
//!   2. construction ........... `build_doc()`  → un `Doc` (tiers + intervalles, silences comblés)
//!   3. corrections ............ `tg_check::normalize()`
//!   4. rendu du texte ......... `tg_check::render()`
//!   5. auto-vérification ...... relecture du texte produit (mêmes contrôles)
//!   6. problèmes restants ..... `tg_check::check()` → .TextGrid si aucun, sinon .txt
//!   7. écriture sur le disque.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;

use super::tg_check::{self, Doc, Interval, Tier};

/// Nombre maximum de messages affichés par fichier (le reste est résumé).
const MAX_MESSAGES: usize = 10;

// ═══════════════════════════════════════════════════════════════════════
// Rapport renvoyé à l'interface
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct ReportLine {
    pub text: String,
    /// true : à afficher comme une erreur (rouge) ; false : information.
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrsFileReport {
    /// Un fichier a bien été écrit (.TextGrid, ou .txt si des bornes restent à corriger).
    pub created: bool,
    pub headline: String,
    pub lines: Vec<ReportLine>,
}

// ═══════════════════════════════════════════════════════════════════════
// 1. Lecture du TRS
// ═══════════════════════════════════════════════════════════════════════

struct Utterance {
    start: f64,
    text: String,
    end: f64,
}

struct TrsData {
    speaker_order: Vec<String>,
    speaker_names: HashMap<String, String>,
    /// Nombre de locuteurs déclarés dans <Speakers> (= l'ancien « size »).
    declared_speakers: usize,
    utterances: HashMap<String, Vec<Utterance>>,
    sections_min: Option<f64>,
    sections_max: Option<f64>,
    /// Plus grand timecode rencontré dans <Turn>, <Sync> ou fin d'énoncé.
    content_max: f64,
    /// Informations à signaler à l'utilisateur (locuteur ajouté, etc.).
    notes: Vec<String>,
}

/// Lit un timecode en tolérant les défauts courants, sans les signaler :
/// espaces autour (y compris insécables), caractères invisibles, virgule
/// décimale. Refuse ce qui n'est pas un nombre fini positif.
fn parse_timecode(raw: &str) -> Option<f64> {
    let cleaned: String = raw
        .chars()
        .filter(|c| !matches!(c, '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'))
        .collect::<String>()
        .trim()
        .replace(',', ".");
    let v: f64 = cleaned.parse().ok()?;
    if v.is_finite() && v >= 0.0 {
        Some(v)
    } else {
        None
    }
}

/// Lit l'attribut `key` comme un timecode. Absent ou vide : None (et une
/// erreur si `required`). Présent mais illisible : erreur précise.
fn read_time(
    attrs: &HashMap<String, String>,
    key: &str,
    tag: &str,
    line_no: usize,
    required: bool,
    errors: &mut Vec<String>,
) -> Option<f64> {
    let raw = attrs.get(key).map(|s| s.as_str()).unwrap_or("");
    if raw.trim().is_empty() {
        if required {
            errors.push(format!("Ligne {} : <{}> : attribut « {} » manquant ou vide", line_no, tag, key));
        }
        return None;
    }
    match parse_timecode(raw) {
        Some(v) => Some(v),
        None => {
            errors.push(format!(
                "Ligne {} : <{}> : attribut « {} » invalide — valeur « {} » (attendu : un nombre de secondes, ex. 918.997)",
                line_no,
                tag,
                key,
                raw.escape_debug()
            ));
            None
        }
    }
}

/// Vrai si la ligne contient la balise ouvrante `needle` (ex. "<Sync") suivie
/// d'un espace, de "/" ou de ">" — pour reconnaître `<Sync/>` sans confondre
/// `<Speaker` avec `<Speakers>`.
fn has_tag(line: &str, needle: &str) -> bool {
    line.match_indices(needle).any(|(i, _)| {
        line[i + needle.len()..]
            .chars()
            .next()
            .map_or(true, |c| c.is_whitespace() || c == '/' || c == '>')
    })
}

fn close_last_utterance(utterances: &mut HashMap<String, Vec<Utterance>>, speakers: &[String], end: f64) {
    for s in speakers {
        if let Some(list) = utterances.get_mut(s) {
            if let Some(last) = list.last_mut() {
                last.end = end;
            }
        }
    }
}

fn parse_trs(content: &str) -> Result<TrsData, Vec<String>> {
    let re_attr = Regex::new(r#"([A-Za-z_][\w:.\-]*)\s*=\s*"([^"]*)""#).expect("regex constante valide");
    let attrs_of = |line: &str| -> HashMap<String, String> {
        re_attr
            .captures_iter(line)
            .map(|c| (c[1].to_string(), c[2].to_string()))
            .collect()
    };

    let mut data = TrsData {
        speaker_order: Vec::new(),
        speaker_names: HashMap::new(),
        declared_speakers: 0,
        utterances: HashMap::new(),
        sections_min: None,
        sections_max: None,
        content_max: 0.0,
        notes: Vec::new(),
    };
    let mut errors: Vec<String> = Vec::new();

    let mut in_turn = false;
    let mut utt_needs_end = false;
    let mut turn_end = 0.0_f64;
    let mut cur_spkr = String::new();
    let mut turn_spkrs: Vec<String> = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let ln = idx + 1;
        let line = raw_line.trim();

        // ── <Speaker id="…" name="…"/> ───────────────────────────────
        if has_tag(line, "<Speaker") {
            let a = attrs_of(line);
            match a.get("id").map(|s| s.trim()).filter(|s| !s.is_empty()) {
                Some(id) => {
                    let id = id.to_string();
                    let name = a
                        .get("name")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| id.clone());
                    if !data.speaker_names.contains_key(&id) {
                        data.speaker_order.push(id.clone());
                        data.declared_speakers += 1;
                    }
                    data.speaker_names.insert(id.clone(), name);
                    data.utterances.entry(id).or_default();
                }
                None => errors.push(format!("Ligne {} : <Speaker> : attribut « id » manquant ou vide", ln)),
            }
            continue;
        }

        // ── <Section startTime="…" endTime="…"> ──────────────────────
        if has_tag(line, "<Section") {
            let a = attrs_of(line);
            if let Some(s) = read_time(&a, "startTime", "Section", ln, false, &mut errors) {
                data.sections_min = Some(data.sections_min.map_or(s, |m| m.min(s)));
            }
            if let Some(e) = read_time(&a, "endTime", "Section", ln, false, &mut errors) {
                data.sections_max = Some(data.sections_max.map_or(e, |m| m.max(e)));
                data.content_max = data.content_max.max(e);
            }
            continue;
        }

        // ── <Turn speaker="…" startTime="…" endTime="…"> ─────────────
        if has_tag(line, "<Turn") {
            let a = attrs_of(line);
            let speakers_attr = a.get("speaker").map(|s| s.trim().to_string()).unwrap_or_default();
            let has_speaker = !speakers_attr.is_empty();

            let start = read_time(&a, "startTime", "Turn", ln, false, &mut errors);
            // Le endTime n'est indispensable que si quelqu'un parle dans ce Turn.
            let end = read_time(&a, "endTime", "Turn", ln, has_speaker, &mut errors);
            if let Some(s) = start {
                data.content_max = data.content_max.max(s);
            }
            if let Some(e) = end {
                data.content_max = data.content_max.max(e);
            }

            let Some(end) = end.filter(|_| has_speaker) else {
                // Turn sans locuteur (silence, bruit…) ou endTime inutilisable
                // (l'erreur éventuelle a déjà été notée) : rien à transcrire.
                in_turn = false;
                continue;
            };

            in_turn = true;
            turn_end = end;
            turn_spkrs = speakers_attr.split_whitespace().map(|s| s.to_string()).collect();
            cur_spkr = turn_spkrs.first().cloned().unwrap_or_default();

            // Locuteur présent dans un Turn mais absent de <Speakers> :
            // on lui crée une tier plutôt que de perdre ses paroles.
            for spkr in &turn_spkrs {
                if !data.speaker_names.contains_key(spkr) {
                    data.speaker_order.push(spkr.clone());
                    data.speaker_names.insert(spkr.clone(), spkr.clone());
                    data.utterances.entry(spkr.clone()).or_default();
                    data.notes.push(format!(
                        "Locuteur « {} » trouvé dans un <Turn> (ligne {}) mais absent de <Speakers> : tier ajoutée",
                        spkr, ln
                    ));
                }
            }
            continue;
        }

        // ── </Turn> ──────────────────────────────────────────────────
        if line.contains("</Turn>") {
            if utt_needs_end {
                close_last_utterance(&mut data.utterances, &turn_spkrs, turn_end);
                utt_needs_end = false;
            }
            in_turn = false;
            continue;
        }

        if !in_turn {
            continue;
        }

        // ── <Sync time="…"/> : début d'un nouvel énoncé ──────────────
        if has_tag(line, "<Sync") {
            let a = attrs_of(line);
            let Some(t) = read_time(&a, "time", "Sync", ln, true, &mut errors) else {
                continue;
            };
            data.content_max = data.content_max.max(t);

            if utt_needs_end {
                close_last_utterance(&mut data.utterances, &turn_spkrs, t);
            }
            for s in &turn_spkrs {
                data.utterances
                    .entry(s.clone())
                    .or_default()
                    .push(Utterance { start: t, text: String::new(), end: 0.0 });
            }
            utt_needs_end = true;
            continue;
        }

        // ── <Who nb="N"/> : locuteur courant dans un Turn à plusieurs ─
        if has_tag(line, "<Who") {
            let a = attrs_of(line);
            let raw = a.get("nb").map(|s| s.trim()).unwrap_or("");
            match raw.parse::<usize>() {
                Ok(nb) if nb >= 1 => {
                    if let Some(s) = turn_spkrs.get(nb - 1) {
                        cur_spkr = s.clone();
                    }
                }
                _ => errors.push(format!(
                    "Ligne {} : <Who> : attribut « nb » invalide — valeur « {} » (attendu : 1, 2, …)",
                    ln,
                    raw.escape_debug()
                )),
            }
            continue;
        }

        // ── Texte transcrit ──────────────────────────────────────────
        if !line.starts_with('<') {
            let text = line.replace('"', "'");
            if !text.is_empty() {
                if let Some(list) = data.utterances.get_mut(&cur_spkr) {
                    if let Some(last) = list.last_mut() {
                        if !last.text.is_empty() {
                            last.text.push(' ');
                        }
                        last.text.push_str(&text);
                    }
                }
            }
        }
    }

    if errors.is_empty() && data.speaker_order.is_empty() {
        errors.push("Aucun locuteur trouvé (ni <Speaker>, ni <Turn speaker=\"…\">)".to_string());
    }
    if errors.is_empty() && data.sections_max.is_none() && data.content_max <= 0.0 {
        errors.push("Aucune borne temporelle exploitable (ni <Section>, ni <Turn>, ni <Sync>)".to_string());
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    if data.sections_max.is_none() {
        data.notes.push(
            "Aucune balise <Section> avec bornes : bornes du TextGrid déduites des <Turn> et <Sync>".to_string(),
        );
    }
    Ok(data)
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Construction du Doc
// ═══════════════════════════════════════════════════════════════════════

/// Une tier par locuteur (triées par identifiant, comme avant), silences
/// comblés entre les énoncés. Les bornes annoncées sont celles du TRS
/// (<Section>, sinon déduites du contenu) : c'est `normalize()` qui les
/// confronte à ce qu'on trouve réellement dans les intervalles.
fn build_doc(data: &TrsData) -> Doc {
    let declared_xmin = data.sections_min.unwrap_or(0.0);
    let declared_xmax = data.sections_max.unwrap_or(data.content_max);

    let mut ids = data.speaker_order.clone();
    ids.sort();

    let tiers: Vec<Tier> = ids
        .iter()
        .map(|id| {
            let mut intervals: Vec<Interval> = Vec::new();
            let mut last_time = 0.0_f64; // le TextGrid commence toujours à 0.0

            if let Some(utts) = data.utterances.get(id) {
                for utt in utts {
                    if last_time < utt.start {
                        intervals.push(Interval { xmin: last_time, xmax: utt.start, text: String::new() });
                    }
                    intervals.push(Interval { xmin: utt.start, xmax: utt.end, text: utt.text.trim().to_string() });
                    last_time = utt.end;
                }
            }
            if last_time < declared_xmax {
                intervals.push(Interval { xmin: last_time, xmax: declared_xmax, text: String::new() });
            }

            Tier {
                name: data.speaker_names.get(id).cloned().unwrap_or_else(|| id.clone()),
                xmin: declared_xmin,
                xmax: declared_xmax,
                declared_size: None,
                intervals,
            }
        })
        .collect();

    Doc { xmin: declared_xmin, xmax: declared_xmax, declared_tiers: Some(data.declared_speakers), tiers }
}

// ═══════════════════════════════════════════════════════════════════════
// 3-7. Conversion d'un fichier
// ═══════════════════════════════════════════════════════════════════════

/// Décode un fichier texte sans perdre les accents : UTF-8 (avec ou sans
/// BOM), UTF-16 (avec BOM), sinon Windows-1252 (≈ ISO-8859-1).
fn decode_trs(raw: &[u8]) -> String {
    if let Some(rest) = raw.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        if let Ok(s) = std::str::from_utf8(rest) {
            return s.to_string();
        }
    }
    if let Some(rest) = raw.strip_prefix(&[0xFF, 0xFE]) {
        return encoding_rs::UTF_16LE.decode(rest).0.into_owned();
    }
    if let Some(rest) = raw.strip_prefix(&[0xFE, 0xFF]) {
        return encoding_rs::UTF_16BE.decode(rest).0.into_owned();
    }
    match std::str::from_utf8(raw) {
        Ok(s) => s.to_string(),
        Err(_) => encoding_rs::WINDOWS_1252.decode(raw).0.into_owned(),
    }
}

pub struct ConversionOutcome {
    pub output_path: PathBuf,
    /// Corrections appliquées automatiquement.
    pub corrections: Vec<String>,
    /// Problèmes restants (non corrigeables) : si non vide, la sortie est un .txt.
    pub problems: Vec<String>,
}

/// Convertit un TRS. Erreur : liste des raisons précises (avec n° de ligne
/// du TRS quand c'est pertinent). Succès : le fichier est écrit dans
/// `out_dir` — en `.TextGrid`, ou en `.txt` si des bornes restent
/// incohérentes (pour pouvoir les corriger à la main dans un éditeur).
pub fn convert_trs_to_textgrid(
    trs_path: &str,
    out_dir: &Path,
    base_name: &str,
) -> Result<ConversionOutcome, Vec<String>> {
    let raw = fs::read(trs_path).map_err(|e| vec![format!("Impossible de lire le fichier : {}", e)])?;
    let content = decode_trs(&raw);

    let data = parse_trs(&content)?;
    let mut corrections: Vec<String> = data.notes.clone();
    let mut doc = build_doc(&data);

    corrections.extend(tg_check::normalize(&mut doc));
    let mut rendered = tg_check::render(&doc);
    let mut problems: Vec<String> = Vec::new();

    // Auto-vérification : on relit le texte produit et on refait les mêmes
    // contrôles (nombre de tiers/d'intervalles annoncés, bornes). Aucun
    // écart n'est attendu ; s'il y en a un, il est corrigé et signalé.
    match tg_check::parse_text(&rendered.text) {
        Ok(mut reread) => {
            let extra = tg_check::normalize(&mut reread);
            if !extra.is_empty() {
                corrections.extend(extra.into_iter().map(|c| format!("(auto-vérification) {}", c)));
                doc = reread;
                rendered = tg_check::render(&doc);
            }
        }
        Err(e) => problems.push(format!("Auto-vérification impossible : {}", e)),
    }

    problems.extend(tg_check::check(&doc, &rendered));

    let extension = if problems.is_empty() { "TextGrid" } else { "txt" };
    let output_path = out_dir.join(format!("{}.{}", base_name, extension));
    fs::write(&output_path, &rendered.text)
        .map_err(|e| vec![format!("Impossible d'écrire {} : {}", output_path.display(), e)])?;

    Ok(ConversionOutcome { output_path, corrections, problems })
}

/// Limite le nombre de messages affichés, en résumant le reste.
fn cap(mut v: Vec<String>) -> Vec<String> {
    if v.len() > MAX_MESSAGES {
        let extra = v.len() - MAX_MESSAGES;
        v.truncate(MAX_MESSAGES);
        v.push(format!("… et {} autre(s) non affiché(s)", extra));
    }
    v
}

/// Convertit un fichier et prépare ce que l'interface affichera pour lui.
pub fn convert_file_to_report(trs_path: &str, desktop: Option<&Path>) -> TrsFileReport {
    let path = Path::new(trs_path);
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trs_path)
        .to_string();

    let fail = |reasons: Vec<String>| TrsFileReport {
        created: false,
        headline: format!("Échec de la conversion : {}", file_name),
        lines: reasons.into_iter().map(|text| ReportLine { text, is_error: true }).collect(),
    };

    if trs_path.is_empty() || !path.exists() {
        return fail(vec![format!("Fichier introuvable : {}", trs_path)]);
    }
    let Some(desktop) = desktop else {
        return fail(vec!["Impossible de localiser le Bureau.".to_string()]);
    };

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let base_name = stem.strip_suffix(".trs").unwrap_or(stem);

    match convert_trs_to_textgrid(trs_path, desktop, base_name) {
        Err(reasons) => fail(cap(reasons)),
        Ok(outcome) => {
            let out_name = outcome
                .output_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let mut lines: Vec<ReportLine> = Vec::new();
            let headline = if outcome.problems.is_empty() {
                format!("Conversion réussie ! Enregistré sur le bureau : {}", out_name)
            } else {
                lines.push(ReportLine {
                    text: "Enregistré en .txt (illisible dans Praat tant que ces erreurs persistent) : corrigez les lignes indiquées, puis renommez le fichier en .TextGrid.".to_string(),
                    is_error: false,
                });
                for p in cap(outcome.problems) {
                    lines.push(ReportLine { text: p, is_error: true });
                }
                format!("Fichier créé, mais des bornes sont à corriger à la main. Enregistré sur le bureau : {}", out_name)
            };
            for c in cap(outcome.corrections) {
                lines.push(ReportLine { text: c, is_error: false });
            }

            TrsFileReport { created: true, headline, lines }
        }
    }
}
