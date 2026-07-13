//! error.rs — one error type for every command.
//!
//! The shell surfaces command errors to the user; keeping a single
//! serializable enum means every caller gets a predictable
//! `{ kind, message }` object it can show.

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum AppError {
    /// Input text/data was invalid or unprocessable.
    Parse(String),
    /// File system problem (load/save).
    Io(String),
    /// Image decode/encode/render problem.
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
