// screenies-native — 2.0 shell entry point (egui/eframe, pure Rust, no webview).
//
// Landing menu → SSRP Editor (a working preview with the basic 1.x features),
// plus Chatlog Parser / Gallery stubs for later phases. All real work lives in
// screenies-core, the same engine the Tauri app uses.
//
// Not yet build-verified locally (needs egui's Linux GUI system-deps), but the
// egui API used here is standard 0.29.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;

use eframe::egui;
use editor::EditorState;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([880.0, 580.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ScreeniesEditor",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
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

#[derive(Default)]
struct App {
    screen: Screen,
    editor: EditorState,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.screen != Screen::Menu {
            egui::TopBottomPanel::top("nav").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("←  Menu").clicked() {
                        self.screen = Screen::Menu;
                    }
                    ui.separator();
                    ui.heading(title_of(self.screen));
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Menu => self.menu(ui),
            Screen::Editor => self.editor.ui(ui),
            Screen::ChatlogParser => {
                stub(ui, "Chatlog Parser", "Fase 3 — buka folder chatlog, cari di aplikasi.")
            }
            Screen::Gallery => stub(ui, "Gallery", "Fase 4 — jelajahi foto SSRP hasil edit."),
        });
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

            ui.add_space(24.0);
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
