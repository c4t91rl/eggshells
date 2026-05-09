// crates/client/src/anti_tamper.rs
//! # Anti-Tamper / Client Hardening
//!
//! Techniki utrudniające modyfikację klienta i debugowanie.
//! Inspirowane systemami anti-cheat i anti-piracy.
//!
//! ## Zaimplementowane techniki:
//! 1. Self-integrity check (hash własnego binarki)
//! 2. Debugger detection (Linux: ptrace, Windows: IsDebuggerPresent)
//! 3. Environment validation
//!
//! ## Ograniczenia:
//! - To są techniki utrudniające, NIE uniemożliwiające
//! - Zdeterminowany atakujący może je obejść
//! - Główna ochrona to kryptografia, nie obfuskacja

use anyhow::Result;
use tracing::{info, warn};

/// Sprawdza integralność własnego pliku wykonywalnego.
///
/// W produkcji: porównuje hash z wbudowanym hashem.
/// W prototypie: po prostu oblicza hash i loguje go.
pub fn perform_self_integrity_check() -> Result<()> {
    let exe_path = std::env::current_exe()?;

    info!("🔒 Self-integrity check: {}", exe_path.display());

    // Sprawdź rozmiar pliku (szybki sanity check)
    let metadata = std::fs::metadata(&exe_path)?;
    info!("   Executable size: {} bytes", metadata.len());

    // Oblicz hash (w produkcji porównać z wbudowanym)
    let exe_data = std::fs::read(&exe_path)?;
    let hash = secure_update_common::compute_sha3_256_hex(&exe_data);
    info!("   Executable SHA3-256: {}...{}", &hash[..16], &hash[hash.len()-16..]);

    Ok(())
}

/// Sprawdza czy proces jest debugowany.
///
/// Na Linux: próbuje ptrace(PTRACE_TRACEME) - jeśli się nie uda,
/// to ktoś już ptrace'uje nasz proces (debugger).
///
/// Na Windows: wywołuje IsDebuggerPresent().
pub fn check_debugger() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Na Linuxie: sprawdź /proc/self/status
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let tracer_pid: i32 = line
                        .split_whitespace()
                        .nth(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    if tracer_pid != 0 {
                        warn!("⚠️ Debugger detected! TracerPid: {}", tracer_pid);
                        return true;
                    }
                }
            }
        }
        false
    }

    #[cfg(target_os = "windows")]
    {
        // Na Windowsie: IsDebuggerPresent
        extern "system" {
            fn IsDebuggerPresent() -> i32;
        }
        let detected = unsafe { IsDebuggerPresent() != 0 };
        if detected {
            warn!("⚠️ Debugger detected! (IsDebuggerPresent)");
        }
        detected
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        false
    }
}

/// Sprawdza podejrzane zmienne środowiskowe
pub fn check_environment() -> Vec<String> {
    let mut warnings = Vec::new();

    // Sprawdź znane zmienne debuggerów
    let suspicious_vars = [
        "LD_PRELOAD",       // Linux: library injection
        "DYLD_INSERT_LIBRARIES", // macOS: library injection
        "_JAVA_OPTIONS",    // JVM injection
    ];

    for var in &suspicious_vars {
        if std::env::var(var).is_ok() {
            let msg = format!("Suspicious environment variable set: {}", var);
            warn!("⚠️ {}", msg);
            warnings.push(msg);
        }
    }

    warnings
}

/// Pełna kontrola hardening - zwraca raport
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

#[derive(Debug, Clone)]
pub struct HardeningReport {
    pub debugger_detected: bool,
    pub environment_warnings: Vec<String>,
    pub self_integrity_ok: bool,
    pub overall_safe: bool,
}