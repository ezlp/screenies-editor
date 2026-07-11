// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;

mod chatlog;

// ── Later-milestone modules (compiled from day one so the structure is real) ──
mod clipboard; // M3
mod config; //    M4
mod files; //     M3
mod fonts;
mod render; //    M3
mod templates; // M4

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::parse_chatlog,
            commands::list_presets,
            commands::list_fonts,
            commands::app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ScreeniesEditor");
}
