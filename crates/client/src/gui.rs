use chrono::Utc;
use eframe::egui;
use secure_update_common::*;

use crate::{anti_tamper, config, updater};
use crate::verifier::VerificationReport;

// ─── Typy pomocnicze ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    ServersApps,
    Dashboard,
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

// ─── Stan aplikacji ─────────────────────────────────────────────

pub struct UpdateApp {
    config: ClientConfig,
    active_tab: Tab,

    // Serwery & Apps
    apps_list: Vec<AppInfo>,
    selected_server: String,
    new_server_input: String,

    // Aktywna operacja
    active_app_id: String,
    update_state: UpdateState,
    pending_metadata: Option<PackageMetadata>,
    pending_publisher_key: Option<HybridPublicKey>,
    downloaded_data: Option<Vec<u8>>,
    verification_report: Option<VerificationReport>,

    // Security
    hardening_report: Option<anti_tamper::HardeningReport>,

    // Logi
    log_messages: Vec<LogEntry>,

    // Settings inputs
    new_server_settings_input: String,
    download_dir_input: String,
    install_dir_input: String,
}

impl UpdateApp {
    pub fn new() -> Self {
        let config = config::load_or_create_config().unwrap_or_default();
        let selected_server = config.selected_server.clone();

        let mut app = Self {
            selected_server: selected_server.clone(),
            new_server_input: String::new(),
            apps_list: Vec::new(),
            active_app_id: config.app_id.clone(),
            update_state: UpdateState::UpToDate,
            pending_metadata: None,
            pending_publisher_key: None,
            downloaded_data: None,
            verification_report: None,
            hardening_report: None,
            log_messages: Vec::new(),
            new_server_settings_input: String::new(),
            download_dir_input: config.download_dir.clone(),
            install_dir_input: config.install_dir.clone(),
            config,
            active_tab: Tab::ServersApps,
        };

        app.add_log(LogLevel::Info, "🚀 Secure Update Manager started");
        app.add_log(LogLevel::Info, "Crypto: Dilithium3 + Ed25519 + SHA3-256");
        app
    }

    fn add_log(&mut self, level: LogLevel, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.log_messages.push(LogEntry {
            timestamp,
            level,
            message: message.to_string(),
        });
        if self.log_messages.len() > 500 {
            self.log_messages.remove(0);
        }
    }

    fn get_installed_app(&self, server_url: &str, app_id: &str) -> Option<InstalledApp> {
        self.config
            .installed_apps
            .iter()
            .find(|ia| ia.server_url == server_url && ia.app_id == app_id)
            .cloned()
    }

    fn refresh_apps_list(&mut self) {
        let server = self.selected_server.clone();
        self.add_log(LogLevel::Info, &format!("Fetching apps from {}...", server));
        match updater::fetch_apps(&server) {
            Ok(resp) => {
                self.apps_list = resp.apps;
                self.add_log(
                    LogLevel::Success,
                    &format!("Found {} apps", self.apps_list.len()),
                );
            }
            Err(e) => {
                self.add_log(LogLevel::Error, &format!("Fetch failed: {}", e));
                self.apps_list.clear();
            }
        }
    }

    fn select_server(&mut self, server: String) {
        self.selected_server = server.clone();
        self.config.selected_server = server;
        self.apps_list.clear();
        self.verification_report = None;
        self.pending_metadata = None;
        self.downloaded_data = None;
        let _ = config::save_config(&self.config);
        self.refresh_apps_list();
    }

    // ─── Pełny flow: check → download → verify → apply ──────────

    fn start_install_or_update(&mut self, app_id: &str) {
        self.active_app_id = app_id.to_string();
        self.config.app_id = app_id.to_string();
        self.verification_report = None;
        self.pending_metadata = None;
        self.downloaded_data = None;

        // 1. Pobierz zainstalowaną wersję (lub 0.0.0 jeśli nie zainstalowana)
        let current_version = self
            .get_installed_app(&self.selected_server, app_id)
            .map(|ia| ia.installed_version.clone())
            .unwrap_or_else(|| SemanticVersion::new(0, 0, 0));

        let server = self.selected_server.clone();
        self.add_log(LogLevel::Info, &format!("Checking {} on {}...", app_id, server));

        // 2. Check
        let check_result = updater::check_for_update(&server, app_id, &current_version);

        match check_result {
            Err(e) => {
                let msg = format!("Check failed: {}", e);
                self.add_log(LogLevel::Error, &msg);
                self.update_state = UpdateState::Error { message: msg };
                return;
            }
            Ok(resp) => {
                if !resp.update_available {
                    self.add_log(LogLevel::Info, "Already up to date");
                    self.update_state = UpdateState::UpToDate;
                    return;
                }

                let metadata = match resp.latest_package {
                    Some(m) => m,
                    None => {
                        self.add_log(LogLevel::Error, "No metadata in response");
                        return;
                    }
                };

                let publisher_key = match resp.publisher_public_key {
                    Some(k) => k,
                    None => {
                        self.add_log(LogLevel::Error, "No publisher key in response");
                        return;
                    }
                };

                // TOFU key pinning
                let pinned = self
                    .config
                    .pinned_publisher_keys_by_server
                    .entry(server.clone())
                    .or_default();
                let already_pinned = pinned.iter().any(|k| k.publisher_id == publisher_key.publisher_id);
                if !already_pinned {
                    pinned.push(publisher_key.clone());
                    self.add_log(LogLevel::Info, &format!("Pinned key: {}", publisher_key.publisher_id));
                }

                let version_str = metadata.version.to_string();
                let app_id_str = metadata.app_id.clone();

                self.add_log(LogLevel::Success, &format!("Update found: v{}", version_str));
                self.update_state = UpdateState::Downloading { progress_percent: 0.0 };

                // 3. Download
                match updater::download_package(&server, &app_id_str, &version_str) {
                    Err(e) => {
                        let msg = format!("Download failed: {}", e);
                        self.add_log(LogLevel::Error, &msg);
                        self.update_state = UpdateState::Error { message: msg };
                        return;
                    }
                    Ok(data) => {
                        self.add_log(LogLevel::Success, &format!("Downloaded {} bytes", data.len()));
                        self.update_state = UpdateState::Verifying;

                        // 4. Verify
                        match crate::verifier::verify_package(
                            &data,
                            &metadata,
                            &publisher_key,
                            &current_version,
                        ) {
                            Err(e) => {
                                let msg = format!("Verification error: {}", e);
                                self.add_log(LogLevel::Error, &msg);
                                self.update_state = UpdateState::Error { message: msg };
                                return;
                            }
                            Ok(report) => {
                                if !report.overall_valid {
                                    self.add_log(LogLevel::Error, "❌ Verification FAILED");
                                    for err in &report.errors {
                                        self.add_log(LogLevel::Error, err);
                                    }
                                    self.update_state = UpdateState::Error {
                                        message: "Verification failed".to_string(),
                                    };
                                    self.verification_report = Some(report);
                                    return;
                                }

                                self.add_log(LogLevel::Success, "✅ Verification PASSED");
                                self.update_state = UpdateState::ReadyToInstall;
                                self.verification_report = Some(report);
                                self.downloaded_data = Some(data);
                                self.pending_metadata = Some(metadata.clone());
                                self.pending_publisher_key = Some(publisher_key);

                                // 5. Apply
                                let install_dir = format!(
                                    "{}/{}",
                                    self.config.install_dir.trim_end_matches('/'),
                                    app_id_str
                                );
                                std::fs::create_dir_all(&install_dir).ok();

                                match updater::apply_update(
                                    self.downloaded_data.as_ref().unwrap(),
                                    &metadata,
                                    &install_dir,
                                ) {
                                    Err(e) => {
                                        let msg = format!("Apply failed: {}", e);
                                        self.add_log(LogLevel::Error, &msg);
                                        self.update_state = UpdateState::Error { message: msg };
                                        return;
                                    }
                                    Ok(_) => {
                                        // Zapisz do installed_apps
                                        self.config.installed_apps.retain(|ia| {
                                            !(ia.server_url == server && ia.app_id == app_id_str)
                                        });
                                        self.config.installed_apps.push(InstalledApp {
                                            server_url: server.clone(),
                                            app_id: app_id_str.clone(),
                                            installed_version: metadata.version.clone(),
                                            install_dir: install_dir.clone(),
                                            installed_at: Utc::now(),
                                            last_verified_at: Some(Utc::now()),
                                        });
                                        self.config.current_version = metadata.version.clone();
                                        let _ = config::save_config(&self.config);
                                        self.update_state = UpdateState::Completed;
                                        self.add_log(
                                            LogLevel::Success,
                                            &format!("✅ {} v{} installed to {}", app_id_str, metadata.version, install_dir),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─── eframe::App ────────────────────────────────────────────────

impl eframe::App for UpdateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top nav
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔒 Secure Update Manager");
                ui.separator();
                ui.selectable_value(&mut self.active_tab, Tab::ServersApps, "🌐 Apps");
                ui.selectable_value(&mut self.active_tab, Tab::Dashboard, "📊 Dashboard");
                ui.selectable_value(&mut self.active_tab, Tab::Security, "🛡️ Security");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "⚙️ Settings");
                ui.selectable_value(&mut self.active_tab, Tab::Logs, "📋 Logs");
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let status = match &self.update_state {
                    UpdateState::UpToDate => "✅ Up to date".to_string(),
                    UpdateState::Checking => "🔄 Checking...".to_string(),
                    UpdateState::UpdateAvailable { version, .. } => {
                        format!("📦 Update available: v{}", version)
                    }
                    UpdateState::Downloading { progress_percent } => {
                        format!("⬇️ Downloading {:.0}%", progress_percent)
                    }
                    UpdateState::Verifying => "🔍 Verifying...".to_string(),
                    UpdateState::ReadyToInstall => "✅ Ready to install".to_string(),
                    UpdateState::Installing => "⚙️ Installing...".to_string(),
                    UpdateState::Completed => "🎉 Completed".to_string(),
                    UpdateState::Error { message } => format!("❌ {}", message),
                };
                ui.label(status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.selected_server);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::ServersApps => self.render_servers_apps(ui),
                Tab::Dashboard => self.render_dashboard(ui),
                Tab::Security => self.render_security(ui),
                Tab::Settings => self.render_settings(ui),
                Tab::Logs => self.render_logs(ui),
            }
        });
    }
}

// ─── Zakładki ───────────────────────────────────────────────────

impl UpdateApp {
    // ════════════════════════════════════════════════════════════
    // 🌐 SERVERS & APPS  (główna zakładka)
    // ════════════════════════════════════════════════════════════
    fn render_servers_apps(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌐 Servers & Applications");
        ui.separator();

        // ── Server selector ──────────────────────────────────────
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Server:");

                let servers = self.config.servers.clone();
                egui::ComboBox::from_id_source("server_combo")
                    .selected_text(&self.selected_server)
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for s in &servers {
                            if ui
                                .selectable_value(&mut self.selected_server, s.clone(), s)
                                .clicked()
                            {
                                let srv = self.selected_server.clone();
                                self.select_server(srv);
                            }
                        }
                    });

                ui.separator();
                ui.label("Add server:");
                ui.text_edit_singleline(&mut self.new_server_input);

                if ui.button("➕ Add").clicked() {
                    let ns = self.new_server_input.trim().to_string();
                    if !ns.is_empty() && !self.config.servers.contains(&ns) {
                        self.config.servers.push(ns.clone());
                        self.new_server_input.clear();
                        let _ = config::save_config(&self.config);
                        self.select_server(ns);
                    }
                }

                if ui.button("🗑 Remove").clicked() {
                    let to_remove = self.selected_server.clone();
                    self.config.servers.retain(|s| *s != to_remove);
                    if self.config.servers.is_empty() {
                        self.config.servers.push("http://127.0.0.1:8443".to_string());
                    }
                    let first = self.config.servers[0].clone();
                    let _ = config::save_config(&self.config);
                    self.select_server(first);
                }

                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_apps_list();
                }
            });
        });

        ui.add_space(8.0);

        if self.apps_list.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label("No applications found on this server.");
                ui.add_space(8.0);
                if ui.button("🔄 Load apps").clicked() {
                    self.refresh_apps_list();
                }
            });
            return;
        }

        // ── Tabela aplikacji ─────────────────────────────────────
        let apps = self.apps_list.clone();
        let server = self.selected_server.clone();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("apps_grid")
                .num_columns(7)
                .spacing([16.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    // Nagłówki
                    ui.strong("App ID");
                    ui.strong("Latest");
                    ui.strong("Publisher");
                    ui.strong("Published");
                    ui.strong("Status");
                    ui.strong("Installed");
                    ui.strong("Action");
                    ui.end_row();

                    for app in &apps {
                        let installed = self.get_installed_app(&server, &app.app_id);

                        // App ID
                        ui.label(&app.app_id);

                        // Latest version
                        ui.label(
                            app.latest_version
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "—".into()),
                        );

                        // Publisher
                        ui.label(
                            app.latest_publisher
                                .as_deref()
                                .unwrap_or("—"),
                        );

                        // Last published
                        ui.label(
                            app.last_published_at
                                .as_ref()
                                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                .unwrap_or_else(|| "—".into()),
                        );

                        // Status + installed version + action
                        match &installed {
                            None => {
                                // Nie zainstalowana
                                ui.colored_label(
                                    egui::Color32::from_rgb(150, 150, 150),
                                    "Not installed",
                                );
                                ui.label("—");
                                if ui
                                    .button("⬇️ Install")
                                    .on_hover_text("Download, verify and install")
                                    .clicked()
                                {
                                    let id = app.app_id.clone();
                                    self.start_install_or_update(&id);
                                }
                            }
                            Some(inst) => {
                                let has_update = app
                                    .latest_version
                                    .as_ref()
                                    .map(|lv| lv.is_newer_than(&inst.installed_version))
                                    .unwrap_or(false);

                                if has_update {
                                    ui.colored_label(
                                        egui::Color32::YELLOW,
                                        "Update available",
                                    );
                                } else {
                                    ui.colored_label(
                                        egui::Color32::GREEN,
                                        "✅ Installed",
                                    );
                                }

                                ui.label(inst.installed_version.to_string());

                                if has_update {
                                    if ui
                                        .button("⬆️ Update")
                                        .on_hover_text("Download and install the new version")
                                        .clicked()
                                    {
                                        let id = app.app_id.clone();
                                        self.start_install_or_update(&id);
                                    }
                                } else {
                                    ui.label("—");
                                }
                            }
                        }

                        ui.end_row();
                    }
                });
        });

        // ── Panel wyników weryfikacji (jeśli coś robiło operację) ─
        if let Some(ref report) = self.verification_report {
            ui.add_space(10.0);
            ui.separator();
            ui.group(|ui| {
                ui.heading("🔍 Last verification");
                egui::Grid::new("ver_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        check_row(ui, "File size",      report.size_check);
                        check_row(ui, "SHA3-256 hash",  report.hash_check);
                        check_row(ui, "Dilithium3 (PQ)", report.dilithium_valid);
                        check_row(ui, "Ed25519",         report.ed25519_valid);
                        check_row(ui, "Anti-downgrade",  report.version_check);
                        check_row(ui, "Publisher",       report.publisher_check);
                    });

                ui.add_space(4.0);
                if report.overall_valid {
                    ui.colored_label(egui::Color32::GREEN, "✅ ALL CHECKS PASSED");
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ VERIFICATION FAILED");
                    for e in &report.errors {
                        ui.colored_label(egui::Color32::RED, format!("  ⚠ {}", e));
                    }
                }
            });
        }
    }

    // ════════════════════════════════════════════════════════════
    // 📊 DASHBOARD
    // ════════════════════════════════════════════════════════════
    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Dashboard");
        ui.separator();

        egui::Grid::new("dash_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("📦 Installed apps");
                    ui.separator();
                    let installed: Vec<_> = self
                        .config
                        .installed_apps
                        .iter()
                        .filter(|ia| ia.server_url == self.selected_server)
                        .collect();
                    if installed.is_empty() {
                        ui.label("None installed from this server");
                    } else {
                        for ia in installed {
                            ui.label(format!("• {} v{}", ia.app_id, ia.installed_version));
                        }
                    }
                });

                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("🌐 Server");
                    ui.separator();
                    ui.label(format!("URL: {}", self.selected_server));
                    ui.label(format!("Known servers: {}", self.config.servers.len()));
                    ui.label(format!("Apps visible: {}", self.apps_list.len()));
                });

                ui.end_row();

                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("🔐 Cryptography");
                    ui.separator();
                    ui.label("PQ:      CRYSTALS-Dilithium3 (ML-DSA-65)");
                    ui.label("Classic: Ed25519");
                    ui.label("Hash:    SHA3-256");
                    ui.label("Scheme:  Hybrid (both required)");
                });

                ui.group(|ui| {
                    ui.set_min_width(350.0);
                    ui.heading("📌 Pinned keys");
                    ui.separator();
                    let keys = self
                        .config
                        .pinned_publisher_keys_by_server
                        .get(&self.selected_server)
                        .map(|v| v.len())
                        .unwrap_or(0);
                    ui.label(format!("Pinned for current server: {}", keys));
                    ui.label("Anti-downgrade: ✅ Enabled");
                });
            });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh apps").clicked() {
                self.refresh_apps_list();
            }
            if ui.button("🛡️ Security check").clicked() {
                self.perform_hardening_check();
            }
            if ui.button("🏥 Health check").clicked() {
                self.perform_health_check();
            }
        });
    }

    // ════════════════════════════════════════════════════════════
    // 🛡️ SECURITY
    // ════════════════════════════════════════════════════════════
    fn render_security(&mut self, ui: &mut egui::Ui) {
        ui.heading("🛡️ Security");
        ui.separator();

        if ui.button("🔍 Run security check").clicked() {
            self.perform_hardening_check();
        }

        ui.add_space(10.0);

        if let Some(ref report) = self.hardening_report {
            ui.group(|ui| {
                ui.heading("Client hardening report");
                ui.separator();
                egui::Grid::new("hard_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Self-integrity:");
                        bool_label(ui, report.self_integrity_ok, "OK", "COMPROMISED");

                        ui.label("Debugger:");
                        bool_label(ui, !report.debugger_detected, "Not detected", "⚠️ DETECTED");

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

                for w in &report.environment_warnings {
                    ui.colored_label(egui::Color32::YELLOW, format!("  ⚠ {}", w));
                }
            });
        }

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("🔑 Pinned keys (current server)");
            ui.separator();
            let keys = self
                .config
                .pinned_publisher_keys_by_server
                .get(&self.selected_server)
                .cloned()
                .unwrap_or_default();
            if keys.is_empty() {
                ui.label("No keys pinned yet (keys pinned on first install)");
            } else {
                for k in &keys {
                    ui.label(format!(
                        "📌 {} (Dilithium: {}...)",
                        k.publisher_id,
                        &k.dilithium_public_key[..20.min(k.dilithium_public_key.len())]
                    ));
                }
            }
            ui.add_space(4.0);
            if ui.button("🗑 Clear pinned keys for this server").clicked() {
                self.config
                    .pinned_publisher_keys_by_server
                    .remove(&self.selected_server);
                let _ = config::save_config(&self.config);
                self.add_log(LogLevel::Warning, "Pinned keys cleared");
            }
        });

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.heading("🎯 Protected against");
            ui.separator();
            ui.label("✅ MITM / package tampering  (SHA3-256 + Dilithium3 + Ed25519)");
            ui.label("✅ Downgrade attacks          (monotonic versioning)");
            ui.label("✅ Quantum threats            (CRYSTALS-Dilithium3)");
            ui.label("✅ Key compromise             (hybrid scheme)");
            ui.label("✅ Replay attacks             (timestamped signatures)");
            ui.label("⚠️ Transport (prototyp HTTP → produkcja TLS 1.3)");
        });
    }

    // ════════════════════════════════════════════════════════════
    // ⚙️ SETTINGS
    // ════════════════════════════════════════════════════════════
    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Settings");
        ui.separator();

        ui.group(|ui| {
            ui.heading("Directories");
            ui.horizontal(|ui| {
                ui.label("Download dir:");
                ui.text_edit_singleline(&mut self.download_dir_input);
            });
            ui.horizontal(|ui| {
                ui.label("Install dir:");
                ui.text_edit_singleline(&mut self.install_dir_input);
            });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.checkbox(&mut self.config.auto_download, "Auto-download updates");
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("Installed apps");
            ui.separator();
            let installed = self.config.installed_apps.clone();
            if installed.is_empty() {
                ui.label("No apps installed");
            } else {
                for ia in &installed {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "  {} | {} | v{} | {}",
                            ia.server_url,
                            ia.app_id,
                            ia.installed_version,
                            ia.install_dir
                        ));
                    });
                }
            }
        });

        ui.add_space(8.0);

        if ui.button("💾 Save").clicked() {
            self.config.download_dir = self.download_dir_input.clone();
            self.config.install_dir = self.install_dir_input.clone();
            if let Err(e) = config::save_config(&self.config) {
                self.add_log(LogLevel::Error, &format!("Save failed: {}", e));
            } else {
                self.add_log(LogLevel::Success, "Settings saved");
            }
        }
    }

    // ════════════════════════════════════════════════════════════
    // 📋 LOGS
    // ════════════════════════════════════════════════════════════
    fn render_logs(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Logs");
        ui.separator();

        if ui.button("🗑 Clear").clicked() {
            self.log_messages.clear();
        }

        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for entry in self.log_messages.iter().rev() {
                    let color = match entry.level {
                        LogLevel::Info    => egui::Color32::LIGHT_GRAY,
                        LogLevel::Success => egui::Color32::GREEN,
                        LogLevel::Warning => egui::Color32::YELLOW,
                        LogLevel::Error   => egui::Color32::RED,
                    };
                    ui.colored_label(
                        color,
                        format!("[{}] {}", entry.timestamp, entry.message),
                    );
                }
            });
    }

    // ─── Helpers (actions) ───────────────────────────────────────

    fn perform_health_check(&mut self) {
        let url = format!("{}/api/health", self.selected_server);
        self.add_log(LogLevel::Info, &format!("Health check: {}", url));
        match reqwest::blocking::get(&url) {
            Ok(r) if r.status().is_success() => {
                self.add_log(LogLevel::Success, "Server is healthy ✅");
            }
            Ok(r) => {
                self.add_log(LogLevel::Error, &format!("Server returned {}", r.status()));
            }
            Err(e) => {
                self.add_log(LogLevel::Error, &format!("Connection failed: {}", e));
            }
        }
    }

    fn perform_hardening_check(&mut self) {
        self.add_log(LogLevel::Info, "Running security checks...");
        let report = anti_tamper::full_hardening_check();
        if report.overall_safe {
            self.add_log(LogLevel::Success, "Environment is safe ✅");
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

// ─── UI helpers ─────────────────────────────────────────────────

fn check_row(ui: &mut egui::Ui, label: &str, ok: bool) {
    ui.label(format!("{}:", label));
    if ok {
        ui.colored_label(egui::Color32::GREEN, "✅ PASS");
    } else {
        ui.colored_label(egui::Color32::RED, "❌ FAIL");
    }
    ui.end_row();
}

fn bool_label(ui: &mut egui::Ui, ok: bool, ok_text: &str, fail_text: &str) {
    if ok {
        ui.colored_label(egui::Color32::GREEN, format!("✅ {}", ok_text));
    } else {
        ui.colored_label(egui::Color32::RED, format!("❌ {}", fail_text));
    }
    ui.end_row();
}