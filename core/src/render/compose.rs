//! compose.rs — the export pipeline, start to finish.

use super::{crop, filters, sticker, text, RenderJob, Size};
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

    // Normal mode: the photo fills the output. Cinematic mode: a solid-color
    // canvas with the photo placed in a centered sub-rect (bars around it).
    // Filters apply to the photo only; the bars stay the pure canvas color.
    let mut out = match &job.canvas {
        None => {
            let mut o = crop::crop_and_resize(&src, &job.crop, &job.output)?;
            filters::apply(&mut o, &job.filters);
            o
        }
        Some(cv) => {
            let mut canvas = image::RgbaImage::from_pixel(
                job.output.w.max(1),
                job.output.h.max(1),
                image::Rgba(cv.color),
            );
            let pw = (cv.photo_w.round() as u32).max(1);
            let ph = (cv.photo_h.round() as u32).max(1);
            let mut photo = crop::crop_and_resize(&src, &job.crop, &Size { w: pw, h: ph })?;
            filters::apply(&mut photo, &job.filters);
            image::imageops::overlay(
                &mut canvas,
                &photo,
                cv.photo_x.round() as i64,
                cv.photo_y.round() as i64,
            );
            canvas
        }
    };

    // Stickers under the text, then text (with its bg strips).
    sticker::overlay_all(&mut out, &job.stickers)?;
    text::draw_blocks(&mut out, job)?;

    // Censor boxes run LAST so blur/pixelate cover whatever sits under them —
    // photo, stickers, AND text (so you can redact a name/plate over anything).
    filters::apply_censors(&mut out, &job.censors);
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
    use crate::render::{Canvas, CropRect, FilterValues, Size};
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
            canvas: None,
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

    #[test]
    fn cinematic_canvas_has_solid_bars_and_a_centered_photo() {
        // 8×12 output: 2px black bars top & bottom, an 8×8 photo band centered.
        let mut job = tiny_job();
        job.output = Size { w: 8, h: 12 };
        job.canvas = Some(Canvas {
            color: [0, 0, 0, 255],
            photo_x: 0.0,
            photo_y: 2.0,
            photo_w: 8.0,
            photo_h: 8.0,
        });
        let img = render(&job).unwrap();
        assert_eq!((img.width(), img.height()), (8, 12));
        // Top and bottom rows are the pure canvas color (black).
        assert_eq!(img.get_pixel(4, 0).0, [0, 0, 0, 255]);
        assert_eq!(img.get_pixel(4, 11).0, [0, 0, 0, 255]);
        // The photo band (red → grayscaled by tiny_job) is not black.
        assert!(img.get_pixel(4, 6).0[0] > 0, "photo band should be visible");
    }
}
