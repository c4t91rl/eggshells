use crate::{
    verifier::{ManifestVerifier, VerificationResult},
    server_registry::{ServerRegistry, RegisteredServer},
    download::{Downloader, DownloadProgress},
    rollback::VersionHistory,
    anti_tamper::AntiTamper,
};
use crypto_core::{SignedManifest, key_management::PublisherIdentity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use futures::future::join_all;

pub struct AppState {
    pub registry: Mutex<ServerRegistry>,
    pub downloader: Downloader,
}

/// Shared reqwest client (reuse connections, set sane timeouts)
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("failed to build reqwest client")
}

// ---- Tauri Commands ----

#[tauri::command]
pub async fn get_servers(
    state: State<'_, AppState>,
) -> Result<Vec<RegisteredServer>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    Ok(registry.servers.clone())
}

#[tauri::command]
pub async fn add_server(
    state: State<'_, AppState>,
    url: String,
) -> Result<RegisteredServer, String> {
    // Load registry off the locked state to avoid blocking other readers/writers longer than needed
    // If ServerRegistry::load is async, prefer `load(...).await` here.
    let registry_path = PathBuf::from("data/servers.json");
    let mut reg = ServerRegistry::load(&registry_path)
        .unwrap_or_else(|_| ServerRegistry::new(registry_path.clone()));

    // Discover + add (async)
    let server = reg.discover_and_add(&url).await.map_err(|e| e.to_string())?;

    // Persist + update in-memory state
    reg.save().map_err(|e| e.to_string())?;
    {
        let mut state_reg = state.registry.lock().map_err(|e| e.to_string())?;
        state_reg.add_server(server.clone());
        // Optionally persist state mirror if your ServerRegistry::save doesn’t cover it
        state_reg.save().map_err(|e| e.to_string())?;
    }

    Ok(server)
}

#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    publisher_id: String,
) -> Result<bool, String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    let removed = registry.remove_server(&publisher_id);
    if removed {
        registry.save().map_err(|e| e.to_string())?;
    }
    Ok(removed)
}

#[tauri::command]
pub async fn check_updates(
    state: State<'_, AppState>,
    publisher_id: String,
    package_name: String,
) -> Result<Option<SignedManifest>, String> {
    // Look up server while holding lock briefly
    let server = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        registry.get_server(&publisher_id).cloned()
    }
    .ok_or_else(|| "Server not found".to_string())?;

    let url = format!(
        "{}/api/packages/{}/latest",
        server.url.trim_end_matches('/'),
        package_name
    );

    let client = http_client();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // Parse JSON (handle non-2xx gracefully)
    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    if body["success"].as_bool().unwrap_or(false) {
        let manifest: SignedManifest = serde_json::from_value(body["data"].clone())
            .map_err(|e| e.to_string())?;
        Ok(Some(manifest))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn verify_manifest(
    _state: State<'_, AppState>,
    manifest: SignedManifest,
) -> Result<VerificationResult, String> {
    // Look up publisher from registry
    let server = {
        let registry = _state.registry.lock().map_err(|e| e.to_string())?;
        registry.get_server(&manifest.manifest.publisher_id).cloned()
    }
    .ok_or_else(|| "Publisher not found in registry".to_string())?;

    Ok(ManifestVerifier::verify(&manifest, &server.publisher))
}

#[derive(Serialize)]
pub struct AvailableUpdate {
    pub manifest: SignedManifest,
    pub verification: VerificationResult,
    pub publisher_name: String,
    pub server_url: String,
}

#[tauri::command]
pub async fn check_all_updates(
    state: State<'_, AppState>,
) -> Result<Vec<AvailableUpdate>, String> {
    // Snapshot enabled servers
    let servers: Vec<RegisteredServer> = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        registry.enabled_servers().into_iter().cloned().collect()
    };

    let client = http_client();
    let mut tasks = Vec::new();

    for server in servers {
        let client = client.clone();
        let server_url = server.url.clone();
        let publisher = server.publisher.clone();
        let publisher_name = publisher.name.clone();

        tasks.push(async move {
            // Fetch packages list
            let packages_url = format!("{}/api/packages", server_url.trim_end_matches('/'));
            let packages_body: serde_json::Value = match client.get(&packages_url).send().await {
                Ok(resp) => match resp.json().await {
                    Ok(v) => v,
                    Err(_) => return Vec::new(),
                },
                Err(_) => return Vec::new(),
            };

            let mut updates_for_server = Vec::new();
            if let Some(packages) = packages_body["data"].as_array() {
                for pkg in packages {
                    let Some(name) = pkg["name"].as_str() else { continue };
                    let manifest_url = format!(
                        "{}/api/packages/{}/latest",
                        server_url.trim_end_matches('/'),
                        name
                    );

                    // Fetch latest manifest
                    let body: serde_json::Value = match client.get(&manifest_url).send().await {
                        Ok(resp) => match resp.json().await {
                            Ok(v) => v,
                            Err(_) => continue,
                        },
                        Err(_) => continue,
                    };

                    if !body["success"].as_bool().unwrap_or(false) {
                        continue;
                    }

                    // Parse + verify
                    if let Ok(manifest) = serde_json::from_value::<SignedManifest>(body["data"].clone()) {
                        let verification = ManifestVerifier::verify(&manifest, &publisher);
                        updates_for_server.push(AvailableUpdate {
                            manifest,
                            verification,
                            publisher_name: publisher_name.clone(),
                            server_url: server_url.clone(),
                        });
                    }
                }
            }

            updates_for_server
        });
    }

    // Run all server/package fetches in parallel, flatten results
    let results = join_all(tasks).await;
    let updates: Vec<AvailableUpdate> = results.into_iter().flatten().collect();
    Ok(updates)
}

#[tauri::command]
pub async fn get_integrity_report() -> Result<crate::anti_tamper::IntegrityReport, String> {
    Ok(AntiTamper::verify_self_integrity())
}

#[derive(Serialize)]
pub struct SecurityInfo {
    pub supported_algorithms: Vec<AlgorithmInfo>,
    pub hash_algorithms: Vec<String>,
    pub tls_version: String,
}

#[derive(Serialize)]
pub struct AlgorithmInfo {
    pub name: String,
    pub algorithm_type: String,
    pub key_size: String,
    pub signature_size: String,
    pub security_level: String,
    pub quantum_safe: bool,
}

#[tauri::command]
pub async fn get_security_info() -> Result<SecurityInfo, String> {
    Ok(SecurityInfo {
        supported_algorithms: vec![
            AlgorithmInfo {
                name: "Ed25519".into(),
                algorithm_type: "Classical".into(),
                key_size: "256-bit".into(),
                signature_size: "512-bit".into(),
                security_level: "128-bit classical".into(),
                quantum_safe: false,
            },
            AlgorithmInfo {
                name: "ML-DSA-65 (Dilithium3)".into(),
                algorithm_type: "Post-Quantum (Lattice)".into(),
                key_size: "~1952 bytes (public)".into(),
                signature_size: "~3293 bytes".into(),
                security_level: "NIST Level 3".into(),
                quantum_safe: true,
            },
            AlgorithmInfo {
                name: "Hybrid Ed25519 + ML-DSA-65".into(),
                algorithm_type: "Hybrid (Classical + PQ)".into(),
                key_size: "Combined".into(),
                signature_size: "Combined".into(),
                security_level: "NIST Level 3 + 128-bit classical".into(),
                quantum_safe: true,
            },
        ],
        hash_algorithms: vec!["SHA3-256".into(), "SHA3-512".into(), "BLAKE3".into()],
        tls_version: "TLS 1.3".into(),
    })
}