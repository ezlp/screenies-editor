//! compose.rs — the export pipeline, start to finish.

use super::{crop, filters, sticker, text, RenderJob};
use crate::error::AppError;
use base64::Engine;
use image::RgbaImage;
use std::io::Cursor;

/// Full render: base64 photo → cropped, resized, filtered, text-stamped RGBA.
pub fn render(job: &RenderJob) -> Result<RgbaImage, AppError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&job.image_base64)
        .map_err(|e| AppError::Parse(format!("base64: {e}")))?;

    let src = image::load_from_memory(&bytes)
        .map_err(|e| AppError::Render(format!("decode foto: {e}")))?
        .to_rgba8();

    // Photo: crop → resize to the output → filters (photo only) → local censors.
    let mut out = crop::crop_and_resize(&src, &job.crop, &job.output)?;
    filters::apply(&mut out, &job.filters);
    filters::apply_censors(&mut out, &job.censors);

    // Stickers under the text, then text (with its bg strips).
    sticker::overlay_all(&mut out, &job.stickers)?;
    text::draw_blocks(&mut out, job)?;
    Ok(out)
}

/// Encode the rendered image as PNG bytes.
pub fn encode_png(img: &RgbaImage) -> Result<Vec<u8>, AppError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| AppError::Render(format!("encode png: {e}")))?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{CropRect, FilterValues, Size};
    use base64::Engine;

    fn tiny_job() -> RenderJob {
        // 4×4 red PNG, generated in-memory so the test has no file deps.
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
        let mut png = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png, image::ImageFormat::Png).unwrap();
        RenderJob {
            image_base64: base64::engine::general_purpose::STANDARD.encode(png.into_inner()),
            crop: CropRect { x: 0.0, y: 0.0, w: 4.0, h: 4.0 },
            output: Size { w: 8, h: 8 },
            stickers: vec![],
            filters: FilterValues {
                brightness: 100.0,
                grayscale: 100.0,
                sepia: 0.0,
                saturate: 100.0,
                contrast: 100.0,
                blur: 0.0,
                pixelate: 0.0,
            },
            censors: vec![],
            font_family: "__none__".into(),
            text_size: 20.0,
            stroke_width: 3.0,
            blocks: vec![], // no text → font never loaded → CI-safe
        }
    }

    #[test]
    fn pipeline_decodes_resizes_filters_and_encodes() {
        let img = render(&tiny_job()).unwrap();
        assert_eq!((img.width(), img.height()), (8, 8));
        let p = img.get_pixel(4, 4);
        assert_eq!(p[0], p[1]); // grayscale 100% → r == g == b
        assert_eq!(p[1], p[2]);
        let png = encode_png(&img).unwrap();
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    }
}
