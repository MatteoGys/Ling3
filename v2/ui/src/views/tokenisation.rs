//! tokenisation.rs (vue) — Remplace ui/views/token_view.py. Le filtrage,
//! le tri et l'affichage des colonnes par tier restent calculés côté
//! frontend (comme en Python, tout était recalculé en mémoire sans nouvel
//! appel réseau) ; seul le chargement initial et l'export passent par
//! `invoke`.

use std::collections::HashMap;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::{self, TokenisationData};
use crate::icons::{IconWarning};

#[derive(Clone)]
struct Row {
    token: String,
    total: u32,
    per_tier: HashMap<String, u32>,
}

fn sort_symbol(col: &str, sort_col: &str, is_asc: bool) -> &'static str {
    if col == sort_col {
        if is_asc { "▲" } else { "▼" }
    } else if is_asc {
        "△"
    } else {
        "▽"
    }
}

fn compute_rows(
    data: &TokenisationData,
    search: &str,
    sort_col: &str,
    col_directions: &HashMap<String, bool>,
    active_tiers: &[String],
) -> Vec<Row> {
    let search = search.to_lowercase();
    let mut rows: Vec<Row> = data
        .global_counts
        .keys()
        .filter(|w| w.contains(&search))
        .map(|w| {
            let mut per_tier = HashMap::new();
            for tier in active_tiers {
                let c = data.tier_counts.get(tier).and_then(|m| m.get(w)).copied().unwrap_or(0);
                per_tier.insert(tier.clone(), c);
            }
            Row {
                token: w.clone(),
                total: *data.global_counts.get(w).unwrap_or(&0),
                per_tier,
            }
        })
        .collect();

    let asc = *col_directions.get(sort_col).unwrap_or(&true);

    rows.sort_by(|a, b| {
        let ord = if sort_col == "Tokens" {
            a.token.cmp(&b.token)
        } else if sort_col == "Occ. Tot." {
            a.total.cmp(&b.total)
        } else {
            let tier = sort_col.strip_prefix("Occ. ").unwrap_or(sort_col);
            let av = a.per_tier.get(tier).copied().unwrap_or(0);
            let bv = b.per_tier.get(tier).copied().unwrap_or(0);
            av.cmp(&bv)
        };
        if asc { ord } else { ord.reverse() }
    });

    rows
}

#[component]
pub fn TokenisationView() -> impl IntoView {
    let data = RwSignal::new(None::<TokenisationData>);
    let search = RwSignal::new(String::new());
    let active_tiers = RwSignal::new(Vec::<String>::new());
    let sort_col = RwSignal::new("Tokens".to_string());
    let col_directions = RwSignal::new({
        let mut m = HashMap::new();
        m.insert("Tokens".to_string(), true);
        m.insert("Occ. Tot.".to_string(), true);
        m
    });
    let menu_open = RwSignal::new(false);
    let status = RwSignal::new("Prêt à analyser.".to_string());
    let status_is_error = RwSignal::new(false);
    let loading = RwSignal::new(false);

    let load = move |_: ()| {
        loading.set(true);
        spawn_local(async move {
            match api::invoke::<_, TokenisationData>("load_tokenisation_data", &api::Empty {}).await {
                Ok(d) => {
                    let mut parts = vec![format!("{} fichier(s) analysé(s)", d.processed_count)];
                    if !d.missing_files.is_empty() {
                        parts.push(format!("{} introuvable(s)", d.missing_files.len()));
                    }
                    if !d.error_files.is_empty() {
                        parts.push(format!("{} en erreur", d.error_files.len()));
                    }
                    status_is_error.set(!d.missing_files.is_empty() || !d.error_files.is_empty());
                    status.set(if status_is_error.get_untracked() { format!("{}.", parts.join(" · ")) } else { String::new() });
                    data.set(Some(d));
                }
                Err(e) => {
                    status.set(e.to_string());
                    status_is_error.set(true);
                }
            }
            loading.set(false);
        });
    };

    // Chargement initial paresseux (⬅ on_show : ne charge que si vide).
    Effect::new(move |_| {
        if data.get_untracked().is_none() {
            load(());
        }
    });

    let handle_sort = move |col: String| {
        let current = sort_col.get_untracked();
        col_directions.update(|dirs| {
            dirs.entry(col.clone()).or_insert(true);
        });
        if current == col {
            col_directions.update(|dirs| {
                if let Some(v) = dirs.get_mut(&col) {
                    *v = !*v;
                }
            });
        } else {
            sort_col.set(col);
        }
    };

    let toggle_tier = move |tier: String| {
        active_tiers.update(|list| {
            if let Some(pos) = list.iter().position(|t| t == &tier) {
                list.remove(pos);
            } else {
                list.push(tier);
            }
        });
    };

    let export = move |_| {
        let d = data.get_untracked();
        let Some(d) = d else { return };
        let tiers = active_tiers.get_untracked();
        let rows = compute_rows(&d, &search.get_untracked(), &sort_col.get_untracked(), &col_directions.get_untracked(), &tiers);

        if rows.is_empty() {
            status.set("Aucune donnée à exporter.".to_string());
            status_is_error.set(true);
            return;
        }

        let mut headers = vec!["Tokens".to_string(), "Occ. Tot.".to_string()];
        for t in &tiers {
            headers.push(format!("Occ. {}", t));
        }

        let table_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|r| {
                let mut cells = vec![r.token.clone(), r.total.to_string()];
                for t in &tiers {
                    cells.push(r.per_tier.get(t).copied().unwrap_or(0).to_string());
                }
                cells
            })
            .collect();

        status.set("Écriture dans Excel...".to_string());
        status_is_error.set(false);

        spawn_local(async move {
            let result: Result<String, String> = api::invoke(
                "export_table_excel",
                &api::ExportTableArgs {
                    sheet_title: "Tokenisation".to_string(),
                    headers,
                    rows: table_rows,
                    default_filename: "tokens_export.xlsx".to_string(),
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
            <h1 class="view-title">"Tokenisation & Occurrences"</h1>
            <hr class="divider" />

            <div class="card" style="display:flex; gap:12px; align-items:center;">
                <input
                    class="field" type="text" placeholder="Rechercher un token..."
                    prop:value=move || search.get()
                    on:input=move |ev| search.set(event_target_value(&ev))
                />

                <div style="position: relative;">
                    <button class="btn btn-primary" on:click=move |_| menu_open.update(|o| *o = !*o)>
                        "Tiers ▾"
                    </button>
                    <Show when=move || menu_open.get()>
                        <div class="dropdown-menu">
                            {move || {
                                data.get()
                                    .map(|d| {
                                        d.tiers
                                            .iter()
                                            .map(|tier| {
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
                                            })
                                            .collect_view()
                                    })
                            }}
                        </div>
                    </Show>
                </div>

                <button class="btn btn-success" on:click=move |_| load(()) disabled=move || loading.get()>
                    {move || if loading.get() { "Chargement..." } else { "Rafraîchir" }}
                </button>
            </div>

            <div class="card" style="overflow-x:auto;">
                <table class="results">
                    <thead>
                        <tr>
                            {move || {
                                let mut cols = vec!["Tokens".to_string(), "Occ. Tot.".to_string()];
                                for t in active_tiers.get() {
                                    cols.push(format!("Occ. {}", t));
                                }
                                let sc = sort_col.get();
                                let dirs = col_directions.get();
                                cols.into_iter()
                                    .map(|col| {
                                        let is_asc = *dirs.get(&col).unwrap_or(&true);
                                        let sym = sort_symbol(&col, &sc, is_asc);
                                        let col_click = col.clone();
                                        view! {
                                            <th
                                                style="cursor:pointer; user-select:none;"
                                                on:click=move |_| handle_sort(col_click.clone())
                                            >
                                                {format!("{} {}", col, sym)}
                                            </th>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            data.get().map(|d| {
                                let tiers = active_tiers.get();
                                let rows = compute_rows(&d, &search.get(), &sort_col.get(), &col_directions.get(), &tiers);
                                rows.into_iter().map(|r| {
                                    view! {
                                        <tr>
                                            <td>{r.token.clone()}</td>
                                            <td>{r.total.to_string()}</td>
                                            {tiers.iter().map(|t| {
                                                let c = r.per_tier.get(t).copied().unwrap_or(0);
                                                view! { <td>{c.to_string()}</td> }
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
