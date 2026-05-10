mod api;
mod auth;
mod db;
mod publisher;
mod rate_limit;
mod storage;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct AppState {
    pub db: db::Database,
    pub storage: storage::PackageStorage,
    pub login_limiter: rate_limit::RateLimiter,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    info!("🚀 Starting Secure Update Server...");

    std::fs::create_dir_all("./server_data/packages")?;
    std::fs::create_dir_all("./server_data/db")?;

    let database =
        db::Database::new("./server_data/db/updates.db")?;
    let storage = storage::PackageStorage::new(
        "./server_data/packages",
    )?;

    let app_state = web::Data::new(Arc::new(RwLock::new(
        AppState {
            db: database,
            storage,
            login_limiter: rate_limit::RateLimiter::new(),
        },
    )));

    // ── Background: czyszczenie wygasłych sesji ───────────────
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
                    tracing::error!(
                        "Session cleanup failed: {}",
                        e
                    );
                }
            }
        }
    });

    info!("🌐 Listening on http://127.0.0.1:8443");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .app_data(
                web::PayloadConfig::new(20 * 1024 * 1024),
            )
            .app_data(
                web::JsonConfig::default()
                    .limit(10 * 1024 * 1024),
            )
            .service(
                web::scope("/api")
                    // ── Public ───────────────────────────
                    .route(
                        "/health",
                        web::get().to(api::health_check),
                    )
                    .route(
                        "/apps",
                        web::get().to(api::list_apps),
                    )
                    .route(
                        "/publishers",
                        web::get().to(api::list_publishers),
                    )
                    .route(
                        "/check/{app_id}",
                        web::post().to(api::check_update),
                    )
                    .route(
                        "/download/{app_id}/{version}",
                        web::get().to(api::download_package),
                    )
                    // ── Auth ─────────────────────────────
                    .route(
                        "/auth/register",
                        web::post().to(api::register_account),
                    )
                    .route(
                        "/auth/login",
                        web::post().to(api::login),
                    )
                    .route(
                        "/auth/logout",
                        web::post().to(api::logout),
                    )
                    // ── Publisher (Bearer token) ──────────
                    .route(
                        "/publishers",
                        web::post()
                            .to(api::register_publisher),
                    )
                    .route(
                        "/packages/metadata",
                        web::post()
                            .to(api::publish_metadata),
                    )
                    .route(
                        "/packages/upload\
                         /{publisher_id}/{app_id}/{version}",
                        web::post().to(api::upload_package),
                    ),
            )
    })
    .bind("127.0.0.1:8443")?
    .run()
    .await?;

    Ok(())
}