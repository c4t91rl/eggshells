use anyhow::Result;
use std::path::Path;
use tracing::{info, warn};

// ── Shared client builder ─────────────────────────────────────────────────────

/// Builds a reqwest blocking client that trusts the given self-signed cert.
/// If no cert_path is provided (or loading fails), falls back to default
/// client (which will fail on self-signed certs — that's intentional in prod).
fn build_client(cert_path: Option<&Path>) -> reqwest::blocking::Client {
    let mut builder = reqwest::blocking::Client::builder()
    .timeout(std::time::Duration::from_secs(10));

    if let Some(path) = cert_path {
        match std::fs::read(path) {
            Ok(pem) => match reqwest::Certificate::from_pem(&pem) {
                Ok(cert) => {
                    builder = builder.add_root_certificate(cert);
                }
                Err(e) => {
                    warn!("Failed to parse cert from {:?}: {}", path, e);
                }
            },
            Err(e) => {
                warn!("Failed to read cert file {:?}: {}", path, e);
            }
        }
    }

    builder.build().unwrap_or_else(|e| {
        warn!("Failed to build HTTP client: {}", e);
        reqwest::blocking::Client::new()
    })
}

// ── Self-integrity check ──────────────────────────────────────────────────────

/// Sprawdza integralność własnej binarki względem hasha
/// pobranego z serwera aktualizacji.
pub fn perform_self_integrity_check_with_server(
    server_url: &str,
    cert_path: Option<&Path>,         // ← NEW: path to cert.pem
) -> Result<()> {
    let exe_path = std::env::current_exe()?;

    info!("Self-integrity check: {}", exe_path.display());

    let metadata = std::fs::metadata(&exe_path)?;
    info!("Executable size: {} bytes", metadata.len());

    let exe_data = std::fs::read(&exe_path)?;
    let actual_hash = secure_update_common::compute_sha3_256_hex(&exe_data);

    info!(
        "Executable SHA3-256: {}...{}",
        &actual_hash[..16],
        &actual_hash[actual_hash.len() - 16..]
    );

    let url = format!("{}/api/client/integrity", server_url);
    info!("Fetching expected hash from {}", url);

    let client = build_client(cert_path);           // ← uses cert-aware client

    let response = match client.get(&url).send() {
        Ok(r) => r,
        Err(e) => {
            warn!("Cannot reach integrity endpoint: {}", e);
            warn!("⚠️  Skipping integrity check (offline mode)");
            return Ok(());
        }
    };

    if !response.status().is_success() {
        warn!("Integrity endpoint returned HTTP {}", response.status());
        return Ok(());
    }

    let body: serde_json::Value = match response.json() {
        Ok(b) => b,
        Err(e) => {
            warn!("Invalid JSON from integrity endpoint: {}", e);
            return Ok(());
        }
    };

    if !body["available"].as_bool().unwrap_or(false) {
        info!("Server reports no integrity hashes configured (dev mode)");
        return Ok(());
    }

    let platform = current_platform_key();
    let expected_hash = body["hashes"][&platform]["sha3_256"]
    .as_str()
    .unwrap_or("")
    .to_lowercase();

    if expected_hash.is_empty() {
        warn!("No hash registered for platform '{}' on server", platform);
        return Ok(());
    }

    info!(
        "Expected SHA3-256: {}...{}",
        &expected_hash[..16.min(expected_hash.len())],
          &expected_hash[expected_hash.len().saturating_sub(16)..]
    );

    if !constant_time_eq(actual_hash.as_bytes(), expected_hash.as_bytes()) {
        warn!("❌ INTEGRITY CHECK FAILED: binary hash does not match server");
        warn!("Expected (server): {}", expected_hash);
        warn!("Actual   (local):  {}", actual_hash);
        anyhow::bail!(
            "Binary integrity check failed: \
executable does not match server-side hash. \
This binary may have been tampered with."
        );
    }

    info!("✅ Self-integrity check passed (verified against server)");
    Ok(())
}

/// Wersja offline — bez weryfikacji z serwerem.
pub fn perform_self_integrity_check() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_data = std::fs::read(&exe_path)?;
    let hash = secure_update_common::compute_sha3_256_hex(&exe_data);

    info!(
        "Self-integrity (offline): {}...{}",
          &hash[..16],
          &hash[hash.len() - 16..]
    );

    Ok(())
}

// ── Hardening reports ─────────────────────────────────────────────────────────

/// Pełny raport z weryfikacją przez serwer.
pub fn full_hardening_check_with_server(
    server_url: &str,
    cert_path: Option<&Path>,         // ← NEW
) -> HardeningReport {
    let debugger_detected = check_debugger();
    let env_warnings = check_environment();
    let self_check =
    perform_self_integrity_check_with_server(server_url, cert_path).is_ok();

    HardeningReport {
        debugger_detected,
        environment_warnings: env_warnings,
        self_integrity_ok: self_check,
        overall_safe: !debugger_detected && self_check,
    }
}

/// Wersja bez serwera (offline / fallback).
pub fn full_hardening_check() -> HardeningReport {
    let debugger_detected = check_debugger();
    let env_warnings = check_environment();
    let self_check = perform_self_integrity_check().is_ok();

    HardeningReport {
        debugger_detected,
        environment_warnings: env_warnings,
        self_integrity_ok: self_check,
        overall_safe: !debugger_detected && self_check,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn current_platform_key() -> String {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x86_64".to_string();

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "linux-aarch64".to_string();

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "windows-x86_64".to_string();

    #[cfg(not(any(
    all(target_os = "linux", target_arch = "x86_64"),
                  all(target_os = "linux", target_arch = "aarch64"),
                  all(target_os = "windows", target_arch = "x86_64"),
    )))]
    return "unknown".to_string();
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn check_debugger() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let tracer_pid: i32 = line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                    if tracer_pid != 0 {
                        warn!("Debugger detected! TracerPid: {}", tracer_pid);
                        return true;
                    }
                }
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn IsDebuggerPresent() -> i32;
        }
        let detected = unsafe { IsDebuggerPresent() != 0 };
        if detected {
            warn!("Debugger detected! (IsDebuggerPresent)");
        }
        detected
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    { false }
}

pub fn check_environment() -> Vec<String> {
    let mut warnings = Vec::new();
    let suspicious = ["LD_PRELOAD", "DYLD_INSERT_LIBRARIES", "_JAVA_OPTIONS"];
    for var in &suspicious {
        if std::env::var(var).is_ok() {
            let msg = format!("Suspicious environment variable set: {}", var);
            warn!("{}", msg);
            warnings.push(msg);
        }
    }
    warnings
}

#[derive(Debug, Clone)]
pub struct HardeningReport {
    pub debugger_detected: bool,
    pub environment_warnings: Vec<String>,
    pub self_integrity_ok: bool,
    pub overall_safe: bool,
}
