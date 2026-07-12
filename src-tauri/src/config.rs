//! config.rs — settings.json in the OS app-config directory (Milestone 4a).
//!
//! Windows: %APPDATA%\com.screenies-editor.app\settings.json
//! Linux:   ~/.config/com.screenies-editor.app/settings.json
//!
//! A corrupt or old-format file never bricks startup: parse failure just
//! falls back to defaults (and serde(default) fills missing fields).

use screenies_core::chatlog::preset::{self, ParsePreset};
use screenies_core::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub theme: String,
    pub font_family: String,
    pub preset: ParsePreset,
    pub file_name_template: String,
    /// Last folder a PNG was saved to — reopened by the next save dialog.
    pub last_save_dir: String,
    /// UI language: "id" (default) or "en".
    pub lang: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            font_family: "Verdana".into(),
            preset: preset::jgrp(),
            file_name_template: "screenie-{tanggal}-{jam}".into(),
            last_save_dir: String::new(),
            lang: "id".into(),
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(dir.join("settings.json"))
}

/// `Ok(None)` = no settings yet (first run) or unreadable file.
pub fn load(app: &AppHandle) -> Result<Option<AppSettings>, AppError> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    match serde_json::from_str::<AppSettings>(&text) {
        Ok(s) => Ok(Some(s)),
        Err(e) => {
            eprintln!("[screenies-editor] settings.json rusak, pakai default: {e}");
            Ok(None)
        }
    }
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let path = settings_path(app)?;
    let text =
        serde_json::to_string_pretty(settings).map_err(|e| AppError::Parse(e.to_string()))?;
    fs::write(&path, text).map_err(|e| AppError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_or_old_settings_json_still_parses() {
        let s: AppSettings = serde_json::from_str(r#"{ "theme": "light" }"#).unwrap();
        assert_eq!(s.theme, "light");
        assert_eq!(s.font_family, "Verdana"); // defaulted
        assert!(s.preset.me_prefix); // defaulted
        assert_eq!(s.file_name_template, "screenie-{tanggal}-{jam}"); // defaulted to JGRP
    }
}
