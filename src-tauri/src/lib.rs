//! SpecLens backend library entry point.
//!
//! Wires the Tauri builder, installs plugins, registers IPC command handlers,
//! and boots the tracing subscriber. Keeps top-level wiring small so individual
//! domains (`commands/*`, `services/*`, `models/*`) can evolve independently.

pub mod commands;
pub mod error;
pub mod models;
pub mod services;
pub mod state;

use tauri::Manager;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::state::AppState;

/// Boot the Tauri application. Called from `main.rs`.
pub fn run() {
    init_tracing();

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Resolve the OS-appropriate app-data directory and mount the
            // long-lived AppState. The directory is created lazily by
            // AppDataStore on first write.
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            app.manage(AppState::new(app_data_dir));
            Ok(())
        });

    let builder = commands::register(builder);

    builder
        .run(tauri::generate_context!())
        .expect("error while running SpecLens application");
}

fn init_tracing() {
    let filter = EnvFilter::try_from_env("SPECLENS_LOG")
        .unwrap_or_else(|_| EnvFilter::new("speclens_lib=info,warn"));
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_target(true)
        .finish();
    // Ignore errors if a global subscriber is already set (e.g. during tests).
    let _ = tracing::subscriber::set_global_default(subscriber);
}
