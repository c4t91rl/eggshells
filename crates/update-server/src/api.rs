use axum::{
    extract::{Path, State, Multipart},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crypto_core::{SignedManifest, key_management::PublisherKeyMaterial};
use crate::{storage::PackageStorage, manifest::ManifestBuilder};

pub struct AppState {
    pub storage: PackageStorage,
    pub publisher_keys: PublisherKeyMaterial,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ServerInfo {
    pub publisher_id: String,
    pub publisher_name: String,
    pub algorithm: String,
    pub key_id: String,
    pub ed25519_public_key: Option<String>,
    pub ml_dsa_public_key: Option<String>,
}

#[derive(Serialize)]
pub struct PackageListItem {
    pub name: String,
    pub latest_version: Option<String>,
    pub versions: Vec<String>,
}

#[derive(Deserialize)]
pub struct PublishRequest {
    pub package_name: String,
    pub version: String,
    pub previous_version: Option<String>,
    pub release_notes: Option<String>,
}

pub fn create_router(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/api/info", get(get_server_info))
        .route("/api/packages", get(list_packages))
        .route("/api/packages/:name/latest", get(get_latest_manifest))
        .route("/api/packages/:name/versions", get(list_versions))
        .route("/api/packages/:name/:version/manifest", get(get_manifest))
        .route("/api/packages/:name/:version/download", get(download_package))
        .route("/api/publish", post(publish_package))
        .with_state(state)
}

async fn get_server_info(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<ServerInfo>> {
    let state = state.read().await;
    let identity = &state.publisher_keys.identity;

    Json(ApiResponse {
        success: true,
        data: Some(ServerInfo {
            publisher_id: identity.id.clone(),
            publisher_name: identity.name.clone(),
            algorithm: format!("{:?}", identity.algorithm),
            key_id: identity.key_id.clone(),
            ed25519_public_key: identity.ed25519_public_key.clone(),
            ml_dsa_public_key: identity.ml_dsa_public_key.clone(),
        }),
        error: None,
    })
}

async fn list_packages(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<ApiResponse<Vec<PackageListItem>>> {
    let state = state.read().await;

    match state.storage.list_packages() {
        Ok(packages) => {
            let items: Vec<PackageListItem> = packages.iter().map(|name| {
                let versions = state.storage.list_versions(name).unwrap_or_default();
                let latest = state.storage.get_latest_manifest(name)
                    .ok()
                    .flatten()
                    .map(|m| m.manifest.version);

                PackageListItem {
                    name: name.clone(),
                    latest_version: latest,
                    versions,
                }
            }).collect();

            Json(ApiResponse { success: true, data: Some(items), error: None })
        }
        Err(e) => {
            Json(ApiResponse { success: false, data: None, error: Some(e.to_string()) })
        }
    }
}

async fn get_latest_manifest(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<SignedManifest>>, StatusCode> {
    let state = state.read().await;

    match state.storage.get_latest_manifest(&name) {
        Ok(Some(manifest)) => {
            Ok(Json(ApiResponse { success: true, data: Some(manifest), error: None }))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            Ok(Json(ApiResponse { success: false, data: None, error: Some(e.to_string()) }))
        }
    }
}

async fn list_versions(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(name): Path<String>,
) -> Json<ApiResponse<Vec<String>>> {
    let state = state.read().await;

    match state.storage.list_versions(&name) {
        Ok(versions) => Json(ApiResponse { success: true, data: Some(versions), error: None }),
        Err(e) => Json(ApiResponse { success: false, data: None, error: Some(e.to_string()) }),
    }
}

async fn get_manifest(
    State(state): State<Arc<RwLock<AppState>>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<ApiResponse<SignedManifest>>, StatusCode> {
    let state = state.read().await;
    let manifest_path = state.storage.manifests_dir_path()
        .join(&name)
        .join(format!("{}.json", version));

    if manifest_path.exists() {
        match std::fs::read_to_string(&manifest_path) {
            Ok(data) => {
                match serde_json::from_str::<SignedManifest>(&data) {
                    Ok(manifest) => {
                        Ok(Json(ApiResponse { success: true, data: Some(manifest), error: None }))
                    }
                    Err(e) => Ok(Json(ApiResponse {
                        success: false, data: None, error: Some(e.to_string()),
                    }))
                }
            }
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn download_package(
    State(state): State<Arc<RwLock<AppState>>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Vec<u8>, StatusCode> {
    let state = state.read().await;
    let path = state.storage.get_package_path(&name, &version);

    if path.exists() {
        std::fs::read(&path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn publish_package(
    State(state): State<Arc<RwLock<AppState>>>,
    mut multipart: Multipart,
) -> Json<ApiResponse<SignedManifest>> {
    let mut package_data: Option<Vec<u8>> = None;
    let mut publish_info: Option<PublishRequest> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "metadata" => {
                if let Ok(text) = field.text().await {
                    publish_info = serde_json::from_str(&text).ok();
                }
            }
            "package" => {
                if let Ok(data) = field.bytes().await {
                    package_data = Some(data.to_vec());
                }
            }
            _ => {}
        }
    }

    let (Some(data), Some(info)) = (package_data, publish_info) else {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Missing package data or metadata".into()),
        });
    };

    let state = state.read().await;

    // Store the package
    let package_path = match state.storage.store_package(&info.package_name, &info.version, &data) {
        Ok(p) => p,
        Err(e) => {
            return Json(ApiResponse {
                success: false, data: None, error: Some(e.to_string()),
            });
        }
    };

    // Create download URL
    let download_url = format!(
        "/api/packages/{}/{}/download",
        info.package_name, info.version
    );

    // Create signed manifest
    let files = vec![(
        format!("{}-{}.bin", info.package_name, info.version),
        package_path.as_path(),
        download_url,
    )];

    match ManifestBuilder::create_signed_manifest(
        &info.package_name,
        &info.version,
        info.previous_version.as_deref(),
        files,
        &state.publisher_keys,
        info.release_notes.as_deref(),
    ) {
        Ok(manifest) => {
            // Store the manifest
            if let Err(e) = state.storage.store_manifest(&manifest) {
                return Json(ApiResponse {
                    success: false, data: None, error: Some(e.to_string()),
                });
            }

            Json(ApiResponse { success: true, data: Some(manifest), error: None })
        }
        Err(e) => {
            Json(ApiResponse { success: false, data: None, error: Some(e.to_string()) })
        }
    }
}