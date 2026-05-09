use crypto_core::{
    CryptoError, SignatureAlgorithm, SignedManifest, ManifestSignature,
    hashing::Hasher,
    hybrid_signature::HybridSignature,
    key_management::PublisherIdentity,
    classical_signature::Ed25519KeyPair,
    pq_signature::MlDsaKeyPair,
    hybrid_signature::HybridKeyPair,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_valid: bool,
    pub checks: Vec<VerificationCheck>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

pub struct ManifestVerifier;

impl ManifestVerifier {
    /// Perform full verification of a signed manifest
    pub fn verify(
        manifest: &SignedManifest,
        publisher: &PublisherIdentity,
    ) -> VerificationResult {
        let mut result = VerificationResult {
            is_valid: true,
            checks: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Check 1: Publisher ID matches
        let publisher_match = manifest.manifest.publisher_id == publisher.id;
        result.checks.push(VerificationCheck {
            name: "Publisher ID Match".into(),
            passed: publisher_match,
            details: format!(
                "Expected: {}, Got: {}",
                publisher.id, manifest.manifest.publisher_id
            ),
        });
        if !publisher_match {
            result.is_valid = false;
            result.errors.push("Publisher ID mismatch".into());
        }

        // Check 2: Manifest has signatures
        if manifest.signatures.is_empty() {
            result.is_valid = false;
            result.checks.push(VerificationCheck {
                name: "Signatures Present".into(),
                passed: false,
                details: "No signatures found".into(),
            });
            result.errors.push("Manifest has no signatures".into());
            return result;
        }

        result.checks.push(VerificationCheck {
            name: "Signatures Present".into(),
            passed: true,
            details: format!("{} signature(s) found", manifest.signatures.len()),
        });

        // Check 3: Verify each signature
        let manifest_bytes = match serde_json::to_vec(&manifest.manifest) {
            Ok(bytes) => bytes,
            Err(e) => {
                result.is_valid = false;
                result.errors.push(format!("Cannot serialize manifest: {}", e));
                return result;
            }
        };

        for sig in &manifest.signatures {
            let sig_result = Self::verify_signature(&manifest_bytes, sig, publisher);
            let passed = sig_result.is_ok();

            result.checks.push(VerificationCheck {
                name: format!("Signature Verification ({:?})", sig.algorithm),
                passed,
                details: if passed {
                    "Signature verified successfully".into()
                } else {
                    format!("Verification failed: {}", sig_result.unwrap_err())
                },
            });

            if !passed {
                result.is_valid = false;
                result.errors.push(format!(
                    "Signature verification failed for key {}",
                    sig.key_id
                ));
            }
        }

        // Check 4: Expiration
        if let Some(expires) = manifest.manifest.expires {
            let now = chrono::Utc::now();
            let expired = now > expires;
            result.checks.push(VerificationCheck {
                name: "Manifest Expiration".into(),
                passed: !expired,
                details: format!("Expires: {}", expires),
            });
            if expired {
                result.warnings.push("Manifest has expired".into());
            }
        }

        // Check 5: Downgrade protection
        if let Some(ref prev) = manifest.manifest.previous_version {
            result.checks.push(VerificationCheck {
                name: "Version Chain".into(),
                passed: true,
                details: format!("{} -> {}", prev, manifest.manifest.version),
            });
        }

        result
    }

    /// Verify a single signature against publisher's keys
    fn verify_signature(
        manifest_bytes: &[u8],
        signature: &ManifestSignature,
        publisher: &PublisherIdentity,
    ) -> Result<(), CryptoError> {
        let sig_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &signature.signature,
        ).map_err(|e| CryptoError::SerializationError(format!("Invalid base64 signature: {}", e)))?;

        match &signature.algorithm {
            SignatureAlgorithm::Ed25519 => {
                let pk_b64 = publisher.ed25519_public_key.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No Ed25519 public key".into()))?;
                let pk_bytes = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    pk_b64,
                ).map_err(|e| CryptoError::SerializationError(e.to_string()))?;

                let key = Ed25519KeyPair::from_public_key(&pk_bytes, &signature.key_id)?;
                key.verify(manifest_bytes, &sig_bytes)?;
            }
            SignatureAlgorithm::MlDsa65 => {
                let pk_b64 = publisher.ml_dsa_public_key.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No ML-DSA public key".into()))?;
                let pk_bytes = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD,
                    pk_b64,
                ).map_err(|e| CryptoError::SerializationError(e.to_string()))?;

                let key = MlDsaKeyPair::from_public_key(&pk_bytes, &signature.key_id)?;
                key.verify(manifest_bytes, &sig_bytes)?;
            }
            SignatureAlgorithm::HybridEd25519MlDsa65 => {
                let ed_pk_b64 = publisher.ed25519_public_key.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No Ed25519 public key".into()))?;
                let pq_pk_b64 = publisher.ml_dsa_public_key.as_ref()
                    .ok_or(CryptoError::InvalidKeyFormat("No ML-DSA public key".into()))?;

                let ed_pk = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD, ed_pk_b64,
                ).map_err(|e| CryptoError::SerializationError(e.to_string()))?;
                let pq_pk = base64::Engine::decode(
                    &base64::engine::general_purpose::STANDARD, pq_pk_b64,
                ).map_err(|e| CryptoError::SerializationError(e.to_string()))?;

                let hybrid_key = HybridKeyPair::from_public_keys(
                    &ed_pk, &pq_pk, &signature.key_id,
                )?;

                // Replaced bincode with serde_json (maintained)
                let hybrid_sig: HybridSignature = serde_json::from_slice(&sig_bytes)
                    .map_err(|e| CryptoError::SerializationError(e.to_string()))?;

                hybrid_key.verify(manifest_bytes, &hybrid_sig)?;
            }
        }

        Ok(())
    }

    /// Verify downloaded file integrity
    pub fn verify_file_integrity(
        file_path: &Path,
        expected: &crypto_core::FileEntry,
    ) -> VerificationCheck {
        match Hasher::verify_file_hash(&expected.hash_algorithm, file_path, &expected.hash) {
            Ok(true) => VerificationCheck {
                name: format!("File Integrity: {}", expected.path),
                passed: true,
                details: format!("Hash verified ({:?}): {}", expected.hash_algorithm, expected.hash),
            },
            Ok(false) | Err(_) => VerificationCheck {
                name: format!("File Integrity: {}", expected.path),
                passed: false,
                details: "Hash mismatch - file may be tampered".into(),
            },
        }
    }
}