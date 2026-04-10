//! Tauri IPC command handlers.
//!
//! Every handler returns `IpcResult<T>` (see `error.rs`) so the frontend
//! receives the canonical `{ code, message, hint? }` shape defined in
//! `contracts/ipc.md`. Handlers are thin: they delegate to services and
//! never touch the filesystem directly.

pub mod agent;
pub mod config;
pub mod phase;
pub mod project;
pub mod speckit;
pub mod tasks;
pub mod terminal;

use tauri::{generate_handler, Runtime};

/// Register all IPC command handlers with the Tauri builder.
pub fn register<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(generate_handler![
        // project.*
        project::project_pick_directory,
        project::project_open,
        project::project_list_recent,
        project::project_remove_recent,
        project::project_pin_recent,
        project::project_close,
        // speckit.*
        speckit::steps_list,
        speckit::steps_get,
        // phase.*
        phase::phase_overview,
        phase::phase_documents_list,
        phase::phase_document_read,
        // tasks.*
        tasks::tasks_scan,
        tasks::tasks_parse,
        tasks::tasks_pick_files,
        tasks::tasks_set_selected,
        // terminal.*
        terminal::terminal_attach,
        terminal::terminal_detach,
        terminal::terminal_write,
        terminal::terminal_resize,
        terminal::terminal_get_slice,
        terminal::terminal_search,
        // agent.*
        agent::agent_detect,
        agent::agent_list_known,
        // config.*
        config::config_get,
        config::config_set_language,
        config::config_set_theme,
        config::config_set_terminal_limits,
    ])
}
