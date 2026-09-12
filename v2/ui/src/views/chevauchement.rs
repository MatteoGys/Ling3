//! chevauchement.rs (vue) — Remplace ui/views/chevauchement_view.py.
//! Toute la logique d'analyse (parsing, recherche dichotomique) vit déjà
//! côté Rust natif (src-tauri/src/textgrid/overlap.rs) ; cette vue ne fait
//! qu'appeler la commande et afficher le résultat.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, OverlapReport};
use crate::icons::{IconWarning};

#[component]
pub fn ChevauchementView() -> impl IntoView {
    let status = RwSignal::new("Prêt à analyser.".to_string());
    let status_is_error = RwSignal::new(false);
    let report = RwSignal::new(None::<OverlapReport>);
    let running = RwSignal::new(false);

    let analyze = move |_| {
        running.set(true);
        status.set("Analyse en cours...".to_string());
        status_is_error.set(false);

        spawn_local(async move {
            match api::invoke::<_, OverlapReport>("analyze_overlaps", &api::Empty {}).await {
                Ok(r) => {
                    let mut parts = vec![
                        format!("{} erreur(s) trouvée(s)", r.total_errors),
                        format!("{} fichier(s) analysé(s)", r.processed_count),
                    ];
                    if !r.missing_files.is_empty() {
                        parts.push(format!("{} introuvable(s)", r.missing_files.len()));
                    }
                    if !r.error_files.is_empty() {
                        parts.push(format!("{} en erreur", r.error_files.len()));
                    }
                    status_is_error.set(!r.missing_files.is_empty() || !r.error_files.is_empty());
                    status.set(format!("{}.", parts.join(" · ")));
                    report.set(Some(r));
                }
                Err(e) => {
                    status.set(e.to_string());
                    status_is_error.set(true);
                }
            }
            running.set(false);
        });
    };

    view! {
        <div class="view-container">
            <h1 class="view-title">"Analyse des chevauchements"</h1>
            <hr class="divider" />

            <div style="display:flex; align-items:center; gap:16px; margin: 16px 0;">
                <button class="btn btn-primary" on:click=analyze disabled=move || running.get()>
                    {move || if running.get() { "Analyse..." } else { "Lancer l'analyse TextGrid" }}
                </button>
                <span
                    class="status-text"
                    class:status-error=move || status_is_error.get()
                    style="display: flex; align-items: center; gap: 6px;"
                >
                    <Show when=move || status_is_error.get()>
                        <IconWarning />
                    </Show>
                    {move || status.get()}
                </span>
            </div>

            <table class="results">
                <thead>
                    <tr>
                        <th>"Tier (Source)"</th>
                        <th>"N° Intervalle"</th>
                        <th>"Timecode (x_max)"</th>
                        <th>"Texte Source"</th>
                        <th style="color: var(--color-red);">"Erreur dans"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        report.get().map(|r| {
                            r.results.into_iter().flat_map(|file_result| {
                                file_result.errors.into_iter().map(|err| {
                                    view! {
                                        <tr>
                                            <td>{err.source_tier}</td>
                                            <td>{err.interval_idx.to_string()}</td>
                                            <td>{format!("{:.4}", err.timecode)}</td>
                                            <td>{err.text}</td>
                                            <td style="color: var(--color-red); font-weight: bold;">{err.error_tiers}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()
                            }).collect_view()
                        })
                    }}
                </tbody>
            </table>
        </div>
    }
}
