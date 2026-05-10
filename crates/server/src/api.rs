use actix_web::{web, HttpRequest, HttpResponse};
use chrono::Utc;
use secure_update_common::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

type SharedState = web::Data<Arc<RwLock<AppState>>>;

// ═══════════════════════════════════════════════════════════════
//  AUTH HELPERS
// ═══════════════════════════════════════════════════════════════

fn extract_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

async fn verify_auth(
    state: &SharedState,
    req: &HttpRequest,
) -> Result<(String, String), HttpResponse> {
    let token = extract_token(req).ok_or_else(|| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Missing Authorization header"
        }))
    })?;

    let app_state = state.read().await;
    match app_state.db.verify_session(&token) {
        Ok(Some((username, publisher_id))) => {
            Ok((username, publisher_id))
        }
        Ok(None) => Err(HttpResponse::Unauthorized().json(
            serde_json::json!({
                "error": "Invalid or expired session"
            }),
        )),
        Err(e) => Err(HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            }))),
    }
}

// ═══════════════════════════════════════════════════════════════
//  AUTH ENDPOINTS
// ═══════════════════════════════════════════════════════════════

pub async fn register_account(
    state: SharedState,
    body: web::Json<crate::auth::RegisterAccountRequest>,
) -> HttpResponse {
    if body.username.trim().is_empty()
        || body.publisher_id.trim().is_empty()
    {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "username and publisher_id required"
            }),
        );
    }
    if body.password.len() < 8 {
        return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Password must be at least 8 characters"
            }),
        );
    }

    let app_state = state.read().await;

    match app_state.db.create_publisher_account(
        &body.username,
        &body.publisher_id,
        &body.display_name,
        &body.password,
    ) {
        Ok(_) => {
            info!(
                "✅ Account created: {} ({})",
                body.username, body.publisher_id
            );
            HttpResponse::Created().json(serde_json::json!({
                "status": "account_created",
                "username": body.username,
                "publisher_id": body.publisher_id,
            }))
        }
        Err(e) => {
            error!("❌ Account creation failed: {}", e);
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("{}", e)
            }))
        }
    }
}

pub async fn login(
    state: SharedState,
    body: web::Json<crate::auth::LoginRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let client_ip = req
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let rate_key = format!("{}:{}", client_ip, body.username);

    let app_state = state.read().await;

    if !app_state.login_limiter.check_and_record(&rate_key)
    {
        warn!("🚫 Rate limit exceeded: {}", rate_key);
        return HttpResponse::TooManyRequests().json(
            serde_json::json!({
                "error": "Too many login attempts. \
                          Wait 60 seconds."
            }),
        );
    }

    match app_state.db.verify_login(
        &body.username,
        &body.password,
    ) {
        Ok(Some((username, publisher_id))) => {
            app_state.login_limiter.reset(&rate_key);

            let token = crate::auth::generate_session_token();
            let expires_at = chrono::Utc::now()
                + chrono::Duration::hours(24);

            if let Err(e) = app_state.db.create_session(
                &token,
                &username,
                &publisher_id,
                &expires_at,
            ) {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": format!(
                            "Session creation failed: {}", e
                        )
                    }));
            }

            info!(
                "✅ Login: {} ({})",
                username, publisher_id
            );
            HttpResponse::Ok().json(crate::auth::LoginResponse {
                token,
                publisher_id,
                expires_at: expires_at.to_rfc3339(),
            })
        }
        Ok(None) => {
            warn!("❌ Login failed: {}", body.username);
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid username or password"
            }))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}

pub async fn logout(
    state: SharedState,
    req: HttpRequest,
) -> HttpResponse {
    if let Some(token) = extract_token(&req) {
        let app_state = state.read().await;
        app_state.db.delete_session(&token).ok();
    }
    HttpResponse::Ok()
        .json(serde_json::json!({"status": "logged_out"}))
}

// ═══════════════════════════════════════════════════════════════
//  PUBLIC ENDPOINTS
// ═══════════════════════════════════════════════════════════════

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

/// GET /api/client/integrity
///
/// Zwraca oczekiwane hashe binarek klienta dla różnych platform.
/// Klient pobiera ten hash przy starcie i porównuje z własnym.
///
/// Plik źródłowy: ./server_data/client_hashes.json
/// Aktualizowany przez skrypt scripts/release_with_server_integrity.sh
pub async fn client_integrity() -> HttpResponse {
    let hashes_path = "./server_data/client_hashes.json";

    let hashes = match std::fs::read_to_string(hashes_path)
    {
        Ok(content) => {
            match serde_json::from_str::<serde_json::Value>(
                &content,
            ) {
                Ok(v) => v,
                Err(e) => {
                    error!(
                        "Invalid client_hashes.json: {}",
                        e
                    );
                    return HttpResponse::InternalServerError(
                    )
                    .json(serde_json::json!({
                        "available": false,
                        "error": format!(
                            "Invalid hashes file: {}", e
                        )
                    }));
                }
            }
        }
        Err(_) => {
            // Brak pliku = brak weryfikacji
            return HttpResponse::Ok().json(
                serde_json::json!({
                    "available": false,
                    "message": "No client integrity hashes configured"
                }),
            );
        }
    };

    HttpResponse::Ok().json(serde_json::json!({
        "available": true,
        "hashes": hashes
    }))
}

pub async fn list_apps(state: SharedState) -> HttpResponse {
    let app_state = state.read().await;
    match app_state.db.list_apps() {
        Ok(apps) => HttpResponse::Ok()
            .json(ListAppsResponse { apps }),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}

pub async fn list_publishers(
    state: SharedState,
) -> HttpResponse {
    let app_state = state.read().await;
    match app_state.db.list_publishers() {
        Ok(p) => HttpResponse::Ok().json(p),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}

pub async fn check_update(
    state: SharedState,
    path: web::Path<String>,
    body: web::Json<CheckUpdateRequest>,
) -> HttpResponse {
    let app_id = path.into_inner();
    let app_state = state.read().await;

    match app_state.db.get_latest_package(&app_id) {
        Ok(Some(latest)) => {
            let update_available = latest
                .version
                .is_newer_than(&body.current_version);

            let publisher_key = app_state
                .db
                .get_publisher(&latest.publisher_id)
                .ok()
                .flatten()
                .map(|p| p.public_key);

            info!(
                "🔍 Check {}: current={}, latest={}, update={}",
                app_id,
                body.current_version,
                latest.version,
                update_available
            );

            HttpResponse::Ok().json(CheckUpdateResponse {
                update_available,
                latest_package: if update_available {
                    Some(latest)
                } else {
                    None
                },
                publisher_public_key: if update_available {
                    publisher_key
                } else {
                    None
                },
            })
        }
        Ok(None) => HttpResponse::Ok().json(
            CheckUpdateResponse {
                update_available: false,
                latest_package: None,
                publisher_public_key: None,
            },
        ),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}

pub async fn download_package(
    state: SharedState,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (app_id, version) = path.into_inner();
    let app_state = state.read().await;

    if !app_state.storage.package_exists(&app_id, &version)
    {
        return HttpResponse::NotFound().json(
            serde_json::json!({
                "error": "Package not found"
            }),
        );
    }

    match app_state.storage.read_package(&app_id, &version)
    {
        Ok(data) => {
            info!(
                "📥 Download: {} v{} ({} bytes)",
                app_id,
                version,
                data.len()
            );
            HttpResponse::Ok()
                .content_type("application/octet-stream")
                .append_header((
                    "Content-Disposition",
                    format!(
                        "attachment; filename=\"{}-{}.pkg\"",
                        app_id, version
                    ),
                ))
                .body(data)
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}

// ═══════════════════════════════════════════════════════════════
//  PUBLISHER ENDPOINTS (require auth token)
// ═══════════════════════════════════════════════════════════════

pub async fn register_publisher(
    state: SharedState,
    body: web::Json<RegisterPublisherRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let (_username, session_pub_id) =
        match verify_auth(&state, &req).await {
            Ok(auth) => auth,
            Err(resp) => return resp,
        };

    if body.public_key.publisher_id != session_pub_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": format!(
                "Key publisher_id '{}' does not match \
                 session publisher_id '{}'",
                body.public_key.publisher_id, session_pub_id
            )
        }));
    }

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
            info!(
                "✅ Publisher keys registered: {}",
                publisher_id
            );
            HttpResponse::Created().json(serde_json::json!({
                "status": "registered",
                "publisher_id": publisher_id,
            }))
        }
        Err(e) => {
            error!("❌ Register failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": format!("{}", e)
                }))
        }
    }
}

pub async fn upload_package(
    state: SharedState,
    path: web::Path<(String, String, String)>,
    body: web::Bytes,
    req: HttpRequest,
) -> HttpResponse {
    let (username, session_pub_id) =
        match verify_auth(&state, &req).await {
            Ok(auth) => auth,
            Err(resp) => return resp,
        };

    let (publisher_id, app_id, version) = path.into_inner();

    if publisher_id != session_pub_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Publisher ID mismatch with session"
        }));
    }

    if body.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Empty package body"
        }));
    }

    let app_state = state.read().await;

    match app_state.storage.store_package(
        &app_id, &version, &body,
    ) {
        Ok(path) => {
            info!(
                "📁 {} uploaded: {}",
                username,
                path.display()
            );
            HttpResponse::Ok().json(serde_json::json!({
                "status": "uploaded",
                "size": body.len(),
                "sha3_256": compute_sha3_256_hex(&body),
            }))
        }
        Err(e) => {
            error!("❌ Upload failed: {}", e);
            HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": format!("{}", e)
                }))
        }
    }
}

pub async fn publish_metadata(
    state: SharedState,
    body: web::Json<PublishPackageRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let (username, session_pub_id) =
        match verify_auth(&state, &req).await {
            Ok(auth) => auth,
            Err(resp) => return resp,
        };

    if body.publisher_id != session_pub_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Publisher ID mismatch with session"
        }));
    }

    let app_state = state.read().await;

    let publisher =
        match app_state.db.get_publisher(&body.publisher_id) {
            Ok(Some(p)) => p,
            Ok(None) => return HttpResponse::NotFound().json(
                serde_json::json!({
                    "error": "Publisher keys not found. Register keys first."
                }),
            ),
            Err(e) => return HttpResponse::InternalServerError()
                .json(serde_json::json!({
                    "error": format!("{}", e)
                })),
        };

    if !publisher.active {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Publisher account is deactivated"
        }));
    }

    let package_data = match app_state.storage.read_package(
        &body.app_id,
        &body.version.to_string(),
    ) {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().json(
            serde_json::json!({
                "error": "Package file not found. Upload first."
            }),
        ),
    };

    let temp_metadata = PackageMetadata {
        package_id: String::new(),
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

    match crate::publisher::verify_package_on_publish(
        &temp_metadata,
        &package_data,
        &publisher,
    ) {
        Ok(result) if result.overall_valid => {
            info!(
                "✅ Signature verified: {} v{} by {}",
                body.app_id, body.version, username
            );
        }
        Ok(result) => {
            warn!(
                "❌ Signature FAILED: {} v{} by {}: {}",
                body.app_id,
                body.version,
                username,
                result.details
            );
            return HttpResponse::BadRequest().json(
                serde_json::json!({
                    "error": "Signature verification failed",
                    "details": result.details
                }),
            );
        }
        Err(e) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("Verification error: {}", e)
            })),
    }

    let metadata = PackageMetadata {
        package_id: Uuid::new_v4().to_string(),
        ..temp_metadata
    };

    match app_state.db.save_package_metadata(&metadata) {
        Ok(_) => {
            info!(
                "✅ Published: {} v{} by {}",
                metadata.app_id, metadata.version, username
            );
            HttpResponse::Created().json(serde_json::json!({
                "status": "published",
                "package_id": metadata.package_id,
                "verified": true,
            }))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({
                "error": format!("{}", e)
            })),
    }
}