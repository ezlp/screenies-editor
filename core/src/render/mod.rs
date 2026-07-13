//! render — the export pipeline (Milestone 3c).
//!
//! The frontend sends a `RenderJob` containing the source photo, crop,
//! output size, filter values, and — crucially — the TEXT LAYOUT it
//! already computed (every token with absolute x/y in output space).
//! Rust never re-wraps text, so the export matches the preview by design.
//!
//! Pipeline (compose.rs): decode → crop → resize → filters → text → done.

pub mod compose;
pub mod crop;
pub mod filters;
pub mod layout;
pub mod sticker;
pub mod text;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderJob {
    /// Source photo bytes, base64 (PNG/JPEG/WebP/BMP).
    pub image_base64: String,
    pub crop: CropRect,
    pub output: Size,
    pub stickers: Vec<StickerJob>,
    pub filters: FilterValues,
    /// Local censor boxes (blur/pixelate a rectangle) — applied to the photo,
    /// under stickers and text. Empty in old payloads via serde(default).
    #[serde(default)]
    pub censors: Vec<CensorRegion>,
    pub font_family: String,
    /// Text size in output px.
    pub text_size: f32,
    /// Outline thickness in output px (same formula as the preview).
    pub stroke_width: f32,
    pub blocks: Vec<ExportBlock>,
}

/// Which effect a censor box applies.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CensorKind {
    Blur,
    Pixelate,
}

/// A rectangular blur/pixelate region in output px — like a sticker, but it
/// censors the photo underneath instead of drawing an image.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CensorRegion {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub kind: CensorKind,
    /// Blur radius (px) or pixelate block size (px), per `kind`.
    pub strength: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterValues {
    pub brightness: f32, // percent, 100 = identity
    pub grayscale: f32,  // percent, 0 = identity
    pub sepia: f32,
    pub saturate: f32,
    pub contrast: f32,
    // ── 2.0 effects (neighborhood passes, run after the per-pixel color ops).
    //    serde(default) keeps old 1.x RenderJob payloads (5 fields) parsing. ──
    /// Box-blur radius in output px. 0 = off.
    #[serde(default)]
    pub blur: f32,
    /// Pixelate block size in output px. 0 or 1 = off.
    #[serde(default)]
    pub pixelate: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBlock {
    pub rows: Vec<ExportRow>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRow {
    /// Top of the row's em box, output px (canvas textBaseline "top").
    pub y: f32,
    pub tokens: Vec<ExportToken>,
    /// Optional dark strip behind the row (BG blok / mask), 55% black.
    pub bg: Option<BgRect>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BgRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StickerJob {
    pub data_base64: String,
    pub x: i64,
    pub y: i64,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportToken {
    pub text: String,
    /// Left edge, output px (measured by the preview canvas).
    pub x: f32,
    /// "#RRGGBB"
    pub color: String,
    pub bold: bool,
}
