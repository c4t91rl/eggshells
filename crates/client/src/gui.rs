use chrono::Utc;
use eframe::egui;
use secure_update_common::*;

use crate::verifier::VerificationReport;
use crate::{anti_tamper, config, updater};

#[derive(Debug, Clone, PartialEq)]
enum Tab {
    Apps,
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

#[derive(Debug, Clone, Default)]
struct HealthInfo {
    is_healthy: bool,
    service_name: String,
    service_version: String,
    pq_algo: String,
    classical_algo: String,
    hash_algo: String,
    timestamp: String,
    latency_ms: u128,
    error: Option<String>,
}

pub struct UpdateApp {
    config: ClientConfig,
    active_tab: Tab,

    apps_list: Vec<AppInfo>,
    selected_server: String,
    new_server_input: String,

    active_app_id: String,
    update_state: UpdateState,
    pending_metadata: Option<PackageMetadata>,
    pending_publisher_key: Option<HybridPublicKey>,
    downloaded_data: Option<Vec<u8>>,
    verification_report: Option<VerificationReport>,

    confirm_uninstall: Option<String>,

    health_info: Option<HealthInfo>,
    hardening_report: Option<anti_tamper::HardeningReport>,

    log_messages: Vec<LogEntry>,

    download_dir_input: String,
    install_dir_input: String,
}

impl UpdateApp {
    pub fn new() -> Self {
        let config =
            config::load_or_create_config().unwrap_or_default();

        let mut app = Self {
            selected_server: config.selected_server.clone(),
            new_server_input: String::new(),
            apps_list: Vec::new(),
            active_app_id: config.app_id.clone(),
            update_state: UpdateState::UpToDate,
            pending_metadata: None,
            pending_publisher_key: None,
            downloaded_data: None,
            verification_report: None,
            confirm_uninstall: None,
            health_info: None,
            hardening_report: None,
            log_messages: Vec::new(),
            download_dir_input: config.download_dir.clone(),
            install_dir_input: config.install_dir.clone(),
            config,
            active_tab: Tab::Apps,
        };

        app.add_log(
            LogLevel::Info,
            "🚀 Secure Update Manager started",
        );
        app
    }

    fn add_log(&mut self, level: LogLevel, msg: &str) {
        self.log_messages.push(LogEntry {
            timestamp: chrono::Local::now()
                .format("%H:%M:%S")
                .to_string(),
            level,
            message: msg.to_string(),
        });
        if self.log_messages.len() > 500 {
            self.log_messages.remove(0);
        }
    }
}

impl eframe::App for UpdateApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
    ) {
        if let Some(ref app_id) =
            self.confirm_uninstall.clone()
        {
            egui::Window::new("⚠️ Confirm Uninstall")
                .collapsible(false)
                .resizable(false)
                .anchor(
                    egui::Align2::CENTER_CENTER,
                    [0.0, 0.0],
                )
                .show(ctx, |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        "Are you sure you want to uninstall",
                    );
                    ui.strong(app_id.as_str());
                    ui.label("This will delete all installed files.");
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let btn = egui::Button::new(
                            "🗑 Yes, uninstall",
                        )
                        .fill(egui::Color32::from_rgb(
                            180, 40, 40,
                        ));
                        if ui.add(btn).clicked() {
                            let id = app_id.clone();
                            self.uninstall_app(&id);
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_uninstall = None;
                        }
                    });
                });
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Secure Update Manager");
                ui.separator();
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Apps,
                    "🌐 Apps",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Dashboard,
                    "📊 Dashboard",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Security,
                    "🛡 Security",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Settings,
                    "⚙ Settings",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    Tab::Logs,
                    "📋 Logs",
                );
            });
        });

        egui::TopBottomPanel::bottom("bottom").show(
            ctx,
            |ui| {
                ui.horizontal(|ui| {
                    let status = match &self.update_state {
                        UpdateState::UpToDate => "✅ Ready",
                        UpdateState::Checking => "🔄 Checking...",
                        UpdateState::UpdateAvailable { .. } => {
                            "📦 Update available"
                        }
                        UpdateState::Downloading { .. } => {
                            "⬇️ Downloading..."
                        }
                        UpdateState::Verifying => "🔍 Verifying...",
                        UpdateState::ReadyToInstall => "✅ Ready to install",
                        UpdateState::Installing => "⚙ Installing...",
                        UpdateState::Completed => "🎉 Done",
                        UpdateState::Error { .. } => "❌ Error",
                    };
                    ui.label(status);

                    ui.with_layout(
                        egui::Layout::right_to_left(
                            egui::Align::Center,
                        ),
                        |ui| {
                            let dot = match &self.health_info {
                                Some(h) if h.is_healthy => "🟢",
                                Some(_) => "🔴",
                                None => "⚪",
                            };
                            ui.label(format!(
                                "{} {}",
                                dot, self.selected_server
                            ));
                        },
                    );
                });
            },
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Apps => self.tab_apps(ui, ctx),
                Tab::Dashboard => self.tab_dashboard(ui),
                Tab::Security => self.tab_security(ui),
                Tab::Settings => self.tab_settings(ui),
                Tab::Logs => self.tab_logs(ui),
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════
//  TABS
// ═══════════════════════════════════════════════════════════════

impl UpdateApp {
    fn tab_apps(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
    ) {
        ui.heading("🌐 Applications");
        ui.add_space(4.0);

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Server:");
                let servers = self.config.servers.clone();
                egui::ComboBox::from_id_source("srv")
                    .selected_text(&self.selected_server)
                    .width(280.0)
                    .show_ui(ui, |ui| {
                        for s in &servers {
                            if ui
                                .selectable_value(
                                    &mut self.selected_server,
                                    s.clone(),
                                    s,
                                )
                                .clicked()
                            {
                                let srv =
                                    self.selected_server.clone();
                                self.select_server(srv);
                            }
                        }
                    });

                ui.text_edit_singleline(
                    &mut self.new_server_input,
                )
                .on_hover_text("https://host:port");

                if ui
                    .button("➕")
                    .on_hover_text("Add server")
                    .clicked()
                {
                    let ns = self
                        .new_server_input
                        .trim()
                        .to_string();
                    if !ns.is_empty()
                        && !self.config.servers.contains(&ns)
                    {
                        self.config.servers.push(ns.clone());
                        self.new_server_input.clear();
                        let _ = config::save_config(
                            &self.config,
                        );
                        self.select_server(ns);
                    }
                }
                if ui
                    .button("🗑")
                    .on_hover_text("Remove server")
                    .clicked()
                {
                    let rm = self.selected_server.clone();
                    self.config
                        .servers
                        .retain(|s| *s != rm);
                    if self.config.servers.is_empty() {
                        self.config.servers.push(
                            "https://127.0.0.1:8443".into(),
                        );
                    }
                    let first = self.config.servers[0].clone();
                    let _ =
                        config::save_config(&self.config);
                    self.select_server(first);
                }
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_apps_list();
                }
            });
        });

        ui.add_space(6.0);

        if self.apps_list.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.heading("No applications found");
                ui.add_space(4.0);
                ui.label(
                    "Click Refresh to load apps from the server.",
                );
                ui.add_space(12.0);
                if ui.button("🔄 Load apps").clicked() {
                    self.refresh_apps_list();
                }
            });
            return;
        }

        let apps = self.apps_list.clone();
        let server = self.selected_server.clone();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for app in &apps {
                let installed =
                    self.get_installed_app(&server, &app.app_id);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(app.app_id.as_str());
                            ui.horizontal(|ui| {
                                ui.label("Latest:");
                                ui.label(
                                    app.latest_version
                                        .as_ref()
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|| {
                                            "—".into()
                                        }),
                                );
                                ui.label("•");
                                ui.label(
                                    app.latest_publisher
                                        .as_deref()
                                        .unwrap_or("—"),
                                );
                            });
                            if let Some(dt) =
                                &app.last_published_at
                            {
                                ui.label(format!(
                                    "Published: {}",
                                    dt.format(
                                        "%Y-%m-%d %H:%M"
                                    )
                                ));
                            }
                        });

                        ui.with_layout(
                            egui::Layout::right_to_left(
                                egui::Align::Center,
                            ),
                            |ui| match &installed {
                                None => {
                                    if ui
                                        .button("⬇️ Install")
                                        .clicked()
                                    {
                                        let id =
                                            app.app_id.clone();
                                        self.start_install_or_update(
                                            &id, ctx,
                                        );
                                    }
                                    ui.colored_label(
                                        egui::Color32::from_rgb(
                                            150, 150, 150,
                                        ),
                                        "Not installed",
                                    );
                                }
                                Some(inst) => {
                                    let has_update = app
                                        .latest_version
                                        .as_ref()
                                        .map(|lv| {
                                            lv.is_newer_than(
                                                &inst
                                                    .installed_version,
                                            )
                                        })
                                        .unwrap_or(false);

                                    let del = egui::Button::new(
                                        "🗑",
                                    )
                                    .fill(
                                        egui::Color32::from_rgb(
                                            120, 30, 30,
                                        ),
                                    );
                                    if ui
                                        .add(del)
                                        .on_hover_text(
                                            "Uninstall",
                                        )
                                        .clicked()
                                    {
                                        self.confirm_uninstall =
                                            Some(
                                                app.app_id
                                                    .clone(),
                                            );
                                    }

                                    if has_update {
                                        if ui
                                            .button("⬆️ Update")
                                            .clicked()
                                        {
                                            let id = app
                                                .app_id
                                                .clone();
                                            self.start_install_or_update(
                                                &id, ctx,
                                            );
                                        }
                                        ui.colored_label(
                                            egui::Color32::YELLOW,
                                            format!(
                                                "v{} → {}",
                                                inst
                                                    .installed_version,
                                                app.latest_version
                                                    .as_ref()
                                                    .unwrap()
                                            ),
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::GREEN,
                                            format!(
                                                "✅ v{}",
                                                inst
                                                    .installed_version
                                            ),
                                        );
                                    }
                                }
                            },
                        );
                    });
                });
                ui.add_space(2.0);
            }
        });

        if let Some(ref report) = self.verification_report {
            ui.add_space(8.0);
            ui.separator();
            self.render_verification_panel(ui, report);
        }
    }

    fn tab_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 Dashboard");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if ui.button("➕ Health Check").clicked() {
                self.perform_health_check();
            }
            if ui.button("🔄 Refresh Apps").clicked() {
                self.refresh_apps_list();
            }
            if ui.button("🛡 Security Check").clicked() {
                self.perform_hardening_check();
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("➕ Server Health");
            ui.separator();

            match &self.health_info {
                None => {
                    ui.label(
                        "No health check performed yet. \
                         Click Health Check above.",
                    );
                }
                Some(h) if !h.is_healthy => {
                    ui.colored_label(
                        egui::Color32::RED,
                        "🔴 Server unreachable",
                    );
                    if let Some(ref err) = h.error {
                        ui.label(format!("Error: {}", err));
                    }
                }
                Some(h) => {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        "🟢 Server is healthy",
                    );
                    ui.add_space(4.0);

                    egui::Grid::new("health_grid")
                        .num_columns(2)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Service:");
                            ui.strong(h.service_name.as_str());
                            ui.end_row();

                            ui.label("Version:");
                            ui.label(h.service_version.as_str());
                            ui.end_row();

                            ui.label("Response time:");
                            let color = if h.latency_ms < 100 {
                                egui::Color32::GREEN
                            } else if h.latency_ms < 500 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::RED
                            };
                            ui.colored_label(
                                color,
                                format!("{} ms", h.latency_ms),
                            );
                            ui.end_row();

                            ui.label("Server time:");
                            ui.label(h.timestamp.as_str());
                            ui.end_row();

                            ui.label("PQ algorithm:");
                            ui.colored_label(
                                egui::Color32::LIGHT_GREEN,
                                h.pq_algo.as_str(),
                            );
                            ui.end_row();

                            ui.label("Classical:");
                            ui.label(h.classical_algo.as_str());
                            ui.end_row();

                            ui.label("Hash:");
                            ui.label(h.hash_algo.as_str());
                            ui.end_row();
                        });
                }
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("📦 Installed Applications");
            ui.separator();
            let installed: Vec<_> = self
                .config
                .installed_apps
                .iter()
                .filter(|ia| {
                    ia.server_url == self.selected_server
                })
                .collect();
            if installed.is_empty() {
                ui.label(
                    "No apps installed from this server. \
                     Go to 🌐 Apps to install.",
                );
            } else {
                egui::Grid::new("installed_dash")
                    .num_columns(3)
                    .spacing([16.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("App");
                        ui.strong("Version");
                        ui.strong("Location");
                        ui.end_row();
                        for ia in &installed {
                            ui.label(ia.app_id.as_str());
                            ui.label(
                                ia.installed_version
                                    .to_string(),
                            );
                            ui.label(ia.install_dir.as_str());
                            ui.end_row();
                        }
                    });
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🔒 Cryptography");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Post-quantum:");
                ui.colored_label(
                    egui::Color32::LIGHT_GREEN,
                    "CRYSTALS-Dilithium3 (ML-DSA-65)",
                );
            });
            ui.horizontal(|ui| {
                ui.label("Classical:");
                ui.label("Ed25519");
            });
            ui.horizontal(|ui| {
                ui.label("Hash:");
                ui.label("SHA3-256 (Keccak)");
            });
            ui.horizontal(|ui| {
                ui.label("Scheme:");
                ui.label("Hybrid AND (both must be valid)");
            });
        });
    }

    fn tab_security(&mut self, ui: &mut egui::Ui) {
        ui.heading("🛡 Security");
        ui.add_space(4.0);

        if ui.button("🔍 Run Security Check").clicked() {
            self.perform_hardening_check();
        }

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🔒 Client Hardening");
            ui.separator();

            match &self.hardening_report {
                None => {
                    ui.label(
                        "No check performed yet. \
                         Click the button above.",
                    );
                }
                Some(report) => {
                    egui::Grid::new("sec_grid")
                        .num_columns(3)
                        .spacing([12.0, 6.0])
                        .show(ui, |ui| {
                            status_row(
                                ui,
                                "Self-integrity",
                                report.self_integrity_ok,
                                "Binary verified by server",
                                "Binary tampered or unreachable",
                                "Compares SHA3-256 with hash from /api/client/integrity",
                            );
                            status_row(
                                ui,
                                "Debugger",
                                !report.debugger_detected,
                                "Not detected",
                                "Debugger attached!",
                                "Checks /proc/self/status or IsDebuggerPresent",
                            );
                            status_row(
                                ui,
                                "Environment",
                                report
                                    .environment_warnings
                                    .is_empty(),
                                "Clean",
                                &format!(
                                    "{} issues",
                                    report
                                        .environment_warnings
                                        .len()
                                ),
                                "Checks LD_PRELOAD, DYLD_INSERT_LIBRARIES",
                            );
                        });

                    if !report.environment_warnings.is_empty()
                    {
                        ui.add_space(4.0);
                        for w in
                            &report.environment_warnings
                        {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("  ⚠ {}", w),
                            );
                        }
                    }

                    ui.add_space(6.0);
                    if report.overall_safe {
                        ui.colored_label(
                            egui::Color32::GREEN,
                            "✅ Environment is safe",
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::RED,
                            "⚠️ Potential issues detected",
                        );
                    }
                }
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🔑 Pinned Publisher Keys");
            ui.separator();
            ui.label(format!(
                "Server: {}",
                self.selected_server
            ));
            ui.add_space(4.0);

            let keys = self
                .config
                .pinned_publisher_keys_by_server
                .get(&self.selected_server)
                .cloned()
                .unwrap_or_default();

            if keys.is_empty() {
                ui.label(
                    "No keys pinned yet. \
                     Keys are pinned automatically on first install.",
                );
            } else {
                for k in &keys {
                    ui.horizontal(|ui| {
                        ui.label("📌");
                        ui.strong(k.publisher_id.as_str());
                        ui.label(format!(
                            "Dilithium: {}…",
                            &k.dilithium_public_key[..24
                                .min(
                                    k.dilithium_public_key
                                        .len(),
                                )]
                        ));
                    });
                }
            }

            ui.add_space(4.0);
            if ui.button("🗑 Clear pinned keys").clicked() {
                self.config
                    .pinned_publisher_keys_by_server
                    .remove(&self.selected_server);
                let _ = config::save_config(&self.config);
                self.add_log(
                    LogLevel::Warning,
                    "Pinned keys cleared for current server",
                );
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🎯 Protection Overview");
            ui.separator();
            let items = [
                ("✅", "MITM / Tampering", "SHA3-256 + dual signatures"),
                ("✅", "Downgrade attacks", "Monotonic version enforcement"),
                ("✅", "Quantum threats", "CRYSTALS-Dilithium3 (NIST Level 3)"),
                ("✅", "Key compromise", "Hybrid scheme (PQ + classical)"),
                ("✅", "Replay attacks", "Timestamped signatures"),
                ("✅", "Client tampering", "Server-verified self-integrity"),
                ("✅", "Brute-force login", "Rate limit (5/60s) + Argon2id"),
                ("✅", "Path traversal", "Filename sanitization"),
                ("✅", "Transport security", "HTTPS fully suported"),
            ];
            egui::Grid::new("prot_grid")
                .num_columns(3)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    for (icon, threat, mitigation) in &items {
                        ui.label(*icon);
                        ui.strong(*threat);
                        ui.label(*mitigation);
                        ui.end_row();
                    }
                });
        });
    }

    fn tab_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙ Settings");
        ui.add_space(4.0);

        ui.group(|ui| {
            ui.heading("📂 Directories");
            ui.separator();
            egui::Grid::new("dir_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Download:");
                    ui.text_edit_singleline(
                        &mut self.download_dir_input,
                    );
                    ui.end_row();
                    ui.label("Install:");
                    ui.text_edit_singleline(
                        &mut self.install_dir_input,
                    );
                    ui.end_row();
                });
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🌐 Known Servers");
            ui.separator();
            for s in self.config.servers.iter() {
                ui.horizontal(|ui| {
                    if *s == self.selected_server {
                        ui.strong(format!("{}", s));
                    } else {
                        ui.label(format!("{}", s));
                    }
                });
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("📦 All Installed Applications");
            ui.separator();
            if self.config.installed_apps.is_empty() {
                ui.label("None");
            } else {
                egui::Grid::new("all_installed")
                    .num_columns(4)
                    .spacing([12.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Server");
                        ui.strong("App");
                        ui.strong("Version");
                        ui.strong("Path");
                        ui.end_row();
                        for ia in
                            &self.config.installed_apps
                        {
                            ui.label(ia.server_url.as_str());
                            ui.label(ia.app_id.as_str());
                            ui.label(
                                ia.installed_version
                                    .to_string(),
                            );
                            ui.label(ia.install_dir.as_str());
                            ui.end_row();
                        }
                    });
            }
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("🔧 Others");
            ui.separator();
            ui.checkbox(
                &mut self.config.auto_download,
                "Auto-download updates on check",
            );
        });

        ui.add_space(12.0);

        if ui.button("💾 Save Settings").clicked() {
            self.config.download_dir =
                self.download_dir_input.clone();
            self.config.install_dir =
                self.install_dir_input.clone();
            match config::save_config(&self.config) {
                Ok(_) => self.add_log(
                    LogLevel::Success,
                    "Settings saved",
                ),
                Err(e) => self.add_log(
                    LogLevel::Error,
                    &format!("Save failed: {}", e),
                ),
            }
        }
    }

    fn tab_logs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("📋 Activity Log");
            ui.with_layout(
                egui::Layout::right_to_left(
                    egui::Align::Center,
                ),
                |ui| {
                    if ui.button("🗑 Clear").clicked() {
                        self.log_messages.clear();
                    }
                    ui.label(format!(
                        "{} entries",
                        self.log_messages.len()
                    ));
                },
            );
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for entry in self.log_messages.iter().rev() {
                    let color = match entry.level {
                        LogLevel::Info => {
                            egui::Color32::LIGHT_GRAY
                        }
                        LogLevel::Success => {
                            egui::Color32::GREEN
                        }
                        LogLevel::Warning => {
                            egui::Color32::YELLOW
                        }
                        LogLevel::Error => {
                            egui::Color32::RED
                        }
                    };
                    ui.colored_label(
                        color,
                        format!(
                            "[{}] {}",
                            entry.timestamp,
                            entry.message
                        ),
                    );
                }
            });
    }
}

// ═══════════════════════════════════════════════════════════════
//  ACTIONS
// ═══════════════════════════════════════════════════════════════

impl UpdateApp {
    fn get_installed_app(
        &self,
        server: &str,
        app_id: &str,
    ) -> Option<InstalledApp> {
        self.config
            .installed_apps
            .iter()
            .find(|ia| {
                ia.server_url == server && ia.app_id == app_id
            })
            .cloned()
    }

    fn refresh_apps_list(&mut self) {
        self.add_log(
            LogLevel::Info,
            &format!(
                "Fetching apps from {}…",
                self.selected_server
            ),
        );
        match updater::fetch_apps(&self.selected_server) {
            Ok(resp) => {
                self.apps_list = resp.apps;
                self.add_log(
                    LogLevel::Success,
                    &format!(
                        "Found {} apps",
                        self.apps_list.len()
                    ),
                );
            }
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    &format!("Fetch failed: {}", e),
                );
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
        self.confirm_uninstall = None;
        self.health_info = None;
        let _ = config::save_config(&self.config);
        self.refresh_apps_list();
    }

    fn perform_health_check(&mut self) {
        let url =
            format!("{}/api/health", self.selected_server);
        self.add_log(
            LogLevel::Info,
            &format!("Health check → {}", url),
        );

        let start = std::time::Instant::now();

        match reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .and_then(|c| c.get(&url).send())
        {
            Ok(resp) if resp.status().is_success() => {
                let latency = start.elapsed().as_millis();
                match resp.json::<serde_json::Value>() {
                    Ok(body) => {
                        let crypto = body
                            .get("crypto")
                            .cloned()
                            .unwrap_or_default();
                        self.health_info = Some(HealthInfo {
                            is_healthy: true,
                            service_name: body["service"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            service_version: body["version"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            pq_algo: crypto["post_quantum"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            classical_algo: crypto
                                ["classical"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            hash_algo: crypto["hash"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            timestamp: body["timestamp"]
                                .as_str()
                                .unwrap_or("?")
                                .to_string(),
                            latency_ms: latency,
                            error: None,
                        });
                        self.add_log(
                            LogLevel::Success,
                            &format!(
                                "Server healthy ({}ms)",
                                latency
                            ),
                        );
                    }
                    Err(e) => {
                        self.health_info = Some(HealthInfo {
                            is_healthy: false,
                            error: Some(format!(
                                "Invalid response: {}",
                                e
                            )),
                            ..Default::default()
                        });
                        self.add_log(
                            LogLevel::Error,
                            "Server returned invalid JSON",
                        );
                    }
                }
            }
            Ok(resp) => {
                self.health_info = Some(HealthInfo {
                    is_healthy: false,
                    error: Some(format!(
                        "HTTP {}",
                        resp.status()
                    )),
                    ..Default::default()
                });
                self.add_log(
                    LogLevel::Error,
                    &format!(
                        "Server returned {}",
                        resp.status()
                    ),
                );
            }
            Err(e) => {
                self.health_info = Some(HealthInfo {
                    is_healthy: false,
                    error: Some(format!("{}", e)),
                    ..Default::default()
                });
                self.add_log(
                    LogLevel::Error,
                    &format!("Connection failed: {}", e),
                );
            }
        }
    }

    fn perform_hardening_check(&mut self) {
        self.add_log(
            LogLevel::Info,
            "Running security checks (with server verification)…",
        );

        let report =
            anti_tamper::full_hardening_check_with_server(
                &self.selected_server,
            );

        if report.overall_safe {
            self.add_log(
                LogLevel::Success,
                "✅ Environment is safe (verified against server)",
            );
        } else {
            if report.debugger_detected {
                self.add_log(
                    LogLevel::Warning,
                    "Debugger detected!",
                );
            }
            if !report.self_integrity_ok {
                self.add_log(
                    LogLevel::Error,
                    "❌ Self-integrity FAILED — \
                     binary tampered or server unreachable",
                );
            }
            for w in &report.environment_warnings {
                self.add_log(LogLevel::Warning, w);
            }
        }
        self.hardening_report = Some(report);
    }

    fn uninstall_app(&mut self, app_id: &str) {
        let server = self.selected_server.clone();
        let installed = match self
            .get_installed_app(&server, app_id)
        {
            Some(ia) => ia,
            None => {
                self.add_log(
                    LogLevel::Warning,
                    &format!(
                        "{} is not installed",
                        app_id
                    ),
                );
                self.confirm_uninstall = None;
                return;
            }
        };

        self.add_log(
            LogLevel::Info,
            &format!("Uninstalling {}…", app_id),
        );

        let path =
            std::path::Path::new(&installed.install_dir);
        if path.exists() {
            match std::fs::remove_dir_all(path) {
                Ok(_) => self.add_log(
                    LogLevel::Success,
                    &format!(
                        "Removed {}",
                        installed.install_dir
                    ),
                ),
                Err(e) => self.add_log(
                    LogLevel::Error,
                    &format!("Remove failed: {}", e),
                ),
            }
        }

        self.config.installed_apps.retain(|ia| {
            !(ia.server_url == server
                && ia.app_id == app_id)
        });
        let _ = config::save_config(&self.config);

        self.add_log(
            LogLevel::Success,
            &format!("✅ {} uninstalled", app_id),
        );
        self.confirm_uninstall = None;
        self.update_state = UpdateState::UpToDate;
    }

    fn start_install_or_update(
        &mut self,
        app_id: &str,
        ctx: &egui::Context,
    ) {
        self.active_app_id = app_id.to_string();
        self.verification_report = None;
        self.pending_metadata = None;
        self.downloaded_data = None;

        let current_version = self
            .get_installed_app(
                &self.selected_server,
                app_id,
            )
            .map(|ia| ia.installed_version.clone())
            .unwrap_or_else(|| {
                SemanticVersion::new(0, 0, 0)
            });

        let server = self.selected_server.clone();
        self.add_log(
            LogLevel::Info,
            &format!("Checking {} on {}…", app_id, server),
        );
        self.update_state = UpdateState::Checking;
        ctx.request_repaint();

        let resp = match updater::check_for_update(
            &server,
            app_id,
            &current_version,
        ) {
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    &format!("Check failed: {}", e),
                );
                self.update_state = UpdateState::Error {
                    message: format!("{}", e),
                };
                ctx.request_repaint();
                return;
            }
            Ok(r) => r,
        };

        if !resp.update_available {
            self.add_log(
                LogLevel::Info,
                &format!("{} is up to date", app_id),
            );
            self.update_state = UpdateState::UpToDate;
            ctx.request_repaint();
            return;
        }

        let metadata = match resp.latest_package {
            Some(m) => m,
            None => {
                self.add_log(
                    LogLevel::Error,
                    "No metadata",
                );
                ctx.request_repaint();
                return;
            }
        };
        let pub_key = match resp.publisher_public_key {
            Some(k) => k,
            None => {
                self.add_log(
                    LogLevel::Error,
                    "No publisher key",
                );
                ctx.request_repaint();
                return;
            }
        };

        let pinned = self
            .config
            .pinned_publisher_keys_by_server
            .entry(server.clone())
            .or_default();
        if !pinned.iter().any(|k| {
            k.publisher_id == pub_key.publisher_id
        }) {
            pinned.push(pub_key.clone());
            self.add_log(
                LogLevel::Info,
                &format!(
                    "Pinned key: {}",
                    pub_key.publisher_id
                ),
            );
        }

        let ver = metadata.version.to_string();
        let app = metadata.app_id.clone();
        self.add_log(
            LogLevel::Success,
            &format!("Found v{}", ver),
        );

        self.update_state = UpdateState::Downloading {
            progress_percent: 0.0,
        };
        ctx.request_repaint();

        let data = match updater::download_package(
            &server, &app, &ver,
        ) {
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    &format!("Download failed: {}", e),
                );
                self.update_state = UpdateState::Error {
                    message: format!("{}", e),
                };
                ctx.request_repaint();
                return;
            }
            Ok(d) => d,
        };
        self.add_log(
            LogLevel::Success,
            &format!("Downloaded {} bytes", data.len()),
        );

        self.update_state = UpdateState::Verifying;
        ctx.request_repaint();

        let report = match crate::verifier::verify_package(
            &data,
            &metadata,
            &pub_key,
            &current_version,
        ) {
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    &format!("Verify error: {}", e),
                );
                self.update_state = UpdateState::Error {
                    message: format!("{}", e),
                };
                ctx.request_repaint();
                return;
            }
            Ok(r) => r,
        };

        self.verification_report = Some(report.clone());

        if !report.overall_valid {
            self.add_log(
                LogLevel::Error,
                "❌ Verification FAILED",
            );
            for e in &report.errors {
                self.add_log(LogLevel::Error, e);
            }
            self.update_state = UpdateState::Error {
                message: "Verification failed".into(),
            };
            ctx.request_repaint();
            return;
        }

        self.add_log(
            LogLevel::Success,
            "✅ Verification PASSED",
        );

        let install_dir = format!(
            "{}/{}",
            self.config
                .install_dir
                .trim_end_matches('/'),
            app
        );
        std::fs::create_dir_all(&install_dir).ok();

        match updater::apply_update(
            &data,
            &metadata,
            &install_dir,
        ) {
            Err(e) => {
                self.add_log(
                    LogLevel::Error,
                    &format!("Apply failed: {}", e),
                );
                self.update_state = UpdateState::Error {
                    message: format!("{}", e),
                };
                ctx.request_repaint();
            }
            Ok(_) => {
                self.config.installed_apps.retain(|ia| {
                    !(ia.server_url == server
                        && ia.app_id == app)
                });
                self.config.installed_apps.push(
                    InstalledApp {
                        server_url: server,
                        app_id: app.clone(),
                        installed_version: metadata
                            .version
                            .clone(),
                        install_dir: install_dir.clone(),
                        installed_at: Utc::now(),
                        last_verified_at: Some(Utc::now()),
                    },
                );
                self.config.current_version =
                    metadata.version.clone();
                let _ =
                    config::save_config(&self.config);

                self.update_state = UpdateState::Completed;
                self.add_log(
                    LogLevel::Success,
                    &format!(
                        "✅ {} v{} → {}",
                        app,
                        metadata.version,
                        install_dir
                    ),
                );
                ctx.request_repaint();
            }
        }
    }

    fn render_verification_panel(
        &self,
        ui: &mut egui::Ui,
        report: &VerificationReport,
    ) {
        ui.group(|ui| {
            ui.heading(format!(
                "🔍 Verification: {}",
                self.active_app_id
            ));
            ui.separator();
            egui::Grid::new("vf")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    check_row(
                        ui,
                        "File size",
                        report.size_check,
                    );
                    check_row(
                        ui,
                        "SHA3-256 hash",
                        report.hash_check,
                    );
                    check_row(
                        ui,
                        "Dilithium3 (PQ)",
                        report.dilithium_valid,
                    );
                    check_row(
                        ui,
                        "Ed25519",
                        report.ed25519_valid,
                    );
                    check_row(
                        ui,
                        "Anti-downgrade",
                        report.version_check,
                    );
                    check_row(
                        ui,
                        "Publisher",
                        report.publisher_check,
                    );
                });
            ui.add_space(4.0);
            if report.overall_valid {
                ui.colored_label(
                    egui::Color32::GREEN,
                    "✅ ALL CHECKS PASSED",
                );
            } else {
                ui.colored_label(
                    egui::Color32::RED,
                    "❌ VERIFICATION FAILED",
                );
                for e in &report.errors {
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("  ⚠ {}", e),
                    );
                }
            }
        });
    }
}

fn check_row(ui: &mut egui::Ui, label: &str, ok: bool) {
    ui.label(format!("{}:", label));
    if ok {
        ui.colored_label(
            egui::Color32::GREEN,
            "✅ PASS",
        );
    } else {
        ui.colored_label(
            egui::Color32::RED,
            "❌ FAIL",
        );
    }
    ui.end_row();
}

fn status_row(
    ui: &mut egui::Ui,
    label: &str,
    ok: bool,
    ok_text: &str,
    fail_text: &str,
    tooltip: &str,
) {
    ui.label(format!("{}:", label));
    if ok {
        ui.colored_label(
            egui::Color32::GREEN,
            format!("✅ {}", ok_text),
        );
    } else {
        ui.colored_label(
            egui::Color32::RED,
            format!("❌ {}", fail_text),
        );
    }
    ui.label(tooltip).on_hover_text(tooltip);
    ui.end_row();
}
