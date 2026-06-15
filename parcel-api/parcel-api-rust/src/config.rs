use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_context_path")]
    pub context_path: String,
    #[serde(default = "default_parcel_data_dir")]
    pub parcel_data_dir: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_port: default_server_port(),
            context_path: default_context_path(),
            parcel_data_dir: default_parcel_data_dir(),
        }
    }
}

fn default_server_port() -> u16 {
    8080
}

fn default_context_path() -> String {
    "/parcel-api".to_string()
}

fn default_parcel_data_dir() -> PathBuf {
    PathBuf::from("../parcel-data")
}

impl AppConfig {
    pub fn load() -> Self {
        let profile = std::env::var("APP_PROFILE").unwrap_or_else(|_| "dev".to_string());
        Self::load_with_profile(&profile)
    }

    pub fn load_with_profile(profile: &str) -> Self {
        let config_dir = std::env::var("APP_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("config"));
        let path = config_dir.join(format!("{profile}.toml"));

        let mut figment = Figment::from(Serialized::defaults(AppConfig::default()));
        if path.exists() {
            figment = figment.merge(Toml::file(&path));
        }
        figment = figment.merge(Env::prefixed("APP_").split("__"));

        figment.extract().unwrap_or_else(|err| {
            eprintln!("config load failed; using defaults: {err}");
            AppConfig::default()
        })
    }
}
