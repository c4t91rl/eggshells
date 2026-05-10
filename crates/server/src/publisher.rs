use anyhow::Result;
use secure_update_common::*;

/// Sprawdza czy publisher jest aktywny
pub fn verify_publisher_authorization(
    publisher: &PublisherInfo,
    _app_id: &str,
) -> Result<bool> {
    Ok(publisher.active)
}

/// Weryfikuje pakiet PRZED zapisaniem metadanych na serwerze
/// Sprawdza: rozmiar, SHA3-256, podpis Dilithium3, podpis Ed25519
pub fn verify_package_on_publish(
    metadata: &PackageMetadata,
    package_data: &[u8],
    publisher: &PublisherInfo,
) -> Result<IntegrityResult> {
    // 1. Rozmiar
    if package_data.len() as u64 != metadata.file_size {
        return Ok(IntegrityResult {
            hash_valid: false,
            dilithium_valid: false,
            ed25519_valid: false,
            overall_valid: false,
            details: format!(
                "Size mismatch: metadata says {} bytes, file is {} bytes",
                metadata.file_size,
                package_data.len()
            ),
        });
    }

    // 2. SHA3-256
    let computed_hash = compute_sha3_256_hex(package_data);
    if !verify_sha3_256(package_data, &metadata.sha3_256_hash) {
        return Ok(IntegrityResult {
            hash_valid: false,
            dilithium_valid: false,
            ed25519_valid: false,
            overall_valid: false,
            details: format!(
                "Hash mismatch: metadata says {}, computed {}",
                metadata.sha3_256_hash, computed_hash
            ),
        });
    }

    // 3. Podpis hybrydowy (Dilithium3 + Ed25519)
    let mut result = verify_hybrid_signature(
        package_data,
        &metadata.signature,
        &publisher.public_key,
    )?;

    result.hash_valid = true;
    result.overall_valid = result.dilithium_valid && result.ed25519_valid && result.hash_valid;

    Ok(result)
}