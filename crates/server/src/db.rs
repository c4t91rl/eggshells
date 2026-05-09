// crates/server/src/db.rs
//! # Database Layer
//!
//! SQLite database przechowująca informacje o publisherach i metadane pakietów.
//! SQLite jest wystarczające dla prototypu - w produkcji można użyć PostgreSQL.

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};
use secure_update_common::*;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .context("Failed to open database")?;

        // Tworzenie tabel
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS publishers (
                id TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                public_key_json TEXT NOT NULL,
                registered_at TEXT NOT NULL,
                active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS packages (
                package_id TEXT PRIMARY KEY,
                app_id TEXT NOT NULL,
                version_major INTEGER NOT NULL,
                version_minor INTEGER NOT NULL,
                version_patch INTEGER NOT NULL,
                publisher_id TEXT NOT NULL,
                sha3_256_hash TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                filename TEXT NOT NULL,
                description TEXT NOT NULL,
                target_platforms_json TEXT NOT NULL,
                signature_json TEXT NOT NULL,
                published_at TEXT NOT NULL,
                min_upgrade_from_json TEXT,
                changelog_json TEXT NOT NULL,
                FOREIGN KEY (publisher_id) REFERENCES publishers(id)
            );

            CREATE INDEX IF NOT EXISTS idx_packages_app_id ON packages(app_id);
            CREATE INDEX IF NOT EXISTS idx_packages_version ON packages(
                app_id, version_major DESC, version_minor DESC, version_patch DESC
            );
            ",
        )
        .context("Failed to create tables")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Rejestruje nowego publishera
    pub fn register_publisher(&self, publisher: &PublisherInfo) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let pk_json = serde_json::to_string(&publisher.public_key)?;

        conn.execute(
            "INSERT INTO publishers (id, display_name, public_key_json, registered_at, active)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                publisher.id,
                publisher.display_name,
                pk_json,
                publisher.registered_at.to_rfc3339(),
                publisher.active as i32,
            ],
        )
        .context("Failed to insert publisher")?;

        Ok(())
    }

    /// Pobiera listę publisherów
    pub fn list_publishers(&self) -> Result<Vec<PublisherInfo>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, display_name, public_key_json, registered_at, active FROM publishers",
        )?;

        let publishers = stmt
            .query_map([], |row| {
                let pk_json: String = row.get(2)?;
                let registered_at_str: String = row.get(3)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    pk_json,
                    registered_at_str,
                    row.get::<_, i32>(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(id, name, pk_json, reg_at, active)| {
                let public_key: HybridPublicKey = serde_json::from_str(&pk_json).ok()?;
                let registered_at = chrono::DateTime::parse_from_rfc3339(&reg_at)
                    .ok()?
                    .with_timezone(&Utc);
                Some(PublisherInfo {
                    id,
                    display_name: name,
                    public_key,
                    registered_at,
                    active: active != 0,
                })
            })
            .collect();

        Ok(publishers)
    }

    /// Pobiera publishera po ID
    pub fn get_publisher(&self, publisher_id: &str) -> Result<Option<PublisherInfo>> {
        let publishers = self.list_publishers()?;
        Ok(publishers.into_iter().find(|p| p.id == publisher_id))
    }

    /// Zapisuje metadane pakietu
    pub fn save_package_metadata(&self, metadata: &PackageMetadata) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        let platforms_json = serde_json::to_string(&metadata.target_platforms)?;
        let signature_json = serde_json::to_string(&metadata.signature)?;
        let min_upgrade_json = metadata
            .min_upgrade_from
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()?;
        let changelog_json = serde_json::to_string(&metadata.changelog)?;

        conn.execute(
            "INSERT OR REPLACE INTO packages
             (package_id, app_id, version_major, version_minor, version_patch,
              publisher_id, sha3_256_hash, file_size, filename, description,
              target_platforms_json, signature_json, published_at,
              min_upgrade_from_json, changelog_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                metadata.package_id,
                metadata.app_id,
                metadata.version.major,
                metadata.version.minor,
                metadata.version.patch,
                metadata.publisher_id,
                metadata.sha3_256_hash,
                metadata.file_size as i64,
                metadata.filename,
                metadata.description,
                platforms_json,
                signature_json,
                metadata.published_at.to_rfc3339(),
                min_upgrade_json,
                changelog_json,
            ],
        )
        .context("Failed to insert package metadata")?;

        Ok(())
    }

    /// Pobiera najnowszy pakiet dla danej aplikacji
    pub fn get_latest_package(&self, app_id: &str) -> Result<Option<PackageMetadata>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT package_id, app_id, version_major, version_minor, version_patch,
                    publisher_id, sha3_256_hash, file_size, filename, description,
                    target_platforms_json, signature_json, published_at,
                    min_upgrade_from_json, changelog_json
             FROM packages
             WHERE app_id = ?1
             ORDER BY version_major DESC, version_minor DESC, version_patch DESC
             LIMIT 1",
        )?;

        let result = stmt
            .query_row(params![app_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, u32>(3)?,
                    row.get::<_, u32>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, String>(14)?,
                ))
            })
            .optional();

        match result {
            Ok(Some(row)) => {
                let (
                    package_id, app_id, major, minor, patch,
                    publisher_id, hash, size, filename, desc,
                    platforms_json, sig_json, published_at_str,
                    min_upgrade_json, changelog_json,
                ) = row;

                let target_platforms: Vec<Platform> = serde_json::from_str(&platforms_json)?;
                let signature: HybridSignature = serde_json::from_str(&sig_json)?;
                let published_at = chrono::DateTime::parse_from_rfc3339(&published_at_str)?
                    .with_timezone(&Utc);
                let min_upgrade_from: Option<SemanticVersion> = min_upgrade_json
                    .map(|j| serde_json::from_str(&j))
                    .transpose()?;
                let changelog: Vec<String> = serde_json::from_str(&changelog_json)?;

                Ok(Some(PackageMetadata {
                    package_id,
                    app_id,
                    version: SemanticVersion::new(major, minor, patch),
                    publisher_id,
                    sha3_256_hash: hash,
                    file_size: size as u64,
                    filename,
                    description: desc,
                    target_platforms,
                    signature,
                    published_at,
                    min_upgrade_from,
                    changelog,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database query failed: {}", e)),
        }
    }
}

// Rusqlite optional helper
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}