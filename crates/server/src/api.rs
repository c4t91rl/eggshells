// crates/server/src/api.rs
//! # REST API Endpoints
//!
//! Implementacja endpointów HTTP dla serwera aktualizacji.

use actix_web::{web, HttpResponse, HttpRequest};
use chrono::Utc;
use secure_update_common::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::AppState;

type SharedState = web::Data<Arc<RwLock<AppState>>>;

/// GET /api/health - Health check
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "Secure Update Server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
        "crypto": {
            "post_quantum": "CRYSTALS-Dilithium3 (ML-DSA-65)",
            "classical": "Ed25519",
            "hash": "SHA3-256"
        }
    }))
}

/// POST /api/publishers - Rejestracja publishera
pub async fn register_publisher(
    state: SharedState,
    body: web::Json<RegisterPublisherRequest>,
) -> HttpResponse {
    let app_state = state.read().await;
    let publisher_id = body.public_key.publisher_id.clone();

    let publisher = PublisherInfo {
        id: publisher_id.clone(),
        display_name: body.display_name.clone(),
        public_key: body.public_key.clone(),
        registered_at: Utc::now(),
        active: true,
    };

    match app_state.db.register_publisher(&publisher) {
        Ok(_) => {
            info!("✅ Registered publisher: {} ({})", publisher.display_name, publisher_id);
            HttpResponse::Created().json(serde_json::json!({
                "status": "registered",
                "publisher_id": publisher_id,
            }))
        }
        Err(e) => {
            error!("❌ Failed to register publisher: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Registration failed: {}", e),
            }))
        }
    }
}

/// GET /api/publishers - Lista publisherów
pub async fn list_publishers(state: SharedState) -> HttpResponse {
    let app_state = state.read().await;
    match app_state.db.list_publishers() {
        Ok(publishers) => HttpResponse::Ok().json(publishers),
        Err(e) => {
            error!("❌ Failed to list publishers: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }))
        }
    }
}

/// POST /api/packages/metadata - Publikacja metadanych pakietu
pub async fn publish_metadata(
    state: SharedState,
    body: web::Json<PublishPackageRequest>,
) -> HttpResponse {
    let app_state = state.read().await;

    // Weryfikuj, że publisher istnieje
    let publisher = match app_state.db.get_publisher(&body.publisher_id) {
        Ok(Some(p)) => p,
        Ok(None) => {
            warn!("⚠️ Unknown publisher: {}", body.publisher_id);
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Publisher not found",
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }));
        }
    };

    if !publisher.active {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Publisher is deactivated",
        }));
    }

    let metadata = PackageMetadata {
        package_id: Uuid::new_v4().to_string(),
        app_id: body.app_id.clone(),
        version: body.version.clone(),
        publisher_id: body.publisher_id.clone(),
        sha3_256_hash: body.sha3_256_hash.clone(),
        file_size: body.file_size,
        filename: body.filename.clone(),
        description: body.description.clone(),
        target_platforms: body.target_platforms.clone(),
        signature: body.signature.clone(),
        published_at: Utc::now(),
        min_upgrade_from: body.min_upgrade_from.clone(),
        changelog: body.changelog.clone(),
    };

    match app_state.db.save_package_metadata(&metadata) {
        Ok(_) => {
            info!(
                "📦 Published package: {} v{} by {}",
                metadata.app_id, metadata.version, metadata.publisher_id
            );
            HttpResponse::Created().json(serde_json::json!({
                "status": "published",
                "package_id": metadata.package_id,
            }))
        }
        Err(e) => {
            error!("❌ Failed to publish metadata: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }))
        }
    }
}

/// POST /api/packages/upload/{publisher_id}/{app_id}/{version} - Upload pliku pakietu
pub async fn upload_package(
    state: SharedState,
    path: web::Path<(String, String, String)>,
    body: web::Bytes,
) -> HttpResponse {
    let (publisher_id, app_id, version) = path.into_inner();
    let app_state = state.read().await;

    // Weryfikuj publishera
    match app_state.db.get_publisher(&publisher_id) {
        Ok(Some(p)) if p.active => {},
        Ok(Some(_)) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Publisher is deactivated",
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Publisher not found",
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }));
        }
    }

    match app_state.storage.store_package(&app_id, &version, &body) {
        Ok(path) => {
            info!("📁 Stored package file: {}", path.display());
            HttpResponse::Ok().json(serde_json::json!({
                "status": "uploaded",
                "size": body.len(),
                "sha3_256": compute_sha3_256_hex(&body),
            }))
        }
        Err(e) => {
            error!("❌ Failed to store package: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }))
        }
    }
}

/// POST /api/check/{app_id} - Sprawdzenie dostępności aktualizacji
pub async fn check_update(
    state: SharedState,
    path: web::Path<String>,
    body: web::Json<CheckUpdateRequest>,
) -> HttpResponse {
    let app_id = path.into_inner();
    let app_state = state.read().await;

    match app_state.db.get_latest_package(&app_id) {
        Ok(Some(latest)) => {
            let update_available = latest.version.is_newer_than(&body.current_version);

            // Pobierz klucz publiczny publishera
            let publisher_key = app_state
                .db
                .get_publisher(&latest.publisher_id)
                .ok()
                .flatten()
                .map(|p| p.public_key);

            info!(
                "🔍 Check update for {}: current={}, latest={}, update={}",
                app_id, body.current_version, latest.version, update_available
            );

            HttpResponse::Ok().json(CheckUpdateResponse {
                update_available,
                latest_package: if update_available { Some(latest) } else { None },
                publisher_public_key: if update_available { publisher_key } else { None },
            })
        }
        Ok(None) => {
            HttpResponse::Ok().json(CheckUpdateResponse {
                update_available: false,
                latest_package: None,
                publisher_public_key: None,
            })
        }
        Err(e) => {
            error!("❌ Failed to check update: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }))
        }
    }
}

/// GET /api/download/{app_id}/{version} - Pobranie pliku pakietu
pub async fn download_package(
    state: SharedState,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();
    let app_state = state.read().await;

    if !app_state.storage.package_exists(&app_id, &version) {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Package file not found",
        }));
    }

    match app_state.storage.read_package(&app_id, &version) {
        Ok(data) => {
            info!("📥 Download: {} v{} ({} bytes)", app_id, version, data.len());
            HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header(("Content-Disposition",
                    format!("attachment; filename=\"{}-{}.pkg\"", app_id, version)))
                .body(data)
        }
        Err(e) => {
            error!("❌ Failed to read package: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("{}", e),
            }))
        }
    }
}