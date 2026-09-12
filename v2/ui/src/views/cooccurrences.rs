//! cooccurrences.rs (vue) — Remplace ui/views/stats_view.py. L'analyse elle-même
//! (fenêtre glissante, gestion des frontières de fichiers) vit côté
//! src-tauri (textgrid::stats) ; cette vue collecte les paramètres et
//! affiche le résultat.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, StatsAnalysisResult, StatsLoadSummary};
use crate::icons::{IconWarning};

#[component]
pub fn CooccurrencesView() -> impl IntoView {
    let summary = RwSignal::new(None::<StatsLoadSummary>);
    let pivot = RwSignal::new(String::new());
    let distances_raw = RwSignal::new("1".to_string());
    let direction_suivant = RwSignal::new(true); // true = "Suivant" (défaut Python)
    let active_tiers = RwSignal::new(Vec::<String>::new());
    let menu_open = RwSignal::new(false);
    let analysis = RwSignal::new(None::<StatsAnalysisResult>);
    let status = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let run_analysis = move || {
        let pivot_v = pivot.get_untracked();
        let distances_v = distances_raw.get_untracked();
        let dir_v = direction_suivant.get_untracked();
        let tiers_v = active_tiers.get_untracked();
        status.set("Analyse en cours...".to_string());
        status_is_error.set(false);
        spawn_local(async move {
            let result: Result<StatsAnalysisResult, String> = api::invoke(
                "run_stats_analysis",
                &api::StatsAnalysisArgs {
                    pivot: pivot_v,
                    distances_raw: distances_v,
                    direction_suivant: dir_v,
                    active_tiers: tiers_v,
                },
            )
            .await;
            match result {
                Ok(r) => {
                    analysis.set(Some(r));
                    status.set("Analyse terminée avec succès.".to_string());
                    status_is_error.set(false);
                }
                Err(e) => {
                    status.set(e.to_string());
                    status_is_error.set(true);
                }
            }
        });
    };

    let load = move || {
        loading.set(true);
        spawn_local(async move {
            let result: Result<StatsLoadSummary, String> =
                api::invoke("load_stats_data", &api::Empty {}).await;
            match result {
                Ok(s) => {
                    active_tiers.set(s.tiers.clone()); // ⬅ toutes actives par défaut, comme en Python
                    summary.set(Some(s));
                }
                Err(e) => {
                    status.set(e.to_string());
                    status_is_error.set(true);
                }
            }
            loading.set(false);
        });
    };

    // Chargement initial (⬅ on_show : charge si vide).
    Effect::new(move |_| {
        if summary.get_untracked().is_none() {
            load();
        }
    });

    let toggle_tier = move |tier: String| {
        active_tiers.update(|list| {
            if let Some(pos) = list.iter().position(|t| t == &tier) {
                list.remove(pos);
            } else {
                list.push(tier);
            }
        });
        run_analysis();
    };

    let export = move |_| {
        let a = analysis.get_untracked();
        let Some(a) = a else {
            status.set("Aucune donnée à exporter.".to_string());
            status_is_error.set(true);
            return;
        };
        if a.rows.is_empty() {
            status.set("Aucune donnée à exporter.".to_string());
            status_is_error.set(true);
            return;
        }

        let tiers = active_tiers.get_untracked();
        let mut headers = vec!["Type".to_string(), "Occ. Tot.".to_string()];
        for t in &tiers {
            headers.push(format!("Occ. {}", t));
        }

        let table_rows: Vec<Vec<String>> = a
            .rows
            .iter()
            .map(|row| {
                let mut cells = vec![
                    row.display.clone(),
                    format!("{} / {}", row.global_count, a.total_pivot_global),
                ];
                for t in &tiers {
                    let c = row.tier_counts.get(t).copied().unwrap_or(0);
                    let tot = a.total_pivot_tier.get(t).copied().unwrap_or(0);
                    cells.push(format!("{} / {}", c, tot));
                }
                cells
            })
            .collect();

        spawn_local(async move {
            let result: Result<String, String> = api::invoke(
                "export_table_excel",
                &api::ExportTableArgs {
                    sheet_title: "Cooccurrences".to_string(),
                    headers,
                    rows: table_rows,
                    default_filename: "cooccurrences_export.xlsx".to_string(),
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
        });
    };

    view! {
        <div class="view-scroll-container">
        <div class="view-scroll-area">
            <h1 class="view-title">"Analyse de la concurrence d'un noeud"</h1>
            <hr class="divider" />

            <div class="card" style="display:flex; gap:12px; align-items:center; flex-wrap: wrap;">
                <input
                    class="field" style="flex: 1; min-width: 180px;" type="text" placeholder="Pivot (ex: du coup)..."
                    prop:value=move || pivot.get()
                    on:input=move |ev| pivot.set(event_target_value(&ev))
                />
                <button
                    class="btn"
                    class:btn-primary=move || !direction_suivant.get()
                    class:btn-outline=move || direction_suivant.get()
                    on:click=move |_| direction_suivant.set(false)
                >
                    "Précédent"
                </button>
                <button
                    class="btn"
                    class:btn-primary=move || direction_suivant.get()
                    class:btn-outline=move || !direction_suivant.get()
                    on:click=move |_| direction_suivant.set(true)
                >
                    "Suivant"
                </button>
                <input
                    class="field" style="max-width: 100px;" type="text" placeholder="1, 2, 3..."
                    prop:value=move || distances_raw.get()
                    on:input=move |ev| distances_raw.set(event_target_value(&ev))
                />

                <div style="position: relative;">
                    <button class="btn btn-primary" on:click=move |_| menu_open.update(|o| *o = !*o)>
                        "Tiers ▾"
                    </button>
                    <Show when=move || menu_open.get()>
                        <div class="dropdown-menu">
                            {move || {
                                summary.get().map(|s| {
                                    s.tiers.iter().map(|tier| {
                                        let tier = tier.clone();
                                        let tier_for_check = tier.clone();
                                        let tier_for_click = tier.clone();
                                        view! {
                                            <label class="dropdown-item">
                                                <input
                                                    type="checkbox"
                                                    prop:checked=move || active_tiers.get().contains(&tier_for_check)
                                                    on:change=move |_| toggle_tier(tier_for_click.clone())
                                                />
                                                {tier}
                                            </label>
                                        }
                                    }).collect_view()
                                })
                            }}
                        </div>
                    </Show>
                </div>

                <button class="btn btn-primary" on:click=move |_| run_analysis()>"Analyser"</button>
            </div>

            <div class="card" style="overflow-x:auto;">
                <table class="results">
                    <thead>
                        <tr>
                            <th>"Type"</th>
                            <th>"Occ. Tot."</th>
                            {move || active_tiers.get().iter().map(|t| {
                                view! { <th>{format!("Occ. {}", t)}</th> }
                            }).collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            analysis.get().map(|a| {
                                let tiers = active_tiers.get();
                                a.rows.iter().map(|row| {
                                    let global_cell = format!("{} / {}", row.global_count, a.total_pivot_global);
                                    view! {
                                        <tr>
                                            <td>{row.display.clone()}</td>
                                            <td>{global_cell}</td>
                                            {tiers.iter().map(|t| {
                                                let c = row.tier_counts.get(t).copied().unwrap_or(0);
                                                let tot = a.total_pivot_tier.get(t).copied().unwrap_or(0);
                                                view! { <td>{format!("{} / {}", c, tot)}</td> }
                                            }).collect_view()}
                                        </tr>
                                    }
                                }).collect_view()
                            })
                        }}
                    </tbody>
                </table>
            </div>
        </div>

            <div class="view-action-bar-fixed">
                <span class="status-text" class:status-error=move || status_is_error.get() style="display: flex; align-items: center; gap: 6px;">
                    <Show when=move || status_is_error.get()>
                        <IconWarning />
                    </Show>
                    {move || status.get()}
                </span>
                <button class="btn btn-success" on:click=export>"Export Excel"</button>
            </div>
        </div>
    }
}
