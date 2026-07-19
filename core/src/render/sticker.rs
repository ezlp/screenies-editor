//! sticker.rs — decode, resize, and alpha-overlay PNG stickers.
//!
//! The preview re-composites every frame while a sticker is dragged, so doing a
//! base64 decode + PNG/WebP decode + Lanczos3 resize per sticker per frame was
//! the dominant interactive cost. Two small caches remove the repeated work:
//!   • `decoded_cache` — full-res source per sticker (keyed on a hash of its
//!     base64), so the expensive image decode happens once, not every frame.
//!   • `resized_cache` — the last resized result per sticker, so a *move*
//!     (size unchanged) is a pure blend and only a *resize* re-runs Lanczos —
//!     and even that scales from the cached source, never re-decoding.
//! Both are keyed by content hash and hold one entry per distinct sticker, so
//! they stay bounded regardless of how far a sticker is dragged or resized.

use super::StickerJob;
use crate::error::AppError;
use base64::Engine;
use image::imageops::{self, FilterType};
use image::RgbaImage;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

/// Full-res decoded sources, keyed on a hash of the base64 payload. Bounded so a
/// long session that cycles through many stickers can't grow without limit.
fn decoded_cache() -> &'static Mutex<HashMap<u64, Arc<RgbaImage>>> {
    static C: OnceLock<Mutex<HashMap<u64, Arc<RgbaImage>>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Last resized result per source: hash → (w, h, image). One entry per sticker,
/// overwritten when the target size changes — a move hits it, a resize replaces
/// it (no unbounded growth as the drag sweeps through sizes).
#[allow(clippy::type_complexity)]
fn resized_cache() -> &'static Mutex<HashMap<u64, (u32, u32, Arc<RgbaImage>)>> {
    static C: OnceLock<Mutex<HashMap<u64, (u32, u32, Arc<RgbaImage>)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Distinct source images to keep decoded before clearing the cache. Stickers
/// are few in practice; this is just a backstop against pathological sessions.
const MAX_DECODED: usize = 32;

fn hash_of(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

pub fn overlay_all(canvas: &mut RgbaImage, stickers: &[StickerJob]) -> Result<(), AppError> {
    for st in stickers {
        let (w, h) = (st.w.max(1), st.h.max(1));
        let key = hash_of(&st.data_base64);

        // Fast path: same source at the same size as last frame → just blend.
        let cached = {
            let cache = resized_cache().lock().unwrap();
            cache
                .get(&key)
                .and_then(|(cw, ch, img)| (*cw == w && *ch == h).then(|| img.clone()))
        };
        let sized = if let Some(img) = cached {
            img
        } else {
            // Decode the source once (memoized), then resize from it as needed.
            let source = decode_source(key, &st.data_base64)?;
            let sized = if source.width() == w && source.height() == h {
                source
            } else {
                Arc::new(imageops::resize(source.as_ref(), w, h, FilterType::Lanczos3))
            };
            resized_cache().lock().unwrap().insert(key, (w, h, sized.clone()));
            sized
        };

        if st.opacity < 1.0 {
            let mut sized_img = sized.as_ref().clone();
            for pixel in sized_img.pixels_mut() {
                pixel.0[3] = (pixel.0[3] as f32 * st.opacity).round() as u8;
            }
            imageops::overlay(canvas, &sized_img, st.x, st.y);
        } else {
            imageops::overlay(canvas, sized.as_ref(), st.x, st.y);
        }
    }
    Ok(())
}

/// The full-res decoded sticker for `key`, decoding + caching on first sight.
fn decode_source(key: u64, data_base64: &str) -> Result<Arc<RgbaImage>, AppError> {
    if let Some(img) = decoded_cache().lock().unwrap().get(&key) {
        return Ok(img.clone());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64)
        .map_err(|e| AppError::Parse(format!("stiker base64: {e}")))?;
    let img = Arc::new(
        image::load_from_memory(&bytes)
            .map_err(|e| AppError::Render(format!("decode stiker: {e}")))?
            .to_rgba8(),
    );
    let mut cache = decoded_cache().lock().unwrap();
    if cache.len() >= MAX_DECODED {
        cache.clear(); // rare backstop; keeps memory bounded
        resized_cache().lock().unwrap().clear(); // also clear resized cache (garbage collection)
    }
    cache.insert(key, img.clone());
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_b64(color: [u8; 4], size: u32) -> String {
        let img = RgbaImage::from_pixel(size, size, image::Rgba(color));
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        base64::engine::general_purpose::STANDARD.encode(buf.into_inner())
    }

    #[test]
    fn overlays_and_repeats_identically_from_cache() {
        // Opaque red sticker over a black canvas, same size → no resize.
        let st = StickerJob { data_base64: png_b64([255, 0, 0, 255], 2), x: 1, y: 1, w: 2, h: 2 };

        let mut canvas = RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 255]));
        overlay_all(&mut canvas, std::slice::from_ref(&st)).unwrap();
        assert_eq!(canvas.get_pixel(1, 1).0, [255, 0, 0, 255]); // sticker drawn
        assert_eq!(canvas.get_pixel(0, 0).0, [0, 0, 0, 255]); // outside untouched

        // Second call hits the resized cache and must produce the same result.
        let mut canvas2 = RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 255]));
        overlay_all(&mut canvas2, std::slice::from_ref(&st)).unwrap();
        assert_eq!(canvas2.get_pixel(1, 1).0, [255, 0, 0, 255]);
    }

    #[test]
    fn resize_reuses_the_decoded_source() {
        // Draw the same source at two different sizes; both must composite.
        let b64 = png_b64([0, 200, 0, 255], 8);
        let big = StickerJob { data_base64: b64.clone(), x: 0, y: 0, w: 6, h: 6 };
        let small = StickerJob { data_base64: b64, x: 0, y: 0, w: 3, h: 3 };

        let mut canvas = RgbaImage::from_pixel(6, 6, image::Rgba([0, 0, 0, 255]));
        overlay_all(&mut canvas, std::slice::from_ref(&big)).unwrap();
        assert_eq!(canvas.get_pixel(0, 0).0, [0, 200, 0, 255]);

        let mut canvas = RgbaImage::from_pixel(6, 6, image::Rgba([0, 0, 0, 255]));
        overlay_all(&mut canvas, std::slice::from_ref(&small)).unwrap();
        assert_eq!(canvas.get_pixel(0, 0).0, [0, 200, 0, 255]);
        assert_eq!(canvas.get_pixel(5, 5).0, [0, 0, 0, 255]); // 3×3 sticker, corner clear
    }

    #[test]
    fn invalid_base64_still_errors() {
        let st = StickerJob { data_base64: "not valid base64!!!".into(), x: 0, y: 0, w: 2, h: 2 };
        let mut canvas = RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 255]));
        assert!(overlay_all(&mut canvas, std::slice::from_ref(&st)).is_err());
    }
}
