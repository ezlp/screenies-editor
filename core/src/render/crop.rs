//! crop.rs — crop the source to the selected region, resize to the output.

use super::{CropRect, Size};
use crate::error::AppError;
use image::imageops::{self, FilterType};
use image::RgbaImage;

pub fn crop_and_resize(src: &RgbaImage, crop: &CropRect, out: &Size) -> Result<RgbaImage, AppError> {
    let (iw, ih) = (src.width() as f64, src.height() as f64);
    let x = crop.x.clamp(0.0, iw - 1.0).round() as u32;
    let y = crop.y.clamp(0.0, ih - 1.0).round() as u32;
    let w = crop.w.round().max(1.0) as u32;
    let h = crop.h.round().max(1.0) as u32;
    let w = w.min(src.width() - x);
    let h = h.min(src.height() - y);
    if out.w == 0 || out.h == 0 {
        return Err(AppError::Render("ukuran output 0".into()));
    }

    let cropped = imageops::crop_imm(src, x, y, w, h).to_image();
    if cropped.width() == out.w && cropped.height() == out.h {
        return Ok(cropped);
    }
    Ok(imageops::resize(&cropped, out.w, out.h, FilterType::Lanczos3))
}

/// Like `crop_and_resize`, but scales the cropped region to FIT inside `out`
/// (preserving its aspect), centers it, and fills the margins with `pad` — a
/// letterbox/pillarbox. Nothing in the crop region is lost or distorted.
pub fn crop_and_fit(
    src: &RgbaImage,
    crop: &CropRect,
    out: &Size,
    pad: [u8; 4],
) -> Result<RgbaImage, AppError> {
    let (iw, ih) = (src.width() as f64, src.height() as f64);
    let x = crop.x.clamp(0.0, iw - 1.0).round() as u32;
    let y = crop.y.clamp(0.0, ih - 1.0).round() as u32;
    let w = crop.w.round().max(1.0) as u32;
    let h = crop.h.round().max(1.0) as u32;
    let w = w.min(src.width() - x);
    let h = h.min(src.height() - y);
    if out.w == 0 || out.h == 0 {
        return Err(AppError::Render("ukuran output 0".into()));
    }
    let cropped = imageops::crop_imm(src, x, y, w, h).to_image();

    // Scale to fit inside the output, preserving aspect.
    let scale =
        (out.w as f32 / cropped.width() as f32).min(out.h as f32 / cropped.height() as f32);
    let dw = ((cropped.width() as f32 * scale).round() as u32).clamp(1, out.w);
    let dh = ((cropped.height() as f32 * scale).round() as u32).clamp(1, out.h);
    let resized = imageops::resize(&cropped, dw, dh, FilterType::Lanczos3);

    // Center the fitted image on a padded canvas.
    let mut canvas = RgbaImage::from_pixel(out.w, out.h, image::Rgba(pad));
    let ox = ((out.w - dw) / 2) as i64;
    let oy = ((out.h - dh) / 2) as i64;
    imageops::overlay(&mut canvas, &resized, ox, oy);
    Ok(canvas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn crops_then_resizes_to_exact_output() {
        let src = RgbaImage::from_pixel(100, 80, Rgba([10, 20, 30, 255]));
        let out = crop_and_resize(
            &src,
            &CropRect { x: 10.0, y: 10.0, w: 60.0, h: 45.0 },
            &Size { w: 800, h: 600 },
        )
        .unwrap();
        assert_eq!((out.width(), out.height()), (800, 600));
    }

    #[test]
    fn oversized_crop_is_clamped_into_the_image() {
        let src = RgbaImage::from_pixel(50, 50, Rgba([0, 0, 0, 255]));
        let out = crop_and_resize(
            &src,
            &CropRect { x: 40.0, y: 40.0, w: 100.0, h: 100.0 },
            &Size { w: 10, h: 10 },
        )
        .unwrap();
        assert_eq!((out.width(), out.height()), (10, 10));
    }
}
