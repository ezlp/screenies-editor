// screenies-native — 2.0 shell entry point (egui/eframe, pure Rust, no webview).
//
// Landing menu → SSRP Editor (a working preview with the basic 1.x features),
// plus Chatlog Parser / Gallery stubs for later phases. All real work lives in
// screenies-core (the shell-independent engine).
//
// Not yet build-verified locally (needs egui's Linux GUI system-deps), but the
// egui API used here is standard 0.29.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chatlog_parser;
mod editor;
mod gallery;

use chatlog_parser::ChatlogParserState;
use eframe::egui;
use editor::EditorState;
use gallery::GalleryState;

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
    ChatlogParser,
    Gallery,
}

struct App {
    screen: Screen,
    editor: EditorState,
    chatlog_parser: ChatlogParserState,
    gallery: GalleryState,
    dark: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            editor: EditorState::default(),
            chatlog_parser: ChatlogParserState::default(),
            gallery: GalleryState::default(),
            dark: true,
        }
    }
}

/// Persisted between launches via eframe storage.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Settings {
    dark: bool,
    font: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { dark: true, font: "Verdana".into() }
    }
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = App::default();
        if let Some(storage) = cc.storage {
            if let Some(s) = eframe::get_value::<Settings>(storage, "settings") {
                app.dark = s.dark;
                app.editor.set_font(s.font);
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

        if self.screen != Screen::Menu {
            egui::TopBottomPanel::top("nav").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("←  Menu").clicked() {
                        self.screen = Screen::Menu;
                    }
                    ui.separator();
                    ui.heading(title_of(self.screen));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(if self.dark { "☀" } else { "🌙" }).on_hover_text("Ganti tema").clicked() {
                            self.dark = !self.dark;
                        }
                    });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Menu => self.menu(ui),
            Screen::Editor => self.editor.ui(ui),
            Screen::ChatlogParser => self.chatlog_parser.ui(ui),
            Screen::Gallery => self.gallery.ui(ui),
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
        let s = Settings { dark: self.dark, font: self.editor.font().to_string() };
        eframe::set_value(storage, "settings", &s);
    }
}

impl App {
    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading(egui::RichText::new("ScreeniesEditor").size(34.0).strong());
            ui.label("Screenshot Roleplay toolkit — komunitas SA-MP");
            ui.add_space(28.0);

            if menu_tile(ui, "🖼  SSRP Editor", "Crop · chatlog · filter · export") {
                self.screen = Screen::Editor;
            }
            if menu_tile(ui, "🔍  Chatlog Parser", "Buka folder chatlog · cari di aplikasi") {
                self.screen = Screen::ChatlogParser;
            }
            if menu_tile(ui, "🗂  Gallery", "Jelajahi foto SSRP hasil edit") {
                self.screen = Screen::Gallery;
            }

            ui.add_space(16.0);
            if ui
                .button(if self.dark { "☀  Mode terang" } else { "🌙  Mode gelap" })
                .clicked()
            {
                self.dark = !self.dark;
            }

            ui.add_space(16.0);
            ui.small(format!("v{} · native (egui) · preview", env!("CARGO_PKG_VERSION")));
        });
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
        Screen::ChatlogParser => "Chatlog Parser",
        Screen::Gallery => "Gallery",
    }
}

fn stub(ui: &mut egui::Ui, title: &str, note: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.heading(title);
        ui.add_space(8.0);
        ui.label(egui::RichText::new(note).weak());
    });
}
