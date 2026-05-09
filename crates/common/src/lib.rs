// crates/common/src/lib.rs
//! # Secure Update Common Library
//!
//! Wspólne typy, kryptografia i narzędzia używane zarówno przez serwer,
//! klienta, jak i narzędzie publishera.
//!
//! ## Moduły:
//! - `crypto` - Kryptografia hybrydowa (Dilithium + Ed25519) i hashing (SHA3-256)
//! - `models` - Struktury danych: metadata pakietów, informacje o publisherach
//! - `version` - Semantic versioning z monotonic check (anti-downgrade)

pub mod crypto;
pub mod models;
pub mod version;

pub use crypto::*;
pub use models::*;
pub use version::*;