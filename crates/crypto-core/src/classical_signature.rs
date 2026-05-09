use crate::CryptoError;
use ed25519_dalek::{
    Signer, SigningKey, Verifier, VerifyingKey, Signature,
    SECRET_KEY_LENGTH,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Ed25519 key pair for classical digital signatures
#[derive(Serialize, Deserialize, Clone)]
pub struct Ed25519KeyPair {
    pub public_key: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_key: Option<Vec<u8>>,
    pub key_id: String,
}

impl Drop for Ed25519KeyPair {
    fn drop(&mut self) {
        if let Some(ref mut sk) = self.secret_key {
            sk.zeroize();
        }
    }
}

impl Ed25519KeyPair {
    /// Generate a new Ed25519 key pair
    pub fn generate() -> Result<Self, CryptoError> {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let key_id = {
            let hash = blake3::hash(&verifying_key.to_bytes());
            hex::encode(&hash.as_bytes()[..8])
        };

        Ok(Self {
            public_key: verifying_key.to_bytes().to_vec(),
            secret_key: Some(signing_key.to_bytes().to_vec()),
            key_id,
        })
    }

    /// Create a public-key-only instance (for verification)
    pub fn from_public_key(public_key: &[u8], key_id: &str) -> Result<Self, CryptoError> {
        if public_key.len() != 32 {
            return Err(CryptoError::InvalidKeyFormat(
                "Ed25519 public key must be 32 bytes".into()
            ));
        }
        Ok(Self {
            public_key: public_key.to_vec(),
            secret_key: None,
            key_id: key_id.to_string(),
        })
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let sk_bytes = self.secret_key.as_ref()
            .ok_or(CryptoError::InvalidKeyFormat("No secret key available".into()))?;

        let mut key_bytes = [0u8; SECRET_KEY_LENGTH];
        key_bytes.copy_from_slice(sk_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        key_bytes.zeroize();

        let signature = signing_key.sign(message);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
        let pk_bytes: [u8; 32] = self.public_key.as_slice().try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid public key length".into()))?;

        let verifying_key = VerifyingKey::from_bytes(&pk_bytes)
            .map_err(|e| CryptoError::InvalidKeyFormat(e.to_string()))?;

        let sig_bytes: [u8; 64] = signature.try_into()
            .map_err(|_| CryptoError::InvalidKeyFormat("Invalid signature length".into()))?;

        let sig = Signature::from_bytes(&sig_bytes);

        match verifying_key.verify(message, &sig) {
            Ok(()) => Ok(true),
            Err(_) => Err(CryptoError::VerificationFailed),
        }
    }

    /// Export public key as base64
    pub fn public_key_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &self.public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let kp = Ed25519KeyPair::generate().unwrap();
        let message = b"test message for signing";
        let signature = kp.sign(message).unwrap();
        assert!(kp.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_wrong_message_fails() {
        let kp = Ed25519KeyPair::generate().unwrap();
        let signature = kp.sign(b"correct message").unwrap();
        assert!(kp.verify(b"wrong message", &signature).is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let kp1 = Ed25519KeyPair::generate().unwrap();
        let kp2 = Ed25519KeyPair::generate().unwrap();
        let signature = kp1.sign(b"message").unwrap();
        assert!(kp2.verify(b"message", &signature).is_err());
    }
}