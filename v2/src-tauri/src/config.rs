use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = "Ling3";

pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir().expect("Impossible de déterminer le dossier de données utilisateur");
    let path = base.join(APP_NAME);
    fs::create_dir_all(&path).expect("Impossible de créer le dossier de données de l'application");
    path
}

pub fn settings_file() -> PathBuf {
    app_data_dir().join("user_settings.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    #[serde(default)]
    pub textgrid_path_show: String,
    #[serde(default)]
    pub textgrid_path_files: Vec<String>,
    #[serde(default)]
    pub excel_path: String,
    #[serde(default = "default_separator")]
    pub separator: String,
}

fn default_separator() -> String {
    "\\".to_string()
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            textgrid_path_show: String::new(),
            textgrid_path_files: Vec::new(),
            excel_path: String::new(),
            separator: default_separator(),
        }
    }
}
