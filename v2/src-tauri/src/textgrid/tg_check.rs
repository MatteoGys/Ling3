//! tg_check.rs — Modèle mémoire d'un TextGrid (format long de Praat) avec
//! vérifications et corrections réutilisables. Aucune dépendance au format
//! TRS : n'importe quelle source (TRS aujourd'hui, autre chose demain) peut
//! construire un `Doc`, puis appeler les fonctions ci-dessous dans l'ordre :
//!
//!   1. `normalize()`  — corrige automatiquement (vérifications 1.1 à 1.4) et
//!                       renvoie la liste des corrections faites ;
//!   2. `render()`     — produit le texte du fichier, en notant le numéro de
//!                       ligne de chaque intervalle ;
//!   3. `check()`      — signale ce qui ne peut PAS être corrigé
//!                       automatiquement (vérification 2 : continuité) ;
//!   4. `parse_text()` — relit un texte TextGrid (sert à l'auto-vérification
//!                       du fichier produit, et à réutiliser ces contrôles sur
//!                       un TextGrid existant).

/// Tolérance de comparaison entre deux timecodes (1 microseconde).
pub const EPS: f64 = 1e-6;

#[derive(Debug, Clone)]
pub struct Interval {
    pub xmin: f64,
    pub xmax: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Tier {
    pub name: String,
    pub xmin: f64,
    pub xmax: f64,
    /// Valeur annoncée par « intervals: size = N » (None : rien à comparer).
    pub declared_size: Option<usize>,
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Clone)]
pub struct Doc {
    pub xmin: f64,
    pub xmax: f64,
    /// Valeur annoncée par « size = N » dans l'en-tête global
    /// (None : rien à comparer).
    pub declared_tiers: Option<usize>,
    pub tiers: Vec<Tier>,
}

/// Texte produit + position de chaque intervalle dans ce texte.
pub struct Rendered {
    pub text: String,
    /// Pour chaque tier, le numéro de ligne (à partir de 1) de chaque
    /// ligne « intervals [j]: ». Le xmin est sur la ligne suivante, le xmax
    /// deux lignes plus bas, le texte trois lignes plus bas.
    pub interval_lines: Vec<Vec<usize>>,
}

fn num(v: f64) -> String {
    format!("{:?}", v)
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// Début du texte d'un intervalle, pour le retrouver facilement.
fn snippet(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        return "(vide)".to_string();
    }
    let short: String = t.chars().take(40).collect();
    if t.chars().count() > 40 {
        format!("{}…", short)
    } else {
        short
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Intervalles vides parasites
// ═══════════════════════════════════════════════════════════════════════

/// Nettoie les intervalles vides d'une tier et renvoie un message par
/// intervalle parasite supprimé :
///   • durée nulle ou négative → retiré SANS rien signaler ;
///   • intervalle vide, pas le premier, dont les voisins se rejoignent déjà
///     (xmax du précédent == xmin du suivant) → parasite, retiré. C'est la
///     preuve qu'on ne perd rien : la chaîne reste continue sans lui, quelles
///     que soient ses propres bornes (typiquement [0.0 ; t] créé par un
///     <Sync time="0"/> égaré dans un TRS) ;
///   • dernier intervalle vide qui recommence à 0.0 (impossible ailleurs
///     qu'en tête) → parasite, retiré.
/// Un intervalle qui porte du texte n'est JAMAIS supprimé ici. Les numéros
/// des intervalles suivants se décalent d'eux-mêmes : `render()` les recalcule
/// ainsi que « intervals: size ».
fn clean_empty_intervals(tier: &mut Tier) {
    tier
        .intervals
        .retain(|iv| !(is_blank(&iv.text) && iv.xmax - iv.xmin <= EPS));

    // Intervalles du milieu : on repart en arrière après chaque suppression,
    // car les voisins ont changé (cas de plusieurs parasites d'affilée).
    let mut j = 1;
    while j + 1 < tier.intervals.len() {
        let neighbours_touch = (tier.intervals[j - 1].xmax - tier.intervals[j + 1].xmin).abs() <= EPS;
        if is_blank(&tier.intervals[j].text) && neighbours_touch {
            tier.intervals.remove(j);
            j = j.saturating_sub(1).max(1);
        } else {
            j += 1;
        }
    }

    // Dernier intervalle vide qui recommence à 0.0.
    while tier.intervals.len() >= 2 {
        let last = tier.intervals.len() - 1;
        if is_blank(&tier.intervals[last].text) && tier.intervals[last].xmin.abs() <= EPS {
            tier.intervals.pop();
        } else {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Corrections automatiques (vérifications 1.1 à 1.4)
// ═══════════════════════════════════════════════════════════════════════

/// Applique les corrections 1.1 à 1.4 et renvoie ce qui a été corrigé
/// (dans l'ordre 1.1, 1.2, 1.3, 1.4). Les intervalles vides de durée nulle
/// ou négative sont retirés sans bruit (rien n'est signalé).
pub fn normalize(doc: &mut Doc) -> Vec<String> {
    let mut n11: Vec<String> = Vec::new();
    let mut n12: Vec<String> = Vec::new();
    let mut n13: Vec<String> = Vec::new();
    let mut n14: Vec<String> = Vec::new();

    // ── 1.1 Nombre de tiers : « size = X » doit valoir le nombre réel ────
    if let Some(declared) = doc.declared_tiers {
        if declared != doc.tiers.len() {
            n11.push(format!(
                "Nombre de tiers : « size = {} » ne correspondait pas au nombre réel de tiers ({}) — corrigé en « size = {} »",
                declared,
                doc.tiers.len(),
                doc.tiers.len()
            ));
        }
    }

    // ── 1.3 Nombre d'intervalles : « intervals: size = X » doit valoir le
    // nombre réel. Comparé AVANT tout retrait/ajout d'intervalle, pour ne
    // signaler que l'écart présent dans la source. ──────────────────────
    for tier in &doc.tiers {
        if let Some(declared) = tier.declared_size {
            if declared != tier.intervals.len() {
                n13.push(format!(
                    "Tier « {} » : « intervals: size = {} » ne correspondait pas au nombre réel d'intervalles ({}) — corrigé en « intervals: size = {} »",
                    tier.name,
                    declared,
                    tier.intervals.len(),
                    tier.intervals.len()
                ));
            }
        }
    }

    // ── Intervalles vides de durée nulle ou négative : retirés, sans bruit.
    for tier in &mut doc.tiers {
        clean_empty_intervals(tier);
    }

    // ── 1.2 Bornes globales : xmin toujours 0.0, xmax = la plus grande
    // valeur trouvée (global, tiers, derniers intervalles), partout. ─────
    if doc.xmin.abs() > EPS {
        n12.push(format!("xmin global : {} ramené à 0.0", num(doc.xmin)));
    }
    doc.xmin = 0.0;
    let mut xmin_fixed: Vec<String> = Vec::new();
    for tier in &mut doc.tiers {
        if tier.xmin.abs() > EPS {
            xmin_fixed.push(format!("« {} »", tier.name));
        }
        tier.xmin = 0.0;
    }
    if !xmin_fixed.is_empty() {
        n12.push(format!("xmin ramené à 0.0 pour les tiers : {}", xmin_fixed.join(", ")));
    }

    let mut max_val = doc.xmax;
    let mut max_src = "l'en-tête global".to_string();
    for tier in &doc.tiers {
        if tier.xmax > max_val + EPS {
            max_val = tier.xmax;
            max_src = format!("l'en-tête de la tier « {} »", tier.name);
        }
        if let Some(last) = tier.intervals.last() {
            if last.xmax > max_val + EPS {
                max_val = last.xmax;
                max_src = format!("le dernier intervalle de la tier « {} »", tier.name);
            }
        }
    }
    if (doc.xmax - max_val).abs() > EPS {
        n12.push(format!(
            "xmax global : {} → {} (plus grande valeur trouvée : {})",
            num(doc.xmax),
            num(max_val),
            max_src
        ));
    }
    doc.xmax = max_val;
    let mut xmax_fixed: Vec<String> = Vec::new();
    for tier in &mut doc.tiers {
        if (tier.xmax - max_val).abs() > EPS {
            xmax_fixed.push(format!("« {} »", tier.name));
        }
        tier.xmax = max_val;
    }
    if !xmax_fixed.is_empty() {
        n12.push(format!(
            "xmax aligné sur le xmax global ({}) pour les tiers : {}",
            num(max_val),
            xmax_fixed.join(", ")
        ));
    }

    // ── 1.4 Bords de chaque tier : le premier intervalle commence à 0.0,
    // le dernier finit au xmax global. Vide → étiré ; avec du texte →
    // un intervalle vide est inséré pour combler. ────────────────────────
    let x = doc.xmax;
    let mut last_stretched: Vec<String> = Vec::new();
    for tier in &mut doc.tiers {
        if tier.intervals.is_empty() {
            if x > EPS {
                tier.intervals.push(Interval { xmin: 0.0, xmax: x, text: String::new() });
                n14.push(format!(
                    "Tier « {} » : aucun intervalle — intervalle vide créé de 0.0 à {}",
                    tier.name,
                    num(x)
                ));
            }
            continue;
        }

        let first_xmin = tier.intervals[0].xmin;
        if first_xmin.abs() > EPS {
            if first_xmin < 0.0 || is_blank(&tier.intervals[0].text) {
                tier.intervals[0].xmin = 0.0;
                n14.push(format!(
                    "Tier « {} » : intervals [1] commençait à {} — ramené à 0.0",
                    tier.name,
                    num(first_xmin)
                ));
            } else {
                tier.intervals.insert(0, Interval { xmin: 0.0, xmax: first_xmin, text: String::new() });
                n14.push(format!(
                    "Tier « {} » : intervals [1] commençait à {} avec du texte — intervalle vide ajouté de 0.0 à {}",
                    tier.name,
                    num(first_xmin),
                    num(first_xmin)
                ));
            }
        }

        let last_idx = tier.intervals.len() - 1;
        let last_xmax = tier.intervals[last_idx].xmax;
        if last_xmax < x - EPS {
            if is_blank(&tier.intervals[last_idx].text) {
                tier.intervals[last_idx].xmax = x;
                last_stretched.push(format!("« {} »", tier.name));
            } else {
                tier.intervals.push(Interval { xmin: last_xmax, xmax: x, text: String::new() });
                n14.push(format!(
                    "Tier « {} » : le dernier intervalle (intervals [{}]) finissait à {} avec du texte — intervalle vide ajouté de {} à {}",
                    tier.name,
                    last_idx + 1,
                    num(last_xmax),
                    num(last_xmax),
                    num(x)
                ));
            }
        }
    }
    if !last_stretched.is_empty() {
        n14.push(format!(
            "Dernier intervalle vide étiré jusqu'à {} pour les tiers : {}",
            num(x),
            last_stretched.join(", ")
        ));
    }

    // Les deux « size » seront désormais écrits d'après les nombres réels.
    doc.declared_tiers = Some(doc.tiers.len());
    for tier in &mut doc.tiers {
        tier.declared_size = Some(tier.intervals.len());
    }

    let mut all = n11;
    all.extend(n12);
    all.extend(n13);
    all.extend(n14);
    all
}

// ═══════════════════════════════════════════════════════════════════════
// Rendu du texte
// ═══════════════════════════════════════════════════════════════════════

/// Produit le texte du TextGrid. Les deux « size » sont toujours écrits
/// d'après les nombres réels de tiers/d'intervalles.
pub fn render(doc: &Doc) -> Rendered {
    let mut lines: Vec<String> = Vec::new();
    let mut interval_lines: Vec<Vec<usize>> = Vec::new();

    lines.push("File type = \"ooTextFile\"".to_string());
    lines.push("Object class = \"TextGrid\"".to_string());
    lines.push(String::new());
    lines.push(format!("xmin = {}", num(doc.xmin)));
    lines.push(format!("xmax = {}", num(doc.xmax)));
    lines.push("tiers? <exists>".to_string());
    lines.push(format!("size = {}", doc.tiers.len()));
    lines.push("item []:".to_string());

    for (i, tier) in doc.tiers.iter().enumerate() {
        lines.push(format!("    item [{}]:", i + 1));
        lines.push("        class = \"IntervalTier\"".to_string());
        lines.push(format!("        name = \"{}\"", tier.name));
        lines.push(format!("        xmin = {}", num(tier.xmin)));
        lines.push(format!("        xmax = {}", num(tier.xmax)));
        lines.push(format!("        intervals: size = {}", tier.intervals.len()));

        let mut tier_lines = Vec::with_capacity(tier.intervals.len());
        for (j, iv) in tier.intervals.iter().enumerate() {
            lines.push(format!("        intervals [{}]:", j + 1));
            tier_lines.push(lines.len()); // numéro de ligne (à partir de 1) de la ligne qu'on vient d'ajouter
            lines.push(format!("            xmin = {}", num(iv.xmin)));
            lines.push(format!("            xmax = {}", num(iv.xmax)));
            lines.push(format!("            text = \"{}\"", iv.text));
        }
        interval_lines.push(tier_lines);
    }

    let mut text = lines.join("\n");
    text.push('\n');
    Rendered { text, interval_lines }
}

// ═══════════════════════════════════════════════════════════════════════
// Problèmes non corrigeables (vérification 2)
// ═══════════════════════════════════════════════════════════════════════

/// Repère ce qui ne peut pas être corrigé automatiquement : intervalle de
/// durée nulle/négative (avec du texte) et rupture de continuité au sein
/// d'une même tier (le xmax de l'intervalle N doit valoir le xmin de N+1).
/// Chaque message donne la tier, le numéro d'intervalle et les numéros de
/// ligne à retoucher dans le texte produit par `render()`.
pub fn check(doc: &Doc, rendered: &Rendered) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();

    for (ti, tier) in doc.tiers.iter().enumerate() {
        let lines = &rendered.interval_lines[ti];
        for (j, iv) in tier.intervals.iter().enumerate() {
            let line = lines.get(j).copied().unwrap_or(0);

            if iv.xmax - iv.xmin <= EPS {
                problems.push(format!(
                    "Tier « {} » · intervals [{}] : xmin = {} (ligne {}), xmax = {} (ligne {}) — durée nulle ou négative — texte : « {} »",
                    tier.name,
                    j + 1,
                    num(iv.xmin),
                    line + 1,
                    num(iv.xmax),
                    line + 2,
                    snippet(&iv.text)
                ));
            }

            if j > 0 {
                let prev = &tier.intervals[j - 1];
                let prev_line = lines.get(j - 1).copied().unwrap_or(0);
                let diff = prev.xmax - iv.xmin;
                if diff.abs() > EPS {
                    let kind = if diff > 0.0 { "chevauchement" } else { "trou" };
                    problems.push(format!(
                        "Tier « {} » · intervals [{}] : xmin = {} (ligne {}) mais l'intervalle précédent (intervals [{}]) se termine à xmax = {} (ligne {}) — {} de {:.3} s — texte : « {} »",
                        tier.name,
                        j + 1,
                        num(iv.xmin),
                        line + 1,
                        j,
                        num(prev.xmax),
                        prev_line + 2,
                        kind,
                        diff.abs(),
                        snippet(&iv.text)
                    ));
                }
            }
        }
    }

    problems
}

// ═══════════════════════════════════════════════════════════════════════
// Relecture d'un texte TextGrid (format long)
// ═══════════════════════════════════════════════════════════════════════

fn value_after<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    line.strip_prefix(prefix).map(|v| v.trim())
}

fn parse_f64(raw: &str, line_no: usize) -> Result<f64, String> {
    raw.trim()
        .parse::<f64>()
        .map_err(|_| format!("Ligne {} : nombre invalide « {} »", line_no, raw))
}

fn parse_usize(raw: &str, line_no: usize) -> Result<usize, String> {
    raw.trim()
        .parse::<usize>()
        .map_err(|_| format!("Ligne {} : entier invalide « {} »", line_no, raw))
}

/// Relit un texte TextGrid au format long, en conservant les valeurs
/// ANNONCÉES (« size = », « intervals: size = ») pour pouvoir les comparer
/// aux nombres réels dans `normalize()`.
pub fn parse_text(text: &str) -> Result<Doc, String> {
    let mut doc = Doc { xmin: 0.0, xmax: 0.0, declared_tiers: None, tiers: Vec::new() };
    let mut cur_tier: Option<usize> = None;
    let mut in_interval = false;

    for (idx, raw) in text.lines().enumerate() {
        let ln = idx + 1;
        let line = raw.trim();

        if line.starts_with("item [") && !line.starts_with("item []") {
            doc.tiers.push(Tier {
                name: String::new(),
                xmin: 0.0,
                xmax: 0.0,
                declared_size: None,
                intervals: Vec::new(),
            });
            cur_tier = Some(doc.tiers.len() - 1);
            in_interval = false;
            continue;
        }

        match cur_tier {
            None => {
                if let Some(v) = value_after(line, "xmin =") {
                    doc.xmin = parse_f64(v, ln)?;
                } else if let Some(v) = value_after(line, "xmax =") {
                    doc.xmax = parse_f64(v, ln)?;
                } else if let Some(v) = value_after(line, "size =") {
                    doc.declared_tiers = Some(parse_usize(v, ln)?);
                }
            }
            Some(t) => {
                let tier = &mut doc.tiers[t];
                if line.starts_with("intervals [") {
                    tier.intervals.push(Interval { xmin: 0.0, xmax: 0.0, text: String::new() });
                    in_interval = true;
                } else if let Some(v) = value_after(line, "intervals: size =") {
                    tier.declared_size = Some(parse_usize(v, ln)?);
                } else if !in_interval {
                    if let Some(v) = value_after(line, "name =") {
                        tier.name = v.trim_matches('"').to_string();
                    } else if let Some(v) = value_after(line, "xmin =") {
                        tier.xmin = parse_f64(v, ln)?;
                    } else if let Some(v) = value_after(line, "xmax =") {
                        tier.xmax = parse_f64(v, ln)?;
                    }
                } else if let Some(iv) = tier.intervals.last_mut() {
                    if let Some(v) = value_after(line, "xmin =") {
                        iv.xmin = parse_f64(v, ln)?;
                    } else if let Some(v) = value_after(line, "xmax =") {
                        iv.xmax = parse_f64(v, ln)?;
                    } else if let Some(v) = value_after(line, "text =") {
                        iv.text = v.trim_matches('"').to_string();
                    }
                }
            }
        }
    }

    Ok(doc)
}
