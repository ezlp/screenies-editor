//! files.rs — file dialogs & disk I/O.
//!
//! ACTIVE: preset import/export as `.toml` — the community-shareable
//! format documented in docs/PRESETS.md (wiki).
//! MILESTONE 3 adds: PNG export with a folder picker + last-folder memory.

use crate::chatlog::preset::ParsePreset;
use crate::error::AppError;
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
    use crate::chatlog::preset;

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
