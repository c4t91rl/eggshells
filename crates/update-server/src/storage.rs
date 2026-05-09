use std::path::{Path, PathBuf};
use crypto_core::SignedManifest;
use anyhow::Result;

pub struct PackageStorage {
    packages_dir: PathBuf,
    manifests_dir: PathBuf,
}

impl PackageStorage {
    pub fn new(packages_dir: PathBuf, manifests_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&packages_dir)?;
        std::fs::create_dir_all(&manifests_dir)?;
        Ok(Self { packages_dir, manifests_dir })
    }

    /// Store a package file
    pub fn store_package(&self, package_name: &str, version: &str, data: &[u8]) -> Result<PathBuf> {
        let dir = self.packages_dir.join(package_name).join(version);
        std::fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{}-{}.bin", package_name, version));
        std::fs::write(&file_path, data)?;
        Ok(file_path)
    }

    /// Store a signed manifest
    pub fn store_manifest(&self, manifest: &SignedManifest) -> Result<PathBuf> {
        let dir = self.manifests_dir.join(&manifest.manifest.package_name);
        std::fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("{}.json", manifest.manifest.version));
        let data = serde_json::to_string_pretty(manifest)?;
        std::fs::write(&file_path, data)?;
        Ok(file_path)
    }

    /// Get latest manifest for a package
    pub fn get_latest_manifest(&self, package_name: &str) -> Result<Option<SignedManifest>> {
        let dir = self.manifests_dir.join(package_name);
        if !dir.exists() {
            return Ok(None);
        }

        let mut versions: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
            .collect();

        versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

        if let Some(entry) = versions.first() {
            let data = std::fs::read_to_string(entry.path())?;
            let manifest: SignedManifest = serde_json::from_str(&data)?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    /// List all versions of a package
    pub fn list_versions(&self, package_name: &str) -> Result<Vec<String>> {
        let dir = self.manifests_dir.join(package_name);
        if !dir.exists() {
            return Ok(vec![]);
        }

        let versions: Vec<String> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path().file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .collect();

        Ok(versions)
    }

    /// Get package file path
    pub fn get_package_path(&self, package_name: &str, version: &str) -> PathBuf {
        self.packages_dir
            .join(package_name)
            .join(version)
            .join(format!("{}-{}.bin", package_name, version))
    }

    /// List all packages
    pub fn list_packages(&self) -> Result<Vec<String>> {
        let packages: Vec<String> = std::fs::read_dir(&self.manifests_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| {
                e.file_name().to_str().map(|s| s.to_string())
            })
            .collect();

        Ok(packages)
    }
}