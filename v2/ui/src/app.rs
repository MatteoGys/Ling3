//! app.rs — Composant racine. Remplace la construction de la sidebar et le
//! routage entre onglets (`switch_view`, `_build_nav_buttons`, etc.) de
//! main.py, ainsi que NAV_ITEMS.

use leptos::prelude::*;

use crate::icons::{IconConcordancier, IconInfo, IconMenu, IconModification, IconOverlap, IconCooccurrences, IconSettings, IconTheme, IconToken};
use crate::theme::provide_theme;
use crate::views::chevauchement::ChevauchementView;
use crate::views::concordancier::ConcordancierView;
use crate::views::cooccurrences::CooccurrencesView;
use crate::views::info::InfoView;
use crate::views::modification_textgrid::ModificationView;
use crate::views::settings::SettingsView;
use crate::views::tokenisation::TokenisationView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Chevauchements,
    Modifications,
    Tokenisation,
    Concordancier,
    Cooccurrences,
    Settings,
    Info,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Chevauchements => "Chevauchements",
            Tab::Modifications => "Modifications",
            Tab::Tokenisation => "Tokenisation",
            Tab::Concordancier => "Concordancier",
            Tab::Cooccurrences => "Cooccurrences",
            Tab::Settings => "Paramètres",
            Tab::Info => "Documentation",
        }
    }
}

/// Icône SVG associée à chaque onglet (remplace les emojis).
fn nav_icon(tab: Tab) -> impl IntoView {
    match tab {
        Tab::Chevauchements => view! { <IconOverlap /> }.into_any(),
        Tab::Modifications => view! { <IconModification /> }.into_any(),
        Tab::Tokenisation => view! { <IconToken /> }.into_any(),
        Tab::Concordancier => view! { <IconConcordancier /> }.into_any(),
        Tab::Cooccurrences => view! { <IconCooccurrences /> }.into_any(),
        Tab::Settings => view! { <IconSettings /> }.into_any(),
        Tab::Info => view! { <IconInfo /> }.into_any(),
    }
}

// ⬅ NAV_ITEMS dans main.py : les onglets principaux visibles dans la
// sidebar (Paramètres/Documentation restent des boutons ronds séparés, en
// bas de sidebar, comme dans l'app Flet).
const NAV_ITEMS: [Tab; 5] = [
    Tab::Chevauchements,
    Tab::Modifications,
    Tab::Tokenisation,
    Tab::Concordancier,
    Tab::Cooccurrences,
];

#[component]
pub fn App() -> impl IntoView {
    let theme = provide_theme();
    let current_tab = RwSignal::new(Tab::Chevauchements);
    let expanded = RwSignal::new(true);

    view! {
        <div id="app-root" data-theme=move || theme.get().as_attr()>
            <aside class="sidebar" class:collapsed=move || !expanded.get()>
                <div class="sidebar-header">
                    <Show when=move || expanded.get()>
                        <span class="app-title">"Ling3"</span>
                    </Show>
                    <button
                        class="icon-btn"
                        title="Réduire/Étendre"
                        on:click=move |_| expanded.update(|e| *e = !*e)
                    >
                        <IconMenu />
                    </button>
                </div>

                <hr class="divider" />

                <nav class="nav-col">
                    {NAV_ITEMS
                        .iter()
                        .map(|tab| {
                            let tab = *tab;
                            view! {
                                <button
                                    class="nav-btn"
                                    class:active=move || current_tab.get() == tab
                                    on:click=move |_| current_tab.set(tab)
                                >
                                    <span class="nav-icon">{nav_icon(tab)}</span>
                                    <span class="nav-label">{tab.label()}</span>
                                </button>
                            }
                        })
                        .collect_view()}
                </nav>

                <hr class="divider" />

                <div class="sidebar-footer">
                    <Show when=move || expanded.get()>
                        <span class="version-label">"v2.0.0 Bêta"</span>
                    </Show>
                    <div class="actions">
                        <button
                            class="round-btn"
                            title="Paramètres"
                            on:click=move |_| current_tab.set(Tab::Settings)
                        >
                            <IconSettings />
                        </button>
                        <button
                            class="round-btn"
                            title="Changer le thème"
                            on:click=move |_| theme.update(|t| *t = t.toggled())
                        >
                            <IconTheme />
                        </button>
                        <button
                            class="round-btn"
                            title="Documentation"
                            on:click=move |_| current_tab.set(Tab::Info)
                        >
                            <IconInfo />
                        </button>
                    </div>
                </div>
            </aside>

            <main class="content-area">
                {move || match current_tab.get() {
                    Tab::Chevauchements => view! { <ChevauchementView /> }.into_any(),
                    Tab::Modifications => view! { <ModificationView /> }.into_any(),
                    Tab::Settings => view! { <SettingsView /> }.into_any(),
                    Tab::Tokenisation => view! { <TokenisationView /> }.into_any(),
                    Tab::Concordancier => view! { <ConcordancierView /> }.into_any(),
                    Tab::Cooccurrences => view! { <CooccurrencesView /> }.into_any(),
                    Tab::Info => view! { <InfoView /> }.into_any(),
                }}
            </main>
        </div>
    }
}
