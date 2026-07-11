// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;

mod chatlog;

// ── Later-milestone modules (compiled from day one so the structure is real) ──
mod clipboard;
mod config;
mod files;
mod fonts;
mod render;
mod templates;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::parse_chatlog,
            commands::list_presets,
            commands::import_preset_toml,
            commands::export_preset_toml,
            commands::export_png,
            commands::copy_png,
            commands::list_templates,
            commands::save_templates,
            commands::load_settings,
            commands::save_settings,
            commands::list_fonts,
            commands::app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ScreeniesEditor");
}
