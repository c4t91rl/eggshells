use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSettings,
    pub publisher: PublisherSettings,
    pub storage: StorageSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherSettings {
    pub id: String,
    pub name: String,
    pub key_file: PathBuf,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSettings {
    pub packages_dir: PathBuf,
    pub manifests_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings {
                host: "0.0.0.0".into(),
                port: 8443,
                tls_cert: None,
                tls_key: None,
            },
            publisher: PublisherSettings {
                id: "default-publisher".into(),
                name: "Default Publisher".into(),
                key_file: PathBuf::from("keys/publisher.json"),
                algorithm: "hybrid".into(),
            },
            storage: StorageSettings {
                packages_dir: PathBuf::from("data/packages"),
                manifests_dir: PathBuf::from("data/manifests"),
            },
        }
    }
}

impl ServerConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn save_default(path: &str) -> anyhow::Result<()> {
        let config = Self::default();
        let content = toml::to_string_pretty(&config)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}