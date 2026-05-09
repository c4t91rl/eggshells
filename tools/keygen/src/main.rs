use clap::{Parser, ValueEnum};
use crypto_core::{
    SignatureAlgorithm,
    key_management::{PublisherKeyMaterial, PublisherIdentity},
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "keygen", about = "Generate publisher keys for KryptoUpdate")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate a new publisher key pair
    Generate {
        /// Publisher ID
        #[arg(short, long)]
        id: String,

        /// Publisher display name
        #[arg(short, long)]
        name: String,

        /// Server URL
        #[arg(short, long)]
        url: String,

        /// Signature algorithm
        #[arg(short, long, value_enum, default_value = "hybrid")]
        algorithm: AlgoChoice,

        /// Output directory for key files
        #[arg(short, long, default_value = "keys")]
        output: PathBuf,
    },

    /// Export public key info
    ExportPublic {
        /// Path to private key file
        #[arg(short, long)]
        key_file: PathBuf,

        /// Output file for public key
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Display key info
    Info {
        /// Path to key file
        #[arg(short, long)]
        key_file: PathBuf,
    },
}

#[derive(Clone, ValueEnum)]
enum AlgoChoice {
    Ed25519,
    MlDsa,
    Hybrid,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { id, name, url, algorithm, output } => {
            let algo = match algorithm {
                AlgoChoice::Ed25519 => SignatureAlgorithm::Ed25519,
                AlgoChoice::MlDsa => SignatureAlgorithm::MlDsa65,
                AlgoChoice::Hybrid => SignatureAlgorithm::HybridEd25519MlDsa65,
            };

            println!("🔑 Generating {:?} key pair for publisher '{}'...", algo, name);

            let keys = PublisherKeyMaterial::generate(&id, &name, &url, algo)?;

            std::fs::create_dir_all(&output)?;

            let private_path = output.join(format!("{}_private.json", id));
            let public_path = output.join(format!("{}_public.json", id));

            keys.save_private(&private_path)?;

            // Save public key info
            let public_info = serde_json::to_string_pretty(&keys.identity)?;
            std::fs::write(&public_path, public_info)?;

            println!("✅ Keys generated successfully!");
            println!("   Private key: {:?}", private_path);
            println!("   Public key:  {:?}", public_path);
            println!("   Key ID:      {}", keys.identity.key_id);
            println!("   Algorithm:   {:?}", keys.identity.algorithm);
        }

        Commands::ExportPublic { key_file, output } => {
            let keys = PublisherKeyMaterial::load_private(&key_file)?;
            let public_info = serde_json::to_string_pretty(&keys.identity)?;
            std::fs::write(&output, public_info)?;
            println!("✅ Public key exported to {:?}", output);
        }

        Commands::Info { key_file } => {
            let data = std::fs::read_to_string(&key_file)?;
            let identity: PublisherIdentity = serde_json::from_str(&data)?;
            println!("Publisher Information:");
            println!("  ID:        {}", identity.id);
            println!("  Name:      {}", identity.name);
            println!("  Algorithm: {:?}", identity.algorithm);
            println!("  Key ID:    {}", identity.key_id);
            println!("  Server:    {}", identity.server_url);
            println!("  Created:   {}", identity.created_at);

            if let Some(ref pk) = identity.ed25519_public_key {
                println!("  Ed25519 PK: {}...{}", &pk[..16], &pk[pk.len()-8..]);
            }
            if let Some(ref pk) = identity.ml_dsa_public_key {
                println!("  ML-DSA PK:  {}... ({} chars)", &pk[..20], pk.len());
            }
        }
    }

    Ok(())
}