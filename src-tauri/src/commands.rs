//! commands.rs — the "menu" of functions the frontend may call.
//!
//! RULE: no logic lives here. Every command is a thin door into a module,
//! so each module stays unit-testable without launching the GUI.
//!
//! Every command registered here needs a matching typed wrapper in
//! `src/ts/tauri-bridge.ts` — keep the two files in sync.

use crate::chatlog::preset::ParsePreset;
use crate::chatlog::{self, ParsedLine};
use crate::error::AppError;

/// Chatlog text → parsed lines: timestamps stripped, `{RRGGBB}` colors
/// parsed (case-insensitive), auto-color (`*` ungu, `(( ))` abu-abu,
/// says/shouts/[phone]/[walkie], /do suffix), system tags typed + bolded.
#[tauri::command]
pub fn parse_chatlog(text: String, preset: ParsePreset) -> Result<Vec<ParsedLine>, AppError> {
    Ok(chatlog::parse(&text, &preset))
}

/// The built-in parsing presets, in dropdown order.
#[tauri::command]
pub fn list_presets() -> Vec<ParsePreset> {
    chatlog::preset::builtin()
}

/// Open-file dialog → preset from a community `.toml` file (None = cancel).
#[tauri::command]
pub fn import_preset_toml(app: tauri::AppHandle) -> Result<Option<ParsePreset>, AppError> {
    crate::files::import_preset(&app)
}

/// Save-file dialog → write the given preset as `.toml` (false = cancel).
#[tauri::command]
pub fn export_preset_toml(
    app: tauri::AppHandle,
    preset: ParsePreset,
) -> Result<bool, AppError> {
    crate::files::export_preset(&app, &preset)
}

/// Installed system font families for the picker — sorted, deduped.
#[tauri::command]
pub fn list_fonts() -> Vec<String> {
    crate::fonts::families()
}

/// App version straight from Cargo.toml — shown in the top-bar badge and
/// handy as a "is the bridge alive?" check.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
