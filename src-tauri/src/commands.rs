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
use crate::config::AppSettings;
use crate::render::{compose, RenderJob};

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
/// MUST be async: blocking dialogs dispatch work to the main thread, and a
/// sync command would occupy that same thread → deadlock (frozen app).
#[tauri::command]
pub async fn import_preset_toml(
    app: tauri::AppHandle,
) -> Result<Option<ParsePreset>, AppError> {
    crate::files::import_preset(&app)
}

/// Save-file dialog → write the given preset as `.toml` (false = cancel).
/// Async for the same deadlock reason as import_preset_toml.
#[tauri::command]
pub async fn export_preset_toml(
    app: tauri::AppHandle,
    preset: ParsePreset,
) -> Result<bool, AppError> {
    crate::files::export_preset(&app, &preset)
}

/// Render the full export and save it via a native dialog ("Save Disk").
/// Async: rendering a 4K image takes real time, and the dialog blocks.
#[tauri::command]
pub async fn export_png(
    app: tauri::AppHandle,
    job: RenderJob,
    file_name: String,
) -> Result<bool, AppError> {
    let img = compose::render(&job)?;
    let png = compose::encode_png(&img)?;
    crate::files::save_png(&app, &png, &file_name)
}

/// Render the full export and put it on the clipboard ("Copy").
#[tauri::command]
pub async fn copy_png(job: RenderJob) -> Result<(), AppError> {
    let img = compose::render(&job)?;
    crate::clipboard::copy_image(&img)
}

/// Saved settings (theme, font, preset). None on first run.
#[tauri::command]
pub fn load_settings(app: tauri::AppHandle) -> Result<Option<AppSettings>, AppError> {
    crate::config::load(&app)
}

/// Persist settings to settings.json in the app config dir.
/// `last_save_dir` is Rust-owned (written by the save dialog) — the
/// frontend never sends it, so we merge the on-disk value to avoid
/// wiping folder memory on every theme/font change.
#[tauri::command]
pub fn save_settings(app: tauri::AppHandle, mut settings: AppSettings) -> Result<(), AppError> {
    if settings.last_save_dir.is_empty() {
        if let Ok(Some(existing)) = crate::config::load(&app) {
            settings.last_save_dir = existing.last_save_dir;
        }
    }
    crate::config::save(&app, &settings)
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
