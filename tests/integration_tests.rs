// tests/integration_tests.rs
//! # Testy integracyjne
//!
//! Testują pełny przepływ: generowanie kluczy → podpisanie → weryfikacja

use secure_update_common::*;

#[test]
fn test_full_signing_and_verification_flow() {
    // 1. Publisher generuje klucze
    let keypair = HybridKeyPair::generate("test-corp").unwrap();
    let public_key = keypair.public_key();

    // 2. Publisher tworzy "pakiet"
    let package_data = b"This is a fake software update package v2.0.0";

    // 3. Publisher oblicza hash
    let hash = compute_sha3_256_hex(package_data);

    // 4. Publisher podpisuje pakiet
    let signature = keypair.sign(package_data).unwrap();

    // 5. Klient weryfikuje
    let result = verify_hybrid_signature(package_data, &signature, &public_key).unwrap();
    assert!(result.overall_valid, "Verification should pass for genuine package");
    assert!(result.dilithium_valid);
    assert!(result.ed25519_valid);

    // 6. Weryfikuj hash
    assert!(verify_sha3_256(package_data, &hash));
}

#[test]
fn test_key_export_import_roundtrip() {
    let original = HybridKeyPair::generate("roundtrip-test").unwrap();
    let exported = original.export_secret_keys().unwrap();
    let imported = HybridKeyPair::import_secret_keys(&exported).unwrap();

    let data = b"Roundtrip test data";
    let sig = imported.sign(data).unwrap();
    let pk = original.public_key();

    let result = verify_hybrid_signature(data, &sig, &pk).unwrap();
    assert!(result.overall_valid);
}

#[test]
fn test_version_anti_downgrade() {
    let current = SemanticVersion::new(2, 0, 0);
    let older = SemanticVersion::new(1, 9, 9);
    let same = SemanticVersion::new(2, 0, 0);
    let newer = SemanticVersion::new(2, 0, 1);

    assert!(!SemanticVersion::is_safe_upgrade(&current, &older), "Downgrade should be blocked");
    assert!(!SemanticVersion::is_safe_upgrade(&current, &same), "Same version should be blocked");
    assert!(SemanticVersion::is_safe_upgrade(&current, &newer), "Upgrade should be allowed");
}

#[test]
fn test_multiple_publishers() {
    let publisher_a = HybridKeyPair::generate("company-a").unwrap();
    let publisher_b = HybridKeyPair::generate("company-b").unwrap();

    let data = b"Software package";

    // A podpisuje
    let sig_a = publisher_a.sign(data).unwrap();

    // Weryfikacja kluczem A → OK
    let result = verify_hybrid_signature(data, &sig_a, &publisher_a.public_key()).unwrap();
    assert!(result.overall_valid);

    // Weryfikacja kluczem B → FAIL (inny publisher)
    let result = verify_hybrid_signature(data, &sig_a, &publisher_b.public_key()).unwrap();
    assert!(!result.overall_valid, "Wrong publisher key should fail");
}