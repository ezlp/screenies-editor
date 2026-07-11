//! render — MILESTONE 3 (not active yet)
//!
//! The export pipeline: everything the preview shows, re-rendered by Rust at
//! full resolution so the saved PNG is pixel-perfect even at 4K.
//!
//! Pipeline (compose.rs): load → crop → filters → backgrounds → text → stickers → PNG.

#![allow(dead_code)]

pub mod background;
pub mod compose;
pub mod crop;
pub mod filters;
pub mod sticker;
pub mod text;

use serde::Deserialize;

/// Everything needed for one export. The frontend (src/ts/export.ts)
/// assembles this and sends it over the bridge in Milestone 3.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderJob {
    /// Placeholder — M3 defines the real fields (image bytes, crop rect,
    /// zones, styles, filter values, stickers, output size).
    pub todo: Option<String>,
}
