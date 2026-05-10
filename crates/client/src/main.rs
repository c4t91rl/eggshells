mod anti_tamper;
mod config;
mod gui;
mod updater;
mod verifier;

use anyhow::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::new("info"),
    )
    .with_target(false)
    .init();

    // ── Hardening checks przy starcie ─────────────────────────

    // 1. Debugger detection — twarde wyjście
    if anti_tamper::check_debugger() {
        eprintln!("❌ SECURITY: Debugger detected. Exiting.");
        std::process::exit(1);
    }

    // 2. Environment checks — ostrzeżenia
    let env_warnings = anti_tamper::check_environment();
    for w in &env_warnings {
        eprintln!("⚠️  SECURITY WARNING: {}", w);
    }

    // 3. Self-integrity z weryfikacją przez serwer
    let cfg = config::load_or_create_config().unwrap_or_default();
    let server_url = cfg.selected_server.clone();

    // Cert path: adjust this to wherever your cert.pem lives
    let cert_path: Option<PathBuf> =
    Some(PathBuf::from("./server_data/certs/cert.pem"))
    .filter(|p| p.exists()); // silently skip if not found

    eprintln!(
        "🔍 Verifying client integrity against: {}",
        server_url
    );

    if let Err(e) = anti_tamper::perform_self_integrity_check_with_server(
        &server_url,
        cert_path.as_deref(),
    ) {
        eprintln!("❌ SECURITY: {}", e);
        eprintln!(
            "⚠️  Continuing despite integrity failure \
(prototype mode — would exit in production)"
        );
    }

    // ── Uruchom GUI ───────────────────────────────────────────

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([900.0, 650.0])
        .with_min_inner_size([700.0, 500.0])
        .with_title("Secure Update Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Secure Update Manager",
        options,
        Box::new(move |cc| {                              // ← add `move`
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.set_fonts(egui::FontDefinitions::default());
            Ok(Box::new(gui::UpdateApp::new(cert_path)))  // ← no .clone() needed, moved in
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}
