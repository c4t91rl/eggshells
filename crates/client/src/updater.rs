use anyhow::{Context, Result};
use secure_update_common::*;
use crate::verifier::{self, VerificationReport};

pub fn fetch_apps(server_url: &str) -> Result<ListAppsResponse> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .get(format!("{}/api/apps", server_url))
        .send()
        .context("Failed to fetch apps")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Server returned {}: {}",
            resp.status(),
            resp.text().unwrap_or_default()
        );
    }

    resp.json().context("Failed to parse apps list")
}

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
        anyhow::bail!(
            "Server returned {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        );
    }

    response.json().context("Failed to parse server response")
}

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
        anyhow::bail!(
            "Download failed {}: {}",
            response.status(),
            response.text().unwrap_or_default()
        );
    }

    Ok(response.bytes()?.to_vec())
}

pub fn apply_update(
    package_data: &[u8],
    metadata: &PackageMetadata,
    install_dir: &str,
) -> Result<()> {
    std::fs::create_dir_all(install_dir)?;
    let output_path = std::path::Path::new(install_dir).join(&metadata.filename);
    std::fs::write(&output_path, package_data)
        .context("Failed to write update file")?;
    tracing::info!(
         "Applied: {} v{} → {}",
        metadata.app_id,
        metadata.version,
        output_path.display()
    );
    Ok(())
}

pub enum UpdateResult {
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
}
