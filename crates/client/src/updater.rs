// crates/client/src/updater.rs
//! # Update Logic
//!
//! Główna logika sprawdzania, pobierania i weryfikowania aktualizacji.

use anyhow::{Context, Result};
use secure_update_common::*;
use crate::verifier::{self, VerificationReport};

/// Sprawdza dostępność aktualizacji na serwerze
pub fn check_for_update(
    server_url: &str,
    app_id: &str,
    current_version: &SemanticVersion,
) -> Result<CheckUpdateResponse> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request = CheckUpdateRequest {
        app_id: app_id.to_string(),
        current_version: current_version.clone(),
        platform: Platform::current(),
    };

    let response = client
        .post(format!("{}/api/check/{}", server_url, app_id))
        .json(&request)
        .send()
        .context("Failed to connect to update server")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Server returned error {}: {}", status, body);
    }

    let check_response: CheckUpdateResponse = response
        .json()
        .context("Failed to parse server response")?;

    Ok(check_response)
}

/// Pobiera pakiet aktualizacji
pub fn download_package(
    server_url: &str,
    app_id: &str,
    version: &str,
) -> Result<Vec<u8>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client
        .get(format!("{}/api/download/{}/{}", server_url, app_id, version))
        .send()
        .context("Failed to download package")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Download failed {}: {}", status, body);
    }

    let data = response.bytes()?.to_vec();
    Ok(data)
}

/// Pełny cykl aktualizacji: sprawdź → pobierz → weryfikuj
pub fn perform_full_update_check( //HOW TA FUNKCJA JEST NIE UŻYWANA??????
    server_url: &str,
    app_id: &str,
    current_version: &SemanticVersion,
    pinned_keys: &[HybridPublicKey],
) -> Result<UpdateResult> {
    // 1. Sprawdź dostępność
    let check = check_for_update(server_url, app_id, current_version)?;

    if !check.update_available {
        return Ok(UpdateResult::UpToDate);
    }

    let metadata = check
        .latest_package
        .ok_or_else(|| anyhow::anyhow!("Update available but no metadata"))?;

    let publisher_key = check
        .publisher_public_key
        .ok_or_else(|| anyhow::anyhow!("No publisher key provided"))?;

    // Opcjonalnie: sprawdź key pinning
    if !pinned_keys.is_empty() {
        let key_pinned = pinned_keys.iter().any(|pk| {
            pk.publisher_id == publisher_key.publisher_id
                && pk.dilithium_public_key == publisher_key.dilithium_public_key
                && pk.ed25519_public_key == publisher_key.ed25519_public_key
        });

        if !key_pinned {
            return Ok(UpdateResult::KeyPinningFailed {
                publisher_id: publisher_key.publisher_id.clone(),
            });
        }
    }

    // 2. Pobierz pakiet
    let package_data = download_package(
        server_url,
        app_id,
        &metadata.version.to_string(),
    )?;

    // 3. Weryfikuj
    let report = verifier::verify_package(
        &package_data,
        &metadata,
        &publisher_key,
        current_version,
    )?;

    if report.overall_valid {
        Ok(UpdateResult::UpdateReady {
            metadata,
            package_data,
            report,
            publisher_key,
        })
    } else {
        Ok(UpdateResult::VerificationFailed { report })
    }
}

/// Symuluje zastosowanie aktualizacji (w prototypie: zapisuje do pliku)
pub fn apply_update(
    package_data: &[u8],
    metadata: &PackageMetadata,
    install_dir: &str,
) -> Result<()> {
    std::fs::create_dir_all(install_dir)?;

    let output_path = std::path::Path::new(install_dir)
        .join(&metadata.filename);

    std::fs::write(&output_path, package_data)
        .context("Failed to write update file")?;

    tracing::info!(
        "✅ Update applied: {} v{} → {}",
        metadata.app_id,
        metadata.version,
        output_path.display()
    );

    Ok(())
}

/// Wynik procesu aktualizacji
#[derive(Debug)]
pub enum UpdateResult { //eeeeeeeeeeeeeeee?
    UpToDate,
    UpdateReady {
        metadata: PackageMetadata,
        package_data: Vec<u8>,
        report: VerificationReport,
        publisher_key: HybridPublicKey,
    },
    VerificationFailed {
        report: VerificationReport,
    },
    KeyPinningFailed {
        publisher_id: String,
    },
}