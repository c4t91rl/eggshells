#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod updater;
mod verifier;
mod server_registry;
mod download;
mod rollback;
mod integrity_check;
mod anti_tamper;

use commands::AppState;
use server_registry::ServerRegistry;
use download::Downloader;
use std::sync::Mutex;

fn main() {
    let registry = ServerRegistry::load(
        &std::path::PathBuf::from("data/servers.json")
    ).unwrap_or_else(|_| {
        ServerRegistry::new(std::path::PathBuf::from("data/servers.json"))
    });

    let downloader = Downloader::new(std::path::PathBuf::from("data/downloads"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            registry: Mutex::new(registry),
            downloader,
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_servers,
            commands::add_server,
            commands::remove_server,
            commands::check_updates,
            commands::verify_manifest,
            commands::check_all_updates,
            commands::get_integrity_report,
            commands::get_security_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}