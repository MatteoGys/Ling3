//! icons.rs — Icônes SVG dessinées à la main (style traits fins), pour
//! remplacer les emojis dans la sidebar. Aucune dépendance externe : zéro
//! risque de conflit de version avec Leptos.
//!
//! Toutes héritent de `currentColor` (le `color` CSS du bouton parent), donc
//! elles suivent automatiquement le thème clair/sombre et l'état actif/hover
//! sans code supplémentaire.

use leptos::prelude::*;


/// --------------------------------------------------
/// Icons Onglets principaux
/// --------------------------------------------------

#[component]
pub fn IconOverlap() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_search_off.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconModification() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_swap.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconToken() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_format_list_numbered_rtl.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconConcordancier() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_playlist_add_check.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconCooccurrences() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_linear_scale.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

/// --------------------------------------------------
/// Icons Barre latérale
/// --------------------------------------------------

#[component]
pub fn IconMenu() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_menu.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconSettings() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_settings.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconTheme() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_brightness.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconInfo() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_info.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}


/// --------------------------------------------------
/// Icons Global
/// --------------------------------------------------

#[component]
pub fn IconWarning() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_error.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconCheck() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_check_circle.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}

#[component]
pub fn IconExport() -> impl IntoView {
    view! {
        <span 
            class="svg-icon" 
            inner_html=include_str!("icons/icons_file_download.svg")
            style="display: inline-flex; align-items: center; justify-content: center;"
        ></span>
    }
}