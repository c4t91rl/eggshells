// crates/server/src/publisher.rs
//! # Publisher Management
//!
//! Zarządzanie publisherami - rejestracja, weryfikacja uprawnień.

use anyhow::Result;
use secure_update_common::*;

/// Weryfikuje, czy publisher ma prawo publikować dla danej aplikacji.
/// W pełnej implementacji byłaby tu lista uprawnień.
/// W prototypie każdy zarejestrowany publisher może publikować.
pub fn verify_publisher_authorization( // ...idk anymore
    publisher: &PublisherInfo,
    _app_id: &str,
) -> Result<bool> {
    // W prototypie: każdy aktywny publisher jest autoryzowany
    Ok(publisher.active)
}

/// Weryfikuje integralność metadanych pakietu względem klucza publishera.
/// Sprawdza, czy podpis na hashu pakietu jest poprawny.
pub fn verify_package_metadata( // to add in later updates
    metadata: &PackageMetadata,
    publisher: &PublisherInfo,
    package_data: &[u8],
) -> Result<IntegrityResult> {
    // 1. Weryfikuj hash
    let computed_hash = compute_sha3_256_hex(package_data);
    let hash_valid = verify_sha3_256(package_data, &metadata.sha3_256_hash);

    if !hash_valid {
        return Ok(IntegrityResult {
            hash_valid: false,
            dilithium_valid: false,
            ed25519_valid: false,
            overall_valid: false,
            details: format!(
                "Hash mismatch: expected {}, got {}",
                metadata.sha3_256_hash, computed_hash
            ),
        });
    }

    // 2. Weryfikuj podpis hybrydowy
    let mut result = verify_hybrid_signature(
        package_data,
        &metadata.signature,
        &publisher.public_key,
    )?;

    result.hash_valid = hash_valid;
    result.overall_valid = result.overall_valid && hash_valid;

    Ok(result)
}