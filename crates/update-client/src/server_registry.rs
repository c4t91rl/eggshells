use crypto_core::key_management::{KeyStore, PublisherIdentity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Manages multiple update servers/publishers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRegistry {
    pub servers: Vec<RegisteredServer>,
    store_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredServer {
    pub url: String,
    pub publisher: PublisherIdentity,
    pub enabled: bool,
    pub last_checked: Option<chrono::DateTime<chrono::Utc>>,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    /// Fully trusted - pinned key
    Pinned,
    /// Trusted on first use (TOFU)
    TrustOnFirstUse,
    /// Manually verified
    Verified,
    /// Untrusted - needs verification
    Untrusted,
}

impl ServerRegistry {
    pub fn new(store_path: PathBuf) -> Self {
        Self {
            servers: Vec::new(),
            store_path,
        }
    }

    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let data = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::new(path.clone()))
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let data = serde_json::to_string_pretty(self)?;
        if let Some(parent) = self.store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.store_path, data)?;
        Ok(())
    }

    /// Add a new server
    pub fn add_server(&mut self, server: RegisteredServer) {
        // Check if server already exists
        if let Some(existing) = self.servers.iter_mut()
            .find(|s| s.publisher.id == server.publisher.id)
        {
            *existing = server;
        } else {
            self.servers.push(server);
        }
    }

    /// Remove a server
    pub fn remove_server(&mut self, publisher_id: &str) -> bool {
        let len = self.servers.len();
        self.servers.retain(|s| s.publisher.id != publisher_id);
        self.servers.len() < len
    }

    /// Get a server by publisher ID
    pub fn get_server(&self, publisher_id: &str) -> Option<&RegisteredServer> {
        self.servers.iter().find(|s| s.publisher.id == publisher_id)
    }

    /// Get all enabled servers
    pub fn enabled_servers(&self) -> Vec<&RegisteredServer> {
        self.servers.iter().filter(|s| s.enabled).collect()
    }

    /// Fetch server info and register (TOFU)
    pub async fn discover_and_add(&mut self, url: &str) -> anyhow::Result<RegisteredServer> {
        let client = reqwest::Client::new();
        let info_url = format!("{}/api/info", url.trim_end_matches('/'));

        let response: serde_json::Value = client
            .get(&info_url)
            .send()
            .await?
            .json()
            .await?;

        let data = response.get("data")
            .ok_or(anyhow::anyhow!("Invalid server response"))?;

        let publisher = PublisherIdentity {
            id: data["publisher_id"].as_str().unwrap_or("unknown").to_string(),
            name: data["publisher_name"].as_str().unwrap_or("Unknown").to_string(),
            description: None,
            server_url: url.to_string(),
            algorithm: serde_json::from_value(
                data.get("algorithm").cloned().unwrap_or(serde_json::json!("Ed25519"))
            ).unwrap_or(crypto_core::SignatureAlgorithm::Ed25519),
            ed25519_public_key: data.get("ed25519_public_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ml_dsa_public_key: data.get("ml_dsa_public_key")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            key_id: data["key_id"].as_str().unwrap_or("unknown").to_string(),
            created_at: chrono::Utc::now(),
        };

        let server = RegisteredServer {
            url: url.to_string(),
            publisher,
            enabled: true,
            last_checked: Some(chrono::Utc::now()),
            trust_level: TrustLevel::TrustOnFirstUse,
        };

        self.add_server(server.clone());
        self.save()?;

        Ok(server)
    }
}