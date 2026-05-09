mod api;
mod config;
mod manifest;
mod storage;

use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crypto_core::{
    SignatureAlgorithm,
    key_management::PublisherKeyMaterial,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load or create config
    let config_path = std::env::args().nth(1).unwrap_or_else(|| "server.toml".into());
    let config = if std::path::Path::new(&config_path).exists() {
        config::ServerConfig::load(&config_path)?
    } else {
        tracing::info!("Creating default config at {}", config_path);
        config::ServerConfig::save_default(&config_path)?;
        config::ServerConfig::default()
    };

    // Load or generate publisher keys
    let publisher_keys = if config.publisher.key_file.exists() {
        tracing::info!("Loading publisher keys from {:?}", config.publisher.key_file);
        PublisherKeyMaterial::load_private(&config.publisher.key_file)?
    } else {
        tracing::info!("Generating new publisher keys");
        let algorithm = match config.publisher.algorithm.as_str() {
            "ed25519" => SignatureAlgorithm::Ed25519,
            "ml-dsa" | "dilithium" => SignatureAlgorithm::MlDsa65,
            _ => SignatureAlgorithm::HybridEd25519MlDsa65,
        };

        let keys = PublisherKeyMaterial::generate(
            &config.publisher.id,
            &config.publisher.name,
            &format!("http://{}:{}", config.server.host, config.server.port),
            algorithm,
        )?;

        // Save keys
        if let Some(parent) = config.publisher.key_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        keys.save_private(&config.publisher.key_file)?;
        tracing::info!("Keys saved to {:?}", config.publisher.key_file);
        tracing::info!("Key ID: {}", keys.identity.key_id);

        keys
    };

    tracing::info!(
        "Publisher: {} ({}), Algorithm: {:?}",
        publisher_keys.identity.name,
        publisher_keys.identity.id,
        publisher_keys.identity.algorithm
    );

    // Initialize storage
    let storage = storage::PackageStorage::new(
        config.storage.packages_dir,
        config.storage.manifests_dir,
    )?;

    // Create app state
    let state = Arc::new(RwLock::new(api::AppState {
        storage,
        publisher_keys,
    }));

    // Build router
    let app = api::create_router(state)
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.server.host, config.server.port);
    tracing::info!("Starting update server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}