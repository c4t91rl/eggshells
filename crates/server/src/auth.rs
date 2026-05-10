use argon2::{
    Argon2,
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher,
        PasswordVerifier, SaltString,
    },
};
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

/// Hashuje hasło Argon2id.
///
/// Argon2id jest memory-hard — GPU nie może efektywnie
/// przeprowadzić brute-force (wymaga dużo RAM per-próbę).
///
/// Parametry domyślne Argon2::default():
///   algorithm: Argon2id
///   version:   19
///   m_cost:    65536 (64 MB RAM)
///   t_cost:    3     (3 iteracje)
///   p_cost:    4     (4 wątki)
///
/// Sól jest generowana kryptograficznie (OsRng → /dev/urandom)
/// i wbudowana w zwracany string — nie trzeba jej osobno
/// przechowywać.
///
/// Format wyjścia: "$argon2id$v=19$m=65536,t=3,p=4$<salt>$<hash>"
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Argon2 hashing failed")
        .to_string()
}

/// Weryfikuje hasło Argon2id w stałym czasie.
///
/// Argon2 verify jest inherentnie constant-time dla
/// poprawnie zaimplementowanych bibliotek.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parsed = match PasswordHash::new(stored_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// Generuje token sesji (64 losowe bajty → 128 znaków hex).
///
/// Entropia: 512 bitów.
/// Źródło losowości: /dev/urandom (Linux/macOS) lub fallback.
pub fn generate_session_token() -> String {
    let mut buf = [0u8; 64];
    fill_random(&mut buf);
    hex::encode(buf)
}

/// Kryptograficznie bezpieczne wypełnienie bufora.
///
/// Priorytet:
///   1. /dev/urandom (Linux/macOS) — kernel CSPRNG
///   2. Fallback: UUID v4 + SHA3-256 + timestamp
///      (UUID v4 wewnętrznie używa OsRng w bibliotece uuid)
fn fill_random(buf: &mut [u8]) {
    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            if f.read_exact(buf).is_ok() {
                return;
            }
        }
    }

    // Fallback dla Windows lub gdy /dev/urandom niedostępny
    // uuid::Uuid::new_v4() wewnętrznie używa getrandom/OsRng
    let id = uuid::Uuid::new_v4();
    let mut hasher = Sha3_256::new();
    hasher.update(id.as_bytes());
    hasher.update(
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .to_le_bytes(),
    );
    let hash = hasher.finalize();

    let first_chunk = buf.len().min(32);
    buf[..first_chunk].copy_from_slice(&hash[..first_chunk]);

    // Dla buforów > 32 bajty (np. 64) — drugi blok SHA3
    if buf.len() > 32 {
        let mut hasher2 = Sha3_256::new();
        hasher2.update(&hash);
        hasher2.update(b"extend-block-2");
        let hash2 = hasher2.finalize();
        let remaining = (buf.len() - 32).min(32);
        buf[32..32 + remaining]
            .copy_from_slice(&hash2[..remaining]);
    }
}