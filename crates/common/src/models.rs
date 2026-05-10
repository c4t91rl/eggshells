use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::HybridPublicKey;
use crate::crypto::HybridSignature;
use crate::version::SemanticVersion;

/// Informacje o zarejestrowanym publisherze
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    pub id: String,
    pub display_name: String,
    pub public_key: HybridPublicKey,
    pub registered_at: DateTime<Utc>,
    pub active: bool,
}

/// Metadata pakietu aktualizacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub package_id: String,
    pub app_id: String,
    pub version: SemanticVersion,
    pub publisher_id: String,
    pub sha3_256_hash: String,
    pub file_size: u64,
    pub filename: String,
    pub description: String,
    pub target_platforms: Vec<Platform>,
    pub signature: HybridSignature,
    pub published_at: DateTime<Utc>,
    pub min_upgrade_from: Option<SemanticVersion>,
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
    pub update_available: bool,
    pub latest_package: Option<PackageMetadata>,
    pub publisher_public_key: Option<HybridPublicKey>,
}

/// Żądanie rejestracji publishera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPublisherRequest {
    pub display_name: String,
    pub public_key: HybridPublicKey,
}

/// Żądanie publikacji pakietu
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

/// Informacja o dostępnej aplikacji (z serwera)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub app_id: String,
    pub latest_version: Option<SemanticVersion>,
    pub latest_publisher: Option<String>,
    pub last_published_at: Option<DateTime<Utc>>,
}

/// Odpowiedź listy aplikacji
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAppsResponse {
    pub apps: Vec<AppInfo>,
}

/// Informacja o zainstalowanej aplikacji (lokalnie)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub server_url: String,
    pub app_id: String,
    pub installed_version: SemanticVersion,
    pub install_dir: String,
    pub installed_at: DateTime<Utc>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

/// Stan aktualizacji po stronie klienta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UpdateState {
    UpToDate,
    Checking,
    UpdateAvailable {
        version: SemanticVersion,
        description: String,
    },
    Downloading {
        progress_percent: f32,
    },
    Verifying,
    ReadyToInstall,
    Installing,
    Completed,
    Error {
        message: String,
    },
}

/// Lokalna konfiguracja klienta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Lista znanych serwerów
    pub servers: Vec<String>,
    /// Aktualnie wybrany serwer
    pub selected_server: String,

    /// Aktywna aplikacja (legacy/compat)
    pub app_id: String,
    /// Aktualna wersja aktywnej aplikacji (legacy/compat)
    pub current_version: SemanticVersion,

    /// Katalogi
    pub download_dir: String,
    pub install_dir: String,

    /// Pinned keys per server
    pub pinned_publisher_keys_by_server: HashMap<String, Vec<HybridPublicKey>>,

    /// Zainstalowane aplikacje (lokalny stan)
    pub installed_apps: Vec<InstalledApp>,

    /// Ustawienia
    pub check_interval_secs: u64,
    pub auto_download: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            servers: vec!["http://127.0.0.1:8443".to_string()],
            selected_server: "http://127.0.0.1:8443".to_string(),
            app_id: "example-app".to_string(),
            current_version: SemanticVersion::new(1, 0, 0),
            download_dir: "./downloads".to_string(),
            install_dir: "./installed".to_string(),
            pinned_publisher_keys_by_server: HashMap::new(),
            installed_apps: Vec::new(),
            check_interval_secs: 3600,
            auto_download: false,
        }
    }
}