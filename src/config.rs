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
    #[serde(default = "default_hospital_name")]
    pub hospital_name: String,
    #[serde(default = "default_governing_body")]
    pub governing_body: String,
    #[serde(default = "default_archive_suffix")]
    pub archive_suffix: String,
    #[serde(default = "default_next_archive_seq")]
    pub next_archive_seq: i64,
    #[serde(default)]
    pub dark_mode: bool,
}

fn default_db_path() -> String { "ksk_qd3176.db".to_string() }
fn default_cskcb() -> String { "52208".to_string() }
fn default_gtin() -> String { "8934286053806".to_string() }
fn default_province() -> String { "52".to_string() }
fn default_ward() -> String { "21550".to_string() }
fn default_hospital_name() -> String { "BỆNH VIỆN BÌNH ĐỊNH".to_string() }
fn default_governing_body() -> String { "BỘ Y TẾ".to_string() }
fn default_archive_suffix() -> String { "GKSK-52208/26".to_string() }
fn default_next_archive_seq() -> i64 { 1 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            default_cskcb: default_cskcb(),
            default_gtin_cskcb: default_gtin(),
            default_province_code: default_province(),
            default_ward_code: default_ward(),
            hospital_name: default_hospital_name(),
            governing_body: default_governing_body(),
            archive_suffix: default_archive_suffix(),
            next_archive_seq: default_next_archive_seq(),
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
