//! concordancier.rs (vue) — Remplace ui/views/concordancier_view.py.
//! L'extraction (regex, contexte inter-intervalles, classification de
//! position) vit côté src-tauri (textgrid::concordance) ; cette vue ne
//! gère que les paramètres et l'ordre des colonnes.
//!
//! Simplification assumée : le glisser-déposer des tuiles de colonnes
//! (Flet Draggable/DragTarget) est remplacé par des boutons ▲/▼/✕ — même
//! résultat fonctionnel (réordonner, ajouter, retirer des colonnes), sans le
//! risque de code de glisser-déposer HTML5 non testé. Facile à embellir
//! plus tard si vous voulez le vrai glisser-déposer.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::icons::{IconWarning};

const DEFAULT_COLUMNS: [&str; 11] = [
    "ID", "Tier Nom", "Tier Numéro", "Tier Lettre", "x_min", "x_max",
    "durée_occurence", "contexte gauche", "pivot", "contexte droit", "POS_PAUSES_VIDES",
];

const DEFAULT_ACTIVE: [&str; 4] = ["ID", "contexte gauche", "pivot", "contexte droit"];

#[component]
pub fn ConcordancierView() -> impl IntoView {
    let pivots_raw = RwSignal::new(String::new());
    let nb_left = RwSignal::new("5".to_string());
    let nb_right = RwSignal::new("5".to_string());
    let active_columns = RwSignal::new(
        DEFAULT_ACTIVE.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    let status = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let generating = RwSignal::new(false);

    let reset_columns = move |_| {
        active_columns.set(DEFAULT_ACTIVE.iter().map(|s| s.to_string()).collect());
    };

    let remove_col = move |col: String| {
        active_columns.update(|list| list.retain(|c| c != &col));
    };

    let add_col = move |col: String| {
        active_columns.update(|list| {
            if !list.contains(&col) {
                list.push(col);
            }
        });
    };

    let move_up = move |col: String| {
        active_columns.update(|list| {
            if let Some(pos) = list.iter().position(|c| c == &col) {
                if pos > 0 {
                    list.swap(pos, pos - 1);
                }
            }
        });
    };

    let move_down = move |col: String| {
        active_columns.update(|list| {
            if let Some(pos) = list.iter().position(|c| c == &col) {
                if pos + 1 < list.len() {
                    list.swap(pos, pos + 1);
                }
            }
        });
    };

    let generate = move |_| {
        let pivots_v = pivots_raw.get_untracked();
        let nb_left_v: i64 = nb_left.get_untracked().parse().unwrap_or(0);
        let nb_right_v: i64 = nb_right.get_untracked().parse().unwrap_or(0);
        let cols_v = active_columns.get_untracked();

        generating.set(true);
        status.set("Analyse en cours...".to_string());
        status_is_error.set(false);

        spawn_local(async move {
            let result: Result<String, String> = api::invoke(
                "generate_concordance_excel",
                &api::ConcordanceArgs {
                    pivots_raw: pivots_v,
                    nb_left: nb_left_v,
                    nb_right: nb_right_v,
                    active_columns: cols_v,
                },
            )
            .await;
            match result {
                Ok(msg) => {
                    status.set(msg);
                    status_is_error.set(false);
                }
                Err(e) => {
                    status.set(e.to_string());
                    status_is_error.set(true);
                }
            }
            generating.set(false);
        });
    };

    view! {
        <div class="view-scroll-container">
        <div class="view-scroll-area">
            <h1 class="view-title">"Concordancier Excel"</h1>
            <hr class="divider" />

            <div class="card" style="display:flex; gap:16px;">
                <input
                    class="field" style="flex: 1;" type="text" placeholder="Ex: euh\\hum\\du coup"
                    prop:value=move || pivots_raw.get()
                    on:input=move |ev| pivots_raw.set(event_target_value(&ev))
                />
                <div>
                    <label class="status-text">"Mots avant (Gauche)"</label>
                    <input
                        class="field" style="width: 100px;" type="text"
                        prop:value=move || nb_left.get()
                        on:input=move |ev| nb_left.set(event_target_value(&ev))
                    />
                </div>
                <div>
                    <label class="status-text">"Mots après (Droite)"</label>
                    <input
                        class="field" style="width: 100px;" type="text"
                        prop:value=move || nb_right.get()
                        on:input=move |ev| nb_right.set(event_target_value(&ev))
                    />
                </div>
            </div>

            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom: 10px;">
                <div>
                    <p style="margin:0; font-weight:bold;">"Ordre des colonnes :"</p>
                    <p class="status-text" style="margin:0;">"Utilisez ▲/▼ pour réordonner l'export."</p>
                </div>
                <button class="btn btn-outline" on:click=reset_columns>"Réinitialiser l'ordre"</button>
            </div>

            <div class="card">
                <p style="margin-top:0; font-weight:bold;">"Colonnes actives"</p>
                <div style="display:flex; flex-direction:column; gap:6px; margin-bottom: 16px;">
                    {move || {
                        let cols = active_columns.get();
                        let len = cols.len();
                        cols.into_iter().enumerate().map(|(i, col)| {
                            let col_up = col.clone();
                            let col_down = col.clone();
                            let col_remove = col.clone();
                            view! {
                                <div class="column-tile">
                                    <span style="flex:1;">{col.clone()}</span>
                                    <button class="icon-btn" disabled=move || i == 0 on:click=move |_| move_up(col_up.clone())>"▲"</button>
                                    <button class="icon-btn" disabled=move || i + 1 == len on:click=move |_| move_down(col_down.clone())>"▼"</button>
                                    <button class="icon-btn" style="color: var(--color-red);" on:click=move |_| remove_col(col_remove.clone())>"✕"</button>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>

                <p style="font-weight:bold;">"Colonnes disponibles"</p>
                <div style="display:flex; flex-wrap:wrap; gap:8px;">
                    {move || {
                        let active = active_columns.get();
                        DEFAULT_COLUMNS.iter()
                            .filter(|c| !active.contains(&c.to_string()))
                            .map(|c| {
                                let c = c.to_string();
                                let c_click = c.clone();
                                view! {
                                    <button class="btn btn-outline" on:click=move |_| add_col(c_click.clone())>
                                        {format!("+ {}", c)}
                                    </button>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>
        </div>

            <div class="view-action-bar-fixed">
                <span class="status-text" class:status-error=move || status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                    <Show when=move || status_is_error.get()>
                        <IconWarning />
                    </Show>
                    {move || status.get()}
                </span>
                <button class="btn btn-primary" on:click=generate disabled=move || generating.get()>
                    {move || if generating.get() { "Génération..." } else { "Générer le fichier Excel" }}
                </button>
            </div>
        </div>
    }
}
