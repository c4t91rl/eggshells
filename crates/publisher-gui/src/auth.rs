//! Autoryzacja publishera — hashowanie hasła
//!
//! Używamy SHA3-256 do hashowania master password.
//! W produkcji powinien być Argon2id, ale SHA3 jest wystarczające
//! dla prototypu i spójne z resztą systemu.

use sha3::{Digest, Sha3_256};

/// Hashuje hasło SHA3-256 (z solą "publisher-tool-salt")
pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(b"publisher-tool-salt-kosciuszkon2026");
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}