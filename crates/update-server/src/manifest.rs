use crypto_core::{
    CryptoError, HashAlgorithm, SignatureAlgorithm,
    SignedManifest, UpdateManifest, FileEntry, ManifestSignature,
    hashing::Hasher,
    key_management::PublisherKeyMaterial,
};
use std::path::Path;

pub struct ManifestBuilder;

impl ManifestBuilder {
    /// Create and sign a manifest for a package
    pub fn create_signed_manifest(
        package_name: &str,
        version: &str,
        previous_version: Option<&str>,
        files: Vec<(String, &Path, String)>,  // (relative_path, file_path, download_url)
        publisher_keys: &PublisherKeyMaterial,
        release_notes: Option<&str>,
    ) -> Result<SignedManifest, CryptoError> {
        // Build file entries with hashes
        let file_entries: Result<Vec<FileEntry>, CryptoError> = files.iter().map(|(rel_path, file_path, url)| {
            let hash = Hasher::hash_file(&HashAlgorithm::Blake3, file_path)?;
            let size = std::fs::metadata(file_path)
                .map_err(|e| CryptoError::SerializationError(e.to_string()))?
                .len();

            Ok(FileEntry {
                path: rel_path.clone(),
                size,
                hash_algorithm: HashAlgorithm::Blake3,
                hash,
                download_url: url.clone(),
            })
        }).collect();

        let manifest = UpdateManifest {
            package_name: package_name.to_string(),
            version: version.to_string(),
            previous_version: previous_version.map(|s| s.to_string()),
            timestamp: chrono::Utc::now(),
            expires: Some(chrono::Utc::now() + chrono::Duration::days(30)),
            files: file_entries?,
            minimum_client_version: None,
            release_notes: release_notes.map(|s| s.to_string()),
            publisher_id: publisher_keys.identity.id.clone(),
        };

        // Serialize manifest for signing
        let manifest_bytes = serde_json::to_vec(&manifest)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;

        // Sign the manifest
        let signature = Self::sign_manifest(&manifest_bytes, publisher_keys)?;

        Ok(SignedManifest {
            manifest,
            signatures: vec![signature],
        })
    }

    fn sign_manifest(
        manifest_bytes: &[u8],
        keys: &PublisherKeyMaterial,
    ) -> Result<ManifestSignature, CryptoError> {
        let (algorithm, sig_bytes) = match &keys.identity.algorithm {
            SignatureAlgorithm::Ed25519 => {
                let kp = keys.ed25519_keypair.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No Ed25519 key".into()))?;
                (SignatureAlgorithm::Ed25519, kp.sign(manifest_bytes)?)
            }
            SignatureAlgorithm::MlDsa65 => {
                let kp = keys.ml_dsa_keypair.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No ML-DSA key".into()))?;
                (SignatureAlgorithm::MlDsa65, kp.sign(manifest_bytes)?)
            }
            SignatureAlgorithm::HybridEd25519MlDsa65 => {
                let kp = keys.hybrid_keypair.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No hybrid key".into()))?;
                let hybrid_sig = kp.sign(manifest_bytes)?;
                let sig_bytes = bincode::serialize(&hybrid_sig)
                    .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
                (SignatureAlgorithm::HybridEd25519MlDsa65, sig_bytes)
            }
        };

        Ok(ManifestSignature {
            algorithm,
            publisher_id: keys.identity.id.clone(),
            key_id: keys.identity.key_id.clone(),
            signature: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &sig_bytes,
            ),
            signed_at: chrono::Utc::now(),
        })
    }
}