// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod files;

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
            commands::load_settings,
            commands::save_settings,
            commands::list_fonts,
            commands::read_dropped_image,
            commands::app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ScreeniesEditor");
}
