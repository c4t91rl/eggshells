// tests/attack_scenarios.rs
//! # Symulacja scenariuszy ataków
//!
//! Testy demonstrujące odporność systemu na różne ataki.

use secure_update_common::*;

#[test]
fn attack_tampering() {
    //! Scenariusz: atakujący modyfikuje pakiet po podpisaniu
    let publisher = HybridKeyPair::generate("legit-publisher").unwrap();
    let original = b"Original legitimate software package";
    let signature = publisher.sign(original).unwrap();
    let pk = publisher.public_key();

    // Atakujący modyfikuje jeden bajt
    let mut tampered = original.to_vec();
    tampered[0] ^= 0xFF; // Flip bits

    let result = verify_hybrid_signature(&tampered, &signature, &pk).unwrap();
    assert!(
        !result.overall_valid,
        "ATTACK BLOCKED: Tampered package rejected"
    );
}

#[test]
fn attack_signature_substitution() {
    //! Scenariusz: atakujący podmienia podpis na swój
    let legit_publisher = HybridKeyPair::generate("legit").unwrap();
    let attacker = HybridKeyPair::generate("attacker").unwrap();

    let legit_data = b"Legitimate package";
    let malicious_data = b"Malicious package with backdoor";

    // Atakujący podpisuje złośliwy pakiet swoim kluczem
    let attacker_sig = attacker.sign(malicious_data).unwrap();

    // Ale weryfikacja odbywa się kluczem legalnego publishera
    let result = verify_hybrid_signature(
        malicious_data,
        &attacker_sig,
        &legit_publisher.public_key(),
    )
    .unwrap();

    assert!(
        !result.overall_valid,
        "ATTACK BLOCKED: Signature from wrong publisher rejected"
    );
}

#[test]
fn attack_downgrade() {
    //! Scenariusz: atakujący próbuje wymusić instalację starszej wersji
    let current = SemanticVersion::new(3, 0, 0);
    let old_vulnerable = SemanticVersion::new(1, 0, 0);

    assert!(
        !SemanticVersion::is_safe_upgrade(&current, &old_vulnerable),
        "ATTACK BLOCKED: Downgrade from 3.0.0 to 1.0.0 rejected"
    );
}

#[test]
fn attack_replay() {
    //! Scenariusz: atakujący ponownie dostarcza stary, ale poprawnie podpisany pakiet
    //! Ochrona: version check zapobiega ponownemu zainstalowaniu
    let current = SemanticVersion::new(2, 0, 0);
    let replayed = SemanticVersion::new(2, 0, 0); // Ta sama wersja

    assert!(
        !SemanticVersion::is_safe_upgrade(&current, &replayed),
        "ATTACK BLOCKED: Replay of same version rejected"
    );
}

#[test]
fn attack_hash_collision_attempt() {
    //! Scenariusz: dwa różne pakiety z tym samym hashem (kolizja)
    //! SHA3-256 zapewnia 128-bit odporność na kolizje - praktycznie niemożliwe
    let data_a = b"Package A content with specific bytes";
    let data_b = b"Package B completely different content";

    let hash_a = compute_sha3_256_hex(data_a);
    let hash_b = compute_sha3_256_hex(data_b);

    assert_ne!(
        hash_a, hash_b,
        "Different data should produce different hashes"
    );

    // Nawet minimalna zmiana
    let mut data_c = data_a.to_vec();
    data_c.push(0x00);
    let hash_c = compute_sha3_256_hex(&data_c);
    assert_ne!(hash_a, hash_c, "Even one byte difference changes the hash");
}

#[test]
fn attack_partial_signature_bypass() {
    //! Scenariusz: atakujący łamie Ed25519 ale nie Dilithium (lub odwrotnie)
    //! Hybrid scheme wymaga OBU podpisów - jeden nie wystarczy
    let publisher = HybridKeyPair::generate("hybrid-test").unwrap();
    let data = b"Test package";
    let mut signature = publisher.sign(data).unwrap();

    // Zepsuj tylko podpis Ed25519 (symulacja złamania klasycznego algorytmu)
    signature.ed25519_signature = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &[0u8; 64],
    );

    let result = verify_hybrid_signature(data, &signature, &publisher.public_key()).unwrap();
    assert!(
        !result.overall_valid,
        "ATTACK BLOCKED: Corrupted Ed25519 signature rejected despite valid Dilithium"
    );
    assert!(result.dilithium_valid, "Dilithium should still be valid");
    assert!(!result.ed25519_valid, "Ed25519 should be invalid");
}