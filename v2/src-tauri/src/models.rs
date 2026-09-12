use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::config::ProfileData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub current_profile: String,
    pub profiles: HashMap<String, ProfileData>,
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("Default".to_string(), ProfileData::default());
        Self {
            current_profile: "Default".to_string(),
            profiles,
        }
    }
}
