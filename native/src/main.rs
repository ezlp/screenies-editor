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

use eframe::egui;
use editor::EditorState;
use gallery::GalleryState;
use i18n::{t, Lang};
use screenies_core::render::FilterValues;

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
    dark: bool,
    lang: Lang,
    ui_scale: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            editor: EditorState::default(),
            gallery: GalleryState::default(),
            dark: true,
            lang: Lang::default(),
            ui_scale: 1.0,
        }
    }
}

/// Persisted between launches via eframe storage.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    dark: bool,
    font: String,
    lang: Lang,
    text_size: f32,
    line_gap: f32,
    filters: FilterValues,
    ui_scale: f32,
    chatlog_folder: Option<String>,
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
                app.dark = s.dark;
                app.lang = s.lang;
                app.ui_scale = s.ui_scale.clamp(0.7, 1.6);
                app.editor.set_font(s.font);
                app.editor.apply_prefs(s.text_size, s.line_gap, s.filters);
                app.editor.set_chatlog_folder(s.chatlog_folder);
            }
        }
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(if self.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        // "Resize the editing space" — scale the whole UI (persisted).
        ctx.set_zoom_factor(self.ui_scale);

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
                        if ui.button(if self.dark { "☀" } else { "🌙" }).on_hover_text(t(self.lang, "Ganti tema")).clicked() {
                            self.dark = !self.dark;
                        }
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
            Screen::Settings => self.settings_screen(ui),
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
            dark: self.dark,
            font: self.editor.font().to_string(),
            lang: self.lang,
            text_size,
            line_gap,
            filters,
            ui_scale: self.ui_scale,
            chatlog_folder: self.editor.chatlog_folder(),
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
            ui.horizontal(|ui| {
                let theme = if self.dark { "☀  Mode terang" } else { "🌙  Mode gelap" };
                if ui.button(t(lang, theme)).clicked() {
                    self.dark = !self.dark;
                }
                if ui.button(lang_label(lang)).on_hover_text("ID / EN").clicked() {
                    self.lang = self.lang.toggled();
                }
            });

            ui.add_space(16.0);
            ui.small(format!("v{} · native (egui) · preview", env!("CARGO_PKG_VERSION")));
        });
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.add_space(12.0);
        ui.heading(t(lang, "Pengaturan"));
        ui.add_space(12.0);
        ui.label(t(lang, "Ukuran ruang edit"));
        ui.add(egui::Slider::new(&mut self.ui_scale, 0.7..=1.6).text("×"));
        ui.small(t(lang, "Perbesar/perkecil seluruh tampilan aplikasi."));
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            let theme = if self.dark { "☀  Mode terang" } else { "🌙  Mode gelap" };
            if ui.button(t(lang, theme)).clicked() {
                self.dark = !self.dark;
            }
            if ui.button(lang_label(lang)).on_hover_text("ID / EN").clicked() {
                self.lang = self.lang.toggled();
            }
        });
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
