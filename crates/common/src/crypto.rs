// crates/common/src/crypto.rs
//! # Moduł kryptograficzny
//!
//! Implementuje hybrydowy schemat podpisów cyfrowych:
//! - **CRYSTALS-Dilithium (Level 3)**: postkwantowy podpis cyfrowy
//! - **Ed25519**: klasyczny podpis cyfrowy
//! - **SHA3-256**: funkcja skrótu dla integralności
//!
//! ## Hybrid Signature
//! Każdy pakiet jest podpisywany OBIE algorytmami. Weryfikacja wymaga,
//! aby OBA podpisy były poprawne. Zapewnia to bezpieczeństwo zarówno
//! przeciwko zagrożeniom klasycznym, jak i kwantowym.

use anyhow::{Context, Result, bail};
use ed25519_dalek::{
    Signer, SigningKey as Ed25519SigningKey, Verifier,
    VerifyingKey as Ed25519VerifyingKey, Signature as Ed25519Signature,
};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{
    PublicKey as PqPublicKey, SecretKey as PqSecretKey,
    SignedMessage, DetachedSignature,
};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Para kluczy Dilithium (postkwantowe)
#[derive(Clone)]
pub struct DilithiumKeyPair {
    pub public_key: dilithium3::PublicKey,
    pub secret_key: dilithium3::SecretKey,
}

/// Para kluczy Ed25519 (klasyczne)
#[derive(Clone)]
pub struct Ed25519KeyPair {
    pub signing_key: Ed25519SigningKey,
    pub verifying_key: Ed25519VerifyingKey,
}

/// Pełna para kluczy publishera (hybrydowa)
#[derive(Clone)]
pub struct HybridKeyPair {
    pub dilithium: DilithiumKeyPair,
    pub ed25519: Ed25519KeyPair,
    pub publisher_id: String,
}

/// Klucz publiczny publishera do dystrybucji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridPublicKey {
    pub publisher_id: String,
    /// Klucz publiczny Dilithium zakodowany w base64
    pub dilithium_public_key: String,
    /// Klucz publiczny Ed25519 zakodowany w base64
    pub ed25519_public_key: String,
}

/// Hybrydowy podpis cyfrowy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSignature {
    /// Podpis Dilithium zakodowany w base64
    pub dilithium_signature: String,
    /// Podpis Ed25519 zakodowany w base64
    pub ed25519_signature: String,
    /// ID publishera który podpisał
    pub publisher_id: String,
    /// Timestamp podpisu (ISO 8601)
    pub signed_at: String,
}

/// Wynik weryfikacji integralności
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityResult {
    pub hash_valid: bool,
    pub dilithium_valid: bool,
    pub ed25519_valid: bool,
    pub overall_valid: bool,
    pub details: String,
}

// ============================================================
// Generowanie kluczy
// ============================================================

impl HybridKeyPair {
    /// Generuje nową hybrydową parę kluczy dla publishera.
    ///
    /// Używa kryptograficznie bezpiecznego generatora liczb losowych (OsRng).
    /// Dilithium3 zapewnia bezpieczeństwo postkwantowe na poziomie NIST Level 3.
    /// Ed25519 zapewnia bezpieczeństwo klasyczne na poziomie ~128 bit.
    pub fn generate(publisher_id: &str) -> Result<Self> {
        // Generowanie kluczy Dilithium (postkwantowe)
        let (dil_pk, dil_sk) = dilithium3::keypair();

        // Generowanie kluczy Ed25519 (klasyczne)
        let ed_sk = Ed25519SigningKey::generate(&mut OsRng);
        let ed_pk = ed_sk.verifying_key();

        Ok(Self {
            dilithium: DilithiumKeyPair {
                public_key: dil_pk,
                secret_key: dil_sk,
            },
            ed25519: Ed25519KeyPair {
                signing_key: ed_sk,
                verifying_key: ed_pk,
            },
            publisher_id: publisher_id.to_string(),
        })
    }

    /// Eksportuje klucz publiczny do formatu nadającego się do dystrybucji
    pub fn public_key(&self) -> HybridPublicKey {
        use pqcrypto_traits::sign::PublicKey;

        HybridPublicKey {
            publisher_id: self.publisher_id.clone(),
            dilithium_public_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                self.dilithium.public_key.as_bytes(),
            ),
            ed25519_public_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                self.ed25519.verifying_key.as_bytes(),
            ),
        }
    }

    /// Podpisuje dane hybrydowo (Dilithium + Ed25519)
    pub fn sign(&self, data: &[u8]) -> Result<HybridSignature> {
        // 1. Oblicz hash SHA3-256 danych
        let hash = compute_sha3_256(data);

        // 2. Podpis Dilithium na hashu
        let dil_sig = dilithium3::detached_sign(&hash, &self.dilithium.secret_key);

        // 3. Podpis Ed25519 na hashu
        let ed_sig = self.ed25519.signing_key.sign(&hash);

        use pqcrypto_traits::sign::DetachedSignature;

        Ok(HybridSignature {
            dilithium_signature: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                dil_sig.as_bytes(),
            ),
            ed25519_signature: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ed_sig.to_bytes(),
            ),
            publisher_id: self.publisher_id.clone(),
            signed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Serializuje klucze prywatne do zapisu na dysk (UWAGA: chronić!)
    pub fn export_secret_keys(&self) -> Result<Vec<u8>> {
        use pqcrypto_traits::sign::SecretKey;

        let data = ExportedKeyPair {
            publisher_id: self.publisher_id.clone(),
            dilithium_secret_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                self.dilithium.secret_key.as_bytes(),
            ),
            dilithium_public_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                self.dilithium.public_key.as_bytes(),
            ),
            ed25519_secret_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                self.ed25519.signing_key.to_bytes(),
            ),
        };
        serde_json::to_vec_pretty(&data).context("Failed to serialize keys")
    }

    /// Importuje klucze prywatne z danych
    pub fn import_secret_keys(data: &[u8]) -> Result<Self> {
        let exported: ExportedKeyPair =
            serde_json::from_slice(data).context("Failed to deserialize keys")?;

        let dil_sk_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &exported.dilithium_secret_key,
        )?;
        let dil_pk_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &exported.dilithium_public_key,
        )?;
        let ed_sk_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &exported.ed25519_secret_key,
        )?;

        let dil_sk = dilithium3::SecretKey::from_bytes(&dil_sk_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid Dilithium secret key: {:?}", e))?;
        let dil_pk = dilithium3::PublicKey::from_bytes(&dil_pk_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid Dilithium public key: {:?}", e))?;

        let ed_sk_array: [u8; 32] = ed_sk_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid Ed25519 key length"))?;
        let ed_sk = Ed25519SigningKey::from_bytes(&ed_sk_array);
        let ed_pk = ed_sk.verifying_key();

        Ok(Self {
            dilithium: DilithiumKeyPair {
                public_key: dil_pk,
                secret_key: dil_sk,
            },
            ed25519: Ed25519KeyPair {
                signing_key: ed_sk,
                verifying_key: ed_pk,
            },
            publisher_id: exported.publisher_id,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct ExportedKeyPair {
    publisher_id: String,
    dilithium_secret_key: String,
    dilithium_public_key: String,
    ed25519_secret_key: String,
}

// ============================================================
// Weryfikacja podpisów
// ============================================================

/// Weryfikuje hybrydowy podpis dla danych.
///
/// Oba podpisy (Dilithium I Ed25519) muszą być poprawne.
/// Używa logiki AND - kompromitacja jednego algorytmu nie wystarcza.
pub fn verify_hybrid_signature(
    data: &[u8],
    signature: &HybridSignature,
    public_key: &HybridPublicKey,
) -> Result<IntegrityResult> {
    let hash = compute_sha3_256(data);

    // 1. Weryfikuj Dilithium
    let dil_sig_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &signature.dilithium_signature,
    ).context("Failed to decode Dilithium signature")?;

    let dil_pk_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &public_key.dilithium_public_key,
    ).context("Failed to decode Dilithium public key")?;

    let dil_pk = dilithium3::PublicKey::from_bytes(&dil_pk_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid Dilithium public key: {:?}", e))?;

    let dil_sig = dilithium3::DetachedSignature::from_bytes(&dil_sig_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid Dilithium signature: {:?}", e))?;

    let dilithium_valid = dilithium3::verify_detached_signature(&dil_sig, &hash, &dil_pk).is_ok();

    // 2. Weryfikuj Ed25519
    let ed_sig_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &signature.ed25519_signature,
    ).context("Failed to decode Ed25519 signature")?;

    let ed_pk_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &public_key.ed25519_public_key,
    ).context("Failed to decode Ed25519 public key")?;

    let ed_pk_array: [u8; 32] = ed_pk_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key length"))?;

    let ed_pk = Ed25519VerifyingKey::from_bytes(&ed_pk_array)
        .map_err(|e| anyhow::anyhow!("Invalid Ed25519 public key: {}", e))?;

    let ed_sig_array: [u8; 64] = ed_sig_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 signature length"))?;

    let ed_sig = Ed25519Signature::from_bytes(&ed_sig_array);
    let ed25519_valid = ed_pk.verify(&hash, &ed_sig).is_ok();

    // 3. Oba muszą być poprawne (AND logic)
    let overall_valid = dilithium_valid && ed25519_valid;

    let details = format!(
        "Dilithium3: {} | Ed25519: {} | Overall: {}",
        if dilithium_valid { "✓ VALID" } else { "✗ INVALID" },
        if ed25519_valid { "✓ VALID" } else { "✗ INVALID" },
        if overall_valid { "✓ PASSED" } else { "✗ FAILED" },
    );

    Ok(IntegrityResult {
        hash_valid: true, // Hash jest obliczany wewnętrznie
        dilithium_valid,
        ed25519_valid,
        overall_valid,
        details,
    })
}

// ============================================================
// Hashing
// ============================================================

/// Oblicza SHA3-256 hash danych.
///
/// SHA3 (Keccak) jest odporny na ataki length-extension
/// i jest uważany za bezpieczny również w kontekście kwantowym
/// (algorytm Grovera daje tylko kwadratowe przyspieszenie,
/// więc SHA3-256 zapewnia ~128 bit bezpieczeństwa post-quantum).
pub fn compute_sha3_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Oblicza SHA3-256 hash i zwraca jako hex string
pub fn compute_sha3_256_hex(data: &[u8]) -> String {
    hex::encode(compute_sha3_256(data))
}

/// Weryfikuje SHA3-256 hash danych
pub fn verify_sha3_256(data: &[u8], expected_hex: &str) -> bool {
    let computed = compute_sha3_256_hex(data);
    // Porównanie w stałym czasie (constant-time comparison)
    // aby zapobiec timing attacks
    constant_time_eq(computed.as_bytes(), expected_hex.as_bytes())
}

/// Porównanie w stałym czasie zapobiegające timing attacks
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_signing() {
        let keypair = HybridKeyPair::generate("test-publisher").unwrap();
        let data = b"Hello, secure update system!";
        let signature = keypair.sign(data).unwrap();
        let public_key = keypair.public_key();

        let result = verify_hybrid_signature(data, &signature, &public_key).unwrap();
        assert!(result.overall_valid);
        assert!(result.dilithium_valid);
        assert!(result.ed25519_valid);
    }

    #[test]
    fn test_tampered_data_fails_verification() {
        let keypair = HybridKeyPair::generate("test-publisher").unwrap();
        let data = b"Original data";
        let signature = keypair.sign(data).unwrap();
        let public_key = keypair.public_key();

        let tampered = b"Tampered data";
        let result = verify_hybrid_signature(tampered, &signature, &public_key).unwrap();
        assert!(!result.overall_valid);
    }

    #[test]
    fn test_wrong_publisher_key_fails() {
        let keypair1 = HybridKeyPair::generate("publisher-a").unwrap();
        let keypair2 = HybridKeyPair::generate("publisher-b").unwrap();
        let data = b"Some data";
        let signature = keypair1.sign(data).unwrap();
        let wrong_public_key = keypair2.public_key();

        let result = verify_hybrid_signature(data, &signature, &wrong_public_key).unwrap();
        assert!(!result.overall_valid);
    }

    #[test]
    fn test_sha3_256() {
        let data = b"test data";
        let hash = compute_sha3_256_hex(data);
        assert!(verify_sha3_256(data, &hash));
        assert!(!verify_sha3_256(b"other data", &hash));
    }

    #[test]
    fn test_key_export_import() {
        let keypair = HybridKeyPair::generate("export-test").unwrap();
        let exported = keypair.export_secret_keys().unwrap();
        let imported = HybridKeyPair::import_secret_keys(&exported).unwrap();

        let data = b"Test roundtrip";
        let sig = imported.sign(data).unwrap();
        let pk = keypair.public_key();
        let result = verify_hybrid_signature(data, &sig, &pk).unwrap();
        assert!(result.overall_valid);
    }
}