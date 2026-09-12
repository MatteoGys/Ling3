//! lib.rs — cœur natif de l'application Tauri.
//! C'est ICI que vit tout ce qui était dans main.rs dans les tranches
//! précédentes. Le main.rs généré par votre scaffold reste inchangé : il se
//! contente d'appeler `run()` ci-dessous (voir mon message pour vérifier son
//! contenu exact).

mod commands;
mod config;
mod excel;
mod models;
mod settings;
mod textgrid;

use commands::StatsState;
use settings::SettingsState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugin "ouvrir avec l'app système" — présent par défaut dans votre
        // scaffold, remis ici car capabilities/default.json le référence.
        .plugin(tauri_plugin_opener::init())
        // Plugin de dialogues natifs (sélection de fichiers/dossiers),
        // utilisé par la vue Paramètres pour remplacer les FilePicker Flet.
        .plugin(tauri_plugin_dialog::init())
        // Équivalent de `sm = SettingsManager()` dans main.py : chargé une
        // fois au démarrage, puis injecté dans toutes les commandes.
        .manage(SettingsState::load())
        // Cache en mémoire des tokens ordonnés pour l'onglet Statistiques
        // (évite de re-parser tous les fichiers à chaque clic "Analyser").
        .manage(StatsState::default())
        .invoke_handler(tauri::generate_handler![
            settings::get_current_data,
            settings::set_textgrid_path_show,
            settings::set_textgrid_path_files,
            settings::set_excel_path,
            settings::set_separator,
            commands::analyze_overlaps,
            commands::apply_text_modifications,
            commands::apply_pivot_extraction,
            commands::convert_trs_files,
            commands::load_tokenisation_data,
            commands::load_stats_data,
            commands::run_stats_analysis,
            commands::export_table_excel,
            commands::generate_concordance_excel,
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de l'application Tauri");
}
