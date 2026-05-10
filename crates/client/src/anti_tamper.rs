//! # Anti-Tamper / Client Hardening
//!
//! Techniki utrudniające modyfikację klienta i debugowanie.
//! Inspirowane systemami anti-cheat i anti-piracy.
//!
//! ## Zaimplementowane techniki:
//! 1. Self-integrity check Z WERYFIKACJĄ PRZEZ SERWER
//!    - Klient oblicza SHA3-256 swojej binarki
//!    - Pobiera oczekiwany hash z /api/client/integrity
//!    - Porównuje → różnica = wykrycie tampera
//! 2. Debugger detection (Linux: ptrace, Windows: IsDebuggerPresent)
//! 3. Environment validation (LD_PRELOAD, etc.)
//!
//! ## Dlaczego serwer:
//! Atakujący który ma dostęp do binarki klienta
//! nie ma jednocześnie dostępu do serwera.
//! Hash trzymany na serwerze nie może być podmieniony
//! razem z binarką → wykrycie nieuniknione.
//!
//! ## Ograniczenia:
//! - To są techniki utrudniające, NIE uniemożliwiające
//! - Atakujący kontrolujący sieć może zwrócić fałszywy hash
//!   (mitigacja: TLS + cert pinning w produkcji)
//! - Główna ochrona to kryptografia paczek, nie obfuskacja

use anyhow::Result;
use tracing::{info, warn};

/// Sprawdza integralność własnej binarki względem hasha
/// pobranego z serwera aktualizacji.
///
/// Pipeline:
/// 1. Oblicz SHA3-256 własnej binarki (std::env::current_exe)
/// 2. GET {server}/api/client/integrity
/// 3. Wyodrębnij hash dla aktualnej platformy
/// 4. Porównaj w stałym czasie
/// 5. Różnica → bail!() → klient powinien zakończyć działanie
///
/// Tryb offline: jeśli serwer nieosiągalny, loguje ostrzeżenie
/// ale nie blokuje (klient mógł zostać uruchomiony bez sieci).
pub fn perform_self_integrity_check_with_server(
    server_url: &str,
) -> Result<()> {
    let exe_path = std::env::current_exe()?;

    info!(
        "Self-integrity check: {}",
        exe_path.display()
    );

    let metadata = std::fs::metadata(&exe_path)?;
    info!("Executable size: {} bytes", metadata.len());

    let exe_data = std::fs::read(&exe_path)?;
    let actual_hash =
        secure_update_common::compute_sha3_256_hex(&exe_data);

    info!(
        "Executable SHA3-256: {}...{}",
        &actual_hash[..16],
        &actual_hash[actual_hash.len() - 16..]
    );

    // Pobierz oczekiwany hash z serwera
    let url =
        format!("{}/api/client/integrity", server_url);
    info!("Fetching expected hash from {}", url);

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to build HTTP client: {}", e);
            return Ok(());
        }
    };

    let response = match client.get(&url).send() {
        Ok(r) => r,
        Err(e) => {
            warn!(
                "Cannot reach integrity endpoint: {}",
                e
            );
            warn!(
                "⚠️  Skipping integrity check (offline mode)"
            );
            return Ok(());
        }
    };

    if !response.status().is_success() {
        warn!(
            "Integrity endpoint returned HTTP {}",
            response.status()
        );
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
        info!(
            "Server reports no integrity hashes \
             configured (dev mode)"
        );
        return Ok(());
    }

    let platform = current_platform_key();
    let expected_hash = body["hashes"][&platform]
        ["sha3_256"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();

    if expected_hash.is_empty() {
        warn!(
            "No hash registered for platform '{}' on server",
            platform
        );
        return Ok(());
    }

    info!(
        "Expected SHA3-256:   {}...{}",
        &expected_hash[..16.min(expected_hash.len())],
        &expected_hash[expected_hash
            .len()
            .saturating_sub(16)..]
    );

    if !constant_time_eq(
        actual_hash.as_bytes(),
        expected_hash.as_bytes(),
    ) {
        warn!(
            "❌ INTEGRITY CHECK FAILED: \
             binary hash does not match server"
        );
        warn!("Expected (server): {}", expected_hash);
        warn!("Actual   (local):  {}", actual_hash);
        anyhow::bail!(
            "Binary integrity check failed: \
             executable does not match server-side hash. \
             This binary may have been tampered with."
        );
    }

    info!(
        "✅ Self-integrity check passed \
         (verified against server)"
    );
    Ok(())
}

/// Wersja offline — bez weryfikacji z serwerem.
/// Tylko loguje hash własnej binarki.
/// Używana w sytuacjach gdy serwer nie jest jeszcze znany.
pub fn perform_self_integrity_check() -> Result<()> {
    let exe_path = std::env::current_exe()?;
    let exe_data = std::fs::read(&exe_path)?;
    let hash =
        secure_update_common::compute_sha3_256_hex(&exe_data);

    info!(
        "Self-integrity (offline): {}...{}",
        &hash[..16],
        &hash[hash.len() - 16..]
    );

    Ok(())
}

/// Klucz platformy używany w pliku client_hashes.json
fn current_platform_key() -> String {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x86_64".to_string();

    #[cfg(all(
        target_os = "linux",
        target_arch = "aarch64"
    ))]
    return "linux-aarch64".to_string();

    #[cfg(all(
        target_os = "windows",
        target_arch = "x86_64"
    ))]
    return "windows-x86_64".to_string();

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(
            target_os = "windows",
            target_arch = "x86_64"
        ),
    )))]
    return "unknown".to_string();
}

/// Porównanie w stałym czasie — chroni przed timing attack.
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

/// Sprawdza czy proces jest debugowany.
///
/// Linux:   /proc/self/status → TracerPid != 0
/// Windows: IsDebuggerPresent() WinAPI
pub fn check_debugger() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) =
            std::fs::read_to_string("/proc/self/status")
        {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let tracer_pid: i32 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    if tracer_pid != 0 {
                        warn!(
                            "Debugger detected! \
                             TracerPid: {}",
                            tracer_pid
                        );
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
            warn!(
                "Debugger detected! (IsDebuggerPresent)"
            );
        }
        detected
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows"
    )))]
    {
        false
    }
}

/// Sprawdza podejrzane zmienne środowiskowe.
pub fn check_environment() -> Vec<String> {
    let mut warnings = Vec::new();

    let suspicious = [
        "LD_PRELOAD",
        "DYLD_INSERT_LIBRARIES",
        "_JAVA_OPTIONS",
    ];

    for var in &suspicious {
        if std::env::var(var).is_ok() {
            let msg = format!(
                "Suspicious environment variable set: {}",
                var
            );
            warn!("{}", msg);
            warnings.push(msg);
        }
    }

    warnings
}

/// Pełny raport z weryfikacją przez serwer.
pub fn full_hardening_check_with_server(
    server_url: &str,
) -> HardeningReport {
    let debugger_detected = check_debugger();
    let env_warnings = check_environment();
    let self_check =
        perform_self_integrity_check_with_server(server_url)
            .is_ok();

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
    let self_check =
        perform_self_integrity_check().is_ok();

    HardeningReport {
        debugger_detected,
        environment_warnings: env_warnings,
        self_integrity_ok: self_check,
        overall_safe: !debugger_detected && self_check,
    }
}

#[derive(Debug, Clone)]
pub struct HardeningReport {
    pub debugger_detected: bool,
    pub environment_warnings: Vec<String>,
    pub self_integrity_ok: bool,
    pub overall_safe: bool,
}