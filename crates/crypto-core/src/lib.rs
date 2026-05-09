pub mod hybrid_signature;
pub mod pq_signature;
pub mod classical_signature;
pub mod hashing;
pub mod key_management;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
    #[error("Expired signature: signed at {signed_at}, expired at {expired_at}")]
    ExpiredSignature { signed_at: String, expired_at: String },
}

/// Supported signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    /// Classical Ed25519
    Ed25519,
    /// Post-quantum ML-DSA (Dilithium3)
    MlDsa65,
    /// Hybrid: Ed25519 + ML-DSA-65 (both must verify)
    HybridEd25519MlDsa65,
}

/// Supported hash algorithms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HashAlgorithm {
    Sha3_256,
    Sha3_512,
    Blake3,
}

/// A signed artifact's metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedManifest {
    pub manifest: UpdateManifest,
    pub signatures: Vec<ManifestSignature>,
}

/// Update manifest describing an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub package_name: String,
    pub version: String,
    pub previous_version: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    pub files: Vec<FileEntry>,
    pub minimum_client_version: Option<String>,
    pub release_notes: Option<String>,
    pub publisher_id: String,
}

/// A file within an update package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub hash_algorithm: HashAlgorithm,
    pub hash: String,
    pub download_url: String,
}

/// A signature over a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSignature {
    pub algorithm: SignatureAlgorithm,
    pub publisher_id: String,
    pub key_id: String,
    pub signature: String,  // base64-encoded
    pub signed_at: chrono::DateTime<chrono::Utc>,
}