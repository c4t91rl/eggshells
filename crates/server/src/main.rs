mod api;
mod auth;
mod db;
mod publisher;
mod rate_limit;
mod storage;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use anyhow::{Context, Result};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub db: db::Database,
    pub storage: storage::PackageStorage,
    pub login_limiter: rate_limit::RateLimiter,
}

fn load_rustls_config() -> Result<ServerConfig> {
    // Install the default rustls crypto provider (required once per process)
    rustls::crypto::aws_lc_rs::default_provider()
    .install_default()
    .ok(); // `.ok()` — ignore error if already installed (e.g. in tests)

    // Load certificate chain — supports multi-cert PEM files
    let cert_chain: Vec<CertificateDer<'static>> =
    CertificateDer::pem_file_iter("./server_data/certs/cert.pem")
    .context("Failed to open certificate file")?
    .collect::<std::result::Result<_, _>>()
    .context("Failed to parse certificate chain")?;

    if cert_chain.is_empty() {
        anyhow::bail!("No certificates found in cert.pem");
    }

    // Load private key — accepts RSA, EC, or PKCS#8 PEM keys
    let key_der = PrivateKeyDer::from_pem_file("./server_data/certs/key.pem")
    .context("Failed to load private key from key.pem")?;

    let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(cert_chain, key_der)
    .context("TLS config error: certificate/key mismatch or invalid format")?;

    Ok(config)
}

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
    .with_env_filter("info")
    .with_target(false)
    .init();

    info!("🚀 Starting Secure Update Server...");

    // Ensure directories exist
    std::fs::create_dir_all("./server_data/packages")?;
    std::fs::create_dir_all("./server_data/db")?;

    // Initialize database and storage
    let database = db::Database::new("./server_data/db/updates.db")?;
    let storage = storage::PackageStorage::new("./server_data/packages")?;

    let app_state = web::Data::new(Arc::new(RwLock::new(AppState {
        db: database,
        storage,
        login_limiter: rate_limit::RateLimiter::new(),
    })));

    // Background task: clean up expired sessions every hour
    let cleanup_state = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(3600),
        );
        loop {
            interval.tick().await;
            let state = cleanup_state.read().await;
            match state.db.cleanup_expired_sessions() {
                Ok(n) if n > 0 => {
                    info!("🧹 Cleaned {} expired sessions", n);
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Session cleanup failed: {}", e);
                }
            }
        }
    });

    // Load TLS configuration
    let tls_config = load_rustls_config()?;

    info!("🔒 Listening on https://127.0.0.1:8443");

    HttpServer::new(move || {
        let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

        App::new()
        .wrap(cors)
        .app_data(app_state.clone())
        .app_data(web::PayloadConfig::new(20 * 1024 * 1024))
        .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
        .service(
            web::scope("/api")
            // ── Public (no auth) ─────────────────────────────────
            .route("/health",                      web::get().to(api::health_check))
            .route("/client/integrity",            web::get().to(api::client_integrity))
            .route("/apps",                        web::get().to(api::list_apps))
            .route("/publishers",                  web::get().to(api::list_publishers))
            .route("/check/{app_id}",              web::post().to(api::check_update))
            .route("/download/{app_id}/{version}", web::get().to(api::download_package))
            // ── Auth ─────────────────────────────────────────────
            .route("/auth/register",               web::post().to(api::register_account))
            .route("/auth/login",                  web::post().to(api::login))
            .route("/auth/logout",                 web::post().to(api::logout))
            // ── Publisher (requires Bearer token) ─────────────────
            .route("/publishers",                  web::post().to(api::register_publisher))
            .route("/packages/metadata",           web::post().to(api::publish_metadata))
            .route(
                "/packages/upload/{publisher_id}/{app_id}/{version}",
                web::post().to(api::upload_package),
            ),
        )
    })
    .bind_rustls_0_23("127.0.0.1:8443", tls_config)?
    .run()
    .await?;

    Ok(())
}
