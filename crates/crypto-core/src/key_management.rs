use crate::{
    CryptoError, SignatureAlgorithm,
    classical_signature::Ed25519KeyPair,
    pq_signature::MlDsaKeyPair,
    hybrid_signature::HybridKeyPair,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a publisher's identity and keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherIdentity {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub server_url: String,
    pub algorithm: SignatureAlgorithm,
    /// Ed25519 public key (base64)
    pub ed25519_public_key: Option<String>,
    /// ML-DSA-65 public key (base64)
    pub ml_dsa_public_key: Option<String>,
    pub key_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Publisher key store - manages trusted publisher keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStore {
    pub publishers: HashMap<String, PublisherIdentity>,
    pub revoked_keys: Vec<String>,
}

impl KeyStore {
    pub fn new() -> Self {
        Self {
            publishers: HashMap::new(),
            revoked_keys: Vec::new(),
        }
    }

    /// Load keystore from file
    pub fn load(path: &Path) -> Result<Self, CryptoError> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| CryptoError::SerializationError(format!("Cannot read keystore: {}", e)))?;
        serde_json::from_str(&data)
            .map_err(|e| CryptoError::SerializationError(format!("Invalid keystore format: {}", e)))
    }

    /// Save keystore to file
    pub fn save(&self, path: &Path) -> Result<(), CryptoError> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        std::fs::write(path, data)
            .map_err(|e| CryptoError::SerializationError(format!("Cannot write keystore: {}", e)))
    }

    /// Add a trusted publisher
    pub fn add_publisher(&mut self, identity: PublisherIdentity) {
        self.publishers.insert(identity.id.clone(), identity);
    }

    /// Remove a publisher
    pub fn remove_publisher(&mut self, publisher_id: &str) -> Option<PublisherIdentity> {
        self.publishers.remove(publisher_id)
    }

    /// Get a publisher's identity
    pub fn get_publisher(&self, publisher_id: &str) -> Option<&PublisherIdentity> {
        self.publishers.get(publisher_id)
    }

    /// Revoke a key
    pub fn revoke_key(&mut self, key_id: &str) {
        self.revoked_keys.push(key_id.to_string());
    }

    /// Check if a key is revoked
    pub fn is_key_revoked(&self, key_id: &str) -> bool {
        self.revoked_keys.contains(&key_id.to_string())
    }

    /// List all trusted publishers
    pub fn list_publishers(&self) -> Vec<&PublisherIdentity> {
        self.publishers.values().collect()
    }
}

/// Full publisher key material (includes private keys - server side only)
#[derive(Serialize, Deserialize)]
pub struct PublisherKeyMaterial {
    pub identity: PublisherIdentity,
    pub ed25519_keypair: Option<Ed25519KeyPair>,
    pub ml_dsa_keypair: Option<MlDsaKeyPair>,
    pub hybrid_keypair: Option<HybridKeyPair>,
}

impl PublisherKeyMaterial {
    /// Generate new publisher key material
    pub fn generate(
        id: &str,
        name: &str,
        server_url: &str,
        algorithm: SignatureAlgorithm,
    ) -> Result<Self, CryptoError> {
        let (ed25519_keypair, ml_dsa_keypair, hybrid_keypair, ed25519_pk, ml_dsa_pk, key_id) =
            match &algorithm {
                SignatureAlgorithm::Ed25519 => {
                    let kp = Ed25519KeyPair::generate()?;
                    let pk = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &kp.public_key,
                    );
                    let kid = kp.key_id.clone();
                    (Some(kp), None, None, Some(pk), None, kid)
                }
                SignatureAlgorithm::MlDsa65 => {
                    let kp = MlDsaKeyPair::generate()?;
                    let pk = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &kp.public_key,
                    );
                    let kid = kp.key_id.clone();
                    (None, Some(kp), None, None, Some(pk), kid)
                }
                SignatureAlgorithm::HybridEd25519MlDsa65 => {
                    let kp = HybridKeyPair::generate()?;
                    let ed_pk = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &kp.ed25519.public_key,
                    );
                    let pq_pk = base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &kp.ml_dsa.public_key,
                    );
                    let kid = kp.key_id.clone();
                    (None, None, Some(kp), Some(ed_pk), Some(pq_pk), kid)
                }
            };

        let identity = PublisherIdentity {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            server_url: server_url.to_string(),
            algorithm,
            ed25519_public_key: ed25519_pk,
            ml_dsa_public_key: ml_dsa_pk,
            key_id,
            created_at: chrono::Utc::now(),
        };

        Ok(Self {
            identity,
            ed25519_keypair,
            ml_dsa_keypair,
            hybrid_keypair,
        })
    }

    /// Save private key material to file (encrypted in production)
    pub fn save_private(&self, path: &Path) -> Result<(), CryptoError> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        std::fs::write(path, data)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))
    }

    /// Load private key material
    pub fn load_private(path: &Path) -> Result<Self, CryptoError> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        serde_json::from_str(&data)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))
    }
}