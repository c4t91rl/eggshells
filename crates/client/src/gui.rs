// crates/client/src/gui.rs
//! # GUI Module
//!
//! Graficzny interfejs użytkownika oparty na egui.
//! Ładny, ciemny motyw z przejrzystym layoutem.

use eframe::egui;
use secure_update_common::*;
use std::sync::{Arc, Mutex};
use crate::{config, updater, anti_tamper};
use crate::verifier::VerificationReport;

/// Stan aplikacji GUI
pub struct UpdateApp {
    // Konfiguracja
    config: ClientConfig,

    // Stan interfejsu
    active_tab: Tab,

    // Stan aktualizacji
    update_state: UpdateState,
    last_check_result: Option<String>,
    verification_report: Option<VerificationReport>,
    downloaded_data: Option<Vec<u8>>,
    pending_metadata: Option<PackageMetadata>,
    pending_publisher_key: Option<HybridPublicKey>,

    // Hardening
    hardening_report: Option<anti_tamper::HardeningReport>,

    // Logi
    log_messages: Vec<LogEntry>,

    // Edytowalne pola
    server_url_input: String,
    app_id_input: String,
    version_input: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Dashboard,
    Update,
    Security,
    Settings,
    Logs,
}

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: String,
    level: LogLevel,
    message: String,
}

#[derive(Debug, Clone)]
enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl UpdateApp {
    pub fn new() -> Self {
        let config = config::load_or_create_config().unwrap_or_default();

        let mut app = Self {
            server_url_input: config.server_url.clone(),
            app_id_input: config.app_id.clone(),
            version_input: config.current_version.to_string(),
            config,
            active_tab: Tab::Dashboard,
            update_state: UpdateState::UpToDate,
            last_check_result: None,
            verification_report: None,
            downloaded_data: None,
            pending_metadata: None,
            pending_publisher_key: None,
            hardening_report: None,
            log_messages: Vec::new(),
        };

        app.add_log(LogLevel::Info, "🚀 Secure Update Manager started");
        app.add_log(
            LogLevel::Info,
            "Cryptography: CRYSTALS-Dilithium3 + Ed25519 + SHA3-256",
        );

        app
    }

    fn add_log(&mut self, level: LogLevel, message: &str) {
        self.log_messages.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            message: message.to_string(),
        });
        // Limituj logi
        if self.log_messages.len() > 500 {
            self.log_messages.remove(0);
        }
    }
}

impl eframe::App for UpdateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel z nawigacją
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔒 Secure Update Manager");
                ui.separator();

                ui.selectable_value(&mut self.active_tab, Tab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.active_tab, Tab::Update, "📦 Update");
                ui.selectable_value(&mut self.active_tab, Tab::Security, "🛡️ Security");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️ Settings");
                ui.selectable_value(&mut self.active_tab, Tab::Logs, "📋 Logs");
            });
        });

        // Status bar na dole
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status_text = match &self.update_state {
                    UpdateState::UpToDate => "✅ Up to date",
                    UpdateState::Checking => "🔄 Checking for updates...",
                    UpdateState::UpdateAvailable { version, .. } => "📦 Update available",
                    UpdateState::Downloading { progress_percent } => "⬇️ Downloading...",
                    UpdateState::Verifying => "🔍 Verifying...",
                    UpdateState::ReadyToInstall => "✅ Ready to install",
                    UpdateState::Installing => "⚙️ Installing...",
                    UpdateState::Completed => "🎉 Update completed",
                    UpdateState::Error { message } => "❌ Error",
                };
                ui.label(status_text);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "v{} | {} | {}",
                        self.config.current_version,
                        self.config.app_id,
                        self.config.server_url
                    ));
                });
            });
        });

        // Central panel z zawartością
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Dashboard => self.render_dashboard(ui),
                Tab::Update => self.render_update(ui),
                Tab::Security => self.render_security(ui),
                Tab::Settings => self.render_settings(ui),
                Tab::Logs => self.render_logs(ui),
            }
        });
    }
}

impl UpdateApp {
    // ============================================================
    // Dashboard Tab
    // ============================================================
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Dashboard");
        ui.separator();

        // Karty informacyjne
        egui::Grid::new("dashboard_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // Application info
                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("📱 Application");
                    ui.separator();
                    ui.label(format!("App ID: {}", self.config.app_id));
                    ui.label(format!("Version: {}", self.config.current_version));
                    ui.label(format!("Platform: {:?}", Platform::current()));
                });

                // Server info
                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("🌐 Server");
                    ui.separator();
                    ui.label(format!("URL: {}", self.config.server_url));
                    ui.label(format!(
                        "Check interval: {}s",
                        self.config.check_interval_secs
                    ));
                    ui.label(format!(
                        "Auto-download: {}",
                        if self.config.auto_download { "Yes" } else { "No" }
                    ));
                });

                ui.end_row();

                // Cryptography info
                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("🔐 Cryptography");
                    ui.separator();
                    ui.label("Post-Quantum: CRYSTALS-Dilithium3 (ML-DSA-65)");
                    ui.label("Classical: Ed25519");
                    ui.label("Hash: SHA3-256 (Keccak)");
                    ui.label("Scheme: Hybrid (both required)");
                });

                // Security status
                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("🛡️ Security Status");
                    ui.separator();
                    ui.label(format!(
                        "Pinned keys: {}",
                        self.config.pinned_publisher_keys.len()
                    ));
                    ui.label(format!(
                        "Anti-downgrade: ✅ Enabled"
                    ));
                    ui.label(format!(
                        "Signature mode: Hybrid (PQ + Classical)"
                    ));

                    if let Some(ref report) = self.hardening_report {
                        if report.overall_safe {
                            ui.colored_label(egui::Color32::GREEN, "Environment: ✅ Safe");
                        } else {
                            ui.colored_label(egui::Color32::RED, "Environment: ⚠️ Warning");
                        }
                    }
                });
            });

        ui.add_space(20.0);

        // Quick actions
        ui.heading("⚡ Quick Actions");
        ui.separator();
        ui.horizontal(|ui| {
            if ui
                .button("🔍 Check for Updates")
                .on_hover_text("Check server for new versions")
                .clicked()
            {
                self.perform_update_check();
            }

            if ui
                .button("🛡️ Run Security Check")
                .on_hover_text("Perform client hardening checks")
                .clicked()
            {
                self.perform_hardening_check();
            }

            if ui
                .button("🏥 Health Check")
                .on_hover_text("Ping the update server")
                .clicked()
            {
                self.perform_health_check();
            }
        });

        // Wyświetl ostatni wynik
        if let Some(ref result) = self.last_check_result {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label(result);
            });
        }
    }

    // ============================================================
    // Update Tab
    // ============================================================
    fn render_update(&mut self, ui: &mut egui::Ui) {
        ui.heading("📦 Update Management");
        ui.separator();

        // Przycisk sprawdzania
        ui.horizontal(|ui| {
            if ui.button("🔍 Check for Updates").clicked() {
                self.perform_update_check();
            }

            match &self.update_state {
                UpdateState::UpToDate => {
                    ui.colored_label(egui::Color32::GREEN, "✅ You are up to date");
                }
                UpdateState::Checking => {
                    ui.spinner();
                    ui.label("Checking...");
                }
                UpdateState::UpdateAvailable { version, description } => {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("📦 Version {} available", version),
                    );
                }
                UpdateState::Error { message } => {
                    ui.colored_label(egui::Color32::RED, format!("❌ {}", message));
                }
                _ => {}
            }
        });

        ui.add_space(10.0);

        // Informacje o oczekującej aktualizacji
        if let Some(ref metadata) = self.pending_metadata.clone() {
            ui.group(|ui| {
                ui.heading("📋 Available Update Details");
                ui.separator();

                egui::Grid::new("update_details")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("App ID:");
                        ui.label(&metadata.app_id);
                        ui.end_row();

                        ui.label("New Version:");
                        ui.colored_label(
                            egui::Color32::GREEN,
                            format!("{}", metadata.version),
                        );
                        ui.end_row();

                        ui.label("Current:");
                        ui.label(format!("{}", self.config.current_version));
                        ui.end_row();

                        ui.label("Publisher:");
                        ui.label(&metadata.publisher_id);
                        ui.end_row();

                        ui.label("File:");
                        ui.label(&metadata.filename);
                        ui.end_row();

                        ui.label("Size:");
                        ui.label(format!("{} bytes", metadata.file_size));
                        ui.end_row();

                        ui.label("SHA3-256:");
                        ui.label(format!("{}...", &metadata.sha3_256_hash[..32.min(metadata.sha3_256_hash.len())]));
                        ui.end_row();

                        ui.label("Published:");
                        ui.label(format!("{}", metadata.published_at));
                        ui.end_row();

                        ui.label("Description:");
                        ui.label(&metadata.description);
                        ui.end_row();
                    });

                if !metadata.changelog.is_empty() {
                    ui.add_space(5.0);
                    ui.label("Changelog:");
                    for entry in &metadata.changelog {
                        ui.label(format!("  • {}", entry));
                    }
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui
                        .button("⬇️ Download & Verify")
                        .on_hover_text("Download the package and verify its signatures")
                        .clicked()
                    {
                        self.perform_download_and_verify();
                    }

                    if self.downloaded_data.is_some() && self.verification_report.as_ref().map_or(false, |r| r.overall_valid) {
                        if ui
                            .button("📥 Apply Update")
                            .on_hover_text("Install the verified update")
                            .clicked()
                        {
                            self.perform_apply_update();
                        }
                    }
                });
            });
        }

        // Raport weryfikacji
        if let Some(ref report) = self.verification_report {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.heading("🔍 Verification Report");
                ui.separator();

                egui::Grid::new("verification_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        render_check_row(ui, "File size", report.size_check);
                        render_check_row(ui, "SHA3-256 hash", report.hash_check);
                        render_check_row(ui, "Dilithium3 signature (PQ)", report.dilithium_valid);
                        render_check_row(ui, "Ed25519 signature", report.ed25519_valid);
                        render_check_row(ui, "Anti-downgrade", report.version_check);
                        render_check_row(ui, "Publisher identity", report.publisher_check);
                    });

                ui.add_space(5.0);
                if report.overall_valid {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        "✅ ALL CHECKS PASSED - Package is authentic and intact",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::RED,
                        "❌ VERIFICATION FAILED - DO NOT INSTALL",
                    );
                    for err in &report.errors {
                        ui.colored_label(egui::Color32::RED, format!("  ⚠ {}", err));
                    }
                }
            });
        }
    }

    // ============================================================
    // Security Tab
    // ============================================================
    fn render_security(&mut self, ui: &mut egui::Ui) {
        ui.heading("🛡️ Security Analysis");
        ui.separator();

        if ui.button("🔍 Run Full Security Check").clicked() {
            self.perform_hardening_check();
        }

        ui.add_space(10.0);

        // Hardening report
        if let Some(ref report) = self.hardening_report {
            ui.group(|ui| {
                ui.heading("Client Hardening Report");
                ui.separator();

                egui::Grid::new("hardening_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Self-integrity:");
                        if report.self_integrity_ok {
                            ui.colored_label(egui::Color32::GREEN, "✅ OK");
                        } else {
                            ui.colored_label(egui::Color32::RED, "❌ COMPROMISED");
                        }
                        ui.end_row();

                        ui.label("Debugger detection:");
                        if !report.debugger_detected {
                            ui.colored_label(egui::Color32::GREEN, "✅ No debugger");
                        } else {
                            ui.colored_label(egui::Color32::RED, "⚠️ Debugger detected!");
                        }
                        ui.end_row();

                        ui.label("Environment:");
                        if report.environment_warnings.is_empty() {
                            ui.colored_label(egui::Color32::GREEN, "✅ Clean");
                        } else {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("⚠️ {} warnings", report.environment_warnings.len()),
                            );
                        }
                        ui.end_row();
                    });

                if !report.environment_warnings.is_empty() {
                    ui.add_space(5.0);
                    ui.label("Environment warnings:");
                    for w in &report.environment_warnings {
                        ui.colored_label(egui::Color32::YELLOW, format!("  ⚠ {}", w));
                    }
                }
            });
        }

        ui.add_space(10.0);

        // Pinned keys
        ui.group(|ui| {
            ui.heading("🔑 Pinned Publisher Keys");
            ui.separator();

            if self.config.pinned_publisher_keys.is_empty() {
                ui.label("No keys pinned. Keys will be pinned on first successful update check.");
            } else {
                for key in &self.config.pinned_publisher_keys {
                    ui.horizontal(|ui| {
                        ui.label(format!("📌 {}", key.publisher_id));
                        ui.label(format!(
                            "(Dilithium: {}...)",
                            &key.dilithium_public_key[..20.min(key.dilithium_public_key.len())]
                        ));
                    });
                }
            }
        });

        ui.add_space(10.0);

        // Threat model info
        ui.group(|ui| {
            ui.heading("🎯 Threat Model");
            ui.separator();
            ui.label("Protected against:");
            ui.label("  ✅ MITM attacks (signature verification)");
            ui.label("  ✅ Package tampering (SHA3-256 + dual signatures)");
            ui.label("  ✅ Downgrade attacks (monotonic versioning)");
            ui.label("  ✅ Key compromise (hybrid PQ + classical)");
            ui.label("  ✅ Quantum threats (CRYSTALS-Dilithium3)");
            ui.label("  ✅ Replay attacks (timestamped signatures)");
            ui.add_space(5.0);
            ui.label("Limitations:");
            ui.label("  ⚠ Prototype uses HTTP (production should use TLS)");
            ui.label("  ⚠ TOFU key pinning (no CA hierarchy)");
            ui.label("  ⚠ Single-signature per package (no threshold)");
        });
    }

    // ============================================================
    // Settings Tab
    // ============================================================
    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Settings");
        ui.separator();

        ui.group(|ui| {
            ui.heading("Server Configuration");
            ui.horizontal(|ui| {
                ui.label("Server URL:");
                ui.text_edit_singleline(&mut self.server_url_input);
            });
            ui.horizontal(|ui| {
                ui.label("App ID:");
                ui.text_edit_singleline(&mut self.app_id_input);
            });
            ui.horizontal(|ui| {
                ui.label("Current Version:");
                ui.text_edit_singleline(&mut self.version_input);
            });

            ui.add_space(5.0);

            ui.checkbox(&mut self.config.auto_download, "Auto-download updates");

            ui.add_space(5.0);

            if ui.button("💾 Save Settings").clicked() {
                self.config.server_url = self.server_url_input.clone();
                self.config.app_id = self.app_id_input.clone();
                if let Ok(v) = SemanticVersion::parse(&self.version_input) {
                    self.config.current_version = v;
                }
                if let Err(e) = config::save_config(&self.config) {
                    self.add_log(LogLevel::Error, &format!("Failed to save config: {}", e));
                } else {
                    self.add_log(LogLevel::Success, "Settings saved");
                }
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("Key Management");
            if ui.button("🗑️ Clear Pinned Keys").clicked() {
                self.config.pinned_publisher_keys.clear();
                let _ = config::save_config(&self.config);
                self.add_log(LogLevel::Warning, "All pinned keys cleared");
            }
        });
    }

    // ============================================================
    // Logs Tab
    // ============================================================
    fn render_logs(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Activity Log");
        ui.separator();

        if ui.button("🗑️ Clear Logs").clicked() {
            self.log_messages.clear();
        }

        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for entry in self.log_messages.iter().rev() {
                    let color = match entry.level {
                        LogLevel::Info => egui::Color32::LIGHT_GRAY,
                        LogLevel::Success => egui::Color32::GREEN,
                        LogLevel::Warning => egui::Color32::YELLOW,
                        LogLevel::Error => egui::Color32::RED,
                    };
                    ui.colored_label(
                        color,
                        format!("[{}] {}", entry.timestamp, entry.message),
                    );
                }
            });
    }

    // ============================================================
    // Actions
    // ============================================================

    fn perform_health_check(&mut self) {
        self.add_log(LogLevel::Info, &format!("Pinging {}...", self.config.server_url));

        match reqwest::blocking::get(format!("{}/api/health", self.config.server_url)) {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().unwrap_or_default();
                let msg = format!("Server OK: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
                self.last_check_result = Some(msg.clone());
                self.add_log(LogLevel::Success, &msg);
            }
            Ok(resp) => {
                let msg = format!("Server returned: {}", resp.status());
                self.last_check_result = Some(msg.clone());
                self.add_log(LogLevel::Error, &msg);
            }
            Err(e) => {
                let msg = format!("Connection failed: {}", e);
                self.last_check_result = Some(msg.clone());
                self.add_log(LogLevel::Error, &msg);
            }
        }
    }

    fn perform_update_check(&mut self) {
        self.add_log(LogLevel::Info, "Checking for updates...");
        self.update_state = UpdateState::Checking;

        match updater::check_for_update(
            &self.config.server_url,
            &self.config.app_id,
            &self.config.current_version,
        ) {
            Ok(response) => {
                if response.update_available {
                    if let Some(ref pkg) = response.latest_package {
                        let msg = format!(
                            "Update available: v{} ({})",
                            pkg.version, pkg.description
                        );
                        self.add_log(LogLevel::Success, &msg);
                        self.update_state = UpdateState::UpdateAvailable {
                            version: pkg.version.clone(),
                            description: pkg.description.clone(),
                        };
                        self.pending_metadata = response.latest_package;
                        self.pending_publisher_key = response.publisher_public_key.clone();

                        // TOFU key pinning
                        if let Some(ref key) = response.publisher_public_key {
                            let already_pinned = self
                                .config
                                .pinned_publisher_keys
                                .iter()
                                .any(|k| k.publisher_id == key.publisher_id);

                            if !already_pinned {
                                self.config.pinned_publisher_keys.push(key.clone());
                                let _ = config::save_config(&self.config);
                                self.add_log(
                                    LogLevel::Info,
                                    &format!("Pinned key for publisher: {}", key.publisher_id),
                                );
                            }
                        }
                    }
                } else {
                    self.update_state = UpdateState::UpToDate;
                    self.add_log(LogLevel::Info, "No updates available");
                    self.last_check_result = Some("✅ You are up to date".to_string());
                }
            }
            Err(e) => {
                let msg = format!("Update check failed: {}", e);
                self.update_state = UpdateState::Error {
                    message: msg.clone(),
                };
                self.add_log(LogLevel::Error, &msg);
            }
        }
    }

    fn perform_download_and_verify(&mut self) {
        let metadata = match &self.pending_metadata {
            Some(m) => m.clone(),
            None => {
                self.add_log(LogLevel::Error, "No pending update to download");
                return;
            }
        };

        let publisher_key = match &self.pending_publisher_key {
            Some(k) => k.clone(),
            None => {
                self.add_log(LogLevel::Error, "No publisher key available");
                return;
            }
        };

        self.add_log(LogLevel::Info, &format!("Downloading v{}...", metadata.version));

        // Download
        match updater::download_package(
            &self.config.server_url,
            &metadata.app_id,
            &metadata.version.to_string(),
        ) {
            Ok(data) => {
                self.add_log(
                    LogLevel::Success,
                    &format!("Downloaded {} bytes", data.len()),
                );

                // Verify
                self.add_log(LogLevel::Info, "Verifying signatures...");
                match crate::verifier::verify_package(
                    &data,
                    &metadata,
                    &publisher_key,
                    &self.config.current_version,
                ) {
                    Ok(report) => {
                        if report.overall_valid {
                            self.add_log(LogLevel::Success, "✅ All verification checks passed!");
                            self.update_state = UpdateState::ReadyToInstall;
                        } else {
                            self.add_log(LogLevel::Error, "❌ Verification FAILED!");
                            for err in &report.errors {
                                self.add_log(LogLevel::Error, err);
                            }
                        }
                        self.verification_report = Some(report);
                        self.downloaded_data = Some(data);
                    }
                    Err(e) => {
                        self.add_log(
                            LogLevel::Error,
                            &format!("Verification error: {}", e),
                        );
                    }
                }
            }
            Err(e) => {
                self.add_log(LogLevel::Error, &format!("Download failed: {}", e));
                self.update_state = UpdateState::Error {
                    message: format!("Download failed: {}", e),
                };
            }
        }
    }

    fn perform_apply_update(&mut self) {
        let metadata = match &self.pending_metadata {
            Some(m) => m.clone(),
            None => return,
        };
        let data = match &self.downloaded_data {
            Some(d) => d.clone(),
            None => return,
        };

        self.add_log(LogLevel::Info, "Applying update...");

        match updater::apply_update(&data, &metadata, &self.config.install_dir) {
            Ok(_) => {
                self.config.current_version = metadata.version.clone();
                self.version_input = self.config.current_version.to_string();
                let _ = config::save_config(&self.config);

                self.update_state = UpdateState::Completed;
                self.add_log(
                    LogLevel::Success,
                    &format!("✅ Updated to v{}!", metadata.version),
                );
                self.pending_metadata = None;
                self.downloaded_data = None;
            }
            Err(e) => {
                self.add_log(LogLevel::Error, &format!("Apply failed: {}", e));
                self.update_state = UpdateState::Error {
                    message: format!("{}", e),
                };
            }
        }
    }

    fn perform_hardening_check(&mut self) {
        self.add_log(LogLevel::Info, "Running security checks...");
        let report = anti_tamper::full_hardening_check();

        if report.overall_safe {
            self.add_log(LogLevel::Success, "Environment is safe");
        } else {
            if report.debugger_detected {
                self.add_log(LogLevel::Warning, "Debugger detected!");
            }
            for w in &report.environment_warnings {
                self.add_log(LogLevel::Warning, w);
            }
        }

        self.hardening_report = Some(report);
    }
}

/// Helper do renderowania wiersza weryfikacji
fn render_check_row(ui: &mut egui::Ui, label: &str, passed: bool) {
    ui.label(format!("{}:", label));
    if passed {
        ui.colored_label(egui::Color32::GREEN, "✅ PASS");
    } else {
        ui.colored_label(egui::Color32::RED, "❌ FAIL");
    }
    ui.end_row();
}