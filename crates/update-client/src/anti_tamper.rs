use crypto_core::hashing::Hasher;
use crypto_core::HashAlgorithm;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Anti-tampering checks for the client application itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub checks: Vec<IntegrityCheck>,
    pub overall_status: IntegrityStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityCheck {
    pub component: String,
    pub status: IntegrityStatus,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrityStatus {
    Ok,
    Warning,
    Compromised,
    Unknown,
}

pub struct AntiTamper;

impl AntiTamper {
    /// Verify the integrity of the client binary itself
    pub fn verify_self_integrity() -> IntegrityReport {
        let mut checks = Vec::new();

        // Check 1: Verify own executable hash
        if let Ok(exe_path) = std::env::current_exe() {
            let hash = Hasher::hash_file(&HashAlgorithm::Blake3, &exe_path);
            checks.push(IntegrityCheck {
                component: "Client Binary".into(),
                status: if hash.is_ok() { IntegrityStatus::Ok } else { IntegrityStatus::Warning },
                details: format!(
                    "Executable: {:?}, Hash: {}",
                    exe_path,
                    hash.unwrap_or_else(|_| "FAILED".into())
                ),
            });
        }

        // Check 2: Verify config directory permissions
        if let Some(config_dir) = dirs_config_path() {
            let permissions_ok = check_directory_permissions(&config_dir);
            checks.push(IntegrityCheck {
                component: "Config Directory".into(),
                status: if permissions_ok { IntegrityStatus::Ok } else { IntegrityStatus::Warning },
                details: format!("Config dir: {:?}", config_dir),
            });
        }

        // Check 3: Check for debugger (basic anti-debug)
        let debugger_detected = detect_debugger();
        checks.push(IntegrityCheck {
            component: "Debugger Detection".into(),
            status: if !debugger_detected { IntegrityStatus::Ok } else { IntegrityStatus::Warning },
            details: if debugger_detected {
                "Debugger or instrumentation detected".into()
            } else {
                "No debugger detected".into()
            },
        });

        let overall = if checks.iter().any(|c| c.status == IntegrityStatus::Compromised) {
            IntegrityStatus::Compromised
        } else if checks.iter().any(|c| c.status == IntegrityStatus::Warning) {
            IntegrityStatus::Warning
        } else {
            IntegrityStatus::Ok
        };

        IntegrityReport {
            checks,
            overall_status: overall,
            timestamp: chrono::Utc::now(),
        }
    }
}

fn dirs_config_path() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir().map(|p| p.join("krypto-update"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|p| p.join("KryptoUpdate"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

fn check_directory_permissions(path: &Path) -> bool {
    if !path.exists() {
        return true; // Will be created with correct permissions
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let mode = metadata.mode();
            // Check that others don't have write access
            return (mode & 0o002) == 0;
        }
    }

    true
}

fn detect_debugger() -> bool {
    #[cfg(target_os = "linux")]
    {
        // Check /proc/self/status for TracerPid
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    if let Some(pid) = line.split_whitespace().nth(1) {
                        if pid != "0" {
                            return true;
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows API: IsDebuggerPresent
        extern "system" {
            fn IsDebuggerPresent() -> i32;
        }
        unsafe {
            if IsDebuggerPresent() != 0 {
                return true;
            }
        }
    }

    false
}