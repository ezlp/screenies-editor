//! theme.rs — theme engine for ScreeniesEditor 3.0.
//!
//! Defines semantic colors, 7 builtin themes, and applies them to egui's Visuals + Style.
//! Themes are data-driven (no hardcoded colors in UI code); screens read from
//! ui.visuals() or pass &Theme for the two semantic colors with no Visuals slot.

use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};

/// A complete, themeable color palette + density.
#[allow(dead_code)]
#[derive(Clone)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub dark: bool,
    // Semantic colors
    pub bg: Color32,         // panel_fill — central + side panels
    pub surface: Color32,    // window_fill / faint_bg — cards, popups, headers
    pub surface_2: Color32,  // extreme_bg — text edits / inset fields
    pub ink: Color32,        // override_text_color
    pub ink_dim: Color32,    // weak text (no Visuals slot → passed via &Theme)
    pub accent: Color32,     // selection / active widget / hyperlink
    pub accent_ink: Color32, // text on accent fills
    pub border: Color32,     // widget + window strokes
    pub error: Color32,
    pub warn: Color32,
    pub good: Color32,
    pub rounding: f32,
    pub dense: bool,
}

impl Theme {
    /// Apply this theme to egui's Visuals + Style, with optional accent override.
    pub fn apply(&self, ctx: &egui::Context, accent: Option<Color32>, scale: f32, dense: bool) {
        let accent = accent.unwrap_or(self.accent);

        let mut v = if self.dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.panel_fill = self.bg;
        v.window_fill = self.surface;
        v.faint_bg_color = self.surface;
        v.extreme_bg_color = self.surface_2;
        v.override_text_color = Some(self.ink);
        v.hyperlink_color = accent;
        v.error_fg_color = self.error;
        v.warn_fg_color = self.warn;
        v.window_stroke = Stroke::new(1.0_f32, self.border);
        v.selection.bg_fill = accent.linear_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0_f32, accent);

        let r = Rounding::same(self.rounding);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.rounding = r;
            w.bg_stroke = Stroke::new(1.0_f32, self.border);
            w.fg_stroke = Stroke::new(1.0_f32, self.ink);
        }
        v.widgets.inactive.weak_bg_fill = self.surface_2; // idle buttons
        v.widgets.active.bg_fill = accent;
        v.widgets.active.fg_stroke = Stroke::new(1.0_f32, self.accent_ink);

        ctx.set_visuals(v);

        let mut style = (*ctx.style()).clone();
        let sp = &mut style.spacing;
        let (gap, pad, h) = if dense {
            (4.0, 3.0, 22.0)
        } else {
            (8.0, 6.0, 26.0)
        };
        sp.item_spacing = egui::vec2(if dense { 6.0 } else { 8.0 }, gap);
        sp.button_padding = egui::vec2(if dense { 6.0 } else { 10.0 }, pad);
        sp.interact_size.y = h;
        ctx.set_style(style);

        ctx.set_zoom_factor(scale);
    }
}

/// Midnight (default): dark with lavender accent — the original app's color.
pub const MIDNIGHT: Theme = Theme {
    id: "midnight",
    name: "Midnight",
    dark: true,
    bg: Color32::from_rgb(20, 22, 28),
    surface: Color32::from_rgb(30, 32, 40),
    surface_2: Color32::from_rgb(24, 26, 32),
    ink: Color32::from_rgb(245, 245, 247),
    ink_dim: Color32::from_rgb(180, 180, 190),
    accent: Color32::from_rgb(194, 162, 218), // lavender
    accent_ink: Color32::from_rgb(20, 22, 28),
    border: Color32::from_rgb(60, 65, 80),
    error: Color32::from_rgb(239, 68, 68),
    warn: Color32::from_rgb(245, 158, 11),
    good: Color32::from_rgb(34, 197, 94),
    rounding: 6.0,
    dense: false,
};

/// Paper: light theme, minimalist.
pub const PAPER: Theme = Theme {
    id: "paper",
    name: "Paper",
    dark: false,
    bg: Color32::from_rgb(255, 255, 255),
    surface: Color32::from_rgb(248, 248, 250),
    surface_2: Color32::from_rgb(240, 240, 244),
    ink: Color32::from_rgb(30, 30, 35),
    ink_dim: Color32::from_rgb(120, 120, 130),
    accent: Color32::from_rgb(124, 58, 255), // vibrant purple
    accent_ink: Color32::from_rgb(255, 255, 255),
    border: Color32::from_rgb(200, 200, 210),
    error: Color32::from_rgb(220, 38, 38),
    warn: Color32::from_rgb(217, 119, 6),
    good: Color32::from_rgb(22, 163, 74),
    rounding: 6.0,
    dense: false,
};

/// Slate: dark neutral, minimal color.
pub const SLATE: Theme = Theme {
    id: "slate",
    name: "Slate",
    dark: true,
    bg: Color32::from_rgb(15, 23, 42),
    surface: Color32::from_rgb(30, 41, 59),
    surface_2: Color32::from_rgb(51, 65, 85),
    ink: Color32::from_rgb(241, 245, 249),
    ink_dim: Color32::from_rgb(148, 163, 184),
    accent: Color32::from_rgb(100, 200, 255), // cyan
    accent_ink: Color32::from_rgb(15, 23, 42),
    border: Color32::from_rgb(71, 85, 105),
    error: Color32::from_rgb(239, 68, 68),
    warn: Color32::from_rgb(245, 158, 11),
    good: Color32::from_rgb(34, 197, 94),
    rounding: 4.0,
    dense: false,
};

/// Ember: warm, sunset-inspired dark theme.
pub const EMBER: Theme = Theme {
    id: "ember",
    name: "Ember",
    dark: true,
    bg: Color32::from_rgb(28, 18, 12),
    surface: Color32::from_rgb(45, 28, 20),
    surface_2: Color32::from_rgb(60, 35, 25),
    ink: Color32::from_rgb(252, 230, 220),
    ink_dim: Color32::from_rgb(200, 160, 140),
    accent: Color32::from_rgb(255, 158, 61), // warm orange
    accent_ink: Color32::from_rgb(28, 18, 12),
    border: Color32::from_rgb(100, 60, 40),
    error: Color32::from_rgb(252, 92, 92),
    warn: Color32::from_rgb(255, 180, 80),
    good: Color32::from_rgb(120, 230, 150),
    rounding: 8.0,
    dense: false,
};

/// San Andreas: GTA:SA inspired — green + black.
pub const SAN_ANDREAS: Theme = Theme {
    id: "san_andreas",
    name: "San Andreas",
    dark: true,
    bg: Color32::from_rgb(5, 10, 5),
    surface: Color32::from_rgb(15, 30, 15),
    surface_2: Color32::from_rgb(25, 50, 25),
    ink: Color32::from_rgb(200, 255, 150),
    ink_dim: Color32::from_rgb(120, 180, 100),
    accent: Color32::from_rgb(0, 255, 100), // neon green
    accent_ink: Color32::from_rgb(5, 10, 5),
    border: Color32::from_rgb(60, 120, 60),
    error: Color32::from_rgb(255, 100, 100),
    warn: Color32::from_rgb(255, 200, 50),
    good: Color32::from_rgb(100, 255, 150),
    rounding: 3.0,
    dense: false,
};

/// Frost: cool, blue-grey minimalist.
pub const FROST: Theme = Theme {
    id: "frost",
    name: "Frost",
    dark: true,
    bg: Color32::from_rgb(14, 28, 35),
    surface: Color32::from_rgb(25, 45, 58),
    surface_2: Color32::from_rgb(40, 65, 80),
    ink: Color32::from_rgb(232, 244, 252),
    ink_dim: Color32::from_rgb(158, 188, 212),
    accent: Color32::from_rgb(91, 198, 255), // sky blue
    accent_ink: Color32::from_rgb(14, 28, 35),
    border: Color32::from_rgb(80, 120, 150),
    error: Color32::from_rgb(255, 120, 120),
    warn: Color32::from_rgb(255, 180, 100),
    good: Color32::from_rgb(120, 255, 180),
    rounding: 5.0,
    dense: false,
};

/// Contrast: high-contrast light theme for a11y.
pub const CONTRAST: Theme = Theme {
    id: "contrast",
    name: "Contrast",
    dark: false,
    bg: Color32::from_rgb(255, 255, 255),
    surface: Color32::from_rgb(250, 250, 250),
    surface_2: Color32::from_rgb(240, 240, 240),
    ink: Color32::from_rgb(0, 0, 0),
    ink_dim: Color32::from_rgb(64, 64, 64),
    accent: Color32::from_rgb(0, 0, 0), // pure black for max contrast
    accent_ink: Color32::from_rgb(255, 255, 255),
    border: Color32::from_rgb(0, 0, 0),
    error: Color32::from_rgb(200, 0, 0),
    warn: Color32::from_rgb(150, 75, 0),
    good: Color32::from_rgb(0, 120, 0),
    rounding: 2.0,
    dense: false,
};

static BUILTINS: [Theme; 7] = [MIDNIGHT, PAPER, SLATE, EMBER, SAN_ANDREAS, FROST, CONTRAST];

/// All builtin themes, in presentation order.
pub fn builtins() -> &'static [Theme] {
    &BUILTINS
}

/// Look up a theme by id, or return the first builtin (Midnight) if not found.
pub fn by_id(id: &str) -> &'static Theme {
    BUILTINS
        .iter()
        .find(|t| t.id == id)
        .unwrap_or(&BUILTINS[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_themes_have_distinct_ids() {
        let ids: Vec<_> = builtins().iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), ids.iter().collect::<std::collections::HashSet<_>>().len());
    }

    #[test]
    fn by_id_finds_all_themes() {
        for theme in builtins() {
            assert_eq!(by_id(theme.id).id, theme.id);
        }
    }

    #[test]
    fn by_id_returns_midnight_for_unknown() {
        assert_eq!(by_id("unknown").id, "midnight");
    }

    #[test]
    fn contrast_luminance_acceptable() {
        // Simple check: ink (black) should contrast well with bg (white)
        let theme = &CONTRAST;
        let ink_lum = relative_luminance(theme.ink);
        let bg_lum = relative_luminance(theme.bg);
        let ratio = (bg_lum + 0.05) / (ink_lum + 0.05); // WCAG contrast formula
        assert!(ratio >= 4.5, "Contrast theme fails a11y: ratio={}", ratio);
    }

    fn relative_luminance(c: Color32) -> f32 {
        let [r, g, b, _] = c.to_normalized_gamma_f32();
        let r = if r <= 0.03928 {
            r / 12.92
        } else {
            ((r + 0.055) / 1.055).powf(2.4)
        };
        let g = if g <= 0.03928 {
            g / 12.92
        } else {
            ((g + 0.055) / 1.055).powf(2.4)
        };
        let b = if b <= 0.03928 {
            b / 12.92
        } else {
            ((b + 0.055) / 1.055).powf(2.4)
        };
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
}
