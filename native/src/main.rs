// screenies-native — 2.0 shell entry point (egui/eframe, pure Rust, no webview).
//
// Landing menu → SSRP Editor (a working preview with the basic 1.x features),
// plus Chatlog Parser / Gallery stubs for later phases. All real work lives in
// screenies-core (the shell-independent engine).
//
// Not yet build-verified locally (needs egui's Linux GUI system-deps), but the
// egui API used here is standard 0.29.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chatlog_browser;
mod editor;
mod gallery;
mod i18n;
mod theme;

use eframe::egui;
use editor::EditorState;
use gallery::GalleryState;
use i18n::{t, Lang};
use screenies_core::render::FilterValues;
use theme::Theme;

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 760.0])
        .with_min_inner_size([880.0, 580.0])
        .with_title("ScreeniesEditor");
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")) {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "ScreeniesEditor",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Screen {
    #[default]
    Menu,
    Editor,
    Gallery,
    Settings,
}

struct App {
    screen: Screen,
    editor: EditorState,
    gallery: GalleryState,
    lang: Lang,
    ui_scale: f32,
    /// Current theme id (e.g., "midnight", "paper").
    theme_id: String,
    /// Optional custom accent override.
    accent: Option<egui::Color32>,
    /// Density toggle: true = compact, false = cozy.
    dense: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            editor: EditorState::default(),
            gallery: GalleryState::default(),
            lang: Lang::default(),
            ui_scale: 1.0,
            theme_id: "midnight".into(),
            accent: None,
            dense: false,
        }
    }
}

/// Persisted between launches via eframe storage.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    /// Legacy light/dark toggle (migrated to theme in App::new).
    dark: bool,
    font: String,
    lang: Lang,
    text_size: f32,
    line_gap: f32,
    filters: FilterValues,
    ui_scale: f32,
    chatlog_folder: Option<String>,
    /// Theme id (e.g., "midnight", "paper"). Empty → migrate from dark.
    #[serde(default)]
    theme: String,
    /// Optional custom accent override as [R, G, B].
    #[serde(default)]
    accent: Option<[u8; 3]>,
    /// Density toggle: true = compact, false = cozy.
    #[serde(default)]
    dense: bool,
    /// Gallery folder path (persisted for Phase D).
    #[serde(default)]
    gallery_folder: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark: true,
            font: "Verdana".into(),
            lang: Lang::Id,
            text_size: 27.0,
            line_gap: 122.0,
            filters: default_filters(),
            ui_scale: 1.0,
            chatlog_folder: None,
            theme: String::new(),
            accent: None,
            dense: false,
            gallery_folder: None,
        }
    }
}

fn default_filters() -> FilterValues {
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

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = App::default();
        if let Some(storage) = cc.storage {
            if let Some(s) = eframe::get_value::<Settings>(storage, "settings") {
                app.lang = s.lang;
                app.ui_scale = s.ui_scale.clamp(0.7, 1.6);
                app.dense = s.dense;

                // Migrate from dark bool to theme id.
                app.theme_id = if s.theme.is_empty() {
                    if s.dark { "midnight" } else { "paper" }.into()
                } else {
                    s.theme.clone()
                };

                // Restore accent override if set.
                app.accent = s.accent.map(|[r, g, b]| egui::Color32::from_rgb(r, g, b));

                app.editor.set_font(s.font);
                app.editor.apply_prefs(s.text_size, s.line_gap, s.filters);
                app.editor.set_chatlog_folder(s.chatlog_folder);
                app.gallery.set_gallery_folder(s.gallery_folder);
            }
        }
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply the current theme.
        let theme_obj = theme::by_id(&self.theme_id).clone();
        theme_obj.apply(ctx, self.accent, self.ui_scale);

        // Propagate the current language to each screen before it draws.
        self.editor.lang = self.lang;
        self.gallery.lang = self.lang;

        if self.screen != Screen::Menu {
            egui::TopBottomPanel::top("nav").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("←  Menu").clicked() {
                        self.screen = Screen::Menu;
                    }
                    ui.separator();
                    ui.heading(title_of(self.screen));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(lang_label(self.lang)).on_hover_text("ID / EN").clicked() {
                            self.lang = self.lang.toggled();
                        }
                    });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Menu => self.menu(ui),
            Screen::Editor => self.editor.ui(ui),
            Screen::Gallery => self.gallery.ui(ui),
            Screen::Settings => self.settings_screen(ui, &theme_obj),
        });

        // Gallery → "Buka di editor": load the photo and jump to the editor.
        if let Some(path) = self.gallery.open_request.take() {
            self.editor.load_photo_path(&path);
            self.screen = Screen::Editor;
        }

        // Drag-and-drop a photo anywhere → load it into the editor.
        let dropped = ctx.input(|i| i.raw.dropped_files.iter().find_map(|f| f.path.clone()));
        if let Some(path) = dropped {
            if screenies_core::gallery::is_image(&path) {
                self.editor.load_photo_path(&path);
                self.screen = Screen::Editor;
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let (text_size, line_gap, filters) = self.editor.prefs();
        let s = Settings {
            dark: false, // legacy, not used by new code but kept for compat
            font: self.editor.font().to_string(),
            lang: self.lang,
            text_size,
            line_gap,
            filters,
            ui_scale: self.ui_scale,
            chatlog_folder: self.editor.chatlog_folder(),
            theme: self.theme_id.clone(),
            accent: self.accent.map(|c| {
                let rgba = c.to_array();
                [rgba[0], rgba[1], rgba[2]]
            }),
            dense: self.dense,
            gallery_folder: self.gallery.gallery_folder(),
        };
        eframe::set_value(storage, "settings", &s);
    }
}

impl App {
    fn menu(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading(egui::RichText::new("ScreeniesEditor").size(34.0).strong());
            ui.label(t(lang, "Screenshot Roleplay toolkit — komunitas SA-MP"));
            ui.add_space(28.0);

            if menu_tile(ui, "🖼  SSRP Editor", t(lang, "Crop · chatlog · filter · export")) {
                self.screen = Screen::Editor;
            }
            if menu_tile(ui, "🗂  Gallery (WIP)", t(lang, "Jelajahi foto SSRP hasil edit")) {
                self.screen = Screen::Gallery;
            }
            if menu_tile(ui, "⚙  Settings", t(lang, "Bahasa · tema · ukuran ruang edit")) {
                self.screen = Screen::Settings;
            }

            ui.add_space(16.0);
            ui.small(format!("v{} · native (egui) · preview", env!("CARGO_PKG_VERSION")));
        });
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui, theme_obj: &Theme) {
        let lang = self.lang;
        ui.add_space(12.0);
        ui.heading(t(lang, "Pengaturan"));
        ui.add_space(12.0);

        // Appearance section
        ui.label(t(lang, "Penampilan"));
        ui.separator();

        // Theme grid (7 swatches)
        ui.label(t(lang, "Tema"));
        ui.horizontal_wrapped(|ui| {
            for builtin in theme::builtins() {
                let is_active = self.theme_id == builtin.id;
                let btn_color = builtin.accent;
                let border_stroke = if is_active {
                    egui::Stroke::new(2.0_f32, ui.visuals().selection.stroke.color)
                } else {
                    egui::Stroke::new(1.0_f32, ui.visuals().window_stroke.color)
                };
                let rect_response = ui.add(
                    egui::Button::new("  ")
                        .fill(btn_color)
                        .stroke(border_stroke)
                        .min_size(egui::Vec2::splat(40.0)),
                );
                if rect_response.clicked() {
                    self.theme_id = builtin.id.to_string();
                }
                rect_response.on_hover_text(builtin.name);
            }
        });
        ui.add_space(8.0);

        // Accent color picker
        ui.label(t(lang, "Accent color"));
        let mut accent_rgb = self.accent
            .map(|c| {
                let rgba = c.to_array();
                [rgba[0], rgba[1], rgba[2]]
            })
            .unwrap_or_else(|| {
                let rgba = theme_obj.accent.to_array();
                [rgba[0], rgba[1], rgba[2]]
            });
        if ui.color_edit_button_srgb(&mut accent_rgb).changed() {
            self.accent = Some(egui::Color32::from_rgb(accent_rgb[0], accent_rgb[1], accent_rgb[2]));
        }
        ui.add_space(8.0);

        // Density toggle
        ui.label(t(lang, "Kepadatan UI"));
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.dense, false, t(lang, "Nyaman"));
            ui.selectable_value(&mut self.dense, true, t(lang, "Kompak"));
        });
        ui.add_space(12.0);

        // Editing defaults section
        ui.label(t(lang, "Default edit"));
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label(t(lang, "Font"));
            ui.label(self.editor.font());
        });
        ui.add_space(8.0);

        // UI scale
        ui.label(t(lang, "Ukuran ruang edit"));
        ui.add(egui::Slider::new(&mut self.ui_scale, 0.7..=1.6).text("×"));
        ui.add_space(12.0);

        // Language section
        ui.label(t(lang, "Bahasa"));
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.lang, Lang::Id, "Bahasa Indonesia");
            ui.selectable_value(&mut self.lang, Lang::En, "English");
        });
        ui.add_space(12.0);

        // About section
        ui.label(t(lang, "Tentang"));
        ui.separator();
        ui.small(format!("v{}", env!("CARGO_PKG_VERSION")));
        ui.small(t(lang, "Screenshot Roleplay toolkit — komunitas SA-MP"));
        ui.hyperlink_to(
            t(lang, "GitHub"),
            "https://github.com/ezlp/screenies-editor",
        );
        ui.small(format!("© 2024 Isut Indraputra & Claude (Anthropic)"));
    }
}

fn lang_label(l: Lang) -> &'static str {
    match l {
        Lang::Id => "ID",
        Lang::En => "EN",
    }
}

fn menu_tile(ui: &mut egui::Ui, title: &str, subtitle: &str) -> bool {
    let clicked = ui
        .add_sized([440.0, 62.0], egui::Button::new(title))
        .clicked();
    ui.small(subtitle);
    ui.add_space(12.0);
    clicked
}

fn title_of(s: Screen) -> &'static str {
    match s {
        Screen::Menu => "ScreeniesEditor",
        Screen::Editor => "SSRP Editor",
        Screen::Gallery => "Gallery",
        Screen::Settings => "Settings",
    }
}
