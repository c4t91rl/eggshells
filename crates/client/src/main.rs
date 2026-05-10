//! # Secure Update Client
//!
//! Klient aktualizacji z GUI (egui), cross-platform.
//! Przy starcie wykonuje hardening checks.

mod anti_tamper;
mod config;
mod gui;
mod updater;
mod verifier;

use anyhow::Result;
use eframe::egui;

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
        eprintln!(
            "❌ SECURITY: Debugger detected. Exiting."
        );
        std::process::exit(1);
    }

    // 2. Environment checks — ostrzeżenia (nie blokujemy,
    //    bo LD_PRELOAD może być używany legalnie w dev)
    let env_warnings = anti_tamper::check_environment();
    for w in &env_warnings {
        eprintln!("⚠️  SECURITY WARNING: {}", w);
    }

    // 3. Self-integrity — ostrzeżenie (w prototypie)
    //    W produkcji: porównać z hashem wkompilowanym w build time
    if let Err(e) = anti_tamper::perform_self_integrity_check() {
        eprintln!("⚠️  Self-integrity check failed: {}", e);
        // Produkcja: std::process::exit(1);
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
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let fonts = egui::FontDefinitions::default();
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(gui::UpdateApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}