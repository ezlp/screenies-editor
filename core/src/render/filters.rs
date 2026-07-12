//! filters.rs — brightness / grayscale / sepia / saturate / contrast,
//! implemented per the CSS Filter Effects spec so the export matches the
//! live preview on every platform (independent of any webview).

use super::FilterValues;
use image::RgbaImage;

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
/// (horizontal then vertical) approximate a Gaussian cheaply; alpha kept.
fn box_blur(img: &mut RgbaImage, radius: u32) {
    let (w, h) = img.dimensions();
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    let r = radius as i64;
    let win = (2 * r + 1) as u32;

    // Horizontal pass into a scratch buffer, then vertical pass back.
    let mut scratch = img.clone();
    for y in 0..h {
        for x in 0..w {
            let (mut sr, mut sg, mut sb) = (0u32, 0u32, 0u32);
            for dx in -r..=r {
                let sx = (x as i64 + dx).clamp(0, w as i64 - 1) as u32;
                let p = img.get_pixel(sx, y).0;
                sr += p[0] as u32;
                sg += p[1] as u32;
                sb += p[2] as u32;
            }
            let p = scratch.get_pixel_mut(x, y);
            p[0] = (sr / win) as u8;
            p[1] = (sg / win) as u8;
            p[2] = (sb / win) as u8;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let (mut sr, mut sg, mut sb) = (0u32, 0u32, 0u32);
            for dy in -r..=r {
                let sy = (y as i64 + dy).clamp(0, h as i64 - 1) as u32;
                let p = scratch.get_pixel(x, sy).0;
                sr += p[0] as u32;
                sg += p[1] as u32;
                sb += p[2] as u32;
            }
            let p = img.get_pixel_mut(x, y);
            p[0] = (sr / win) as u8;
            p[1] = (sg / win) as u8;
            p[2] = (sb / win) as u8;
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
