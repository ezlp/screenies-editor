//! compose.rs — the export pipeline, start to finish.

use super::{crop, filters, sticker, text, RenderJob};
use crate::error::AppError;
use base64::Engine;
use image::RgbaImage;
use std::io::Cursor;

/// Decode + crop/resize/fit the photo into the output-sized BASE image (no
/// filters, effects or text yet). This is the expensive part; a shell can cache
/// it and call `render_onto` when only filter/effect/text params change. Fit mode
/// scales the whole crop inside the output with black padding (keeps everything);
/// otherwise it crops to fill.
pub fn prepare_base(job: &RenderJob) -> Result<RgbaImage, AppError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&job.image_base64)
        .map_err(|e| AppError::Parse(format!("base64: {e}")))?;
    let src = image::load_from_memory(&bytes)
        .map_err(|e| AppError::Render(format!("decode foto: {e}")))?
        .to_rgba8();
    if job.fit {
        crop::crop_and_fit(&src, &job.crop, &job.output, [0, 0, 0, 255])
    } else {
        crop::crop_and_resize(&src, &job.crop, &job.output)
    }
}

/// Apply everything on top of a prepared base: filters, cinematic bars, stickers,
/// text, then censor boxes LAST (so blur/pixelate cover whatever sits under them
/// — photo, stickers, AND text). Takes the base by value.
pub fn render_onto(mut out: RgbaImage, job: &RenderJob) -> Result<RgbaImage, AppError> {
    filters::apply(&mut out, &job.filters);
    if let Some(cv) = &job.canvas {
        draw_letterbox(&mut out, cv);
    }
    sticker::overlay_all(&mut out, &job.stickers)?;
    text::draw_blocks(&mut out, job)?;
    filters::apply_censors(&mut out, &job.censors);
    Ok(out)
}

/// Full render: base64 photo → cropped/resized/fitted → filtered, text-stamped.
pub fn render(job: &RenderJob) -> Result<RgbaImage, AppError> {
    render_onto(prepare_base(job)?, job)
}

/// Cinematic letterbox: paint solid top & bottom bars over the image. The bars
/// grow inward (output size unchanged) and are clamped so they never overlap.
fn draw_letterbox(img: &mut RgbaImage, cv: &super::Canvas) {
    use super::BarPos;
    let (w, h) = img.dimensions();
    let bar = cv.bar.min(h / 2);
    if bar == 0 {
        return;
    }
    let top = matches!(cv.bars, BarPos::Both | BarPos::Top);
    let bottom = matches!(cv.bars, BarPos::Both | BarPos::Bottom);
    let color = image::Rgba(cv.color);
    for y in 0..bar {
        for x in 0..w {
            if top {
                img.put_pixel(x, y, color);
            }
            if bottom {
                img.put_pixel(x, h - 1 - y, color);
            }
        }
    }
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
    use crate::render::{BarPos, Canvas, CropRect, FilterValues, Size};
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
            fit: false,
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
    fn cinematic_letterbox_paints_bars_over_photo_top_and_bottom() {
        // Output stays 8×12; 3px black bars cover the top and bottom rows, the
        // photo shows through the middle band.
        let mut job = tiny_job();
        job.output = Size { w: 8, h: 12 };
        job.canvas = Some(Canvas { color: [0, 0, 0, 255], bar: 3, bars: BarPos::Both });
        let img = render(&job).unwrap();
        assert_eq!((img.width(), img.height()), (8, 12)); // size unchanged by bars
        // Top 3 and bottom 3 rows are the bar color (black).
        assert_eq!(img.get_pixel(4, 0).0, [0, 0, 0, 255]);
        assert_eq!(img.get_pixel(4, 2).0, [0, 0, 0, 255]);
        assert_eq!(img.get_pixel(4, 9).0, [0, 0, 0, 255]);
        assert_eq!(img.get_pixel(4, 11).0, [0, 0, 0, 255]);
        // The middle band shows the photo (red → grayscaled), not black.
        assert!(img.get_pixel(4, 6).0[0] > 0, "photo band should show through");
    }

    #[test]
    fn cinematic_bottom_only_leaves_the_top_untouched() {
        let mut job = tiny_job();
        job.output = Size { w: 8, h: 12 };
        job.canvas = Some(Canvas { color: [0, 0, 0, 255], bar: 3, bars: BarPos::Bottom });
        let img = render(&job).unwrap();
        // Bottom rows are the bar; top row still shows the photo (not black).
        assert_eq!(img.get_pixel(4, 11).0, [0, 0, 0, 255]);
        assert!(img.get_pixel(4, 0).0[0] > 0, "top must be untouched when bottom-only");
    }

    #[test]
    fn fit_keeps_the_whole_image_with_padding() {
        // 4×4 (square) crop fit into an 8×4 output → a 4×4 image centered, with
        // black side-padding (nothing cropped).
        let mut job = tiny_job();
        job.output = Size { w: 8, h: 4 };
        job.fit = true;
        let img = render(&job).unwrap();
        assert_eq!((img.width(), img.height()), (8, 4));
        // Left & right columns are black padding.
        assert_eq!(img.get_pixel(0, 2).0, [0, 0, 0, 255]);
        assert_eq!(img.get_pixel(7, 2).0, [0, 0, 0, 255]);
        // The centered photo shows through (grayscaled red, not pure black).
        assert!(img.get_pixel(4, 2).0[0] > 0, "fitted photo should be centered");
    }
}
