//! theme.rs — Remplace ui/theme.py. Les valeurs de couleurs elles-mêmes
//! vivent dans styles/theme.css (variables CSS `--color-*`) ; ce module ne
//! gère que l'état réactif "clair/sombre" et l'attribut `data-theme` qui
//! sélectionne la bonne palette en CSS.

use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_attr(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn toggled(&self) -> Theme {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        }
    }
}

/// À appeler une seule fois, à la racine de <App/> : crée le signal de
/// thème et le rend disponible à tous les composants enfants.
pub fn provide_theme() -> RwSignal<Theme> {
    let theme = RwSignal::new(Theme::Light);
    provide_context(theme);
    theme
}

/// À appeler depuis n'importe quelle vue pour lire/écrire le thème courant.
pub fn use_theme() -> RwSignal<Theme> {
    use_context::<RwSignal<Theme>>().expect("provide_theme() doit être appelé dans <App/>")
}
