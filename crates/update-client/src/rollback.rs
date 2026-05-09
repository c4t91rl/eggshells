use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Version history for rollback support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub package_name: String,
    pub current_version: String,
    pub installed_versions: Vec<InstalledVersion>,
    pub max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub files: Vec<PathBuf>,
    pub manifest_hash: String,
}

impl VersionHistory {
    pub fn new(package_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            current_version: "0.0.0".to_string(),
            installed_versions: Vec::new(),
            max_history: 5,
        }
    }

    /// Record a new installed version
    pub fn record_installation(
        &mut self,
        version: &str,
        files: Vec<PathBuf>,
        manifest_hash: &str,
    ) {
        self.current_version = version.to_string();
        self.installed_versions.push(InstalledVersion {
            version: version.to_string(),
            installed_at: chrono::Utc::now(),
            files,
            manifest_hash: manifest_hash.to_string(),
        });

        // Keep only max_history versions
        while self.installed_versions.len() > self.max_history {
            self.installed_versions.remove(0);
        }
    }

    /// Check if a version would be a downgrade
    pub fn is_downgrade(&self, new_version: &str) -> bool {
        // Simple string comparison - in production use semver
        new_version <= self.current_version.as_str()
    }

    /// Get available rollback targets
    pub fn rollback_targets(&self) -> Vec<&InstalledVersion> {
        self.installed_versions.iter()
            .filter(|v| v.version != self.current_version)
            .collect()
    }
}