// crates/client/src/main.rs
//! # Secure Update Client
//!
//! Klient aktualizacji z graficznym interfejsem użytkownika (egui).
//! Cross-platform: działa na Windows i Linux.

mod gui;
mod updater;
mod verifier;
mod config;
mod anti_tamper;

use anyhow::Result;
use eframe::egui;

fn main() -> Result<()> {
    // Inicjalizacja logowania
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
        .with_target(false)
        .init();

    // Anti-tamper check przy starcie
    if let Err(e) = anti_tamper::perform_self_integrity_check() {
        eprintln!("⚠️ Self-integrity check warning: {}", e);
        // W produkcji: odmów uruchomienia
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([700.0, 500.0])
            .with_title("🔒 Secure Update Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Secure Update Manager",
        options,
        Box::new(|cc| {
            // Ustawiamy ciemny motyw
            cc.egui_ctx.set_visuals(egui::Visuals::dark());

            // Konfigurujemy czcionki
            let fonts = egui::FontDefinitions::default(); // mut - ponoć nie musi być
            // Można dodać custom fonty tutaj
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(gui::UpdateApp::new()))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}