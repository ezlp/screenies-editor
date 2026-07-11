//! error.rs — one error type for every command.
//!
//! Tauri serializes command errors to the frontend; keeping a single
//! serializable enum means TypeScript always receives a predictable
//! `{ kind, message }` object it can show to the user.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AppError {
    /// Input text/data was invalid or unprocessable.
    #[allow(dead_code)] // first used in Milestone 2
    Parse(String),
    /// File system problem (load/save). Used from Milestone 3.
    #[allow(dead_code)]
    Io(String),
    /// Image decode/encode/render problem. Used from Milestone 3.
    #[allow(dead_code)]
    Render(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Parse(m) => write!(f, "parse error: {m}"),
            AppError::Io(m) => write!(f, "io error: {m}"),
            AppError::Render(m) => write!(f, "render error: {m}"),
        }
    }
}

impl std::error::Error for AppError {}
