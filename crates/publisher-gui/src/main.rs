mod publisher_logic;

use eframe::egui;
use secure_update_common::*;
use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("🔐 Secure Update Publisher"),
        ..Default::default()
    };

    eframe::run_native(
        "Secure Update Publisher",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(PublisherApp::new()))
        }),
    )
}

// ═══════════════════════════════════════════════════════════════
//  TYPES
// ═══════════════════════════════════════════════════════════════

#[derive(PartialEq)]
enum Screen {
    Login,
    CreateAccount,
    Main,
}

#[derive(PartialEq)]
enum Tab {
    Keys,
    Register,
    Publish,
    History,
}

#[derive(Debug, Clone)]
struct LogEntry {
    time: String,
    level: LogLevel,
    msg: String,
}

#[derive(Debug, Clone)]
enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PublishRecord {
    app_id: String,
    version: String,
    publisher_id: String,
    hash: String,
    timestamp: String,
    server: String,
}

// ═══════════════════════════════════════════════════════════════
//  STATE
// ═══════════════════════════════════════════════════════════════

struct PublisherApp {
    screen: Screen,

    // Auth
    server_url: String,
    username: String,
    password: String,
    password_confirm: String,
    publisher_id_create: String,
    display_name_create: String,
    auth_error: String,
    auth_token: Option<String>,
    session_publisher_id: Option<String>,

    // Keys
    loaded_keypair: Option<HybridKeyPair>,
    loaded_keys_path: Option<PathBuf>,
    publisher_id_input: String,
    keys_output_dir: String,

    // Register publisher keys
    display_name: String,

    // Publish
    app_id: String,
    version: String,
    description: String,
    changelog_text: String,
    selected_file: Option<PathBuf>,
    selected_file_size: u64,
    selected_file_hash: String,

    // UI
    current_tab: Tab,
    logs: Vec<LogEntry>,
    publish_history: Vec<PublishRecord>,
}

impl PublisherApp {
    fn new() -> Self {
        let history: Vec<PublishRecord> = std::fs::read_to_string("publisher_history.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Self {
            screen: Screen::Login,
            server_url: "http://127.0.0.1:8443".into(),
            username: String::new(),
            password: String::new(),
            password_confirm: String::new(),
            publisher_id_create: "my-publisher".into(),
            display_name_create: "My Organization".into(),
            auth_error: String::new(),
            auth_token: None,
            session_publisher_id: None,

            loaded_keypair: None,
            loaded_keys_path: None,
            publisher_id_input: "my-publisher".into(),
            keys_output_dir: "./keys".into(),

            display_name: "My Organization".into(),

            app_id: String::new(),
            version: "1.0.0".into(),
            description: String::new(),
            changelog_text: String::new(),
            selected_file: None,
            selected_file_size: 0,
            selected_file_hash: String::new(),

            current_tab: Tab::Keys,
            logs: Vec::new(),
            publish_history: history,
        }
    }

    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push(LogEntry {
            time: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            msg: msg.into(),
        });
        if self.logs.len() > 200 {
            self.logs.remove(0);
        }
    }

    fn save_history(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&self.publish_history) {
            std::fs::write("publisher_history.json", data).ok();
        }
    }

    fn http_client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap()
    }
}

// ═══════════════════════════════════════════════════════════════
//  eframe::App
// ═══════════════════════════════════════════════════════════════

impl eframe::App for PublisherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::Login => self.screen_login(ctx),
            Screen::CreateAccount => self.screen_create_account(ctx),
            Screen::Main => self.screen_main(ctx),
        }
    }
}

// ═══════════════════════════════════════════════════════════════
//  SCREENS
// ═══════════════════════════════════════════════════════════════

impl PublisherApp {
    // ──────────────────────────────────────────────────────────
    //  LOGIN
    // ──────────────────────────────────────────────────────────
    fn screen_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(80.0);
                ui.heading("🔐 Publisher Login");
                ui.add_space(20.0);

                egui::Grid::new("login")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Server:");
                        ui.text_edit_singleline(&mut self.server_url);
                        ui.end_row();
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.username);
                        ui.end_row();
                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.password)
                                .password(true),
                        );
                        ui.end_row();
                    });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("🔓 Login").clicked() {
                        self.do_login();
                    }
                    if ui.button("📝 Create Account").clicked() {
                        self.screen = Screen::CreateAccount;
                        self.auth_error.clear();
                    }
                });

                if !self.auth_error.is_empty() {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::RED, &self.auth_error);
                }

                ui.add_space(40.0);
                ui.colored_label(
                    egui::Color32::from_rgb(100, 100, 100),
                    "⚠️ Authorized publishers only.",
                );
            });
        });
    }

    // ──────────────────────────────────────────────────────────
    //  CREATE ACCOUNT
    // ──────────────────────────────────────────────────────────
    fn screen_create_account(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.heading("📝 Create Publisher Account");
                ui.add_space(16.0);

                egui::Grid::new("create")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Server:");
                        ui.text_edit_singleline(&mut self.server_url);
                        ui.end_row();
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.username);
                        ui.end_row();
                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.password)
                                .password(true),
                        );
                        ui.end_row();
                        ui.label("Confirm:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.password_confirm)
                                .password(true),
                        );
                        ui.end_row();
                        ui.label("Publisher ID:");
                        ui.text_edit_singleline(&mut self.publisher_id_create);
                        ui.end_row();
                        ui.label("Display Name:");
                        ui.text_edit_singleline(&mut self.display_name_create);
                        ui.end_row();
                    });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("📝 Create Account").clicked() {
                        self.do_create_account();
                    }
                    if ui.button("← Back to Login").clicked() {
                        self.screen = Screen::Login;
                        self.auth_error.clear();
                    }
                });

                if !self.auth_error.is_empty() {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::RED, &self.auth_error);
                }
            });
        });
    }

    // ──────────────────────────────────────────────────────────
    //  MAIN
    // ──────────────────────────────────────────────────────────
    fn screen_main(&mut self, ctx: &egui::Context) {
        // Top
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📤 Publisher Tool");
                ui.separator();
                ui.selectable_value(&mut self.current_tab, Tab::Keys, "🔑 Keys");
                ui.selectable_value(&mut self.current_tab, Tab::Register, "📝 Register");
                ui.selectable_value(&mut self.current_tab, Tab::Publish, "🚀 Publish");
                ui.selectable_value(&mut self.current_tab, Tab::History, "📜 History");

                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.button("🔒 Logout").clicked() {
                            self.do_logout();
                        }
                        ui.colored_label(
                            egui::Color32::GREEN,
                            format!("👤 {} • {}", self.username, self.server_url),
                        );
                    },
                );
            });
        });

        // Logs
        egui::TopBottomPanel::bottom("logs")
            .min_height(120.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("📋 Logs");
                    if ui.button("Clear").clicked() {
                        self.logs.clear();
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for e in self.logs.iter().rev() {
                            let c = match e.level {
                                LogLevel::Info => egui::Color32::LIGHT_GRAY,
                                LogLevel::Success => egui::Color32::GREEN,
                                LogLevel::Warning => egui::Color32::YELLOW,
                                LogLevel::Error => egui::Color32::RED,
                            };
                            ui.colored_label(c, format!("[{}] {}", e.time, e.msg));
                        }
                    });
            });

        // Central
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Keys => self.tab_keys(ui),
                Tab::Register => self.tab_register(ui),
                Tab::Publish => self.tab_publish(ui),
                Tab::History => self.tab_history(ui),
            }
        });
    }
}

// ═══════════════════════════════════════════════════════════════
//  TABS
// ═══════════════════════════════════════════════════════════════

impl PublisherApp {
    fn tab_keys(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔑 Key Management");
        ui.separator();

        ui.group(|ui| {
            ui.heading("Generate New Keys");
            egui::Grid::new("kg")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Publisher ID:");
                    ui.text_edit_singleline(&mut self.publisher_id_input);
                    ui.end_row();
                    ui.label("Output dir:");
                    ui.text_edit_singleline(&mut self.keys_output_dir);
                    ui.end_row();
                });
            ui.add_space(8.0);
            if ui.button("🔑 Generate Dilithium3 + Ed25519").clicked() {
                self.generate_keys();
            }
        });

        ui.add_space(12.0);

        ui.group(|ui| {
            ui.heading("Load Existing Keys");
            if ui.button("📂 Select .keys.json").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Keys", &["json"])
                    .pick_file()
                {
                    self.load_keys(&path);
                }
            }
            if let Some(ref p) = self.loaded_keys_path {
                ui.label(format!("Loaded: {}", p.display()));
            }
        });

        ui.add_space(12.0);

        ui.group(|ui| {
            ui.heading("Status");
            ui.separator();
            match &self.loaded_keypair {
                None => {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ No keys loaded");
                }
                Some(kp) => {
                    let pk = kp.public_key();
                    ui.label(format!("Publisher: {}", kp.publisher_id));
                    ui.label(format!(
                        "Dilithium: {}… ({} chars)",
                        &pk.dilithium_public_key[..32],
                        pk.dilithium_public_key.len()
                    ));
                    ui.label(format!(
                        "Ed25519:   {}… ({} chars)",
                        &pk.ed25519_public_key[..32],
                        pk.ed25519_public_key.len()
                    ));
                    ui.colored_label(egui::Color32::GREEN, "✅ Keys ready");
                }
            }
        });
    }

    fn tab_register(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Register Publisher Keys on Server");
        ui.separator();

        if self.loaded_keypair.is_none() {
            ui.colored_label(
                egui::Color32::RED,
                "❌ Load keys first (🔑 Keys tab)",
            );
            return;
        }

        ui.group(|ui| {
            ui.label(format!("Server: {}", self.server_url));
            ui.label(format!(
                "Publisher: {}",
                self.loaded_keypair.as_ref().unwrap().publisher_id
            ));
            egui::Grid::new("rg")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Display Name:");
                    ui.text_edit_singleline(&mut self.display_name);
                    ui.end_row();
                });
            ui.add_space(8.0);
            if ui.button("📝 Register Keys").clicked() {
                self.do_register_publisher();
            }
        });
    }

    fn tab_publish(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Publish Package");
        ui.separator();

        if self.loaded_keypair.is_none() {
            ui.colored_label(
                egui::Color32::RED,
                "❌ Load keys first (🔑 Keys tab)",
            );
            return;
        }

        ui.group(|ui| {
            ui.heading("Package Details");
            egui::Grid::new("pg")
                .num_columns(2)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label("App ID:");
                    ui.text_edit_singleline(&mut self.app_id);
                    ui.end_row();
                    ui.label("Version:");
                    ui.text_edit_singleline(&mut self.version);
                    ui.end_row();
                    ui.label("Description:");
                    ui.text_edit_singleline(&mut self.description);
                    ui.end_row();
                });
            ui.label("Changelog (one per line):");
            ui.text_edit_multiline(&mut self.changelog_text);
        });

        ui.add_space(8.0);

        ui.group(|ui| {
            ui.heading("File");
            if ui.button("📂 Select File").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    if let Ok(data) = std::fs::read(&path) {
                        self.selected_file_size = data.len() as u64;
                        self.selected_file_hash = compute_sha3_256_hex(&data);
                        self.selected_file = Some(path);
                    }
                }
            }
            if let Some(ref f) = self.selected_file {
                ui.label(format!("File: {}", f.display()));
                ui.label(format!("Size: {} bytes", self.selected_file_size));
                ui.label(format!(
                    "SHA3: {}…",
                    &self.selected_file_hash
                        [..32.min(self.selected_file_hash.len())]
                ));
            }
        });

        ui.add_space(12.0);

        let ready = self.selected_file.is_some()
            && !self.app_id.is_empty()
            && !self.version.is_empty();

        let btn = egui::Button::new("🚀 Sign & Publish").fill(if ready {
            egui::Color32::from_rgb(30, 120, 30)
        } else {
            egui::Color32::from_rgb(80, 80, 80)
        });

        if ui.add_enabled(ready, btn).clicked() {
            self.do_publish();
        }
    }

    fn tab_history(&mut self, ui: &mut egui::Ui) {
        ui.heading("📜 History");
        ui.separator();

        if self.publish_history.is_empty() {
            ui.label("No packages published yet.");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("hg")
                .num_columns(5)
                .spacing([12.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("App");
                    ui.strong("Version");
                    ui.strong("Publisher");
                    ui.strong("Server");
                    ui.strong("Date");
                    ui.end_row();

                    for r in self.publish_history.iter().rev() {
                        ui.label(r.app_id.as_str());
                        ui.label(r.version.as_str());
                        ui.label(r.publisher_id.as_str());
                        ui.label(r.server.as_str());
                        ui.label(r.timestamp.as_str());
                        ui.end_row();
                    }
                });
        });
    }
}

// ═══════════════════════════════════════════════════════════════
//  ACTIONS
// ═══════════════════════════════════════════════════════════════

impl PublisherApp {
    fn do_login(&mut self) {
        let client = self.http_client();
        let body = serde_json::json!({
            "username": self.username,
            "password": self.password,
        });

        match client
            .post(format!("{}/api/auth/login", self.server_url))
            .json(&body)
            .send()
        {
            Ok(resp) if resp.status().is_success() => {
                let data: serde_json::Value = resp.json().unwrap_or_default();
                self.auth_token = data["token"].as_str().map(|s| s.into());
                self.session_publisher_id =
                    data["publisher_id"].as_str().map(|s| s.into());
                self.screen = Screen::Main;
                self.password.clear();
                self.auth_error.clear();
                self.log(LogLevel::Success, "✅ Logged in");
            }
            Ok(resp) => {
                let body = resp.text().unwrap_or_default();
                self.auth_error = format!("Login failed: {}", body);
            }
            Err(e) => {
                self.auth_error = format!("Connection error: {}", e);
            }
        }
    }

    fn do_create_account(&mut self) {
        if self.password != self.password_confirm {
            self.auth_error = "Passwords do not match".into();
            return;
        }
        if self.password.len() < 6 {
            self.auth_error = "Password must be at least 6 characters".into();
            return;
        }

        let client = self.http_client();
        let body = serde_json::json!({
            "username": self.username,
            "password": self.password,
            "publisher_id": self.publisher_id_create,
            "display_name": self.display_name_create,
        });

        match client
            .post(format!("{}/api/auth/register", self.server_url))
            .json(&body)
            .send()
        {
            Ok(resp) if resp.status().is_success() => {
                self.auth_error.clear();
                self.screen = Screen::Login;
                self.password.clear();
                self.password_confirm.clear();
                self.log(LogLevel::Success, "✅ Account created! You can now login.");
            }
            Ok(resp) => {
                let body = resp.text().unwrap_or_default();
                self.auth_error = format!("Failed: {}", body);
            }
            Err(e) => {
                self.auth_error = format!("Connection error: {}", e);
            }
        }
    }

    fn do_logout(&mut self) {
        if let Some(ref token) = self.auth_token {
            let client = self.http_client();
            client
                .post(format!("{}/api/auth/logout", self.server_url))
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .ok();
        }
        self.auth_token = None;
        self.session_publisher_id = None;
        self.loaded_keypair = None;
        self.screen = Screen::Login;
        self.log(LogLevel::Info, "Logged out");
    }

    fn generate_keys(&mut self) {
        self.log(
            LogLevel::Info,
            &format!("Generating keys for '{}'…", self.publisher_id_input),
        );

        match HybridKeyPair::generate(&self.publisher_id_input) {
            Ok(kp) => {
                std::fs::create_dir_all(&self.keys_output_dir).ok();

                let sp = std::path::Path::new(&self.keys_output_dir)
                    .join(format!("{}.keys.json", self.publisher_id_input));
                let pp = std::path::Path::new(&self.keys_output_dir)
                    .join(format!("{}.pub.json", self.publisher_id_input));

                if let Ok(d) = kp.export_secret_keys() {
                    std::fs::write(&sp, d).ok();
                }
                if let Ok(d) = serde_json::to_vec_pretty(&kp.public_key()) {
                    std::fs::write(&pp, d).ok();
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(
                        &sp,
                        std::fs::Permissions::from_mode(0o600),
                    )
                    .ok();
                }

                self.log(
                    LogLevel::Success,
                    &format!("✅ Keys saved to {}", sp.display()),
                );
                self.loaded_keys_path = Some(sp);
                self.loaded_keypair = Some(kp);
            }
            Err(e) => self.log(LogLevel::Error, &format!("Keygen failed: {}", e)),
        }
    }

    fn load_keys(&mut self, path: &PathBuf) {
        match std::fs::read(path) {
            Ok(data) => match HybridKeyPair::import_secret_keys(&data) {
                Ok(kp) => {
                    self.log(
                        LogLevel::Success,
                        &format!("✅ Keys loaded: {}", kp.publisher_id),
                    );
                    self.loaded_keys_path = Some(path.clone());
                    self.loaded_keypair = Some(kp);
                }
                Err(e) => {
                    self.log(LogLevel::Error, &format!("Import failed: {}", e));
                }
            },
            Err(e) => {
                self.log(LogLevel::Error, &format!("Read failed: {}", e));
            }
        }
    }

    fn do_register_publisher(&mut self) {
        let kp = match &self.loaded_keypair {
            Some(k) => k,
            None => return,
        };
        let token = match &self.auth_token {
            Some(t) => t.clone(),
            None => {
                self.log(LogLevel::Error, "Not logged in");
                return;
            }
        };

        let client = self.http_client();
        let req = RegisterPublisherRequest {
            display_name: self.display_name.clone(),
            public_key: kp.public_key(),
        };

        match client
            .post(format!("{}/api/publishers", self.server_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&req)
            .send()
        {
            Ok(r) if r.status().is_success() => {
                self.log(LogLevel::Success, "✅ Publisher keys registered on server");
            }
            Ok(r) => {
                let body = r.text().unwrap_or_default();
                self.log(LogLevel::Error, &format!("Failed: {}", body));
            }
            Err(e) => {
                self.log(LogLevel::Error, &format!("Error: {}", e));
            }
        }
    }

    fn do_publish(&mut self) {
        let kp = match self.loaded_keypair.clone() {
            Some(k) => k,
            None => return,
        };
        let publisher_id = kp.publisher_id.clone();
        let token = match &self.auth_token {
            Some(t) => t.clone(),
            None => {
                self.log(LogLevel::Error, "Not logged in");
                return;
            }
        };
        let fp = match &self.selected_file {
            Some(p) => p.clone(),
            None => return,
        };
        let sv = match SemanticVersion::parse(&self.version) {
            Ok(v) => v,
            Err(e) => {
                self.log(LogLevel::Error, &format!("Bad version: {}", e));
                return;
            }
        };
        let data = match std::fs::read(&fp) {
            Ok(d) => d,
            Err(e) => {
                self.log(LogLevel::Error, &format!("Read: {}", e));
                return;
            }
        };

        let hash = compute_sha3_256_hex(&data);
        self.log(LogLevel::Info, &format!("SHA3-256: {}", hash));

        self.log(LogLevel::Info, "Signing with Dilithium3 + Ed25519…");
        let sig = match kp.sign(&data) {
            Ok(s) => s,
            Err(e) => {
                self.log(LogLevel::Error, &format!("Sign: {}", e));
                return;
            }
        };
        self.log(LogLevel::Success, "✅ Signed");

        let filename = fp
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("{}-{}.bin", self.app_id, self.version));

        let changelog: Vec<String> = self
            .changelog_text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect();

        let client = self.http_client();

        // 1. Upload
        self.log(LogLevel::Info, "Uploading…");
        match client
            .post(format!(
                "{}/api/packages/upload/{}/{}/{}",
                self.server_url, publisher_id, self.app_id, self.version
            ))
            .header("Authorization", format!("Bearer {}", token))
            .body(data.clone())
            .send()
        {
            Ok(r) if r.status().is_success() => {
                self.log(LogLevel::Success, "✅ Uploaded");
            }
            Ok(r) => {
                let body = r.text().unwrap_or_default();
                self.log(LogLevel::Error, &format!("Upload: {}", body));
                return;
            }
            Err(e) => {
                self.log(LogLevel::Error, &format!("Upload: {}", e));
                return;
            }
        }

        // 2. Metadata
        self.log(LogLevel::Info, "Publishing metadata…");
        let meta = PublishPackageRequest {
            app_id: self.app_id.clone(),
            version: sv.clone(),
            publisher_id: publisher_id.clone(),
            sha3_256_hash: hash.clone(),
            file_size: data.len() as u64,
            filename,
            description: self.description.clone(),
            target_platforms: vec![Platform::current()],
            signature: sig,
            min_upgrade_from: None,
            changelog,
        };

        match client
            .post(format!("{}/api/packages/metadata", self.server_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&meta)
            .send()
        {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().unwrap_or_default();
                self.log(
                    LogLevel::Success,
                    &format!(
                        "✅ Published! ID: {} (verified: {})",
                        body["package_id"], body["verified"]
                    ),
                );
                self.publish_history.push(PublishRecord {
                    app_id: self.app_id.clone(),
                    version: self.version.clone(),
                    publisher_id: publisher_id.clone(),
                    hash,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    server: self.server_url.clone(),
                });
                self.save_history();
            }
            Ok(r) => {
                let body = r.text().unwrap_or_default();
                self.log(LogLevel::Error, &format!("Metadata: {}", body));
            }
            Err(e) => {
                self.log(LogLevel::Error, &format!("Metadata: {}", e));
            }
        }
    }
}