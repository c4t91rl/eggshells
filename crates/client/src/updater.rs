use anyhow::{Context, Result};
use secure_update_common::*;
use crate::verifier::VerificationReport;

pub fn fetch_apps(server_url: &str) -> Result<ListAppsResponse> {
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .get(format!("{}/api/apps", server_url.trim_end_matches('/')))
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
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let request = CheckUpdateRequest {
        app_id: app_id.to_string(),
        current_version: current_version.clone(),
        platform: Platform::current(),
    };

    let response = client
        .post(format!("{}/api/check/{}", server_url.trim_end_matches('/'), app_id))
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
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client
        .get(format!(
            "{}/api/download/{}/{}",
            server_url, app_id, version
        ))
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
    use std::ffi::OsStr;

    std::fs::create_dir_all(install_dir)
        .context("Failed to create install directory")?;

    // ── Krok 1: wyodrębnij TYLKO nazwę pliku ──────────────────
    // Path::file_name() zwraca None jeśli filename kończy się na ".."
    // lub jest pusty. Dla "../../etc/evil" zwraca "evil" — bezpieczne.
    // Dla "../" zwraca None → fallback.
    let safe_filename = std::path::Path::new(&metadata.filename)
        .file_name()
        .unwrap_or_else(|| OsStr::new("package.bin"))
        .to_owned();

    // Dodatkowa weryfikacja: nazwa nie może zawierać separatorów
    // (na wypadek platform-specific edge cases)
    let filename_str = safe_filename.to_string_lossy();
    if filename_str.contains('/') || filename_str.contains('\\') {
        anyhow::bail!(
            "Invalid filename: contains path separator: {}",
            filename_str
        );
    }
    if filename_str == ".." || filename_str == "." {
        anyhow::bail!("Invalid filename: {}", filename_str);
    }

    let output_path = std::path::Path::new(install_dir)
        .join(&safe_filename);

    // ── Krok 2: zapisz do pliku tymczasowego ──────────────────
    // Atomowe: najpierw .tmp, potem rename.
    // Jeśli coś się posypie w trakcie zapisu — nie nadpiszemy
    // istniejącego pliku częściowymi danymi.
    let tmp_path = output_path.with_extension("tmp");

    std::fs::write(&tmp_path, package_data)
        .context("Failed to write temp file")?;

    // ── Krok 3: weryfikacja przez canonicalize ─────────────────
    // canonicalize() rozwiązuje symlinki i ".." w ścieżce.
    // Nawet jeśli ktoś podmienił install_dir na symlink
    // prowadzący gdzie indziej — to wykryjemy.
    let canonical_install =
        std::path::Path::new(install_dir)
            .canonicalize()
            .context("Failed to canonicalize install directory")?;

    let canonical_tmp = tmp_path
        .canonicalize()
        .context("Failed to canonicalize temp file path")?;

    if !canonical_tmp.starts_with(&canonical_install) {
        // Usuń plik tymczasowy przed zwróceniem błędu
        std::fs::remove_file(&tmp_path).ok();
        anyhow::bail!(
            "Path traversal detected: {} is outside install dir {}",
            canonical_tmp.display(),
            canonical_install.display()
        );
    }

    // ── Krok 4: atomowe przeniesienie ─────────────────────────
    // std::fs::rename jest atomowe na tym samym filesystemie.
    // Klient nie zobaczy częściowo zapisanego pliku.
    std::fs::rename(&tmp_path, &output_path)
        .context("Failed to finalize installation (rename failed)")?;

    tracing::info!(
        "Applied: {} v{} → {}",
        metadata.app_id,
        metadata.version,
        output_path.display()
    );

    Ok(())
}

/// Wynik pełnego pipeline'u aktualizacji
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