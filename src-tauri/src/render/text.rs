//! text.rs — draw the pre-laid-out tokens onto the output image.
//!
//! Fonts come from the same fontdb the picker uses, loaded into ab_glyph.
//! The outline ("stroke") reproduces canvas strokeText by stamping the
//! glyph coverage in black at offsets around a circle of radius
//! strokeWidth/2, then filling the color on top — visually equivalent for
//! SSRP-scale strokes (2–7 px) and rock-solid across platforms.

use super::{ExportBlock, RenderJob};
use crate::error::AppError;
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::RgbaImage;

struct Faces {
    bold: FontVec,
    heavy: FontVec,
}

pub fn draw_blocks(img: &mut RgbaImage, job: &RenderJob) -> Result<(), AppError> {
    if job.blocks.iter().all(|b| b.rows.is_empty()) {
        return Ok(());
    }
    let faces = load_faces(&job.font_family)?;
    let scale = PxScale::from(job.text_size);
    // Precomputed once (v0.12.0 fix: was re-allocated per token). Empty
    // when stroke is 0 → the outline pass is skipped entirely.
    let offsets = if job.stroke_width > 0.0 {
        stroke_offsets((job.stroke_width / 2.0).max(0.5))
    } else {
        Vec::new()
    };

    for block in &job.blocks {
        draw_block(img, block, &faces, scale, &offsets);
    }
    Ok(())
}

fn draw_block(img: &mut RgbaImage, block: &ExportBlock, faces: &Faces, scale: PxScale, offsets: &[(f32, f32)]) {
    for row in &block.rows {
        // Pass 1: black outline for every token in the row…
        for token in &row.tokens {
            let font = if token.bold { &faces.heavy } else { &faces.bold };
            for &(dx, dy) in offsets {
                draw_token(img, font, scale, token.x + dx, row.y + dy, [0, 0, 0], &token.text);
            }
        }
        // …pass 2: colored fill on top.
        for token in &row.tokens {
            let font = if token.bold { &faces.heavy } else { &faces.bold };
            let color = parse_hex(&token.color);
            draw_token(img, font, scale, token.x, row.y, color, &token.text);
        }
    }
}

/// Draw one token with its left edge at `x` and its em-box TOP at `y`
/// (canvas textBaseline = "top").
fn draw_token(img: &mut RgbaImage, font: &FontVec, scale: PxScale, x: f32, y: f32, rgb: [u8; 3], text: &str) {
    let scaled = font.as_scaled(scale);
    let baseline = y + scaled.ascent();
    let mut pen = x;
    let mut prev = None;

    for ch in text.chars() {
        let id = scaled.glyph_id(ch);
        if let Some(p) = prev {
            pen += scaled.kern(p, id);
        }
        let glyph = id.with_scale_and_position(scale, ab_glyph::point(pen, baseline));
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, cov| {
                if cov <= 0.0 {
                    return;
                }
                let px = bounds.min.x as i32 + gx as i32;
                let py = bounds.min.y as i32 + gy as i32;
                blend(img, px, py, rgb, cov.min(1.0));
            });
        }
        pen += scaled.h_advance(id);
        prev = Some(id);
    }
}

fn blend(img: &mut RgbaImage, x: i32, y: i32, rgb: [u8; 3], a: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let dst = img.get_pixel_mut(x as u32, y as u32);
    for i in 0..3 {
        let s = rgb[i] as f32;
        let d = dst[i] as f32;
        dst[i] = (s * a + d * (1.0 - a)).round() as u8;
    }
    dst[3] = 255;
}

/// 16 directions at full radius + 8 at 60% for a solid round outline.
fn stroke_offsets(r: f32) -> Vec<(f32, f32)> {
    let mut out = Vec::with_capacity(24);
    for i in 0..16 {
        let a = (i as f32) * std::f32::consts::TAU / 16.0;
        out.push((a.cos() * r, a.sin() * r));
    }
    let r2 = r * 0.6;
    for i in 0..8 {
        let a = (i as f32) * std::f32::consts::TAU / 8.0 + 0.39;
        out.push((a.cos() * r2, a.sin() * r2));
    }
    out
}

fn parse_hex(s: &str) -> [u8; 3] {
    let h = s.trim_start_matches('#');
    if h.len() != 6 {
        return [255, 255, 255];
    }
    let v = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(255);
    [v(0), v(2), v(4)]
}

/// Load bold(700) + heavy(900) faces of the family from the system, with
/// graceful fallbacks (900→700, family→generic sans).
fn load_faces(family: &str) -> Result<Faces, AppError> {
    let db = crate::fonts::database(); // shared, scanned once per app run

    let bold = face_bytes(db, family, fontdb::Weight::BOLD)
        .or_else(|| face_bytes(db, family, fontdb::Weight::NORMAL))
        .or_else(|| generic_bytes(db, fontdb::Weight::BOLD))
        .ok_or_else(|| AppError::Render(format!("font '{family}' tidak ditemukan")))?;
    let heavy = face_bytes(db, family, fontdb::Weight::BLACK)
        .or_else(|| face_bytes(db, family, fontdb::Weight::EXTRA_BOLD))
        .unwrap_or_else(|| bold.clone());

    Ok(Faces {
        bold: FontVec::try_from_vec_and_index(bold.0, bold.1)
            .map_err(|e| AppError::Render(format!("font rusak: {e}")))?,
        heavy: FontVec::try_from_vec_and_index(heavy.0, heavy.1)
            .map_err(|e| AppError::Render(format!("font rusak: {e}")))?,
    })
}

type FaceData = (Vec<u8>, u32);

fn face_bytes(db: &fontdb::Database, family: &str, weight: fontdb::Weight) -> Option<FaceData> {
    let q = fontdb::Query {
        families: &[fontdb::Family::Name(family)],
        weight,
        ..Default::default()
    };
    let id = db.query(&q)?;
    db.with_face_data(id, |data, index| (data.to_vec(), index))
}

fn generic_bytes(db: &fontdb::Database, weight: fontdb::Weight) -> Option<FaceData> {
    let q = fontdb::Query {
        families: &[fontdb::Family::SansSerif],
        weight,
        ..Default::default()
    };
    let id = db.query(&q)?;
    db.with_face_data(id, |data, index| (data.to_vec(), index))
}
