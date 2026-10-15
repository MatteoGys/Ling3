//! modification_textgrid.rs (vue) — Remplace ui/views/modification_TextGrid_view.py.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::icons::{IconCheck, IconWarning};

/// Rapport de conversion d'un fichier TRS, tel que renvoyé par la commande
/// `convert_trs_files` (miroir de `trs_convert::TrsFileReport` côté src-tauri).
#[derive(Debug, Clone, serde::Deserialize)]
struct TrsFileReport {
    /// Un fichier a été écrit (.TextGrid, ou .txt si des bornes sont à corriger).
    created: bool,
    headline: String,
    lines: Vec<TrsReportLine>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TrsReportLine {
    text: String,
    /// true : à afficher en rouge ; false : simple information.
    is_error: bool,
}

#[component]
pub fn ModificationView() -> impl IntoView {
    // ── Conversion TRS → TextGrid ─────────────────────────────────────────
    // Un seul bouton : "Parcourir" ouvre le sélecteur, et dès que des
    // fichiers en ressortent, la conversion s'enchaîne aussitôt dans la
    // même action — plus besoin d'un second bouton "Convertir".
    // `trs_status` ne sert qu'aux messages généraux (en cours, erreur du
    // sélecteur) ; le résultat de chaque fichier est dans `trs_reports`.
    let trs_status = RwSignal::new(String::new());
    let trs_status_is_error = RwSignal::new(false);
    let trs_reports = RwSignal::new(Vec::<TrsFileReport>::new());

    let pick_and_convert_trs = move |_| {
        spawn_local(async move {
            match api::pick_files("Sélectionner un ou plusieurs fichiers .trs", &["trs", "xml"], true).await {
                Ok(paths) if !paths.is_empty() => {
                    trs_reports.set(Vec::new());
                    trs_status.set("Conversion en cours...".to_string());
                    trs_status_is_error.set(false);

                    let result: Result<Vec<TrsFileReport>, String> =
                        api::invoke("convert_trs_files", &api::TrsPathsArg { trs_paths: paths }).await;
                    match result {
                        Ok(reports) => {
                            trs_status.set(String::new());
                            trs_reports.set(reports);
                        }
                        Err(e) => {
                            trs_status.set(e);
                            trs_status_is_error.set(true);
                        }
                    }
                }
                Ok(_) => {} // annulé, rien à faire
                Err(e) => {
                    trs_status.set(e);
                    trs_status_is_error.set(true);
                }
            }
        });
    };

    // ── Remplacement de texte + fusion des silences ──────────────────────
    let find_text = RwSignal::new(String::new());
    let replace_text = RwSignal::new(String::new());
    let merge_silences = RwSignal::new(false);
    let mod_status = RwSignal::new(String::new());
    let mod_status_is_error = RwSignal::new(false);

    let apply_modifications = move |_| {
        let find_v = find_text.get_untracked();
        let replace_v = replace_text.get_untracked();
        let merge_v = merge_silences.get_untracked();
        mod_status.set("Traitement en cours...".to_string());
        mod_status_is_error.set(false);
        spawn_local(async move {
            let result: Result<String, String> = api::invoke(
                "apply_text_modifications",
                &api::TextModArgs { find: find_v, replace: replace_v, merge_silences: merge_v },
            )
            .await;
            match result {
                Ok(msg) => mod_status.set(msg),
                Err(e) => {
                    mod_status.set(e.to_string());
                    mod_status_is_error.set(true);
                }
            }
        });
    };

    // ── Extraction par pivots ─────────────────────────────────────────────
    let pivots_raw = RwSignal::new(String::new());
    let pivot_status = RwSignal::new(String::new());
    let pivot_status_is_error = RwSignal::new(false);

    let apply_pivots = move |_| {
        let pivots_v = pivots_raw.get_untracked();
        pivot_status.set("Traitement en cours...".to_string());
        pivot_status_is_error.set(false);
        spawn_local(async move {
            let result: Result<String, String> =
                api::invoke("apply_pivot_extraction", &api::PivotArgs { pivots_raw: pivots_v }).await;
            match result {
                Ok(msg) => pivot_status.set(msg),
                Err(e) => {
                    pivot_status.set(e.to_string());
                    pivot_status_is_error.set(true);
                }
            }
        });
    };

    view! {
        <div class="view-container">
            <h1 class="view-title">"Modifications"</h1>
            <hr class="divider" />

            <div class="card">
                <p style="margin-top:0;"><strong>"Conversion TRS → TextGrid"</strong></p>
                <div class="pivot-action-row">
                    <button
                        class="btn btn-primary"
                        on:click=pick_and_convert_trs
                    >
                        "Parcourir"
                    </button>

                    <p class="status-text"
                        class:status-error=move || trs_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                        <Show when=move || trs_status_is_error.get()>
                            <IconWarning />
                        </Show>
                        {move || trs_status.get()}
                    </p>
                </div>

                // Un bloc par fichier : icône + titre, puis le détail tabulé dessous.
                {move || {
                    trs_reports
                        .get()
                        .into_iter()
                        .map(|report| {
                            let created = report.created;
                            view! {
                                <div style="padding: 10px 0; margin-top: 10px; border-top: 1px solid var(--color-bg-container-border);">
                                    <div
                                        class="status-text"
                                        class:status-error=move || !created
                                        style="display: flex; align-items: center; gap: 8px; margin: 0;"
                                    >
                                        {if created {
                                            view! { <IconCheck /> }.into_any()
                                        } else {
                                            view! { <IconWarning /> }.into_any()
                                        }}
                                        <span>{report.headline.clone()}</span>
                                    </div>
                                    {report
                                        .lines
                                        .into_iter()
                                        .map(|line| {
                                            let is_error = line.is_error;
                                            view! {
                                                <div
                                                    class="status-text"
                                                    class:status-error=move || is_error
                                                    style="margin: 4px 0 0 28px;"
                                                >
                                                    {line.text}
                                                </div>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>

            <div class="card">
                <p style="margin-top:0;"><strong>"Remplacement de texte & fusion des silences"</strong></p>
                <div style="display:flex; gap:12px; margin-bottom: 10px;">
                    <input
                        class="field" type="text" placeholder="Texte cible..."
                        prop:value=move || find_text.get()
                        on:input=move |ev| find_text.set(event_target_value(&ev))
                    />
                    <input
                        class="field" type="text" placeholder="Texte de remplacement..."
                        prop:value=move || replace_text.get()
                        on:input=move |ev| replace_text.set(event_target_value(&ev))
                    />
                </div>
                <label style="display:flex; align-items:center; gap:8px; margin-bottom: 10px;">
                    <input
                        type="checkbox"
                        prop:checked=move || merge_silences.get()
                        on:change=move |ev| merge_silences.set(event_target_checked(&ev))
                    />
                    "Fusionner les intervalles de silence consécutifs"
                </label>
                
                <div class="replace-action-row">
                    <button 
                        class="btn btn-primary" 
                        on:click=apply_modifications
                    >
                        "Appliquer"
                    </button>

                    <p class="status-text" 
                        class:status-error=move || mod_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                            <Show when=move || mod_status_is_error.get()>
                                <IconWarning />
                            </Show>
                            {move || mod_status.get()}
                    </p>
                </div>
            </div>

            <div class="card">
                <p style="margin-top:0;"><strong>"Extraction par mots pivots"</strong></p>
                <input
                    class="field" type="text" placeholder="pivot1\\pivot2\\pivot3..."
                    prop:value=move || pivots_raw.get()
                    on:input=move |ev| pivots_raw.set(event_target_value(&ev))
                />
                <div class="pivot-action-row">
                    <button
                        class="btn btn-primary"
                        on:click=apply_pivots
                    >
                        "Créer les fichiers *_pivot"
                    </button>

                    <p class="status-text"
                        class:status-error=move || pivot_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                            <Show when=move || pivot_status_is_error.get()>
                                <IconWarning />
                            </Show>
                            {move || pivot_status.get()}
                    </p>
                </div>
            </div>
        </div>
    }
}
