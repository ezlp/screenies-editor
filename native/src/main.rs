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
mod icons;
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

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
enum Tool {
    Photo,
    Crop,
    Chatlog,
    Text,
    Fx,
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Photo
    }
}

impl Tool {
    fn id(self) -> &'static str {
        match self {
            Tool::Photo => "photo",
            Tool::Crop => "crop",
            Tool::Chatlog => "chatlog",
            Tool::Text => "text",
            Tool::Fx => "fx",
        }
    }

    fn from_id(s: &str) -> Self {
        match s {
            "crop" => Tool::Crop,
            "chatlog" => Tool::Chatlog,
            "text" => Tool::Text,
            "fx" => Tool::Fx,
            _ => Tool::Photo,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn tool_from_id_roundtrip() {
        assert_eq!(Tool::from_id("photo"), Tool::Photo);
        assert_eq!(Tool::from_id("crop"), Tool::Crop);
        assert_eq!(Tool::from_id("chatlog"), Tool::Chatlog);
        assert_eq!(Tool::from_id("text"), Tool::Text);
        assert_eq!(Tool::from_id("fx"), Tool::Fx);
        assert_eq!(Tool::from_id("unknown"), Tool::Photo);
    }

    #[test]
    fn settings_serialize_active_tool() {
        let s = Settings { active_tool: "crop".into(), ..Settings::default() };
        let js = serde_json::to_string(&s).expect("serialize");
        let des: Settings = serde_json::from_str(&js).expect("deserialize");
        assert_eq!(des.active_tool, "crop");
    }
}


struct App {
    screen: Screen,
    editor: EditorState,
    gallery: GalleryState,
    lang: Lang,
    ui_scale: f32,
    /// Current tool (persisted in settings)
    active_tool: Tool,
    /// Current theme id (e.g., "midnight", "paper").
    theme_id: String,
    /// Optional custom accent override.
    accent: Option<egui::Color32>,
    /// Density toggle: true = compact, false = cozy.
    dense: bool,
    source_shots_folder: Option<String>,
    imgbb_api_key: Option<String>,
    unified_layout: bool,
    hotkeys: std::collections::HashMap<String, String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::default(),
            editor: EditorState::default(),
            gallery: GalleryState::default(),
            lang: Lang::default(),
            ui_scale: 1.0,
            active_tool: Tool::default(),
            theme_id: "midnight".into(),
            accent: None,
            dense: false,
            source_shots_folder: None,
            imgbb_api_key: None,
            unified_layout: false,
            hotkeys: default_hotkeys(),
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
    #[serde(default = "default_true")]
    stroke_auto: bool,
    #[serde(default = "default_stroke_width")]
    stroke_width: f32,
    #[serde(default)]
    cinematic: bool,
    #[serde(default = "default_cinematic_bar")]
    cinematic_bar: f32,
    #[serde(default)]
    cinematic_bar_pos: screenies_core::render::BarPos,
    #[serde(default)]
    cinematic_color: [u8; 3],
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
    /// Active tool id (photo/crop/chatlog/text/fx)
    #[serde(default)]
    active_tool: String,
    /// Last folders used by the photo picker and PNG exporter.
    #[serde(default)]
    last_open_folder: Option<String>,
    #[serde(default)]
    last_save_folder: Option<String>,
    #[serde(default)]
    source_shots_folder: Option<String>,
    #[serde(default)]
    imgbb_api_key: Option<String>,
    #[serde(default)]
    unified_layout: bool,
    #[serde(default = "default_hotkeys")]
    hotkeys: std::collections::HashMap<String, String>,
}

fn default_true() -> bool { true }
fn default_stroke_width() -> f32 { 3.0 }
fn default_cinematic_bar() -> f32 { 12.0 }
fn default_hotkeys() -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("Open".to_string(), "Ctrl+O".to_string());
    map.insert("Paste".to_string(), "Ctrl+V".to_string());
    map.insert("Export".to_string(), "Ctrl+E".to_string());
    map.insert("Undo".to_string(), "Ctrl+Z".to_string());
    map.insert("Redo".to_string(), "Ctrl+Y".to_string());
    map.insert("Cinematic".to_string(), "Ctrl+M".to_string());
    map
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
            stroke_auto: true,
            stroke_width: 3.0,
            cinematic: false,
            cinematic_bar: 12.0,
            cinematic_bar_pos: screenies_core::render::BarPos::Both,
            cinematic_color: [0, 0, 0],
            ui_scale: 1.0,
            chatlog_folder: None,
            theme: String::new(),
            accent: None,
            dense: false,
            gallery_folder: None,
            active_tool: "photo".into(),
            last_open_folder: None,
            last_save_folder: None,
            source_shots_folder: None,
            imgbb_api_key: None,
            unified_layout: false,
            hotkeys: default_hotkeys(),
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
        // Load the custom icon font.
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "icons".into(),
            egui::FontData::from_static(include_bytes!("../assets/icons.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Name("icons".into()))
            .or_default()
            .insert(0, "icons".into());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("icons".into());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("icons".into());
        cc.egui_ctx.set_fonts(fonts);

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

                // Restore active tool if present
                app.active_tool = Tool::from_id(&s.active_tool);

                app.editor.set_font(s.font);
                app.editor.apply_prefs(
                    s.text_size, s.line_gap, s.filters, s.stroke_auto, s.stroke_width,
                    s.cinematic, s.cinematic_bar, s.cinematic_bar_pos, s.cinematic_color,
                );
                app.editor.set_chatlog_folder(s.chatlog_folder);
                app.editor.set_file_folders(s.last_open_folder, s.last_save_folder);
                app.gallery.set_gallery_folder(s.gallery_folder);
                app.source_shots_folder = s.source_shots_folder;
                app.imgbb_api_key = s.imgbb_api_key;
                app.unified_layout = s.unified_layout;
                for (k, v) in s.hotkeys {
                    app.hotkeys.insert(k, v);
                }
            }
        }
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply the current theme.
        let theme_obj = theme::by_id(&self.theme_id).clone();
        theme_obj.apply(ctx, self.accent, self.ui_scale, self.dense);

        // Propagate the current language to each screen before it draws.
        self.editor.lang = self.lang;
        self.gallery.lang = self.lang;

        // Persistent Nav Rail (left side of window)
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(56.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(12.0);
                    
                    // Home/Menu
                    let is_home = self.screen == Screen::Menu;
                    let resp = ui.add_sized(
                        [40.0, 40.0],
                        egui::SelectableLabel::new(is_home, egui::RichText::new(icons::ICON_HOME).size(22.0)),
                    );
                    if resp.on_hover_text(t(self.lang, "Menu Utama")).clicked() {
                        self.screen = Screen::Menu;
                    }
                    ui.add_space(8.0);

                    // Editor
                    let is_editor = self.screen == Screen::Editor;
                    let resp = ui.add_sized(
                        [40.0, 40.0],
                        egui::SelectableLabel::new(is_editor, egui::RichText::new(icons::ICON_IMAGE).size(22.0)),
                    );
                    if resp.on_hover_text(t(self.lang, "SSRP Editor")).clicked() {
                        self.screen = Screen::Editor;
                    }
                    ui.add_space(8.0);

                    // Gallery
                    let is_gallery = self.screen == Screen::Gallery;
                    let resp = ui.add_sized(
                        [40.0, 40.0],
                        egui::SelectableLabel::new(is_gallery, egui::RichText::new(icons::ICON_FOLDER).size(22.0)),
                    );
                    if resp.on_hover_text(t(self.lang, "Gallery")).clicked() {
                        self.screen = Screen::Gallery;
                    }
                    ui.add_space(8.0);

                    // Settings
                    let is_settings = self.screen == Screen::Settings;
                    let resp = ui.add_sized(
                        [40.0, 40.0],
                        egui::SelectableLabel::new(is_settings, egui::RichText::new(icons::ICON_SETTINGS).size(22.0)),
                    );
                    if resp.on_hover_text(t(self.lang, "Pengaturan")).clicked() {
                        self.screen = Screen::Settings;
                    }

                    // Bottom items: quick theme cycle
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(12.0);
                        let resp = ui.add_sized(
                            [40.0, 40.0],
                            egui::Button::new(egui::RichText::new(icons::ICON_SPARKLES).size(20.0)),
                        );
                        if resp.on_hover_text(t(self.lang, "Ganti tema")).clicked() {
                            let current_idx = theme::builtins()
                                .iter()
                                .position(|t| t.id == self.theme_id)
                                .unwrap_or(0);
                            let next_idx = (current_idx + 1) % theme::builtins().len();
                            self.theme_id = theme::builtins()[next_idx].id.to_string();
                        }
                    });
                });
            });

        // Sync active tool into the editor before drawing, and read it back
        // afterwards so clicks inside the editor update App state.
        self.editor.active_tool = self.active_tool;
        egui::CentralPanel::default().show(ctx, |ui| match self.screen {
            Screen::Menu => self.menu(ui),
            Screen::Editor => self.editor.ui(ui),
            Screen::Gallery => self.gallery.ui(ui),
            Screen::Settings => self.settings_screen(ui, &theme_obj),
        });
        // read back the active tool (user may have clicked the rail)
        self.active_tool = self.editor.active_tool;

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
        let prefs = self.editor.prefs();
        let s = Settings {
            dark: false, // legacy, not used by new code but kept for compat
            font: self.editor.font().to_string(),
            lang: self.lang,
            text_size: prefs.text_size,
            line_gap: prefs.line_gap,
            filters: prefs.filters,
            stroke_auto: prefs.stroke_auto,
            stroke_width: prefs.stroke_width,
            cinematic: prefs.cinematic,
            cinematic_bar: prefs.cinematic_bar,
            cinematic_bar_pos: prefs.cinematic_bar_pos,
            cinematic_color: prefs.cinematic_color,
            ui_scale: self.ui_scale,
            chatlog_folder: self.editor.chatlog_folder(),
            theme: self.theme_id.clone(),
            accent: self.accent.map(|c| {
                let rgba = c.to_array();
                [rgba[0], rgba[1], rgba[2]]
            }),
            dense: self.dense,
            gallery_folder: self.gallery.gallery_folder(),
            active_tool: self.active_tool.id().to_string(),
            last_open_folder: self.editor.last_open_folder(),
            last_save_folder: self.editor.last_save_folder(),
            source_shots_folder: self.source_shots_folder.clone(),
            imgbb_api_key: self.imgbb_api_key.clone(),
            unified_layout: self.unified_layout,
            hotkeys: self.hotkeys.clone(),
        };
        eframe::set_value(storage, "settings", &s);
    }
}

impl App {
    fn menu(&mut self, ui: &mut egui::Ui) {
        let lang = self.lang;
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            
            // Header / Title
            ui.heading(egui::RichText::new("ScreeniesEditor").size(36.0).strong());
            ui.label(t(lang, "Screenshot Roleplay toolkit — komunitas SA-MP"));
            ui.add_space(32.0);

            // Row of Cards
            ui.horizontal(|ui| {
                let card_w = 240.0;
                let card_h = 160.0;
                let num_cards = 3;
                let total_w = card_w * num_cards as f32 + ui.spacing().item_spacing.x * (num_cards - 1) as f32;
                let avail_w = ui.available_width();
                if avail_w > total_w {
                    ui.add_space((avail_w - total_w) / 2.0);
                }

                // Editor Card
                if entry_card(
                    ui,
                    icons::ICON_IMAGE,
                    t(lang, "SSRP Editor"),
                    t(lang, "Crop · chatlog · filter · export"),
                    card_w,
                    card_h,
                ).clicked() {
                    self.screen = Screen::Editor;
                }

                // Gallery Card
                if entry_card(
                    ui,
                    icons::ICON_FOLDER,
                    t(lang, "Gallery"),
                    t(lang, "Jelajahi foto SSRP hasil edit"),
                    card_w,
                    card_h,
                ).clicked() {
                    self.screen = Screen::Gallery;
                }

                // Settings Card
                if entry_card(
                    ui,
                    icons::ICON_SETTINGS,
                    t(lang, "Settings"),
                    t(lang, "Bahasa · tema · ukuran ruang edit"),
                    card_w,
                    card_h,
                ).clicked() {
                    self.screen = Screen::Settings;
                }
            });

            // Recent shots strip
            if let Some(_folder) = self.gallery.gallery_folder() {
                if !self.gallery.items.is_empty() {
                    ui.add_space(32.0);
                    ui.label(egui::RichText::new(t(lang, "Hasil Edit Terbaru")).strong().size(18.0));
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        let limit = self.gallery.items.len().min(5);
                        let thumb_w = 120.0;
                        let thumb_h = 80.0;
                        let total_thumbs_w = thumb_w * limit as f32 + ui.spacing().item_spacing.x * (limit - 1) as f32;
                        let avail_w = ui.available_width();
                        if avail_w > total_thumbs_w {
                            ui.add_space((avail_w - total_thumbs_w) / 2.0);
                        }

                        for i in 0..limit {
                            let path = self.gallery.items[i].path.clone();
                            if !self.gallery.thumbs.contains_key(&path) {
                                self.gallery.load_thumb(ui.ctx(), &path);
                            }

                            let clicked = match self.gallery.thumbs.get(&path) {
                                Some(Some(tex)) => {
                                    let s = tex.size_vec2();
                                    let scale = (thumb_w / s.x).min(thumb_h / s.y).min(1.0);
                                    let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), s * scale));
                                    ui.add(egui::ImageButton::new(img))
                                        .on_hover_text(&self.gallery.items[i].name)
                                        .clicked()
                                }
                                _ => ui.add_sized([thumb_w, thumb_h], egui::Button::new("…")).clicked(),
                            };

                            if clicked {
                                self.editor.load_photo_path(&path);
                                self.screen = Screen::Editor;
                            }
                        }
                    });
                }
            }

            ui.add_space(40.0);
            
            // Footer
            ui.small(format!("v{} · native (egui) · stable", env!("CARGO_PKG_VERSION")));
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
        ui.small(t(lang, "Pilih skema warna dasar untuk seluruh antarmuka aplikasi."));
        ui.add_space(4.0);
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
        ui.small(t(lang, "Sesuaikan warna penekanan untuk tombol, seleksi, dan tautan."));
        ui.add_space(4.0);
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
        ui.small(t(lang, "Sesuaikan jarak antar elemen UI (Nyaman atau Kompak)."));
        ui.add_space(4.0);
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
        ui.small(t(lang, "Skala perbesaran antarmuka editor dan teks."));
        ui.add_space(4.0);
        ui.add(egui::Slider::new(&mut self.ui_scale, 0.7..=1.6).text("×"));
        ui.add_space(12.0);

        // Language section
        ui.label(t(lang, "Bahasa"));
        ui.small(t(lang, "Ganti bahasa pelokalan aplikasi."));
        ui.add_space(4.0);
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

fn entry_card(ui: &mut egui::Ui, icon: &str, title: &str, sub: &str, width: f32, height: f32) -> egui::Response {
    let margin = 12.0;
    let frame = egui::Frame::none()
        .fill(ui.visuals().faint_bg_color)
        .rounding(ui.visuals().widgets.inactive.rounding)
        .inner_margin(margin);
        
    let response = frame.show(ui, |ui| {
        ui.set_width(width);
        ui.set_height(height);
        ui.vertical_centered(|ui| {
            ui.add_space(6.0);
            ui.label(egui::RichText::new(icon).size(32.0).color(ui.visuals().selection.stroke.color));
            ui.add_space(8.0);
            ui.heading(title);
            ui.add_space(4.0);
            ui.small(sub);
            ui.add_space(6.0);
        });
    }).response;

    let response = ui.interact(response.rect, response.id, egui::Sense::click());
    if response.hovered() {
        ui.painter().rect_stroke(
            response.rect,
            ui.visuals().widgets.inactive.rounding,
            egui::Stroke::new(1.5_f32, ui.visuals().selection.stroke.color),
        );
    } else {
        ui.painter().rect_stroke(
            response.rect,
            ui.visuals().widgets.inactive.rounding,
            egui::Stroke::new(1.0_f32, ui.visuals().widgets.inactive.bg_stroke.color),
        );
    }
    response
}
