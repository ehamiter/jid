use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::theme::ThemeMode;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeMode,
    pub focus_mode: bool,
    pub documents_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            focus_mode: false,
            documents_dir: Self::default_documents_dir(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("jid")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("jid.toml")
    }

    fn default_documents_dir() -> PathBuf {
        dirs::document_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("jid")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config dir: {}", e);
                return;
            }
        }
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    eprintln!("Failed to write config: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize config: {}", e),
        }
    }
}
