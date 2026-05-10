// crates/publisher-tool/src/main.rs
//! # Publisher Tool (CLI)
//!
//! Narzędzie wiersza poleceń dla publisherów do:
//! - Generowania par kluczy (Dilithium + Ed25519)
//! - Podpisywania pakietów aktualizacji
//! - Rejestracji na serwerze
//! - Publikacji pakietów
//!
//! ## Użycie:
//! ```bash
//! # Generuj klucze
//! publisher-tool generate-keys --publisher-id "my-company" --output ./keys/
//!
//! # Zarejestruj na serwerze
//! publisher-tool register --keys ./keys/my-company.keys.json --server http://127.0.0.1:8443 --name "My Company"
//!
//! # Podpisz i opublikuj pakiet
//! publisher-tool publish --keys ./keys/my-company.keys.json --server http://127.0.0.1:8443 \
//!     --app-id "example-app" --version "1.1.0" --file ./build/app-v1.1.0.bin \
//!     --description "Bug fixes" --changelog "Fixed crash on startup" "Improved performance"
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use secure_update_common::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use reqwest::Certificate;

#[derive(Parser)]
#[command(name = "publisher-tool")]
#[command(about = "Secure Update Publisher Tool - sign and publish packages")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generuj nową parę kluczy hybrydowych (Dilithium + Ed25519)
    GenerateKeys {
        /// ID publishera
        #[arg(long)]
        publisher_id: String,
        /// Katalog wyjściowy
        #[arg(long, default_value = "./keys")]
        output: PathBuf,
    },
    /// Utwórz konto publishera na serwerze
    CreateAccount {
        /// Nazwa użytkownika do logowania
        #[arg(long)]
        username: String,
        /// Hasło do konta
        #[arg(long)]
        password: String,
        /// ID publishera powiązane z kontem
        #[arg(long)]
        publisher_id: String,
        /// Nazwa wyświetlana
        #[arg(long)]
        display_name: String,
        /// URL serwera
        #[arg(long, default_value = "https://127.0.0.1:8443")]
        server: String,
    },
    /// Zaloguj się i pobierz token sesji
    Login {
        /// Nazwa użytkownika
        #[arg(long)]
        username: String,
        /// Hasło
        #[arg(long)]
        password: String,
        /// URL serwera
        #[arg(long, default_value = "https://127.0.0.1:8443")]
        server: String,
    },
    /// Zarejestruj publishera na serwerze
    Register {
        /// Ścieżka do pliku kluczy
        #[arg(long)]
        keys: PathBuf,
        /// URL serwera
        #[arg(long, default_value = "https://127.0.0.1:8443")]
        server: String,
        /// Nazwa wyświetlana
        #[arg(long)]
        name: String,
        /// Token sesji otrzymany po logowaniu
        #[arg(long)]
        token: String,
    },
    /// Podpisz i opublikuj pakiet aktualizacji
    Publish {
        /// Ścieżka do pliku kluczy
        #[arg(long)]
        keys: PathBuf,
        /// URL serwera
        #[arg(long, default_value = "https://127.0.0.1:8443")]
        server: String,
        /// Token sesji otrzymany po logowaniu
        #[arg(long)]
        token: String,
        /// ID aplikacji
        #[arg(long)]
        app_id: String,
        /// Wersja (X.Y.Z)
        #[arg(long)]
        version: String,
        /// Ścieżka do pliku pakietu
        #[arg(long)]
        file: PathBuf,
        /// Opis aktualizacji
        #[arg(long, default_value = "Update")]
        description: String,
        /// Changelog (można podać wielokrotnie)
        #[arg(long)]
        changelog: Vec<String>,
    },
    /// Podpisz plik lokalnie (bez wysyłania na serwer)
    Sign {
        /// Ścieżka do pliku kluczy
        #[arg(long)]
        keys: PathBuf,
        /// Plik do podpisania
        #[arg(long)]
        file: PathBuf,
        /// Ścieżka wyjściowa dla sygnatury
        #[arg(long)]
        output: PathBuf,
    },
    /// Weryfikuj podpis lokalnie
    Verify {
        /// Ścieżka do pliku klucza publicznego
        #[arg(long)]
        public_key: PathBuf,
        /// Plik do weryfikacji
        #[arg(long)]
        file: PathBuf,
        /// Plik sygnatury
        #[arg(long)]
        signature: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenerateKeys {
            publisher_id,
            output,
        } => generate_keys(&publisher_id, &output),

        Commands::CreateAccount {
            username,
            password,
            publisher_id,
            display_name,
            server,
        } => create_account(&username, &password, &publisher_id, &display_name, &server).await,

        Commands::Login {
            username,
            password,
            server,
        } => login(&username, &password, &server).await,

        Commands::Register {
            keys,
            server,
            name,
            token,
        } => register_publisher(&keys, &server, &name, &token).await,

        Commands::Publish {
            keys,
            server,
            token,
            app_id,
            version,
            file,
            description,
            changelog,
        } => publish_package(&keys, &server, &token, &app_id, &version, &file, &description, &changelog).await,

        Commands::Sign {
            keys,
            file,
            output,
        } => sign_file(&keys, &file, &output),

        Commands::Verify {
            public_key,
            file,
            signature,
        } => verify_file(&public_key, &file, &signature),
    }
}

fn generate_keys(publisher_id: &str, output: &PathBuf) -> Result<()> {
    println!( "Generating hybrid key pair for publisher '{}'...", publisher_id);
    println!( " Algorithms: CRYSTALS-Dilithium3 + Ed25519");

    let keypair = HybridKeyPair::generate(publisher_id)?;

    // Zapisz klucze prywatne
    std::fs::create_dir_all(output)?;
    let secret_path = output.join(format!("{}.keys.json", publisher_id));
    let secret_data = keypair.export_secret_keys()?;
    std::fs::write(&secret_path, &secret_data)?;

    // Zapisz klucz publiczny osobno
    let public_key = keypair.public_key();
    let public_path = output.join(format!("{}.pub.json", publisher_id));
    let public_data = serde_json::to_vec_pretty(&public_key)?;
    std::fs::write(&public_path, &public_data)?;

    println!( "Keys generated successfully!");
    println!( " Private keys: {}", secret_path.display());
    println!( " Public key:   {}", public_path.display());
    println!();
    println!( " PROTECT YOUR PRIVATE KEYS! Never share {}",
        secret_path.display());

    // Ustawienie uprawnień (Linux/macOS)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&secret_path,
            std::fs::Permissions::from_mode(0o600))?;
        println!( "Set permissions 600 on private key file");
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateAccountRequest<'a> {
    username: &'a str,
    password: &'a str,
    publisher_id: &'a str,
    display_name: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    token: String,
    publisher_id: String,
    expires_at: String,
}

async fn create_account(
    username: &str,
    password: &str,
    publisher_id: &str,
    display_name: &str,
    server: &str,
) -> Result<()> {
    println!("Creating publisher account on {}...", server);

    let request = CreateAccountRequest {
        username,
        password,
        publisher_id,
        display_name,
    };

    let cert_pem = std::fs::read("server_data/certs/cert.pem")?;
    let cert = Certificate::from_pem(&cert_pem)?;

    let client = reqwest::Client::builder()
    .add_root_certificate(cert)          // ← trust only your cert
    .build()?;

    let response = client
        .post(format!("{}/api/auth/register", server))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to server")?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        println!("Account created successfully!");
        println!("Publisher ID: {}", body["publisher_id"]);
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await?;
        anyhow::bail!("Account creation failed: {} - {}", status, body)
    }
}

async fn login(username: &str, password: &str, server: &str) -> Result<()> {
    println!("Logging in to {}...", server);

    let request = AuthRequest { username, password };

    let cert_pem = std::fs::read("server_data/certs/cert.pem")?;
    let cert = Certificate::from_pem(&cert_pem)?;

    let client = reqwest::Client::builder()
    .add_root_certificate(cert)          // ← trust only your cert
    .build()?;

    let response = client
        .post(format!("{}/api/auth/login", server))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to server")?;

    if response.status().is_success() {
        let body: LoginResponse = response.json().await?;
        println!("Login successful!");
        println!("token: {}", body.token);
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await?;
        anyhow::bail!("Login failed: {} - {}", status, body)
    }
}

async fn register_publisher(
    keys: &PathBuf,
    server: &str,
    name: &str,
    token: &str,
) -> Result<()> {
    println!("Registering publisher on {}...", server);

    let key_data = std::fs::read(keys).context("Failed to read key file")?;
    let keypair = HybridKeyPair::import_secret_keys(&key_data)?;
    let public_key = keypair.public_key();

    let request = RegisterPublisherRequest {
        display_name: name.to_string(),
        public_key,
    };

    let cert_pem = std::fs::read("server_data/certs/cert.pem")?;
    let cert = Certificate::from_pem(&cert_pem)?;

    let client = reqwest::Client::builder()
    .add_root_certificate(cert)          // ← trust only your cert
    .build()?;

    let response = client
        .post(format!("{}/api/publishers", server))
        .header("Authorization", format!("Bearer {}", token))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to server")?;

    if response.status().is_success() {
        let body: serde_json::Value = response.json().await?;
        println!("Publisher registered successfully!");
        println!("Publisher ID: {}", body["publisher_id"]);
    } else {
        let status = response.status();
        let body = response.text().await?;
        anyhow::bail!("Registration failed: {} - {}", status, body);
    }

    Ok(())
}

async fn publish_package(
    keys: &PathBuf,
    server: &str,
    token: &str,
    app_id: &str,
    version: &str,
    file: &PathBuf,
    description: &str,
    changelog: &[String],
) -> Result<()> {
    println!( "Publishing package {} v{}...", app_id, version);

    // Wczytaj klucze
    let key_data = std::fs::read(keys).context("Failed to read key file")?;
    let keypair = HybridKeyPair::import_secret_keys(&key_data)?;

    // Wczytaj plik pakietu
    let package_data = std::fs::read(file)
        .context(format!("Failed to read package file: {}", file.display()))?;

    println!( " File size: {} bytes", package_data.len());

    // Oblicz hash
    let hash = compute_sha3_256_hex(&package_data);
    println!( " SHA3-256: {}", hash);

    // Podpisz pakiet
    println!( " Signing with Dilithium3 + Ed25519...");
    let signature = keypair.sign(&package_data)?;
    println!(  " Signature created");

    // Parsuj wersję
    let sem_version = SemanticVersion::parse(version)
        .map_err(|e| anyhow::anyhow!(e))?;

    let filename = file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{}-{}.pkg", app_id, version));

    // 1. Upload pliku
    println!( " Uploading package file...");

    let cert_pem = std::fs::read("server_data/certs/cert.pem")?;
    let cert = Certificate::from_pem(&cert_pem)?;

    let client = reqwest::Client::builder()
    .add_root_certificate(cert)          // ← trust only your cert
    .build()?;

    let upload_resp = client
        .post(format!(
            "{}/api/packages/upload/{}/{}/{}",
            server.trim_end_matches('/'), keypair.publisher_id, app_id, version
        ))
        .header("Authorization", format!("Bearer {}", token))
        .body(package_data.clone())
        .send()
        .await
        .context("Failed to upload package")?;

    if !upload_resp.status().is_success() {
        let body = upload_resp.text().await?;
        anyhow::bail!("Upload failed: {}", body);
    }
    println!(  " File uploaded");

    // 2. Publikuj metadane
    println!( " Publishing metadata...");
    let metadata_req = PublishPackageRequest {
        app_id: app_id.to_string(),
        version: sem_version,
        publisher_id: keypair.publisher_id.clone(),
        sha3_256_hash: hash,
        file_size: package_data.len() as u64,
        filename,
        description: description.to_string(),
        target_platforms: vec![Platform::current()],
        signature,
        min_upgrade_from: None,
        changelog: changelog.to_vec(),
    };

    let meta_resp = client
        .post(format!("{}/api/packages/metadata", server))
        .header("Authorization", format!("Bearer {}", token))
        .json(&metadata_req)
        .send()
        .await
        .context("Failed to publish metadata")?;

    if meta_resp.status().is_success() {
        let body: serde_json::Value = meta_resp.json().await?;
        println!( "Package published successfully!");
        println!( " Package ID: {}", body["package_id"]);
    } else {
        let body = meta_resp.text().await?;
        println!( "Metadata publication failed: {}", body);
    }

    Ok(())
}

fn sign_file(keys: &PathBuf, file: &PathBuf, output: &PathBuf) -> Result<()> {
    println!( "Signing file: {}", file.display());

    let key_data = std::fs::read(keys)?;
    let keypair = HybridKeyPair::import_secret_keys(&key_data)?;

    let file_data = std::fs::read(file)?;
    let hash = compute_sha3_256_hex(&file_data);
    let signature = keypair.sign(&file_data)?;

    let sig_data = serde_json::json!({
        "file_hash": hash,
        "file_size": file_data.len(),
        "signature": signature,
        "public_key": keypair.public_key(),
    });

    std::fs::write(output, serde_json::to_vec_pretty(&sig_data)?)?;
    println!( "Signature saved to: {}", output.display());

    Ok(())
}

fn verify_file(public_key_path: &PathBuf, file: &PathBuf, signature_path: &PathBuf) -> Result<()> {
    println!( "Verifying file: {}", file.display());

    let pk_data = std::fs::read(public_key_path)?;
    let public_key: HybridPublicKey = serde_json::from_slice(&pk_data)?;

    let sig_data = std::fs::read(signature_path)?;
    let sig_json: serde_json::Value = serde_json::from_slice(&sig_data)?;
    let signature: HybridSignature = serde_json::from_value(sig_json["signature"].clone())?;

    let file_data = std::fs::read(file)?;

    let result = verify_hybrid_signature(&file_data, &signature, &public_key)?;

    println!( " {}", result.details);
    if result.overall_valid {
        println!( "Verification PASSED");
    } else {
        println!( "Verification FAILED");
    }

    Ok(())
}
