//! settings.rs (vue) — Remplace ui/views/settings_view.py.
//!
//! Changement par rapport à la version précédente : un seul champ pour les
//! chemins TextGrid (zone de texte multi-lignes, un chemin par ligne) sert à
//! la fois d'affichage ET de source de la liste réelle utilisée par les
//! analyses — que ce soit rempli via "Parcourir" ou tapé à la main. Avant,
//! la saisie manuelle ne mettait à jour qu'un champ d'affichage séparé,
//! jamais la vraie liste, donc les autres onglets voyaient toujours une
//! liste vide dans ce cas.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::icons::IconWarning;

/// Découpe le contenu de la zone de texte en chemins (un par ligne),
/// en ignorant les lignes vides.
fn parse_paths(raw: &str) -> Vec<String> {
    raw.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect()
}

#[component]
pub fn SettingsView() -> impl IntoView {
    let tg_paths_raw = RwSignal::new(String::new());
    let excel_path = RwSignal::new(String::new());
    let separator = RwSignal::new(String::from("\\"));

    let status = RwSignal::new(String::new());
    let status_is_error = RwSignal::new(false);

    // ── Chargement initial (équivalent de on_show()) ─────────────────────
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(data) = api::invoke::<_, api::ProfileData>("get_current_data", &api::Empty {}).await {
                tg_paths_raw.set(data.textgrid_path_files.join("\n"));
                excel_path.set(data.excel_path);
                separator.set(if data.separator.is_empty() { "\\".to_string() } else { data.separator });
            }
        });
    });

    // ── Sélection du/des fichier(s) TextGrid ─────────────────────────────
    let pick_textgrid = move |_| {
        spawn_local(async move {
            match api::pick_files("Sélectionner un ou plusieurs fichiers TextGrid", &["TextGrid", "txt"], true).await {
                Ok(paths) if !paths.is_empty() => {
                    // Un chemin par ligne : visible, modifiable à la main ensuite.
                    tg_paths_raw.set(paths.join("\n"));
                }
                Ok(_) => {} // annulé
                Err(e) => {
                    status.set(format!("Erreur de sélection : {}", e));
                    status_is_error.set(true);
                }
            }
        });
    };

    // ── Sélection du fichier Excel ────────────────────────────────────────
    let pick_excel = move |_| {
        spawn_local(async move {
            match api::pick_files("Sélectionner un fichier Excel", &["xlsx"], false).await {
                Ok(paths) if !paths.is_empty() => excel_path.set(paths[0].clone()),
                Ok(_) => {}
                Err(e) => {
                    status.set(format!("Erreur de sélection : {}", e));
                    status_is_error.set(true);
                }
            }
        });
    };

    // ── Enregistrement ────────────────────────────────────────────────────
    let save = move |_| {
        let tg_paths_v = parse_paths(&tg_paths_raw.get_untracked());
        let excel_v = excel_path.get_untracked();
        let sep_v = separator.get_untracked();

        spawn_local(async move {
            let mut changed = false;

            if !tg_paths_v.is_empty() {
                let joined = tg_paths_v.join("\n");
                let _ = api::invoke::<_, ()>("set_textgrid_path_show", &api::PathArg { path: joined }).await;
                let _ = api::invoke::<_, ()>("set_textgrid_path_files", &api::PathsArg { paths: tg_paths_v }).await;
                changed = true;
            }
            if !excel_v.trim().is_empty() {
                let _ = api::invoke::<_, ()>("set_excel_path", &api::PathArg { path: excel_v }).await;
                changed = true;
            }
            if !sep_v.trim().is_empty() {
                let _ = api::invoke::<_, ()>("set_separator", &api::SeparatorArg { separator: sep_v }).await;
                changed = true;
            }

            if changed {
                status.set("Paramètres enregistrés avec succès.".to_string());
                status_is_error.set(false);
            } else {
                status.set("Aucune modification à enregistrer.".to_string());
                status_is_error.set(false);
            }
        });
    };

    view! {
        <div class="view-container">
            <h1 class="view-title">"Paramètres"</h1>
            <hr class="divider" />

            <div class="card">
                <p><strong>"Fichier(s) .TextGrid ou .txt"</strong></p>
                <p class="status-text" style="margin-top:0;">
                    "Un chemin par ligne — vous pouvez utiliser \"Parcourir\" ou taper/coller "
                    "directement les chemins."
                </p>
                <div style="display:flex; gap:12px; align-items:flex-start;">
                    <textarea
                        class="field"
                        rows="4"
                        style="resize: vertical; font-family: monospace; font-size: 13px;"
                        prop:value=move || tg_paths_raw.get()
                        on:input=move |ev| tg_paths_raw.set(event_target_value(&ev))
                    ></textarea>
                    <button class="btn btn-primary" on:click=pick_textgrid>"Parcourir"</button>
                </div>
            </div>

            <div class="card">
                <p><strong>"Fichier Excel"</strong></p>
                <div style="display:flex; gap:12px; align-items:center;">
                    <input
                        class="field"
                        type="text"
                        prop:value=move || excel_path.get()
                        on:input=move |ev| excel_path.set(event_target_value(&ev))
                    />
                    <button class="btn btn-primary" on:click=pick_excel>"Parcourir"</button>
                </div>
            </div>

            <div class="card">
                <p><strong>"Séparateur de mots clés / pivots"</strong></p>
                <input
                    class="field"
                    style="max-width: 150px;"
                    type="text"
                    prop:value=move || separator.get()
                    on:input=move |ev| separator.set(event_target_value(&ev))
                />
            </div>

            <button class="btn btn-success" on:click=save>"Enregistrer"</button>

            <p
                class="status-text"
                class:status-error=move || status_is_error.get()
                style="display: flex; align-items: center; gap: 6px;"
            >
                <Show when=move || status_is_error.get()>
                    <IconWarning />
                </Show>
                {move || status.get()}
            </p>
        </div>
    }
}
