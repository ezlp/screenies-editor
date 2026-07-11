//! sticker.rs — decode, resize, and alpha-overlay PNG stickers.

use super::StickerJob;
use crate::error::AppError;
use base64::Engine;
use image::imageops::{self, FilterType};
use image::RgbaImage;

pub fn overlay_all(canvas: &mut RgbaImage, stickers: &[StickerJob]) -> Result<(), AppError> {
    for st in stickers {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&st.data_base64)
            .map_err(|e| AppError::Parse(format!("stiker base64: {e}")))?;
        let img = image::load_from_memory(&bytes)
            .map_err(|e| AppError::Render(format!("decode stiker: {e}")))?
            .to_rgba8();
        let sized = if img.width() == st.w && img.height() == st.h {
            img
        } else {
            imageops::resize(&img, st.w.max(1), st.h.max(1), FilterType::Lanczos3)
        };
        imageops::overlay(canvas, &sized, st.x, st.y); // proper alpha blend
    }
    Ok(())
}
