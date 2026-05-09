use crate::CryptoError;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage, DetachedSignature};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// ML-DSA-65 (Dilithium3) key pair for post-quantum signatures
#[derive(Serialize, Deserialize, Clone)]
pub struct MlDsaKeyPair {
    pub public_key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_key: Option<Vec<u8>>,
    pub key_id: String,
}

impl Drop for MlDsaKeyPair {
    fn drop(&mut self) {
        if let Some(ref mut sk) = self.secret_key {
            sk.zeroize();
        }
    }
}

impl MlDsaKeyPair {
    /// Generate a new ML-DSA-65 (Dilithium3) key pair
    pub fn generate() -> Result<Self, CryptoError> {
        let (pk, sk) = dilithium3::keypair();

        let key_id = {
            let hash = blake3::hash(pk.as_bytes());
            format!("pq-{}", hex::encode(&hash.as_bytes()[..8]))
        };

        Ok(Self {
            public_key: pk.as_bytes().to_vec(),
            secret_key: Some(sk.as_bytes().to_vec()),
            key_id,
        })
    }

    /// Create a public-key-only instance
    pub fn from_public_key(public_key: &[u8], key_id: &str) -> Result<Self, CryptoError> {
        // Validate that it's a valid Dilithium3 public key
        dilithium3::PublicKey::from_bytes(public_key)
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid ML-DSA-65 public key".into()))?;

        Ok(Self {
            public_key: public_key.to_vec(),
            secret_key: None,
            key_id: key_id.to_string(),
        })
    }

    /// Sign a message (detached signature)
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let sk_bytes = self.secret_key.as_ref()
            .ok_or(CryptoError::InvalidKeyFormat("No secret key available".into()))?;

        let sk = dilithium3::SecretKey::from_bytes(sk_bytes)
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid secret key".into()))?;

        let sig = dilithium3::detached_sign(message, &sk);
        Ok(sig.as_bytes().to_vec())
    }

    /// Verify a detached signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        let pk = dilithium3::PublicKey::from_bytes(&self.public_key)
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid public key".into()))?;

        let sig = dilithium3::DetachedSignature::from_bytes(signature)
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid signature format".into()))?;

        match dilithium3::verify_detached_signature(&sig, message, &pk) {
            Ok(()) => Ok(true),
            Err(_) => Err(CryptoError::VerificationFailed),
        }
    }

    /// Public key size info (useful for UI display)
    pub fn public_key_size(&self) -> usize {
        self.public_key.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_sign_verify() {
        let kp = MlDsaKeyPair::generate().unwrap();
        let message = b"post-quantum test message";
        let signature = kp.sign(message).unwrap();
        assert!(kp.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_pq_tampered_message() {
        let kp = MlDsaKeyPair::generate().unwrap();
        let signature = kp.sign(b"original").unwrap();
        assert!(kp.verify(b"tampered", &signature).is_err());
    }
}