use crate::{
    CryptoError, SignatureAlgorithm,
    classical_signature::Ed25519KeyPair,
    pq_signature::MlDsaKeyPair,
};
use serde::{Deserialize, Serialize};

/// Hybrid signature combining Ed25519 + ML-DSA-65
/// Both signatures must verify for the hybrid to be considered valid.
/// This provides security against both classical and quantum attacks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignature {
    pub classical_signature: Vec<u8>,
    pub pq_signature: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
}

/// Hybrid key pair containing both classical and post-quantum keys
#[derive(Serialize, Deserialize, Clone)]
pub struct HybridKeyPair {
    pub ed25519: Ed25519KeyPair,
    pub ml_dsa: MlDsaKeyPair,
    pub key_id: String,
}

impl HybridKeyPair {
    /// Generate a new hybrid key pair
    pub fn generate() -> Result<Self, CryptoError> {
        let ed25519 = Ed25519KeyPair::generate()?;
        let ml_dsa = MlDsaKeyPair::generate()?;

        let key_id = format!("hybrid-{}-{}", &ed25519.key_id[..8], &ml_dsa.key_id[3..11]);

        Ok(Self { ed25519, ml_dsa, key_id })
    }

    /// Create a public-key-only instance
    pub fn from_public_keys(
        ed25519_pk: &[u8],
        ml_dsa_pk: &[u8],
        key_id: &str,
    ) -> Result<Self, CryptoError> {
        let ed25519 = Ed25519KeyPair::from_public_key(ed25519_pk, &format!("{}-ed", key_id))?;
        let ml_dsa = MlDsaKeyPair::from_public_key(ml_dsa_pk, &format!("{}-pq", key_id))?;

        Ok(Self {
            ed25519,
            ml_dsa,
            key_id: key_id.to_string(),
        })
    }

    /// Sign a message with both algorithms
    /// The message is domain-separated to prevent cross-protocol attacks
    pub fn sign(&self, message: &[u8]) -> Result<HybridSignature, CryptoError> {
        // Domain separation: prefix the message differently for each algorithm
        let classical_message = Self::domain_separate(b"classical-ed25519", message);
        let pq_message = Self::domain_separate(b"post-quantum-ml-dsa-65", message);

        let classical_sig = self.ed25519.sign(&classical_message)?;
        let pq_sig = self.ml_dsa.sign(&pq_message)?;

        Ok(HybridSignature {
            classical_signature: classical_sig,
            pq_signature: pq_sig,
            algorithm: SignatureAlgorithm::HybridEd25519MlDsa65,
        })
    }

    /// Verify both signatures - BOTH must pass
    pub fn verify(&self, message: &[u8], signature: &HybridSignature) -> Result<bool, CryptoError> {
        let classical_message = Self::domain_separate(b"classical-ed25519", message);
        let pq_message = Self::domain_separate(b"post-quantum-ml-dsa-65", message);

        // Both must verify - fail if either fails
        self.ed25519.verify(&classical_message, &signature.classical_signature)?;
        self.ml_dsa.verify(&pq_message, &signature.pq_signature)?;

        Ok(true)
    }

    /// Domain separation to prevent cross-protocol attacks
    fn domain_separate(domain: &[u8], message: &[u8]) -> Vec<u8> {
        let mut separated = Vec::with_capacity(domain.len() + 1 + message.len());
        separated.extend_from_slice(domain);
        separated.push(0x00); // null separator
        separated.extend_from_slice(message);
        separated
    }
}

/// Unified signer that supports all signature modes
pub struct UnifiedSigner;

impl UnifiedSigner {
    /// Sign with the appropriate algorithm
    pub fn sign(
        algorithm: &SignatureAlgorithm,
        message: &[u8],
        ed25519_key: Option<&Ed25519KeyPair>,
        ml_dsa_key: Option<&MlDsaKeyPair>,
        hybrid_key: Option<&HybridKeyPair>,
    ) -> Result<Vec<u8>, CryptoError> {
        match algorithm {
            SignatureAlgorithm::Ed25519 => {
                let key = ed25519_key.ok_or(
                    CryptoError::InvalidKeyFormat("Ed25519 key required".into())
                )?;
                key.sign(message)
            }
            SignatureAlgorithm::MlDsa65 => {
                let key = ml_dsa_key.ok_or(
                    CryptoError::InvalidKeyFormat("ML-DSA key required".into())
                )?;
                key.sign(message)
            }
            SignatureAlgorithm::HybridEd25519MlDsa65 => {
                let key = hybrid_key.ok_or(
                    CryptoError::InvalidKeyFormat("Hybrid key required".into())
                )?;
                let hybrid_sig = key.sign(message)?;
                // Serialize the hybrid signature
                bincode::serialize(&hybrid_sig)
                    .map_err(|e| CryptoError::SerializationError(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_sign_verify() {
        let kp = HybridKeyPair::generate().unwrap();
        let message = b"hybrid security test";
        let signature = kp.sign(message).unwrap();
        assert!(kp.verify(message, &signature).unwrap());
    }

    #[test]
    fn test_hybrid_tampered_fails() {
        let kp = HybridKeyPair::generate().unwrap();
        let signature = kp.sign(b"original message").unwrap();
        assert!(kp.verify(b"tampered message", &signature).is_err());
    }

    #[test]
    fn test_hybrid_partial_signature_fails() {
        let kp = HybridKeyPair::generate().unwrap();
        let message = b"test message";
        let mut signature = kp.sign(message).unwrap();

        // Tamper with the classical signature
        signature.classical_signature[0] ^= 0xFF;
        assert!(kp.verify(message, &signature).is_err());
    }

    #[test]
    fn test_domain_separation() {
        let msg = b"test";
        let sep1 = HybridKeyPair::domain_separate(b"domain1", msg);
        let sep2 = HybridKeyPair::domain_separate(b"domain2", msg);
        assert_ne!(sep1, sep2);
    }
}