//! files.rs — file dialogs & disk I/O.
//!
//! ACTIVE: preset import/export as `.toml`, and PNG export via a native
//! save dialog ("Save Disk"). Last-folder memory arrives with M4 config.

use screenies_core::chatlog::preset::ParsePreset;
use screenies_core::error::AppError;
use std::fs;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

/// Open-file dialog → parse the chosen `.toml` into a preset.
/// `Ok(None)` = the user cancelled the dialog.
pub fn import_preset(app: &AppHandle) -> Result<Option<ParsePreset>, AppError> {
    let Some(picked) = app
        .dialog()
        .file()
        .add_filter("Preset ScreeniesEditor", &["toml"])
        .blocking_pick_file()
    else {
        return Ok(None);
    };

    let path = picked.into_path().map_err(|e| AppError::Io(e.to_string()))?;
    let text = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    let preset: ParsePreset =
        toml::from_str(&text).map_err(|e| AppError::Parse(format!("TOML tidak valid: {e}")))?;
    Ok(Some(preset))
}

/// Save-file dialog → write the preset as pretty TOML.
/// `Ok(false)` = the user cancelled the dialog.
pub fn export_preset(app: &AppHandle, preset: &ParsePreset) -> Result<bool, AppError> {
    let Some(picked) = app
        .dialog()
        .file()
        .add_filter("Preset ScreeniesEditor", &["toml"])
        .set_file_name("preset-screenies.toml")
        .blocking_save_file()
    else {
        return Ok(false);
    };

    let path = picked.into_path().map_err(|e| AppError::Io(e.to_string()))?;
    let text =
        toml::to_string_pretty(preset).map_err(|e| AppError::Parse(e.to_string()))?;
    fs::write(&path, text).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use screenies_core::chatlog::preset;

    #[test]
    fn preset_roundtrips_through_toml() {
        let original = preset::jgrp();
        let text = toml::to_string_pretty(&original).unwrap();
        let back: preset::ParsePreset = toml::from_str(&text).unwrap();
        assert_eq!(original, back);
        assert!(text.contains("stripTimestamps"), "keys must stay camelCase:\n{text}");
    }

    #[test]
    fn partial_toml_uses_defaults() {
        let p: preset::ParsePreset =
            toml::from_str("name = \"Wiki\"\ndoSuffix = false\n").unwrap();
        assert_eq!(p.name, "Wiki");
        assert!(!p.do_suffix);
        assert!(p.me_prefix); // defaulted
    }
}

/// Save-file dialog → write the rendered PNG. `Ok(false)` = cancelled.
/// `file_name` comes pre-expanded from the frontend template; it is
/// sanitized here as the last line of defense.
pub fn save_png(app: &AppHandle, png: &[u8], file_name: &str) -> Result<bool, AppError> {
    let mut dialog = app
        .dialog()
        .file()
        .add_filter("Gambar PNG", &["png"])
        .set_file_name(&sanitize_png_name(file_name));

    // Last-folder memory: reopen where the user saved last time.
    if let Ok(Some(settings)) = crate::config::load(app) {
        if !settings.last_save_dir.is_empty() {
            let dir = std::path::PathBuf::from(&settings.last_save_dir);
            if dir.is_dir() {
                dialog = dialog.set_directory(dir);
            }
        }
    }

    let Some(picked) = dialog.blocking_save_file() else {
        return Ok(false);
    };

    let path = picked.into_path().map_err(|e| AppError::Io(e.to_string()))?;
    fs::write(&path, png).map_err(|e| AppError::Io(e.to_string()))?;

    if let Some(parent) = path.parent() {
        let mut settings = crate::config::load(app)?.unwrap_or_default();
        settings.last_save_dir = parent.to_string_lossy().into_owned();
        let _ = crate::config::save(app, &settings); // best-effort
    }
    Ok(true)
}

/// Strip filesystem-hostile characters, guarantee a non-empty ".png" name.
fn sanitize_png_name(raw: &str) -> String {
    let mut name: String = raw
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();
    if name.is_empty() {
        name = "screenie".into();
    }
    if !name.to_lowercase().ends_with(".png") {
        name.push_str(".png");
    }
    name
}

#[cfg(test)]
mod name_tests {
    use super::sanitize_png_name;

    #[test]
    fn sanitizes_and_ensures_extension() {
        assert_eq!(sanitize_png_name("rp/rampok: part2"), "rp-rampok- part2.png");
        assert_eq!(sanitize_png_name("sudah.png"), "sudah.png");
        assert_eq!(sanitize_png_name("   "), "screenie.png");
    }
}
