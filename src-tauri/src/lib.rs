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

use tauri::{Emitter, Manager};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::services::BridgeEvent;
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

            // Pull the terminal event receiver and spawn a forwarder that
            // re-emits BridgeEvent payloads as Tauri events the frontend
            // subscribes to via `subscribe('pty_output', ...)`.
            let state = app.state::<AppState>();
            let rx = state
                .terminal_event_rx
                .lock()
                .expect("terminal_event_rx mutex")
                .take()
                .expect("terminal_event_rx already taken");
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                for event in rx {
                    match event {
                        BridgeEvent::Lines { session_id, lines } => {
                            let _ = handle.emit(
                                "pty_output",
                                serde_json::json!({
                                    "sessionId": session_id,
                                    "lines": lines,
                                }),
                            );
                        }
                        BridgeEvent::Spilled {
                            session_id,
                            evicted_count,
                        } => {
                            let _ = handle.emit(
                                "buffer_spilled",
                                serde_json::json!({
                                    "sessionId": session_id,
                                    "evictedCount": evicted_count,
                                }),
                            );
                        }
                        BridgeEvent::Closed { session_id } => {
                            let _ = handle
                                .emit("pty_closed", serde_json::json!({ "sessionId": session_id }));
                        }
                    }
                }
            });
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
