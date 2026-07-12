//! screenies-core — the shared heart of ScreeniesEditor.
//!
//! Shell-independent by design: no Tauri, no windowing, no dialogs.
//! Anything that touches the OS shell (config paths, file dialogs, IPC)
//! lives in the shell (src-tauri/) — everything that computes
//! lives here, tested once, reused everywhere (including WASM later).

pub mod chatlog;
pub mod chatlog_library;
pub mod clipboard;
pub mod error;
pub mod fonts;
pub mod gallery;
pub mod render;
