//! settings.rs — Gestion de la configuration utilisateur.
//! Remplace backend/settings_manager.py (+ le rôle de backend/__init__.py,
//! voir la réponse à la question 3 : Rust n'a pas besoin d'un fichier "d'init").

use std::fs;
use std::sync::Mutex;
use tauri::State;

use crate::config::{self, ProfileData};
use crate::models::AppSettings;

/// État global partagé, injecté dans toutes les commandes via `.manage(...)`
/// dans main.rs. Un seul verrou en mémoire, sauvegardé sur disque à chaque
/// modification — équivalent du `self.settings` + `self.save()` de Python.
pub struct SettingsState(pub Mutex<AppSettings>);

impl SettingsState {
    /// Charge le fichier JSON s'il existe, sinon démarre avec un profil
    /// "Default" vide (même logique que SettingsManager._load()).
    pub fn load() -> Self {
        let settings = fs::read_to_string(config::settings_file())
            .ok()
            .and_then(|content| serde_json::from_str::<AppSettings>(&content).ok())
            .unwrap_or_default();
        Self(Mutex::new(settings))
    }
}

fn save(settings: &AppSettings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(config::settings_file(), json).map_err(|e| e.to_string())
}

/// Retourne (en la créant si besoin) les données du profil actuellement actif.
fn current_profile_mut(settings: &mut AppSettings) -> &mut ProfileData {
    let key = settings.current_profile.clone();
    settings.profiles.entry(key).or_insert_with(ProfileData::default)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_current_data(state: State<'_, SettingsState>) -> ProfileData {
    let settings = state.0.lock().unwrap();
    settings
        .profiles
        .get(&settings.current_profile)
        .cloned()
        .unwrap_or_default()
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_textgrid_path_show(state: State<'_, SettingsState>, path: String) -> Result<(), String> {
    let mut settings = state.0.lock().unwrap();
    current_profile_mut(&mut settings).textgrid_path_show = path;
    save(&settings)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_textgrid_path_files(state: State<'_, SettingsState>, paths: Vec<String>) -> Result<(), String> {
    let mut settings = state.0.lock().unwrap();
    current_profile_mut(&mut settings).textgrid_path_files = paths;
    save(&settings)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_excel_path(state: State<'_, SettingsState>, path: String) -> Result<(), String> {
    let mut settings = state.0.lock().unwrap();
    current_profile_mut(&mut settings).excel_path = path;
    save(&settings)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_separator(state: State<'_, SettingsState>, separator: String) -> Result<(), String> {
    let mut settings = state.0.lock().unwrap();
    current_profile_mut(&mut settings).separator = separator;
    save(&settings)
}
