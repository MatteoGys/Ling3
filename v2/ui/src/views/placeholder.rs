//! placeholder.rs — Vue générique pour les onglets pas encore convertis
//! (Tokenisation, Concordancier, Statistiques, Documentation). Évite que la
//! navigation ne pointe vers du vide pendant qu'on avance tranche par
//! tranche.

use leptos::prelude::*;

#[component]
pub fn PlaceholderView(title: String) -> impl IntoView {
    view! {
        <div class="view-container">
            <h1 class="view-title">{title}</h1>
            <hr class="divider" />
            <p class="status-text">"Cette section arrive dans une prochaine tranche."</p>
        </div>
    }
}
