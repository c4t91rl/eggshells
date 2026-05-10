// crates/server/src/main.rs
//! # Secure Update Server
//!
//! Serwer aktualizacji obsługujący wielu publisherów.
//! Każdy publisher rejestruje swój klucz publiczny i publikuje
//! podpisane pakiety aktualizacji.
//!
//! ## Endpointy API:
//! - `POST   /api/publishers`                    - Rejestracja publishera
//! - `GET    /api/publishers`                    - Lista publisherów
//! - `POST   /api/packages/upload`               - Upload pakietu
//! - `POST   /api/packages/metadata`             - Publikacja metadanych
//! - `GET    /api/check/{app_id}`                - Sprawdzenie aktualizacji
//! - `GET    /api/download/{app_id}/{version}`   - Pobranie pakietu
//! - `GET    /api/health`                        - Health check

mod api;
mod db;
mod publisher;
mod storage;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware}; // <= ignore
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Współdzielony stan serwera
pub struct AppState {
    pub db: db::Database,
    pub storage: storage::PackageStorage,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Inicjalizacja logowania
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .with_target(false)
        .init();

    info!("🚀 Starting Secure Update Server...");

    // Tworzenie katalogów
    std::fs::create_dir_all("./server_data/packages")?;
    std::fs::create_dir_all("./server_data/db")?;

    // Inicjalizacja bazy danych i storage
    let database = db::Database::new("./server_data/db/updates.db")?;
    let storage = storage::PackageStorage::new("./server_data/packages")?;

    let app_state = web::Data::new(Arc::new(RwLock::new(AppState {
        db: database,
        storage,
    })));

    info!("📦 Server data directory: ./server_data/");
    info!("🌐 Starting HTTP server on http://127.0.0.1:8443");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(api::health_check))
                    .route("/publishers", web::post().to(api::register_publisher))
                    .route("/publishers", web::get().to(api::list_publishers))
                    .route("/packages/metadata", web::post().to(api::publish_metadata))
                    .route("/packages/upload/{publisher_id}/{app_id}/{version}",
                           web::post().to(api::upload_package))
                    .route("/check/{app_id}", web::post().to(api::check_update))
                    .route("/download/{app_id}/{version}",
                           web::get().to(api::download_package)),
            )
    })
    .bind("127.0.0.1:8443")?
    .run()
    .await?;

    Ok(())
}