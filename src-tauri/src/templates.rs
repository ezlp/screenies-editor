//! templates.rs — quick-text templates in templates.json (Milestone 4c).
//!
//! Same philosophy as presets: plain JSON in the app config dir,
//! forward-compatible, shareable, never bricks startup when corrupt.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct QuickText {
    pub label: String,
    pub text: String,
}

impl Default for QuickText {
    fn default() -> Self {
        Self { label: String::new(), text: String::new() }
    }
}

fn templates_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(dir.join("templates.json"))
}

pub fn load(app: &AppHandle) -> Result<Vec<QuickText>, AppError> {
    let path = templates_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    match serde_json::from_str::<Vec<QuickText>>(&text) {
        Ok(items) => Ok(items),
        Err(e) => {
            eprintln!("[screenies-editor] templates.json rusak, mulai kosong: {e}");
            Ok(Vec::new())
        }
    }
}

pub fn save(app: &AppHandle, items: &[QuickText]) -> Result<(), AppError> {
    let path = templates_path(app)?;
    let text = serde_json::to_string_pretty(items).map_err(|e| AppError::Parse(e.to_string()))?;
    fs::write(&path, text).map_err(|e| AppError::Io(e.to_string()))
}
