use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(default = "default_db_path")]
    pub db_path: String,
    #[serde(default = "default_cskcb")]
    pub default_cskcb: String,
    #[serde(default = "default_gtin")]
    pub default_gtin_cskcb: String,
    #[serde(default = "default_province")]
    pub default_province_code: String,
    #[serde(default = "default_ward")]
    pub default_ward_code: String,
    #[serde(default)]
    pub dark_mode: bool,
}

fn default_db_path() -> String { "ksk_qd3176.db".to_string() }
fn default_cskcb() -> String { "77005".to_string() }
fn default_gtin() -> String { "8934286053806".to_string() }
fn default_province() -> String { "64".to_string() }
fn default_ward() -> String { "23992".to_string() }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            default_cskcb: default_cskcb(),
            default_gtin_cskcb: default_gtin(),
            default_province_code: default_province(),
            default_ward_code: default_ward(),
            dark_mode: false,
        }
    }
}

impl AppConfig {
    pub fn save(&self) -> Result<(), String> {
        let s = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write("config.toml", s).map_err(|e| e.to_string())
    }

    pub fn load() -> Self {
        fs::read_to_string("config.toml")
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }
}
