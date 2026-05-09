use crate::{
    verifier::{ManifestVerifier, VerificationResult},
    server_registry::{ServerRegistry, RegisteredServer, TrustLevel},
    download::{Downloader, DownloadProgress},
    rollback::VersionHistory,
    anti_tamper::AntiTamper,
};
use crypto_core::{SignedManifest, key_management::PublisherIdentity};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub registry: Mutex<ServerRegistry>,
    pub downloader: Downloader,
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
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    // We need to drop the lock before the async call, so clone what we need
    drop(registry);

    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    // For simplicity in the prototype, we do a blocking call
    // In production, use proper async state management
    let server = tokio::runtime::Handle::current()
        .block_on(async {
            let mut reg = ServerRegistry::load(&std::path::PathBuf::from("data/servers.json"))
                .unwrap_or_else(|_| ServerRegistry::new(std::path::PathBuf::from("data/servers.json")));
            reg.discover_and_add(&url).await
        })
        .map_err(|e| e.to_string())?;

    registry.add_server(server.clone());
    registry.save().map_err(|e| e.to_string())?;

    Ok(server)
}

#[tauri::command]
pub async fn remove_server(
    state: State<'_, AppState>,
    publisher_id: String,
) -> Result<bool, String> {
    let mut registry = state.registry.lock().map_err(|e| e.to_string())?;
    let removed = registry.remove_server(&publisher_id);
    registry.save().map_err(|e| e.to_string())?;
    Ok(removed)
}

#[tauri::command]
pub async fn check_updates(
    state: State<'_, AppState>,
    publisher_id: String,
    package_name: String,
) -> Result<Option<SignedManifest>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let server = registry.get_server(&publisher_id)
        .ok_or("Server not found")?;

    let url = format!(
        "{}/api/packages/{}/latest",
        server.url.trim_end_matches('/'),
        package_name
    );

    drop(registry);

    let client = reqwest::Client::new();
    let response: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if response["success"].as_bool().unwrap_or(false) {
        let manifest: SignedManifest = serde_json::from_value(response["data"].clone())
            .map_err(|e| e.to_string())?;
        Ok(Some(manifest))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn verify_manifest(
    state: State<'_, AppState>,
    manifest: SignedManifest,
) -> Result<VerificationResult, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let server = registry.get_server(&manifest.manifest.publisher_id)
        .ok_or("Publisher not found in registry")?;

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
    let registry = state.registry.lock().map_err(|e| e.to_string())?;
    let servers: Vec<RegisteredServer> = registry.enabled_servers()
        .iter()
        .map(|s| (*s).clone())
        .collect();
    drop(registry);

    let mut updates = Vec::new();
    let client = reqwest::Client::new();

    for server in servers {
        let packages_url = format!("{}/api/packages", server.url.trim_end_matches('/'));

        if let Ok(response) = client.get(&packages_url).send().await {
            if let Ok(body) = response.json::<serde_json::Value>().await {
                if let Some(packages) = body["data"].as_array() {
                    for pkg in packages {
                        if let Some(name) = pkg["name"].as_str() {
                            let manifest_url = format!(
                                "{}/api/packages/{}/latest",
                                server.url.trim_end_matches('/'),
                                name
                            );

                            if let Ok(resp) = client.get(&manifest_url).send().await {
                                if let Ok(body) = resp.json::<serde_json::Value>().await {
                                    if body["success"].as_bool().unwrap_or(false) {
                                        if let Ok(manifest) = serde_json::from_value::<SignedManifest>(
                                            body["data"].clone()
                                        ) {
                                            let verification = ManifestVerifier::verify(
                                                &manifest,
                                                &server.publisher,
                                            );

                                            updates.push(AvailableUpdate {
                                                manifest,
                                                verification,
                                                publisher_name: server.publisher.name.clone(),
                                                server_url: server.url.clone(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(updates)
}

#[tauri::command]
pub async fn get_integrity_report() -> Result<crate::anti_tamper::IntegrityReport, String> {
    Ok(AntiTamper::verify_self_integrity())
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