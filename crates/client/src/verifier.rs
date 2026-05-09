// crates/client/src/verifier.rs
//! # Package Verifier
//!
//! Weryfikacja integralności i autentyczności pobranych pakietów.
//! Implementuje pełny pipeline weryfikacji:
//! 1. Sprawdzenie rozmiaru pliku
//! 2. Weryfikacja SHA3-256
//! 3. Weryfikacja podpisu Dilithium (postkwantowy)
//! 4. Weryfikacja podpisu Ed25519 (klasyczny)
//! 5. Anti-downgrade check

use anyhow::{Context, Result, bail};
use secure_update_common::*;

/// Pełna weryfikacja pakietu aktualizacji
pub fn verify_package(
    package_data: &[u8],
    metadata: &PackageMetadata,
    publisher_key: &HybridPublicKey,
    current_version: &SemanticVersion,
) -> Result<VerificationReport> {
    let mut report = VerificationReport::default();

    // 1. Sprawdź rozmiar
    report.size_check = package_data.len() as u64 == metadata.file_size;
    if !report.size_check {
        report.errors.push(format!(
            "Size mismatch: expected {} bytes, got {} bytes",
            metadata.file_size,
            package_data.len()
        ));
    }

    // 2. Weryfikuj hash SHA3-256
    let computed_hash = compute_sha3_256_hex(package_data);
    report.hash_check = verify_sha3_256(package_data, &metadata.sha3_256_hash);
    report.computed_hash = computed_hash.clone();
    report.expected_hash = metadata.sha3_256_hash.clone();

    if !report.hash_check {
        report.errors.push(format!(
            "Hash mismatch: expected {}, computed {}",
            metadata.sha3_256_hash, computed_hash
        ));
    }

    // 3. Weryfikuj podpisy hybrydowe (Dilithium + Ed25519)
    match verify_hybrid_signature(package_data, &metadata.signature, publisher_key) {
        Ok(integrity) => {
            report.dilithium_valid = integrity.dilithium_valid;
            report.ed25519_valid = integrity.ed25519_valid;
            report.signature_details = integrity.details;

            if !integrity.dilithium_valid {
                report.errors.push("Dilithium signature INVALID".to_string());
            }
            if !integrity.ed25519_valid {
                report.errors.push("Ed25519 signature INVALID".to_string());
            }
        }
        Err(e) => {
            report.errors.push(format!("Signature verification error: {}", e));
        }
    }

    // 4. Anti-downgrade check
    report.version_check =
        SemanticVersion::is_safe_upgrade(current_version, &metadata.version);
    if !report.version_check {
        report.errors.push(format!(
            "Downgrade attempt blocked: {} → {} is not allowed",
            current_version, metadata.version
        ));
    }

    // 5. Publisher ID match
    report.publisher_check = metadata.publisher_id == publisher_key.publisher_id;
    if !report.publisher_check {
        report.errors.push(format!(
            "Publisher mismatch: metadata says '{}', key belongs to '{}'",
            metadata.publisher_id, publisher_key.publisher_id
        ));
    }

    // Overall result
    report.overall_valid = report.size_check
        && report.hash_check
        && report.dilithium_valid
        && report.ed25519_valid
        && report.version_check
        && report.publisher_check;

    Ok(report)
}

/// Raport z weryfikacji pakietu
#[derive(Debug, Clone, Default)]
pub struct VerificationReport {
    pub overall_valid: bool,
    pub size_check: bool,
    pub hash_check: bool,
    pub computed_hash: String,
    pub expected_hash: String,
    pub dilithium_valid: bool,
    pub ed25519_valid: bool,
    pub version_check: bool,
    pub publisher_check: bool,
    pub signature_details: String,
    pub errors: Vec<String>,
}

impl VerificationReport {
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("═══ Verification Report ═══"));
        lines.push(format!("  File size:      {}", status_icon(self.size_check)));
        lines.push(format!("  SHA3-256 hash:  {}", status_icon(self.hash_check)));
        lines.push(format!("  Dilithium sig:  {}", status_icon(self.dilithium_valid)));
        lines.push(format!("  Ed25519 sig:    {}", status_icon(self.ed25519_valid)));
        lines.push(format!("  Version check:  {}", status_icon(self.version_check)));
        lines.push(format!("  Publisher:      {}", status_icon(self.publisher_check)));
        lines.push(format!("  ─────────────────────"));
        lines.push(format!(
            "  OVERALL:        {}",
            if self.overall_valid {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        ));

        if !self.errors.is_empty() {
            lines.push(format!(""));
            lines.push(format!("  Errors:"));
            for err in &self.errors {
                lines.push(format!("    ⚠ {}", err));
            }
        }

        lines.join("\n")
    }
}

fn status_icon(ok: bool) -> &'static str {
    if ok {
        "✓ PASS"
    } else {
        "✗ FAIL"
    }
}