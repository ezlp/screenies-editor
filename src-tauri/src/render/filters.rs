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

    let identity =
        b == 1.0 && g == 0.0 && s == 0.0 && (sat - 1.0).abs() < f32::EPSILON && c == 1.0;
    if identity {
        return;
    }

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
        FilterValues { brightness: 100.0, grayscale: 0.0, sepia: 0.0, saturate: 100.0, contrast: 100.0 }
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
}
