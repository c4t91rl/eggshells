// crates/client/src/config.rs
//! # Client Configuration
//!
//! Zarządzanie konfiguracją klienta - wczytywanie, zapisywanie, wartości domyślne.

use anyhow::{Context, Result};
use secure_update_common::*;
use std::path::{ PathBuf};//Path, - nieużywane

const CONFIG_FILENAME: &str = "update-client-config.json";

/// Ścieżka do pliku konfiguracji
pub fn config_path() -> PathBuf {
    // Użyj katalogu bieżącego dla prototypu
    PathBuf::from(CONFIG_FILENAME)
}

/// Wczytaj konfigurację lub utwórz domyślną
pub fn load_or_create_config() -> Result<ClientConfig> {
    let path = config_path();

    if path.exists() {
        let data = std::fs::read_to_string(&path)
            .context("Failed to read config file")?;
        let config: ClientConfig = serde_json::from_str(&data)
            .context("Failed to parse config file")?;
        Ok(config)
    } else {
        let config = ClientConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

/// Zapisz konfigurację
pub fn save_config(config: &ClientConfig) -> Result<()> {
    let path = config_path();
    let data = serde_json::to_string_pretty(config)?;
    std::fs::write(&path, data).context("Failed to write config file")?;
    Ok(())
}