// crates/common/src/models.rs
//! # Modele danych
//!
//! Definicje struktur danych używanych w komunikacji między
//! serwerem, klientem i narzędziem publishera.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::HybridPublicKey;
use crate::crypto::HybridSignature;
use crate::version::SemanticVersion;

/// Informacje o zarejestrowanym publisherze
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    /// Unikalny identyfikator publishera
    pub id: String,
    /// Nazwa wyświetlana
    pub display_name: String,
    /// Klucz publiczny (hybrydowy: Dilithium + Ed25519)
    pub public_key: HybridPublicKey,
    /// Data rejestracji
    pub registered_at: DateTime<Utc>,
    /// Czy publisher jest aktywny
    pub active: bool,
}

/// Metadata pakietu aktualizacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Unikalny ID pakietu
    pub package_id: String,
    /// ID aplikacji której dotyczy aktualizacja
    pub app_id: String,
    /// Wersja pakietu
    pub version: SemanticVersion,
    /// ID publishera który podpisał pakiet
    pub publisher_id: String,
    /// SHA3-256 hash pliku pakietu (hex)
    pub sha3_256_hash: String,
    /// Rozmiar pliku w bajtach
    pub file_size: u64,
    /// Nazwa pliku
    pub filename: String,
    /// Opis aktualizacji
    pub description: String,
    /// Platformy docelowe
    pub target_platforms: Vec<Platform>,
    /// Hybrydowy podpis cyfrowy
    pub signature: HybridSignature,
    /// Data publikacji
    pub published_at: DateTime<Utc>,
    /// Minimalna wersja od której można aktualizować
    pub min_upgrade_from: Option<SemanticVersion>,
    /// Changelog
    pub changelog: Vec<String>,
}

/// Platforma docelowa
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    #[serde(rename = "linux-x86_64")]
    LinuxX86_64,
    #[serde(rename = "linux-aarch64")]
    LinuxAarch64,
    #[serde(rename = "windows-x86_64")]
    WindowsX86_64,
    #[serde(rename = "cross-platform")]
    CrossPlatform,
}

impl Platform {
    /// Zwraca platformę bieżącego systemu
    pub fn current() -> Self {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Platform::LinuxX86_64;
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Platform::LinuxAarch64;
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Platform::WindowsX86_64;
        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64"),
        )))]
        return Platform::CrossPlatform;
    }
}

/// Żądanie sprawdzenia aktualizacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckUpdateRequest {
    pub app_id: String,
    pub current_version: SemanticVersion,
    pub platform: Platform,
}

/// Odpowiedź na sprawdzenie aktualizacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckUpdateResponse {
    /// Czy dostępna jest nowsza wersja
    pub update_available: bool,
    /// Metadata najnowszego pakietu (jeśli dostępny)
    pub latest_package: Option<PackageMetadata>,
    /// Klucz publiczny publishera (do weryfikacji podpisu)
    pub publisher_public_key: Option<HybridPublicKey>,
}

/// Żądanie rejestracji publishera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPublisherRequest {
    pub display_name: String,
    pub public_key: HybridPublicKey,
}

/// Żądanie publikacji pakietu (metadata, bez pliku binarnego)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishPackageRequest {
    pub app_id: String,
    pub version: SemanticVersion,
    pub publisher_id: String,
    pub sha3_256_hash: String,
    pub file_size: u64,
    pub filename: String,
    pub description: String,
    pub target_platforms: Vec<Platform>,
    pub signature: HybridSignature,
    pub min_upgrade_from: Option<SemanticVersion>,
    pub changelog: Vec<String>,
}

/// Stan aktualizacji po stronie klienta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateState {
    /// Brak aktualizacji
    UpToDate,
    /// Sprawdzanie aktualizacji
    Checking,
    /// Dostępna aktualizacja
    UpdateAvailable {
        version: SemanticVersion,
        description: String,
    },
    /// Pobieranie
    Downloading {
        progress_percent: f32,
    },
    /// Weryfikacja podpisu i integralności
    Verifying,
    /// Gotowy do instalacji
    ReadyToInstall,
    /// Instalowanie
    Installing,
    /// Zakończono
    Completed,
    /// Błąd
    Error {
        message: String,
    },
}

/// Lokalna konfiguracja klienta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// URL serwera aktualizacji
    pub server_url: String,
    /// ID aplikacji
    pub app_id: String,
    /// Obecna zainstalowana wersja
    pub current_version: SemanticVersion,
    /// Ścieżka do katalogu z pobranymi aktualizacjami
    pub download_dir: String,
    /// Ścieżka do katalogu z zainstalowaną aplikacją
    pub install_dir: String,
    /// Zapisane klucze publiczne publisherów (key pinning)
    pub pinned_publisher_keys: Vec<HybridPublicKey>,
    /// Interwał sprawdzania aktualizacji (sekundy)
    pub check_interval_secs: u64,
    /// Czy automatycznie pobierać aktualizacje
    pub auto_download: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:8443".to_string(),
            app_id: "example-app".to_string(),
            current_version: SemanticVersion::new(1, 0, 0),
            download_dir: "./downloads".to_string(),
            install_dir: "./installed".to_string(),
            pinned_publisher_keys: Vec::new(),
            check_interval_secs: 3600,
            auto_download: false,
        }
    }
}