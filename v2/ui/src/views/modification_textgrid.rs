//! modification_textgrid.rs (vue) — Remplace ui/views/modification_TextGrid_view.py.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::icons::{IconCheck, IconWarning};

#[component]
pub fn ModificationView() -> impl IntoView {
    // ── Conversion TRS → TextGrid ─────────────────────────────────────────
    let trs_paths = RwSignal::new(Vec::<String>::new());
    let trs_label = RwSignal::new("Aucun fichier .trs.xml sélectionné".to_string());
    let trs_status = RwSignal::new(String::new());
    let trs_status_is_error = RwSignal::new(false);

    let pick_trs = move |_| {
        spawn_local(async move {
            match api::pick_files("Sélectionner un ou plusieurs fichiers .trs", &["trs", "xml"], true).await {
                Ok(paths) if !paths.is_empty() => {
                    trs_label.set(format!("{} fichier(s) sélectionné(s)", paths.len()));
                    trs_paths.set(paths);
                }
                Ok(_) => {}
                Err(e) => {
                    trs_status.set(e.to_string());
                    trs_status_is_error.set(true);
                }
            }
        });
    };

    let convert_trs = move |_| {
        let paths = trs_paths.get_untracked();
        if paths.is_empty() {
            return;
        }
        trs_status.set("Conversion en cours...".to_string());
        trs_status_is_error.set(false);
        spawn_local(async move {
            let result: Result<Vec<Result<String, String>>, String> =
                api::invoke("convert_trs_files", &api::TrsPathsArg { trs_paths: paths }).await;
            match result {
                Ok(results) => {
                    let has_error = results.iter().any(|r| r.is_err());
                    let summary = results
                        .into_iter()
                        .map(|r| r.unwrap_or_else(|e| e))
                        .collect::<Vec<_>>()
                        .join(" | ");
                    trs_status.set(summary);
                    trs_status_is_error.set(has_error);
                }
                Err(e) => {
                    trs_status.set(e.to_string());
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
                <p><strong>"Conversion TRS → TextGrid"</strong></p>
                <div style="display:flex; gap:12px; align-items:center;">
                    <span class="status-text" class:status-error=move || trs_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                        <Show when=move || trs_status_is_error.get()>
                            <IconWarning />
                        </Show>
                        <Show when=move || !trs_status_is_error.get() && !trs_status.get().is_empty()>
                            <IconCheck />
                        </Show>
                        {move || trs_status.get()}
                    </span>
                    <button class="btn btn-primary" on:click=pick_trs>"Parcourir"</button>
                    <button class="btn btn-success" on:click=convert_trs>"Convertir"</button>
                </div>
            </div>

            <div class="card">
                <p><strong>"Remplacement de texte & fusion des silences"</strong></p>
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
                <button class="btn btn-primary" on:click=apply_modifications>"Appliquer"</button>
                <p class="status-text" class:status-error=move || mod_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                    <Show when=move || mod_status_is_error.get()>
                        <IconWarning />
                    </Show>
                    {move || mod_status.get()}
                </p>
            </div>

            <div class="card">
                <p><strong>"Extraction par mots pivots"</strong></p>
                <input
                    class="field" type="text" placeholder="pivot1\\pivot2\\pivot3..."
                    prop:value=move || pivots_raw.get()
                    on:input=move |ev| pivots_raw.set(event_target_value(&ev))
                />
                <button class="btn btn-primary" style="margin-top:10px;" on:click=apply_pivots>
                    "Créer les fichiers *_pivot"
                </button>
                <p class="status-text" class:status-error=move || pivot_status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                    <Show when=move || pivot_status_is_error.get()>
                        <IconWarning />
                    </Show>
                    {move || pivot_status.get()}
                </p>
            </div>
        </div>
    }
}
