// screenies-native — 2.0 shell entry point (egui/eframe, pure Rust, no webview).
//
// Boots a window with a landing menu that routes to the SSRP Editor, the
// Chatlog Parser, and the Gallery. All real work lives in screenies-core, so
// these screens are thin views over the same engine the Tauri app uses.
//
// Phase-0 scaffold: the menu + stub screens. Editor/parser/gallery are built
// out phase by phase (docs/2.0-MIGRATION.md). Not yet build-verified locally
// (needs egui's Linux GUI system-deps), but the egui API here is standard.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ScreeniesEditor",
        options,
        Box::new(|_cc| Ok(Box::<App>::default())),
    )
}

#[derive(Default, PartialEq, Eq)]
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Back bar (hidden on the menu itself).
        if self.screen != Screen::Menu {
            egui::TopBottomPanel::top("nav").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("← Menu").clicked() {
                        self.screen = Screen::Menu;
                    }
                    ui.heading(self.screen_title());
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Menu => self.menu(ui),
            Screen::Editor => stub(ui, "SSRP Editor", "phase 2 — ports the current editor onto screenies-core"),
            Screen::ChatlogParser => stub(ui, "Chatlog Parser", "phase 3 — open a chatlog folder, search in-app"),
            Screen::Gallery => stub(ui, "Gallery", "phase 4 — browse exported SSRP photos"),
        });
    }
}

impl App {
    fn screen_title(&self) -> &'static str {
        match self.screen {
            Screen::Menu => "ScreeniesEditor",
            Screen::Editor => "SSRP Editor",
            Screen::ChatlogParser => "Chatlog Parser",
            Screen::Gallery => "Gallery",
        }
    }

    fn menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading(egui::RichText::new("ScreeniesEditor").size(34.0).strong());
            ui.label("Screenshot Roleplay toolkit — SA-MP community");
            ui.add_space(28.0);

            let tile = |ui: &mut egui::Ui, title: &str, subtitle: &str| -> bool {
                let resp = ui.add_sized(
                    [420.0, 64.0],
                    egui::Button::new(format!("{title}\n{subtitle}")),
                );
                ui.add_space(12.0);
                resp.clicked()
            };

            if tile(ui, "SSRP Editor", "Crop · chatlog · filters · export") {
                self.screen = Screen::Editor;
            }
            if tile(ui, "Chatlog Parser", "Open a chatlog folder · search in-app") {
                self.screen = Screen::ChatlogParser;
            }
            if tile(ui, "Gallery", "Browse your exported SSRP photos") {
                self.screen = Screen::Gallery;
            }

            ui.add_space(24.0);
            ui.small(format!("v{} · native (egui)", env!("CARGO_PKG_VERSION")));
        });
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
