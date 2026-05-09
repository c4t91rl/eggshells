// crates/server/src/storage.rs
//! # Package Storage
//!
//! Przechowywanie plików pakietów aktualizacji na dysku.
//! W produkcji można zastąpić S3, Azure Blob Storage, itp.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct PackageStorage {
    base_path: PathBuf,
}

impl PackageStorage {
    pub fn new(base_path: &str) -> Result<Self> {
        let path = PathBuf::from(base_path);
        std::fs::create_dir_all(&path)
            .context("Failed to create package storage directory")?;
        Ok(Self { base_path: path })
    }

    /// Ścieżka do pliku pakietu
    pub fn package_path(&self, app_id: &str, version: &str) -> PathBuf {
        self.base_path
            .join(app_id)
            .join(format!("{}.pkg", version))
    }

    /// Zapisuje plik pakietu
    pub fn store_package(
        &self,
        app_id: &str,
        version: &str,
        data: &[u8],
    ) -> Result<PathBuf> {
        let dir = self.base_path.join(app_id);
        std::fs::create_dir_all(&dir)?;

        let path = self.package_path(app_id, version);
        std::fs::write(&path, data)
            .context("Failed to write package file")?;

        Ok(path)
    }

    /// Odczytuje plik pakietu
    pub fn read_package(&self, app_id: &str, version: &str) -> Result<Vec<u8>> {
        let path = self.package_path(app_id, version);
        std::fs::read(&path)
            .context(format!("Failed to read package: {}", path.display()))
    }

    /// Sprawdza czy pakiet istnieje
    pub fn package_exists(&self, app_id: &str, version: &str) -> bool {
        self.package_path(app_id, version).exists()
    }
}