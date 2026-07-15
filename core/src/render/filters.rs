//! filters.rs — brightness / grayscale / sepia / saturate / contrast,
//! implemented per the CSS Filter Effects spec so the export matches the
//! live preview on every platform (independent of any webview).

use super::{CensorKind, CensorRegion, FilterValues};
use image::RgbaImage;

/// Apply each censor box to the image: copy the rectangle out, run the same
/// blur/pixelate pass on it, and write it back. Rectangles are clamped to the
/// image; degenerate/off-image boxes are skipped.
pub fn apply_censors(img: &mut RgbaImage, regions: &[CensorRegion]) {
    let (iw, ih) = img.dimensions();
    for r in regions {
        let x0 = r.x.max(0.0).min(iw as f32) as u32;
        let y0 = r.y.max(0.0).min(ih as f32) as u32;
        let x1 = (r.x + r.w).max(0.0).min(iw as f32) as u32;
        let y1 = (r.y + r.h).max(0.0).min(ih as f32) as u32;
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        let (rw, rh) = (x1 - x0, y1 - y0);

        // Extract the region into its own buffer.
        let mut sub = RgbaImage::new(rw, rh);
        for yy in 0..rh {
            for xx in 0..rw {
                sub.put_pixel(xx, yy, *img.get_pixel(x0 + xx, y0 + yy));
            }
        }

        match r.kind {
            CensorKind::Blur if r.strength >= 1.0 => box_blur(&mut sub, r.strength.round() as u32),
            CensorKind::Pixelate if r.strength >= 2.0 => pixelate(&mut sub, r.strength.round() as u32),
            _ => continue, // strength too low to have an effect
        }

        // Write the censored region back.
        for yy in 0..rh {
            for xx in 0..rw {
                img.put_pixel(x0 + xx, y0 + yy, *sub.get_pixel(xx, yy));
            }
        }
    }
}

pub fn apply(img: &mut RgbaImage, f: &FilterValues) {
    let b = f.brightness / 100.0;
    let g = (f.grayscale / 100.0).clamp(0.0, 1.0);
    let s = (f.sepia / 100.0).clamp(0.0, 1.0);
    let sat = f.saturate / 100.0;
    let c = f.contrast / 100.0;

    // Per-pixel color ops are skipped when they'd be a no-op, but the
    // neighborhood passes (blur/pixelate) below may still need to run.
    let color_identity =
        b == 1.0 && g == 0.0 && s == 0.0 && (sat - 1.0).abs() < f32::EPSILON && c == 1.0;

    if !color_identity {
    for px in img.pixels_mut() {
        let mut r = px[0] as f32 / 255.0;
        let mut gr = px[1] as f32 / 255.0;
        let mut bl = px[2] as f32 / 255.0;

        // Order matches the preview's filter string: brightness → grayscale
        // → sepia → saturate → contrast (identity steps are no-ops).
        r *= b;
        gr *= b;
        bl *= b;

        if g > 0.0 {
            // CSS grayscale: interpolate toward luma with Rec.709 weights.
            let luma = 0.2126 * r + 0.7152 * gr + 0.0722 * bl;
            r += (luma - r) * g;
            gr += (luma - gr) * g;
            bl += (luma - bl) * g;
        }

        if s > 0.0 {
            // CSS sepia matrix, interpolated by s.
            let nr = 0.393 * r + 0.769 * gr + 0.189 * bl;
            let ng = 0.349 * r + 0.686 * gr + 0.168 * bl;
            let nb = 0.272 * r + 0.534 * gr + 0.131 * bl;
            r += (nr - r) * s;
            gr += (ng - gr) * s;
            bl += (nb - bl) * s;
        }

        if (sat - 1.0).abs() > f32::EPSILON {
            // CSS saturate matrix (Rec.709 luma constants).
            let nr = (0.213 + 0.787 * sat) * r + (0.715 - 0.715 * sat) * gr + (0.072 - 0.072 * sat) * bl;
            let ng = (0.213 - 0.213 * sat) * r + (0.715 + 0.285 * sat) * gr + (0.072 - 0.072 * sat) * bl;
            let nb = (0.213 - 0.213 * sat) * r + (0.715 - 0.715 * sat) * gr + (0.072 + 0.928 * sat) * bl;
            r = nr;
            gr = ng;
            bl = nb;
        }

        if c != 1.0 {
            r = (r - 0.5) * c + 0.5;
            gr = (gr - 0.5) * c + 0.5;
            bl = (bl - 0.5) * c + 0.5;
        }

        px[0] = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        px[1] = (gr.clamp(0.0, 1.0) * 255.0).round() as u8;
        px[2] = (bl.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }

    // Neighborhood passes run last, on the color-corrected pixels. Order:
    // pixelate (blocky mosaic) then blur (soften) — both are opt-in (0 = off).
    if f.pixelate >= 2.0 {
        pixelate(img, f.pixelate.round() as u32);
    }
    if f.blur >= 1.0 {
        box_blur(img, f.blur.round() as u32);
    }
}

/// Mosaic: replace each `block`×`block` cell with its average RGB. Alpha is
/// left untouched (SSRP photos are opaque; stickers blend separately).
fn pixelate(img: &mut RgbaImage, block: u32) {
    let (w, h) = img.dimensions();
    let block = block.max(1);
    let mut y = 0;
    while y < h {
        let mut x = 0;
        while x < w {
            let (mut sr, mut sg, mut sb, mut n) = (0u32, 0u32, 0u32, 0u32);
            for yy in y..(y + block).min(h) {
                for xx in x..(x + block).min(w) {
                    let p = img.get_pixel(xx, yy).0;
                    sr += p[0] as u32;
                    sg += p[1] as u32;
                    sb += p[2] as u32;
                    n += 1;
                }
            }
            if n > 0 {
                let (ar, ag, ab) = ((sr / n) as u8, (sg / n) as u8, (sb / n) as u8);
                for yy in y..(y + block).min(h) {
                    for xx in x..(x + block).min(w) {
                        let p = img.get_pixel_mut(xx, yy);
                        p[0] = ar;
                        p[1] = ag;
                        p[2] = ab;
                    }
                }
            }
            x += block;
        }
        y += block;
    }
}

/// Separable box blur of the given radius (edges clamped). Two passes
/// (horizontal then vertical) approximate a Gaussian cheaply; alpha kept. Uses a
/// running window sum, so cost is O(pixels) — independent of the radius.
fn box_blur(img: &mut RgbaImage, radius: u32) {
    let (w, h) = img.dimensions();
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    let r = radius as i64;
    let win = (2 * r + 1) as i64;

    // Each pass slides a window sum: at every step drop the pixel leaving on one
    // edge and add the one entering on the other (both edge-clamped) — the same
    // clamped average as the naive version, without the per-radius inner loop.
    //
    // Pass 1 writes every pixel of `scratch` (RGB) from `img`, and pass 2 reads
    // back only its RGB — never its alpha — so `scratch` just needs allocating,
    // not cloning. Final alpha comes from `img`, which pass 2 leaves untouched.
    let mut scratch = RgbaImage::new(w, h);
    for y in 0..h {
        let (mut sr, mut sg, mut sb) = (0i64, 0i64, 0i64);
        for dx in -r..=r {
            let sx = dx.clamp(0, w as i64 - 1) as u32;
            let p = img.get_pixel(sx, y).0;
            sr += p[0] as i64;
            sg += p[1] as i64;
            sb += p[2] as i64;
        }
        for x in 0..w {
            let p = scratch.get_pixel_mut(x, y);
            p[0] = (sr / win) as u8;
            p[1] = (sg / win) as u8;
            p[2] = (sb / win) as u8;
            let x_out = (x as i64 - r).clamp(0, w as i64 - 1) as u32;
            let x_in = (x as i64 + r + 1).clamp(0, w as i64 - 1) as u32;
            let pin = img.get_pixel(x_in, y).0;
            let pout = img.get_pixel(x_out, y).0;
            sr += pin[0] as i64 - pout[0] as i64;
            sg += pin[1] as i64 - pout[1] as i64;
            sb += pin[2] as i64 - pout[2] as i64;
        }
    }
    for x in 0..w {
        let (mut sr, mut sg, mut sb) = (0i64, 0i64, 0i64);
        for dy in -r..=r {
            let sy = dy.clamp(0, h as i64 - 1) as u32;
            let p = scratch.get_pixel(x, sy).0;
            sr += p[0] as i64;
            sg += p[1] as i64;
            sb += p[2] as i64;
        }
        for y in 0..h {
            let p = img.get_pixel_mut(x, y);
            p[0] = (sr / win) as u8;
            p[1] = (sg / win) as u8;
            p[2] = (sb / win) as u8;
            let y_out = (y as i64 - r).clamp(0, h as i64 - 1) as u32;
            let y_in = (y as i64 + r + 1).clamp(0, h as i64 - 1) as u32;
            let pin = scratch.get_pixel(x, y_in).0;
            let pout = scratch.get_pixel(x, y_out).0;
            sr += pin[0] as i64 - pout[0] as i64;
            sg += pin[1] as i64 - pout[1] as i64;
            sb += pin[2] as i64 - pout[2] as i64;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn one(px: [u8; 4], f: FilterValues) -> [u8; 4] {
        let mut img = RgbaImage::from_pixel(1, 1, Rgba(px));
        apply(&mut img, &f);
        img.get_pixel(0, 0).0
    }

    fn base() -> FilterValues {
        FilterValues {
            brightness: 100.0,
            grayscale: 0.0,
            sepia: 0.0,
            saturate: 100.0,
            contrast: 100.0,
            blur: 0.0,
            pixelate: 0.0,
        }
    }

    #[test]
    fn identity_leaves_pixels_alone() {
        assert_eq!(one([120, 80, 200, 255], base()), [120, 80, 200, 255]);
    }

    #[test]
    fn full_grayscale_matches_rec709_luma() {
        let out = one([255, 0, 0, 255], FilterValues { grayscale: 100.0, ..base() });
        let luma = (0.2126f32 * 255.0).round() as u8;
        assert_eq!(out[0], luma);
        assert_eq!(out[0], out[1]);
        assert_eq!(out[1], out[2]);
    }

    #[test]
    fn brightness_zero_is_black_and_200_doubles() {
        assert_eq!(one([90, 90, 90, 255], FilterValues { brightness: 0.0, ..base() }), [0, 0, 0, 255]);
        assert_eq!(one([90, 90, 90, 255], FilterValues { brightness: 200.0, ..base() })[0], 180);
    }

    #[test]
    fn contrast_pivots_at_middle_gray() {
        let out = one([128, 128, 128, 255], FilterValues { contrast: 200.0, ..base() });
        assert!(out[0].abs_diff(128) <= 1); // mid-gray is the fixed point
    }

    #[test]
    fn pixelate_averages_each_block() {
        // 2×2 image, block size 2 → every pixel becomes the mean of all four.
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([100, 100, 100, 255]));
        img.put_pixel(0, 1, Rgba([100, 100, 100, 255]));
        img.put_pixel(1, 1, Rgba([200, 200, 200, 255]));
        apply(&mut img, &FilterValues { pixelate: 2.0, ..base() });
        // mean = (0+100+100+200)/4 = 100 for every cell.
        for p in img.pixels() {
            assert_eq!(p.0[0], 100);
            assert_eq!(p.0[3], 255); // alpha preserved
        }
    }

    #[test]
    fn blur_moves_a_lone_bright_pixel_toward_its_neighbors() {
        let mut img = RgbaImage::from_pixel(5, 5, Rgba([0, 0, 0, 255]));
        img.put_pixel(2, 2, Rgba([255, 255, 255, 255]));
        apply(&mut img, &FilterValues { blur: 1.0, ..base() });
        // Center is pulled down from 255; a neighbor is pulled up from 0.
        assert!(img.get_pixel(2, 2).0[0] < 255);
        assert!(img.get_pixel(2, 1).0[0] > 0);
    }

    #[test]
    fn censor_region_only_touches_its_rectangle() {
        // 6×2 image: left half black (cols 0–2), right half white (cols 3–5).
        let mut img = RgbaImage::new(6, 2);
        for y in 0..2 {
            for x in 0..6 {
                let v = if x < 3 { 0 } else { 255 };
                img.put_pixel(x, y, Rgba([v, v, v, 255]));
            }
        }
        // Pixelate a 2×2 box over cols 2–3 (straddles the black/white edge).
        let region = CensorRegion {
            x: 2.0,
            y: 0.0,
            w: 2.0,
            h: 2.0,
            kind: CensorKind::Pixelate,
            strength: 2.0,
        };
        apply_censors(&mut img, &[region]);
        // Inside the box: cols 2 (black) + 3 (white) average to mid-gray.
        let inside = img.get_pixel(2, 0).0[0];
        assert!(inside > 100 && inside < 160, "region should be averaged, got {inside}");
        // Outside the box: untouched.
        assert_eq!(img.get_pixel(0, 0).0[0], 0);
        assert_eq!(img.get_pixel(5, 0).0[0], 255);
    }

    #[test]
    fn effects_run_even_when_color_ops_are_identity() {
        // Regression: the early-return for identity color ops must not skip
        // the neighborhood passes.
        let mut img = RgbaImage::from_pixel(4, 4, Rgba([10, 20, 30, 255]));
        img.put_pixel(0, 0, Rgba([250, 250, 250, 255]));
        apply(&mut img, &FilterValues { pixelate: 4.0, ..base() });
        // One 4×4 block → all pixels share the block average (not the original).
        let a = img.get_pixel(0, 0).0;
        let b = img.get_pixel(3, 3).0;
        assert_eq!(a, b);
    }
}
