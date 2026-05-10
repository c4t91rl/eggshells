use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Request logowania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Request rejestracji konta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAccountRequest {
    pub username: String,
    pub password: String,
    pub publisher_id: String,
    pub display_name: String,
}

/// Odpowiedź logowania
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub publisher_id: String,
    pub expires_at: String,
}

/// Generuje losową sól (32 bajty → hex)
pub fn generate_salt() -> String {
    let mut buf = [0u8; 32];
    getrandom(&mut buf);
    hex::encode(buf)
}

/// Hashuje hasło z solą (SHA3-256)
pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hasher.update(b"secure-update-system-2026");
    hex::encode(hasher.finalize())
}

/// Weryfikuje hasło (constant-time)
pub fn verify_password(password: &str, salt: &str, expected_hash: &str) -> bool {
    let computed = hash_password(password, salt);
    if computed.len() != expected_hash.len() {
        return false;
    }
    let mut result = 0u8;
    for (a, b) in computed.bytes().zip(expected_hash.bytes()) {
        result |= a ^ b;
    }
    result == 0
}

/// Generuje token sesji (64 losowe bajty → hex)
pub fn generate_session_token() -> String {
    let mut buf = [0u8; 64];
    getrandom(&mut buf);
    hex::encode(buf)
}

/// Kryptograficznie bezpieczne losowe bajty (bez zależności od rand)
fn getrandom(buf: &mut [u8]) {
    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            f.read_exact(buf).ok();
            return;
        }
    }

    #[cfg(windows)]
    {
        // Na Windows używamy BCryptGenRandom przez std
        // ale to wymaga dodatkowej zależności, więc fallback:
    }

    // Fallback: uuid + sha3 (nie idealne, ale działa)
    let id = uuid::Uuid::new_v4();
    let mut hasher = Sha3_256::new();
    hasher.update(id.as_bytes());
    hasher.update(&std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_le_bytes());
    let hash = hasher.finalize();
    let len = buf.len().min(hash.len());
    buf[..len].copy_from_slice(&hash[..len]);
    // Dla buf > 32 bajtów, powtórz z innym seedem
    if buf.len() > 32 {
        let mut hasher2 = Sha3_256::new();
        hasher2.update(&hash);
        hasher2.update(b"extend");
        let hash2 = hasher2.finalize();
        let remaining = buf.len() - 32;
        let copy_len = remaining.min(hash2.len());
        buf[32..32 + copy_len].copy_from_slice(&hash2[..copy_len]);
    }
}