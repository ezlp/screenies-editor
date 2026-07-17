// editor.rs — the SSRP editor screen (2.0 preview / MVP).
//
// Basic parity with 1.x: load a photo, paste a chatlog, tweak text + filters,
// see a LIVE preview, and export a PNG. The preview and the export are the
// SAME core::compose::render call, so what you see is what you save.
//
// Deliberately NOT in this MVP (they land in later phases): crop editor,
// stickers, color palette, undo/redo, multi-block, i18n, settings persistence.

use base64::Engine;
use eframe::egui;
use std::sync::Arc;
use screenies_core::chatlog::{self, preset::ParsePreset};
use screenies_core::render::compose;
use screenies_core::render::layout::{self, Anchor, BgMode, LayoutBlock, LayoutParams};
use screenies_core::render::text::GlyphMeasure;
use screenies_core::render::{
    BarPos, Canvas, CensorKind, CensorRegion, CropRect, FilterValues, RenderJob, Size, StickerJob,
};

use crate::Tool;

/// A loaded photo: base64 for the render pipeline + its pixel dimensions.
#[derive(Clone)]
struct Photo {
    /// Encoded photo bytes, shared behind `Arc` so cloning the doc (tab switch)
    /// or building a per-frame RenderJob is a refcount bump, not a deep copy of
    /// several MB. Only `prepare_base`/export actually decode it.
    base64: Arc<str>,
    w: u32,
    h: u32,
}

/// One photo's full editable state (a "document"). Text controls (font, size,
/// spacing) stay global; everything else is per-photo so tabs are independent.
#[derive(Clone)]
struct Doc {
    photo: Option<Photo>,
    blocks: Vec<ChatBlock>,
    censors: Vec<CensorRegion>,
    stickers: Vec<Sticker>,
    crop: Option<CropRect>,
    crop_ratio: Option<f32>,
    output_override: Option<(u32, u32)>,
    filters: FilterValues,
}

impl Default for Doc {
    fn default() -> Self {
        Self {
            photo: None,
            blocks: vec![ChatBlock::new(0)],
            censors: Vec::new(),
            stickers: Vec::new(),
            crop: None,
            crop_ratio: None,
            output_override: None,
            filters: identity_filters(),
        }
    }
}

/// A placed sticker: image data + output-space rect. `aspect` (w/h) keeps
/// resize proportional. The pixels are composited by core into the preview.
#[derive(Clone, PartialEq)]
struct Sticker {
    base64: String,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    aspect: f32,
}

/// One chatlog block: its raw text + placement. Multiple blocks let you put
/// different chat at different positions (mirrors 1.x "+ Tambah Chatlog").
#[derive(Clone, PartialEq)]
struct ChatBlock {
    text: String,
    anchor: Anchor,
    bg_mode: BgMode,
    x: f32,
    y: f32,
}

impl ChatBlock {
    /// A fresh block, staggered by index so new ones don't stack exactly.
    fn new(index: usize) -> Self {
        Self {
            text: String::new(),
            anchor: Anchor::Free,
            bg_mode: BgMode::None,
            x: 40.0,
            y: 40.0 + 70.0 * index as f32,
        }
    }
}

/// One undo step: the editable content (photo/textures/selection excluded).
#[derive(Clone, PartialEq)]
struct Snapshot {
    blocks: Vec<ChatBlock>,
    censors: Vec<CensorRegion>,
    stickers: Vec<Sticker>,
    crop: Option<CropRect>,
    crop_ratio: Option<f32>,
    crop_fit: bool,
    filters: FilterValues,
    cinematic: bool,
    cinematic_bar: f32,
    cinematic_bar_pos: BarPos,
    cinematic_color: [u8; 3],
    font_family: String,
    text_size: f32,
    line_gap: f32,
    stroke_auto: bool,
    stroke_width: f32,
}

/// A cached preview base: the decoded/cropped/resized photo before any filters,
/// effects or text. Reused while only those params change; keyed on the inputs
/// that affect it (crop, output size, fit mode) and cleared when the photo changes.
struct BaseCache {
    crop: CropRect,
    out_w: u32,
    out_h: u32,
    fit: bool,
    img: image::RgbaImage,
}

/// A cached FILTERED base: the base image with filters + cinematic bars already
/// applied, before any stickers/text/censors. Reused while only those overlay
/// params change, so dragging a sticker, censor box or text block skips the
/// whole-image color pass. Keyed on the base inputs PLUS filters + canvas; sits
/// on top of `BaseCache` (a filtered miss rebuilds from the base, a base miss
/// rebuilds both).
struct FilteredCache {
    crop: CropRect,
    out_w: u32,
    out_h: u32,
    fit: bool,
    filters: FilterValues,
    canvas: Option<Canvas>,
    img: image::RgbaImage,
}

pub struct EditorState {
    preset: ParsePreset,
    photo: Option<Photo>,

    // Multiple photos as tabs. `docs[active]` is a stale mirror of the working
    // fields below; they're synced on switch/add. Text controls stay global.
    docs: Vec<Doc>,
    active: usize,

    // Chatlog blocks (each: text + anchor + bg + position). One is selected
    // for editing at a time.
    blocks: Vec<ChatBlock>,
    selected_block: usize,

    // Color palette: last non-empty text selection (block, start, end char),
    // + the custom-picker color.
    text_selection: Option<(usize, usize, usize)>,
    custom_color: [u8; 3],

    /// UI language, set by the App each frame.
    pub lang: crate::i18n::Lang,

    /// Chatlog-folder browser popup (grab chatlog text to paste).
    chatlog: crate::chatlog_browser::ChatlogBrowser,

    // Text controls (shared across blocks).
    font_family: String,
    /// Installed font families for the picker dropdown (populated lazily).
    font_list: Vec<String>,
    text_size: f32,
    line_gap: f32,
    stroke_auto: bool,
    stroke_width: f32,

    filters: FilterValues,

    // Cinematic mode: solid bars painted over the photo's top/bottom (global,
    // like the text controls; captured in undo snapshots).
    cinematic: bool,
    cinematic_bar: f32,      // bar height as % of output height (0 = no bars)
    cinematic_bar_pos: BarPos,
    cinematic_color: [u8; 3],

    // Local censor boxes (blur/pixelate a region), placed + resized on the preview.
    censors: Vec<CensorRegion>,
    selected_censor: Option<usize>,

    // Sticker overlays (PNG/WebP), placed + resized on the preview.
    stickers: Vec<Sticker>,
    selected_sticker: Option<usize>,

    // Crop / output. crop = None → whole photo. crop_ratio locks the aspect.
    // output_override forces a fixed output resolution (e.g. 800×600).
    crop: Option<CropRect>,
    crop_ratio: Option<f32>,
    output_override: Option<(u32, u32)>,
    /// Fit (global): keep the whole image (pad with bars) for fixed sizes instead
    /// of cropping to fill. Applies when output_override is set.
    crop_fit: bool,
    crop_editing: bool,
    source_img: Option<egui::ColorImage>, // decoded photo, for the crop-edit view
    source_tex: Option<egui::TextureHandle>,

    // Undo/redo: snapshots of the editable content.
    history: Vec<Snapshot>,
    future: Vec<Snapshot>,

    // Live preview: re-rendered only when `dirty`.
    dirty: bool,
    texture: Option<egui::TextureHandle>,
    error: Option<String>,
    /// Cached decoded/cropped/resized photo (before filters/text), reused across
    /// filter/effect/text changes so sliders don't re-decode + re-resize.
    base_cache: Option<BaseCache>,
    /// Cached filtered base (photo + filters + bars), reused across overlay-only
    /// edits so dragging a sticker/censor/text skips the whole-image filter pass.
    filtered_cache: Option<FilteredCache>,
    // Active tool for the editor UI (mirrors App.active_tool)
    pub active_tool: Tool,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            preset: chatlog::preset::jgrp(),
            photo: None,
            docs: vec![Doc::default()],
            active: 0,
            blocks: vec![ChatBlock::new(0)],
            selected_block: 0,
            text_selection: None,
            custom_color: [255, 255, 255],
            lang: crate::i18n::Lang::default(),
            chatlog: crate::chatlog_browser::ChatlogBrowser::default(),
            font_family: "Verdana".into(),
            font_list: Vec::new(),
            text_size: 27.0,
            line_gap: 122.0,
            stroke_auto: true,
            stroke_width: 3.0,
            filters: identity_filters(),
            cinematic: false,
            cinematic_bar: 12.0,
            cinematic_bar_pos: BarPos::Both,
            cinematic_color: [0, 0, 0],
            censors: Vec::new(),
            selected_censor: None,
            stickers: Vec::new(),
            selected_sticker: None,
            crop: None,
            crop_ratio: None,
            output_override: None,
            crop_fit: false,
            crop_editing: false,
            source_img: None,
            source_tex: None,
            history: Vec::new(),
            future: Vec::new(),
            dirty: false,
            texture: None,
            error: None,
            base_cache: None,
            filtered_cache: None,
            active_tool: Tool::default(),
        }
    }
}

impl EditorState {
    fn t(&self, s: &'static str) -> &'static str {
        crate::i18n::t(self.lang, s)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Keyboard handling — skip while a text field is focused so the
        // textarea's own editing keys win.
        let typing = ui.ctx().memory(|m| m.focused().is_some());
        if !typing {
            // Undo/Redo (Ctrl/Cmd+Z, Ctrl+Y)
            let (do_undo, do_redo) = ui.ctx().input(|i| {
                let cmd = i.modifiers.command;
                let undo = cmd && !i.modifiers.shift && i.key_pressed(egui::Key::Z);
                let redo = (cmd && i.key_pressed(egui::Key::Y))
                    || (cmd && i.modifiers.shift && i.key_pressed(egui::Key::Z));
                (undo, redo)
            });
            if do_undo {
                self.undo();
            }
            if do_redo {
                self.redo();
            }

            // Tool shortcuts: 1..5 select Photo, Crop, Chatlog, Text, Fx
            if let Some(key_tool) = ui.ctx().input(|i| {
                if i.key_pressed(egui::Key::Num1) { Some(Tool::Photo) }
                else if i.key_pressed(egui::Key::Num2) { Some(Tool::Crop) }
                else if i.key_pressed(egui::Key::Num3) { Some(Tool::Chatlog) }
                else if i.key_pressed(egui::Key::Num4) { Some(Tool::Text) }
                else if i.key_pressed(egui::Key::Num5) { Some(Tool::Fx) }
                else { None }
            }) {
                self.active_tool = key_tool;
            }
        }

        // Tool rail (left, 56px). Sits left of the controls panel.
        egui::SidePanel::left("tool_rail")
            .resizable(false)
            .default_width(56.0)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    // Photo
                    if ui
                        .add(egui::Button::new("🖼").min_size(egui::Vec2::splat(40.0)))
                        .on_hover_text(self.t("Foto")).clicked()
                    {
                        self.active_tool = Tool::Photo;
                    }
                    ui.add_space(6.0);
                    // Crop
                    if ui
                        .add(egui::Button::new("✂️").min_size(egui::Vec2::splat(40.0)))
                        .on_hover_text(self.t("Potong")).clicked()
                    {
                        self.active_tool = Tool::Crop;
                    }
                    ui.add_space(6.0);
                    // Chatlog
                    if ui
                        .add(egui::Button::new("💬").min_size(egui::Vec2::splat(40.0)))
                        .on_hover_text(self.t("Chatlog")).clicked()
                    {
                        self.active_tool = Tool::Chatlog;
                    }
                    ui.add_space(6.0);
                    // Text
                    if ui
                        .add(egui::Button::new("🔤").min_size(egui::Vec2::splat(40.0)))
                        .on_hover_text(self.t("Teks")).clicked()
                    {
                        self.active_tool = Tool::Text;
                    }
                    ui.add_space(6.0);
                    // Fx
                    if ui
                        .add(egui::Button::new("✨").min_size(egui::Vec2::splat(40.0)))
                        .on_hover_text(self.t("Efek")).clicked()
                    {
                        self.active_tool = Tool::Fx;
                    }
                });
            });

        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(320.0)
            .show_inside(ui, |ui| self.controls(ui));

        // Refresh the output texture from control edits BEFORE drawing the
        // preview. Skip while crop-editing — that view uses the source photo;
        // the full output re-render happens once you click "Selesai crop".
        if self.dirty && !self.crop_editing {
            self.refresh(ui.ctx());
        }

        egui::CentralPanel::default().show_inside(ui, |ui| self.preview(ui));

        // Preview drags (censor/crop) may have set dirty after the refresh
        // above — ask for one more frame so the change shows up.
        if self.dirty && !self.crop_editing {
            ui.ctx().request_repaint();
        }

        // Record an undo step once the edit settles (pointer released).
        self.maybe_commit(ui.ctx());

        // Chatlog-folder popup (overlay).
        self.chatlog.window(ui.ctx());
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        // Main scrollable content
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);

            // Dispatch to the active tool panel.
            match self.active_tool {
                Tool::Photo => self.tool_photo(ui),
                Tool::Crop => self.tool_crop(ui),
                Tool::Chatlog => self.tool_chatlog(ui),
                Tool::Text => self.tool_text(ui),
                Tool::Fx => self.tool_fx(ui),
            }
        });

        // Fixed action bar at bottom of the controls panel (Export + future actions)
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(self.t("💾  Export PNG")).clicked() {
                self.export();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(self.history.len() > 1, egui::Button::new("↩  Undo"))
                    .on_hover_text("Ctrl+Z")
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(!self.future.is_empty(), egui::Button::new("↪  Redo"))
                    .on_hover_text("Ctrl+Y")
                    .clicked()
                {
                    self.redo();
                }
            });
        });
    }

    // --- Tool panels (refactor of the original controls body) ---
    fn tool_photo(&mut self, ui: &mut egui::Ui) {
        // Photo tabs (multiple photos).
        ui.horizontal_wrapped(|ui| {
            for i in 0..self.docs.len() {
                if ui.selectable_label(self.active == i, format!("Foto {}", i + 1)).clicked() {
                    self.switch_doc(i);
                }
            }
            if ui.button("➕").on_hover_text(self.t("Tambah foto")).clicked() {
                self.add_doc();
            }
            if self.docs.len() > 1
                && ui.button("✕").on_hover_text(self.t("Tutup foto ini")).clicked()
            {
                self.close_doc(self.active);
            }
        });

        if ui.button(self.t("📂  Muat Foto")).clicked() {
            self.pick_photo();
        }
        if let Some(p) = &self.photo {
            ui.small(format!("{}×{} px", p.w, p.h));
        } else {
            ui.small(self.t("Belum ada foto."));
        }

        ui.separator();
        // Export lives on the Photo panel.
        if ui.button(self.t("💾  Export PNG")).clicked() {
            self.export();
        }
        if let Some(err) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, err);
        }
    }

    fn tool_crop(&mut self, ui: &mut egui::Ui) {
        ui.label(self.t("Crop / Resolusi"));
        ui.horizontal_wrapped(|ui| {
            if ui.button(self.t("Bebas")).clicked() {
                self.set_ratio(None);
            }
            if ui.button("1:1").clicked() {
                self.set_ratio(Some(1.0));
            }
            if ui.button("4:3").clicked() {
                self.set_ratio(Some(4.0 / 3.0));
            }
            if ui.button("16:9").clicked() {
                self.set_ratio(Some(16.0 / 9.0));
            }
            if ui.button("21:9").clicked() {
                self.set_ratio(Some(21.0 / 9.0));
            }
            if ui.button("800×600").clicked() {
                self.set_resolution(800, 600);
            }
        });
        ui.horizontal(|ui| {
            let (potong, muat) = (self.t("Potong"), self.t("Muat penuh"));
            let prev = self.crop_fit;
            ui.selectable_value(&mut self.crop_fit, false, potong);
            ui.selectable_value(&mut self.crop_fit, true, muat);
            if self.crop_fit != prev {
                if self.crop_fit {
                    self.crop_editing = false; // fit needs no crop step
                }
                self.dirty = true;
            }
        });
        if self.crop_fit {
            ui.small(self.t("Muat: simpan seluruh gambar + bar (ukuran tetap)."));
        }
        let crop_btn = self.t(if self.crop_editing { "✓ Selesai crop" } else { "✏ Edit crop" });
        if ui.add_enabled(self.photo.is_some(), egui::Button::new(crop_btn)).clicked() {
            self.toggle_crop_edit();
        }
        if self.crop.is_some() || self.output_override.is_some() || self.cinematic {
            let (ow, oh) = self.output_dims();
            ui.small(format!("Output: {}×{}", ow.round() as u32, oh.round() as u32));
        }
    }

    fn tool_chatlog(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Chatlog:");
            for i in 0..self.blocks.len() {
                if ui
                    .selectable_label(self.selected_block == i, format!("#{}", i + 1))
                    .clicked()
                {
                    self.selected_block = i;
                }
            }
            if ui.button("➕").on_hover_text(self.t("Tambah chatlog")).clicked() {
                self.blocks.push(ChatBlock::new(self.blocks.len()));
                self.selected_block = self.blocks.len() - 1;
                self.dirty = true;
            }
            if ui.button("📂").on_hover_text(self.t("Chatlog dari folder")).clicked() {
                self.chatlog.open();
            }
        });

        let bi = self.selected_block.min(self.blocks.len() - 1);
        self.selected_block = bi;
        let out = egui::TextEdit::multiline(&mut self.blocks[bi].text)
            .desired_rows(6)
            .desired_width(f32::INFINITY)
            .hint_text("[12:34:56] Budi_Santoso says: contoh chat")
            .show(ui);
        if out.response.changed() {
            self.dirty = true;
        }
        // Remember the current selection so a swatch can wrap it, even
        // after the click moves focus off the textarea.
        if let Some(range) = out.cursor_range {
            let a = range.primary.ccursor.index;
            let b = range.secondary.ccursor.index;
            let (s, e) = if a <= b { (a, b) } else { (b, a) };
            if s != e {
                self.text_selection = Some((bi, s, e));
            }
        }

        self.palette_ui(ui);
        self.combo_anchor(ui, bi);
        self.combo_bg(ui, bi);
        let del_lbl = self.t("🗑 Hapus chatlog ini");
        if self.blocks.len() > 1 && ui.button(del_lbl).clicked() {
            self.blocks.remove(bi);
            self.selected_block = 0;
            self.dirty = true;
        }
    }

    fn tool_text(&mut self, ui: &mut egui::Ui) {
        ui.label(self.t("Teks"));
        let font_lbl = self.t("Font");
        ui.horizontal(|ui| {
            ui.label(font_lbl);
            if self.font_list.is_empty() {
                self.font_list = screenies_core::fonts::families();
            }
            let mut chosen: Option<String> = None;
            egui::ComboBox::from_id_salt("font")
                .selected_text(self.font_family.clone())
                .width(190.0)
                .show_ui(ui, |ui| {
                    for i in 0..self.font_list.len() {
                        let fam = self.font_list[i].clone();
                        if ui.selectable_label(self.font_family == fam, &fam).clicked() {
                            chosen = Some(fam);
                        }
                    }
                });
            if let Some(f) = chosen {
                self.font_family = f;
                self.dirty = true;
            }
        });
        let size_lbl = self.t("Ukuran");
        if ui
            .add(egui::Slider::new(&mut self.text_size, 8.0..=60.0).text(size_lbl))
            .changed()
        {
            self.dirty = true;
        }
        let gap_lbl = self.t("Jarak baris %");
        if ui
            .add(egui::Slider::new(&mut self.line_gap, 80.0..=200.0).text(gap_lbl))
            .changed()
        {
            self.dirty = true;
        }
        let auto_lbl = self.t("Outline otomatis");
        if ui.checkbox(&mut self.stroke_auto, auto_lbl).changed() {
            self.dirty = true;
        }
        let outline_lbl = self.t("Outline px");
        if !self.stroke_auto
            && ui
                .add(egui::Slider::new(&mut self.stroke_width, 0.0..=10.0).text(outline_lbl))
                .changed()
        {
            self.dirty = true;
        }
    }

    fn tool_fx(&mut self, ui: &mut egui::Ui) {
        ui.collapsing(self.t("Filter"), |ui| {
            self.filter_slider(ui, "Brightness", 0.0..=300.0, |f| &mut f.brightness);
            self.filter_slider(ui, "Contrast", 0.0..=200.0, |f| &mut f.contrast);
            self.filter_slider(ui, "Grayscale", 0.0..=100.0, |f| &mut f.grayscale);
            self.filter_slider(ui, "Sepia", 0.0..=100.0, |f| &mut f.sepia);
            self.filter_slider(ui, "Saturate", 0.0..=300.0, |f| &mut f.saturate);
        });

        ui.collapsing(self.t("Sensor area (blur/pixelate lokal)"), |ui| {
            ui.horizontal(|ui| {
                if ui.button("+ Blur").clicked() {
                    self.add_censor(CensorKind::Blur);
                }
                if ui.button("+ Pixelate").clicked() {
                    self.add_censor(CensorKind::Pixelate);
                }
            });
            ui.small(self.t("Klik kotak di preview untuk pilih · seret badan untuk geser · seret pojok untuk resize."));

            if let Some(i) = self.selected_censor {
                if i < self.censors.len() {
                    let kind = self.censors[i].kind;
                    let label = self.t(match kind {
                        CensorKind::Blur => "Blur radius (px)",
                        CensorKind::Pixelate => "Blok (px)",
                    });
                    let mut strength = self.censors[i].strength;
                    if ui.add(egui::Slider::new(&mut strength, 1.0..=64.0).text(label)).changed() {
                        self.censors[i].strength = strength;
                        self.dirty = true;
                    }
                    let del = self.t("🗑 Hapus area");
                    if ui.button(del).clicked() {
                        self.censors.remove(i);
                        self.selected_censor = None;
                        self.dirty = true;
                    }
                }
            }
            ui.small(format!("{} area sensor", self.censors.len()));
        });

        ui.collapsing(self.t("Stiker"), |ui| {
            if ui.button(self.t("+ Tambah stiker")).clicked() {
                self.add_sticker();
            }
            ui.small(self.t("Klik stiker di preview untuk pilih · seret untuk geser · pojok untuk resize."));
            if let Some(i) = self.selected_sticker {
                if i < self.stickers.len() {
                    let (out_w, _) = self.output_dims();
                    let wl = self.t("Lebar (px)");
                    let mut w = self.stickers[i].w;
                    if ui.add(egui::Slider::new(&mut w, 16.0..=out_w).text(wl)).changed() {
                        self.stickers[i].w = w;
                        self.stickers[i].h = w / self.stickers[i].aspect;
                        self.dirty = true;
                    }
                    let del = self.t("🗑 Hapus stiker");
                    if ui.button(del).clicked() {
                        self.stickers.remove(i);
                        self.selected_sticker = None;
                        self.dirty = true;
                    }
                }
            }
            ui.small(format!("{} stiker", self.stickers.len()));
        });

        ui.separator();
        ui.label(self.t("Mode"));
        ui.horizontal(|ui| {
            if ui.selectable_label(!self.cinematic, self.t("Normal")).clicked() {
                self.cinematic = false;
                self.dirty = true;
            }
            if ui.selectable_label(self.cinematic, self.t("🎬 Sinema")).clicked() {
                self.cinematic = true;
                self.dirty = true;
            }
        });
        if self.cinematic {
            let bar_lbl = self.t("Tinggi bar %");
            if ui
                .add(egui::Slider::new(&mut self.cinematic_bar, 0.0..=40.0).text(bar_lbl))
                .changed()
            {
                self.dirty = true;
            }
            ui.horizontal(|ui| {
                ui.label(self.t("Posisi bar"));
                let (both, atas, bawah) = (self.t("Keduanya"), self.t("Atas"), self.t("Bawah"));
                let prev = self.cinematic_bar_pos;
                ui.selectable_value(&mut self.cinematic_bar_pos, BarPos::Both, both);
                ui.selectable_value(&mut self.cinematic_bar_pos, BarPos::Top, atas);
                ui.selectable_value(&mut self.cinematic_bar_pos, BarPos::Bottom, bawah);
                if self.cinematic_bar_pos != prev {
                    self.dirty = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label(self.t("Warna bar"));
                if ui.color_edit_button_srgb(&mut self.cinematic_color).changed() {
                    self.dirty = true;
                }
            });
            ui.small(self.t("Bar sinema digambar di dalam foto (pilih posisi)."));
        }
    }

    fn combo_anchor(&mut self, ui: &mut egui::Ui, bi: usize) {
        let prev = self.blocks[bi].anchor;
        let sel = format!("{}: {}", self.t("Posisi"), self.t(anchor_label(prev)));
        let (free, atas, bawah) = (self.t("Bebas"), self.t("Kiri Atas"), self.t("Kiri Bawah"));
        egui::ComboBox::from_id_salt(("anchor", bi))
            .selected_text(sel)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.blocks[bi].anchor, Anchor::Free, free);
                ui.selectable_value(&mut self.blocks[bi].anchor, Anchor::KiriAtas, atas);
                ui.selectable_value(&mut self.blocks[bi].anchor, Anchor::KiriBawah, bawah);
            });
        if self.blocks[bi].anchor != prev {
            self.dirty = true;
        }
        if self.blocks[bi].anchor == Anchor::Free {
            let a = ui.add(egui::Slider::new(&mut self.blocks[bi].x, 0.0..=4000.0).text("X")).changed();
            let b = ui.add(egui::Slider::new(&mut self.blocks[bi].y, 0.0..=4000.0).text("Y")).changed();
            if a || b {
                self.dirty = true;
            }
        }
    }

    fn combo_bg(&mut self, ui: &mut egui::Ui, bi: usize) {
        let prev = self.blocks[bi].bg_mode;
        let sel = format!("BG: {}", self.t(bg_label(prev)));
        let (none, blok, mask) = (self.t("Tidak ada"), self.t("Blok"), self.t("Mask"));
        egui::ComboBox::from_id_salt(("bg", bi))
            .selected_text(sel)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.blocks[bi].bg_mode, BgMode::None, none);
                ui.selectable_value(&mut self.blocks[bi].bg_mode, BgMode::Block, blok);
                ui.selectable_value(&mut self.blocks[bi].bg_mode, BgMode::Mask, mask);
            });
        if self.blocks[bi].bg_mode != prev {
            self.dirty = true;
        }
    }

    fn palette_ui(&mut self, ui: &mut egui::Ui) {
        let header = self.t("Palet warna");
        let hint = self.t("Pilih teks di chatlog, klik warna untuk membungkus {RRGGBB}.");
        let apply_lbl = self.t("Terapkan kustom");
        let sel_yes = self.t("✓ ada teks terpilih");
        let sel_no = self.t("(pilih teks di chatlog dulu)");
        ui.collapsing(header, |ui| {
            ui.small(hint);
            ui.horizontal_wrapped(|ui| {
                for &(name, hex) in PALETTE {
                    let btn = egui::Button::new("   ").fill(hex_to_color32(hex));
                    if ui.add(btn).on_hover_text(self.t(name)).clicked() {
                        self.apply_color(hex);
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.color_edit_button_srgb(&mut self.custom_color);
                if ui.button(apply_lbl).clicked() {
                    let hex = format!(
                        "{:02X}{:02X}{:02X}",
                        self.custom_color[0], self.custom_color[1], self.custom_color[2]
                    );
                    self.apply_color(&hex);
                }
            });
            ui.small(if self.text_selection.is_some() { sel_yes } else { sel_no });
        });
    }

    /// Wrap the remembered selection with `{RRGGBB}…{FFFFFF}` (char-indexed,
    /// so multibyte text is safe).
    fn apply_color(&mut self, hex: &str) {
        let Some((bi, s, e)) = self.text_selection else {
            return;
        };
        if bi >= self.blocks.len() {
            return;
        }
        let chars: Vec<char> = self.blocks[bi].text.chars().collect();
        if s >= e || e > chars.len() {
            return;
        }
        let before: String = chars[..s].iter().collect();
        let sel: String = chars[s..e].iter().collect();
        let after: String = chars[e..].iter().collect();
        self.blocks[bi].text = format!("{before}{{{hex}}}{sel}{{FFFFFF}}{after}");
        self.text_selection = None;
        self.dirty = true;
    }

    fn filter_slider(
        &mut self,
        ui: &mut egui::Ui,
        label: &str,
        range: std::ops::RangeInclusive<f32>,
        field: impl Fn(&mut FilterValues) -> &mut f32,
    ) {
        if ui
            .add(egui::Slider::new(field(&mut self.filters), range).text(label))
            .changed()
        {
            self.dirty = true;
        }
    }

    fn preview(&mut self, ui: &mut egui::Ui) {
        // Crop-edit mode shows the SOURCE photo with an editable crop box.
        if self.crop_editing {
            if let Some((pw, ph)) = self.photo.as_ref().map(|p| (p.w, p.h)) {
                self.preview_crop(ui, pw, ph);
                return;
            }
        }

        let Some(tex) = self.texture.as_ref() else {
            let msg = self.t("Muat foto untuk mulai mengedit.");
            ui.centered_and_justified(|ui| {
                ui.label(msg);
            });
            return;
        };
        let tex_id = tex.id();
        let img_px = tex.size_vec2(); // output-space size
        let avail = ui.available_size();
        let scale = (avail.x / img_px.x).min(avail.y / img_px.y).min(1.0);
        let disp = img_px * scale;

        // Reserve the area, then draw the rendered image centered in it.
        let (area, _) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let origin = area.min + (avail - disp) * 0.5; // image top-left, screen px
        let img_rect = egui::Rect::from_min_size(origin, disp);
        let painter = ui.painter_at(area);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(tex_id, img_rect, uv, egui::Color32::WHITE);

        // Floating preview toolbar (top-right) with Undo/Redo and other quick actions.
        // Position it relative to the preview area so it overlays the image.
        let toolbar_w = 160.0;
        let toolbar_h = 40.0;
        let toolbar_x = area.max.x - toolbar_w - 12.0;
        let toolbar_y = area.min.y + 12.0;
        let toolbar_pos = egui::pos2(toolbar_x, toolbar_y);
        egui::Area::new("preview_toolbar")
            .fixed_pos(toolbar_pos)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(self.history.len() > 1, egui::Button::new("↩  Undo"))
                        .on_hover_text("Ctrl+Z")
                        .clicked()
                    {
                        self.undo();
                    }
                    if ui
                        .add_enabled(!self.future.is_empty(), egui::Button::new("↪  Redo"))
                        .on_hover_text("Ctrl+Y")
                        .clicked()
                    {
                        self.redo();
                    }
                });
            });

        // Censor boxes: output coords → screen via (origin + coord*scale).
        // The blur/pixelate itself is already baked into the image by core;
        // here we only draw the editable outline + handle and take the input.
        const HANDLE: f32 = 12.0;
        for i in 0..self.censors.len() {
            let c = self.censors[i];
            let min = egui::pos2(origin.x + c.x * scale, origin.y + c.y * scale);
            let rect = egui::Rect::from_min_size(min, egui::vec2(c.w * scale, c.h * scale));
            let selected = self.selected_censor == Some(i);

            let body = ui.interact(rect, egui::Id::new(("censor", i)), egui::Sense::click_and_drag());
            if body.clicked() {
                self.selected_censor = Some(i);
            }
            if body.dragged() {
                let d = body.drag_delta() / scale;
                self.censors[i].x += d.x;
                self.censors[i].y += d.y;
                self.selected_censor = Some(i);
                self.dirty = true;
            }

            let color = if selected {
                ui.visuals().selection.stroke.color
            } else {
                ui.visuals().text_color()
            };
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(if selected { 2.0_f32 } else { 1.0_f32 }, color));
            let tag = match c.kind {
                CensorKind::Blur => "blur",
                CensorKind::Pixelate => "pixel",
            };
            painter.text(
                rect.min + egui::vec2(3.0, 2.0),
                egui::Align2::LEFT_TOP,
                tag,
                egui::FontId::monospace(11.0),
                color,
            );

            if selected {
                let hrect = egui::Rect::from_min_size(rect.max - egui::vec2(HANDLE, HANDLE), egui::vec2(HANDLE, HANDLE));
                painter.rect_filled(hrect, 0.0, color);
                let hr = ui.interact(hrect, egui::Id::new(("censor-resize", i)), egui::Sense::drag());
                if hr.dragged() {
                    let d = hr.drag_delta() / scale;
                    self.censors[i].w = (self.censors[i].w + d.x).max(8.0);
                    self.censors[i].h = (self.censors[i].h + d.y).max(8.0);
                    self.dirty = true;
                }
            }
        }

        // Stickers: the image is composited by core into the texture; here we
        // only outline the selected one + take move/resize (aspect-locked).
        for i in 0..self.stickers.len() {
            let (sx, sy, sw, sh) = {
                let s = &self.stickers[i];
                (s.x, s.y, s.w, s.h)
            };
            let min = egui::pos2(origin.x + sx * scale, origin.y + sy * scale);
            let rect = egui::Rect::from_min_size(min, egui::vec2(sw * scale, sh * scale));
            let selected = self.selected_sticker == Some(i);

            let body = ui.interact(rect, egui::Id::new(("sticker", i)), egui::Sense::click_and_drag());
            if body.clicked() {
                self.selected_sticker = Some(i);
                self.selected_censor = None;
            }
            if body.dragged() {
                let d = body.drag_delta() / scale;
                self.stickers[i].x += d.x;
                self.stickers[i].y += d.y;
                self.selected_sticker = Some(i);
                self.dirty = true;
            }

            if selected {
                let color = ui.visuals().warn_fg_color;
                painter.rect_stroke(rect, 0.0, egui::Stroke::new(2.0_f32, color));
                let hrect = egui::Rect::from_min_size(rect.max - egui::vec2(HANDLE, HANDLE), egui::vec2(HANDLE, HANDLE));
                painter.rect_filled(hrect, 0.0, color);
                let hr = ui.interact(hrect, egui::Id::new(("sticker-resize", i)), egui::Sense::drag());
                if hr.dragged() {
                    let d = hr.drag_delta() / scale;
                    let neww = (self.stickers[i].w + d.x).max(16.0);
                    self.stickers[i].w = neww;
                    self.stickers[i].h = neww / self.stickers[i].aspect;
                    self.dirty = true;
                }
            }
        }
    }

    /// Crop-edit view: the SOURCE photo with a draggable/resizable crop box
    /// and the outside dimmed. Coordinates are source px; screen = origin + c*scale.
    fn preview_crop(&mut self, ui: &mut egui::Ui, pw: u32, ph: u32) {
        if self.source_tex.is_none() {
            if let Some(img) = self.source_img.clone() {
                self.source_tex =
                    Some(ui.ctx().load_texture("source", img, egui::TextureOptions::LINEAR));
            }
        }
        let load_msg = self.t("Muat foto dulu.");
        let Some(tex) = self.source_tex.as_ref() else {
            ui.centered_and_justified(|ui| {
                ui.label(load_msg);
            });
            return;
        };
        let tex_id = tex.id();
        let src = egui::vec2(pw as f32, ph as f32);
        let avail = ui.available_size();
        let scale = (avail.x / src.x).min(avail.y / src.y).min(1.0);
        let disp = src * scale;
        let (area, _) = ui.allocate_exact_size(avail, egui::Sense::hover());
        let origin = area.min + (avail - disp) * 0.5;
        let img_rect = egui::Rect::from_min_size(origin, disp);
        let painter = ui.painter_at(area);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(tex_id, img_rect, uv, egui::Color32::WHITE);

        let mut crop = self
            .crop
            .unwrap_or(CropRect { x: 0.0, y: 0.0, w: pw as f64, h: ph as f64 });
        let cr_min = egui::pos2(origin.x + crop.x as f32 * scale, origin.y + crop.y as f32 * scale);
        let cr = egui::Rect::from_min_size(cr_min, egui::vec2(crop.w as f32 * scale, crop.h as f32 * scale));

        // Dim the four strips outside the crop box.
        let dim = egui::Color32::from_black_alpha(130);
        let z = egui::Rounding::ZERO;
        painter.rect_filled(egui::Rect::from_min_max(img_rect.min, egui::pos2(img_rect.max.x, cr.min.y)), z, dim);
        painter.rect_filled(egui::Rect::from_min_max(egui::pos2(img_rect.min.x, cr.max.y), img_rect.max), z, dim);
        painter.rect_filled(egui::Rect::from_min_max(egui::pos2(img_rect.min.x, cr.min.y), egui::pos2(cr.min.x, cr.max.y)), z, dim);
        painter.rect_filled(egui::Rect::from_min_max(egui::pos2(cr.max.x, cr.min.y), egui::pos2(img_rect.max.x, cr.max.y)), z, dim);

        let accent = ui.visuals().selection.stroke.color;
        painter.rect_stroke(cr, z, egui::Stroke::new(2.0_f32, accent));

        // Move the whole box: the inner area (inset so it doesn't fight the edge
        // handles) drags with a "grab" cursor.
        const H: f32 = 12.0;
        let body = ui
            .interact(cr.shrink(H), egui::Id::new("crop-body"), egui::Sense::drag())
            .on_hover_cursor(egui::CursorIcon::Grab);
        if body.dragged() {
            let d = body.drag_delta() / scale;
            crop.x += d.x as f64;
            crop.y += d.y as f64;
        }

        // Eight resize handles (4 corners + 4 edges) — drag any side like a
        // window, each with a matching resize cursor. Fields per handle:
        // (fraction x, fraction y, left, right, top, bottom, cursor).
        let handles: [(f32, f32, bool, bool, bool, bool, egui::CursorIcon); 8] = [
            (0.0, 0.0, true, false, true, false, egui::CursorIcon::ResizeNwSe), // NW
            (0.5, 0.0, false, false, true, false, egui::CursorIcon::ResizeVertical), // N
            (1.0, 0.0, false, true, true, false, egui::CursorIcon::ResizeNeSw), // NE
            (1.0, 0.5, false, true, false, false, egui::CursorIcon::ResizeHorizontal), // E
            (1.0, 1.0, false, true, false, true, egui::CursorIcon::ResizeNwSe), // SE
            (0.5, 1.0, false, false, false, true, egui::CursorIcon::ResizeVertical), // S
            (0.0, 1.0, true, false, false, true, egui::CursorIcon::ResizeNeSw), // SW
            (0.0, 0.5, true, false, false, false, egui::CursorIcon::ResizeHorizontal), // W
        ];
        let mut resized = false;
        for (i, &(fx, fy, left, right, top, bottom, cursor)) in handles.iter().enumerate() {
            let center = egui::pos2(cr.min.x + fx * cr.width(), cr.min.y + fy * cr.height());
            let hrect = egui::Rect::from_center_size(center, egui::vec2(H, H));
            painter.rect_filled(hrect, z, accent);
            let hr = ui
                .interact(hrect, egui::Id::new(("crop-h", i)), egui::Sense::drag())
                .on_hover_cursor(cursor);
            if hr.dragged() {
                let d = hr.drag_delta() / scale;
                let (dx, dy) = (d.x as f64, d.y as f64);
                if left {
                    crop.x += dx;
                    crop.w -= dx;
                }
                if right {
                    crop.w += dx;
                }
                if top {
                    crop.y += dy;
                    crop.h -= dy;
                }
                if bottom {
                    crop.h += dy;
                }
                resized = true;
            }
        }

        if body.dragged() || resized {
            clamp_crop(&mut crop, pw, ph, self.crop_ratio);
            self.crop = Some(crop);
            self.dirty = true;
        }

        let crop_hint = self.t("Seret kotak untuk framing · sisi/pojok untuk resize · klik “✓ Selesai crop”");
        painter.text(
            img_rect.min + egui::vec2(6.0, 6.0),
            egui::Align2::LEFT_TOP,
            crop_hint,
            egui::FontId::proportional(12.0),
            egui::Color32::WHITE,
        );
    }

    fn pick_photo(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Gambar", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            self.load_photo_path(&path);
        }
    }

    /// Persisted settings accessors.
    pub fn font(&self) -> &str {
        &self.font_family
    }
    pub fn set_font(&mut self, f: String) {
        if !f.is_empty() {
            self.font_family = f;
            self.dirty = true;
        }
    }
    pub fn chatlog_folder(&self) -> Option<String> {
        self.chatlog.folder_path()
    }
    pub fn set_chatlog_folder(&mut self, p: Option<String>) {
        self.chatlog.set_folder_path(p);
    }
    pub fn prefs(&self) -> (f32, f32, FilterValues) {
        (self.text_size, self.line_gap, self.filters)
    }
    pub fn apply_prefs(&mut self, size: f32, gap: f32, filters: FilterValues) {
        self.text_size = size.clamp(8.0, 60.0);
        self.line_gap = gap.clamp(80.0, 200.0);
        self.filters = filters;
        self.dirty = true;
    }

    /// Load a photo from a path (used by the file picker and the Gallery's
    /// "open in editor"). Resets the crop to the whole photo.
    pub fn load_photo_path(&mut self, path: &std::path::Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(format!("Gagal baca file: {e}"));
                return;
            }
        };
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let (w, h) = (rgba.width(), rgba.height());
                self.source_img = Some(egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    rgba.as_raw(),
                ));
                self.source_tex = None;
                self.photo = Some(Photo {
                    base64: base64::engine::general_purpose::STANDARD.encode(&bytes).into(),
                    w,
                    h,
                });
                self.crop = None;
                self.crop_ratio = None;
                self.output_override = None;
                self.crop_editing = false;
                self.error = None;
                self.base_cache = None; // new photo → invalidate the preview base
                self.filtered_cache = None;
                self.dirty = true;
            }
            Err(e) => self.error = Some(format!("Gagal decode gambar: {e}")),
        }
    }

    fn capture_doc(&self) -> Doc {
        Doc {
            photo: self.photo.clone(),
            blocks: self.blocks.clone(),
            censors: self.censors.clone(),
            stickers: self.stickers.clone(),
            crop: self.crop,
            crop_ratio: self.crop_ratio,
            output_override: self.output_override,
            filters: self.filters,
        }
    }

    fn load_doc(&mut self, d: Doc) {
        self.source_img = d.photo.as_ref().and_then(|p| decode_color_image(&p.base64));
        self.source_tex = None;
        self.photo = d.photo;
        self.blocks = d.blocks;
        self.censors = d.censors;
        self.stickers = d.stickers;
        self.crop = d.crop;
        self.crop_ratio = d.crop_ratio;
        self.output_override = d.output_override;
        self.filters = d.filters;
        self.selected_block = 0;
        self.selected_censor = None;
        self.selected_sticker = None;
        self.crop_editing = false;
        self.history.clear();
        self.future.clear();
        self.base_cache = None; // switched document/photo → invalidate the base
        self.filtered_cache = None;
        self.dirty = true;
    }

    fn switch_doc(&mut self, target: usize) {
        if target == self.active || target >= self.docs.len() {
            return;
        }
        self.docs[self.active] = self.capture_doc();
        self.active = target;
        let d = self.docs[target].clone();
        self.load_doc(d);
    }

    fn add_doc(&mut self) {
        self.docs[self.active] = self.capture_doc();
        self.docs.push(Doc::default());
        self.active = self.docs.len() - 1;
        let d = self.docs[self.active].clone();
        self.load_doc(d);
    }

    fn close_doc(&mut self, i: usize) {
        if self.docs.len() <= 1 || i >= self.docs.len() {
            return;
        }
        if i == self.active {
            self.docs.remove(i);
            self.active = self.active.min(self.docs.len() - 1);
            let d = self.docs[self.active].clone();
            self.load_doc(d);
        } else {
            self.docs[self.active] = self.capture_doc();
            self.docs.remove(i);
            if i < self.active {
                self.active -= 1;
            }
        }
    }

    /// Full render/output size — a fixed-resolution override wins, else the crop
    /// size, else the whole photo. This is the coordinate space stickers, censor
    /// boxes and text live in. Cinematic bars are drawn INSIDE this (they cover
    /// the photo's top/bottom and don't change the output size).
    fn output_dims(&self) -> (f32, f32) {
        if let Some((w, h)) = self.output_override {
            (w as f32, h as f32)
        } else if let Some(c) = self.crop {
            (c.w as f32, c.h as f32)
        } else if let Some(p) = &self.photo {
            (p.w as f32, p.h as f32)
        } else {
            (400.0, 300.0)
        }
    }

    /// Cinematic bar height (px) — each bar covers this much of the output's top
    /// and bottom (grows inward; 0 = off).
    fn cinematic_bar_px(&self) -> f32 {
        if self.cinematic {
            let (_, oh) = self.output_dims();
            (oh * (self.cinematic_bar / 100.0)).round().max(0.0)
        } else {
            0.0
        }
    }

    /// Add a sticker (PNG/WebP/…) centered, sized to ~30% of the output width.
    fn add_sticker(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Gambar", &["png", "webp", "jpg", "jpeg", "bmp"])
            .pick_file()
        else {
            return;
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                self.error = Some(format!("Gagal baca stiker: {e}"));
                return;
            }
        };
        let (ow_px, oh_px) = match image::load_from_memory(&bytes) {
            Ok(img) => (img.width().max(1), img.height().max(1)),
            Err(e) => {
                self.error = Some(format!("Gagal decode stiker: {e}"));
                return;
            }
        };
        let aspect = ow_px as f32 / oh_px as f32;
        let (out_w, out_h) = self.output_dims();
        let w = (out_w * 0.30).clamp(48.0, out_w);
        let h = w / aspect;
        self.stickers.push(Sticker {
            base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            x: (out_w - w) / 2.0,
            y: (out_h - h) / 2.0,
            w,
            h,
            aspect,
        });
        self.selected_sticker = Some(self.stickers.len() - 1);
        self.error = None;
        self.dirty = true;
    }

    /// Add a censor box centered on the output (sized relative to it).
    fn add_censor(&mut self, kind: CensorKind) {
        let (ow, oh) = self.output_dims();
        let w = (ow * 0.28).max(40.0);
        let h = (oh * 0.14).max(24.0);
        self.censors.push(CensorRegion {
            x: (ow - w) / 2.0,
            y: (oh - h) / 2.0,
            w,
            h,
            kind,
            strength: match kind {
                CensorKind::Blur => 10.0,
                CensorKind::Pixelate => 14.0,
            },
        });
        self.selected_censor = Some(self.censors.len() - 1);
        self.dirty = true;
    }

    /// Pick an aspect ratio (None = free): reset the crop to the largest
    /// centered box of that ratio and jump into crop-edit mode.
    fn set_ratio(&mut self, ratio: Option<f32>) {
        if let Some(p) = &self.photo {
            self.crop_ratio = ratio;
            self.crop = Some(centered_crop(p.w, p.h, ratio));
            self.output_override = None;
            self.crop_editing = true;
            self.dirty = true;
        }
    }

    /// Fixed output resolution (e.g. 800×600): crop to that aspect ratio and scale
    /// the result to exactly w×h. The crop box is editable (drag to reframe), so
    /// nothing is stretched; the output is a true w×h file — unlike the plain 4:3
    /// preset, which keeps the source resolution.
    fn set_resolution(&mut self, w: u32, h: u32) {
        if let Some(p) = &self.photo {
            let ratio = w as f32 / h as f32;
            self.crop_ratio = Some(ratio);
            self.crop = Some(centered_crop(p.w, p.h, Some(ratio)));
            self.output_override = Some((w, h));
            self.crop_editing = !self.crop_fit; // fit keeps the whole image — no crop step
            self.dirty = true;
        }
    }

    fn toggle_crop_edit(&mut self) {
        self.crop_editing = !self.crop_editing;
        if self.crop_editing && self.crop.is_none() {
            if let Some(p) = &self.photo {
                self.crop = Some(centered_crop(p.w, p.h, self.crop_ratio));
            }
        }
        self.dirty = true;
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            blocks: self.blocks.clone(),
            censors: self.censors.clone(),
            stickers: self.stickers.clone(),
            crop: self.crop,
            crop_ratio: self.crop_ratio,
            crop_fit: self.crop_fit,
            filters: self.filters,
            cinematic: self.cinematic,
            cinematic_bar: self.cinematic_bar,
            cinematic_bar_pos: self.cinematic_bar_pos,
            cinematic_color: self.cinematic_color,
            font_family: self.font_family.clone(),
            text_size: self.text_size,
            line_gap: self.line_gap,
            stroke_auto: self.stroke_auto,
            stroke_width: self.stroke_width,
        }
    }

    /// True when the live editable state equals `s`, compared in place so
    /// `maybe_commit` can decide "did anything change?" without materializing a
    /// `Snapshot` (which deep-clones every block, sticker and its base64 string)
    /// on frames where nothing changed. Keep the field list in sync with
    /// `snapshot`/`restore`.
    fn matches_snapshot(&self, s: &Snapshot) -> bool {
        self.blocks == s.blocks
            && self.censors == s.censors
            && self.stickers == s.stickers
            && self.crop == s.crop
            && self.crop_ratio == s.crop_ratio
            && self.crop_fit == s.crop_fit
            && self.filters == s.filters
            && self.cinematic == s.cinematic
            && self.cinematic_bar == s.cinematic_bar
            && self.cinematic_bar_pos == s.cinematic_bar_pos
            && self.cinematic_color == s.cinematic_color
            && self.font_family == s.font_family
            && self.text_size == s.text_size
            && self.line_gap == s.line_gap
            && self.stroke_auto == s.stroke_auto
            && self.stroke_width == s.stroke_width
    }

    fn restore(&mut self, s: Snapshot) {
        self.blocks = s.blocks;
        self.censors = s.censors;
        self.stickers = s.stickers;
        self.crop = s.crop;
        self.crop_ratio = s.crop_ratio;
        self.crop_fit = s.crop_fit;
        self.filters = s.filters;
        self.cinematic = s.cinematic;
        self.cinematic_bar = s.cinematic_bar;
        self.cinematic_bar_pos = s.cinematic_bar_pos;
        self.cinematic_color = s.cinematic_color;
        self.font_family = s.font_family;
        self.text_size = s.text_size;
        self.line_gap = s.line_gap;
        self.stroke_auto = s.stroke_auto;
        self.stroke_width = s.stroke_width;
        self.selected_block = self.selected_block.min(self.blocks.len().saturating_sub(1));
        self.selected_censor = None;
        self.selected_sticker = None;
        self.dirty = true;
    }

    /// Record an undo step once edits settle (pointer up), coalescing drags.
    fn maybe_commit(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.any_down()) {
            return; // mid-drag — wait for release
        }
        // Compare live state to the last commit in place; only build a snapshot
        // (which deep-clones blocks/stickers/base64) once we're actually pushing.
        let changed = self.history.last().map_or(true, |last| !self.matches_snapshot(last));
        if changed {
            self.history.push(self.snapshot());
            if self.history.len() > 80 {
                self.history.remove(0);
            }
            self.future.clear();
        }
    }

    fn undo(&mut self) {
        if self.history.len() < 2 {
            return;
        }
        let cur = self.history.pop().unwrap();
        self.future.push(cur);
        let prev = self.history.last().unwrap().clone();
        self.restore(prev);
    }

    fn redo(&mut self) {
        if let Some(snap) = self.future.pop() {
            self.history.push(snap.clone());
            self.restore(snap);
        }
    }

    /// Auto outline thickness — matches the 1.x rule (size/9, floored).
    fn effective_stroke(&self) -> f32 {
        if self.stroke_auto {
            let min = if self.text_size < 14.0 { 1.0 } else { 2.0 };
            (self.text_size / 9.0).round().max(min)
        } else {
            self.stroke_width
        }
    }

    /// Assemble the render job from the current state (None if no photo).
    /// Text is laid out in core; font-load failure just drops the text layer
    /// (the photo still renders) so the preview never goes blank on a typo.
    fn current_job(&self) -> Option<RenderJob> {
        let photo = self.photo.as_ref()?;
        let full = CropRect { x: 0.0, y: 0.0, w: photo.w as f64, h: photo.h as f64 };
        // Fit mode (fixed sizes): keep the WHOLE image — the crop region is the
        // full photo, and compose pads it with bars instead of cropping to fill.
        let use_fit = self.crop_fit && self.output_override.is_some();
        let crop = if use_fit { full } else { self.crop.unwrap_or(full) };
        // Output size (fixed-resolution override, else the crop). Cinematic mode
        // keeps this size — its bars are painted INSIDE the photo, not added.
        let output = match self.output_override {
            Some((w, h)) => Size { w, h },
            None => Size {
                w: (crop.w.round() as u32).max(1),
                h: (crop.h.round() as u32).max(1),
            },
        };

        // Cinematic mode: solid bars over the output's top & bottom (inward).
        let canvas = if self.cinematic {
            Some(Canvas {
                color: [
                    self.cinematic_color[0],
                    self.cinematic_color[1],
                    self.cinematic_color[2],
                    255,
                ],
                bar: self.cinematic_bar_px() as u32,
                bars: self.cinematic_bar_pos,
            })
        } else {
            None
        };

        let blocks = if let Ok(measure) = GlyphMeasure::new(&self.font_family, self.text_size) {
            let params = LayoutParams {
                text_size: self.text_size,
                line_gap: self.line_gap,
                bg_offset: 0.0,
                output_w: output.w as f32,
                output_h: output.h as f32,
            };
            let lblocks: Vec<LayoutBlock> = self
                .blocks
                .iter()
                .filter(|b| !b.text.trim().is_empty())
                .map(|b| LayoutBlock {
                    lines: chatlog::parse(&b.text, &self.preset),
                    anchor: b.anchor,
                    bg_mode: b.bg_mode,
                    x: b.x,
                    y: b.y,
                })
                .collect();
            layout::layout_blocks(&lblocks, &params, &measure)
                .into_iter()
                .map(|l| l.block)
                .collect()
        } else {
            Vec::new()
        };

        let stickers = self
            .stickers
            .iter()
            .map(|s| StickerJob {
                data_base64: s.base64.clone(),
                x: s.x.round() as i64,
                y: s.y.round() as i64,
                w: (s.w.round() as u32).max(1),
                h: (s.h.round() as u32).max(1),
            })
            .collect();

        Some(RenderJob {
            image_base64: photo.base64.clone(),
            crop,
            output,
            stickers,
            filters: self.filters,
            censors: self.censors.clone(),
            canvas,
            fit: use_fit,
            font_family: self.font_family.clone(),
            text_size: self.text_size,
            stroke_width: self.effective_stroke(),
            blocks,
        })
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        self.dirty = false;
        let Some(job) = self.current_job() else {
            self.texture = None;
            return;
        };

        // Layer 1: the filtered base (photo + filters + cinematic bars). When it
        // hits, an overlay-only edit (sticker/censor/text drag) skips BOTH the
        // base prep AND the whole-image filter pass — we composite straight onto
        // the cached filtered image. Keyed on the base inputs plus filters/canvas.
        let filt_hit = self.filtered_cache.as_ref().map_or(false, |c| {
            c.crop == job.crop
                && c.out_w == job.output.w
                && c.out_h == job.output.h
                && c.fit == job.fit
                && c.filters == job.filters
                && c.canvas == job.canvas
        });
        let filtered = if filt_hit {
            self.filtered_cache.as_ref().unwrap().img.clone()
        } else {
            // Layer 0: decoded/cropped/resized photo. Its own cache is keyed on
            // just crop/size/fit, so a filter change reuses the base and rebuilds
            // only the filtered layer. Cleared (both layers) when the photo changes.
            let base_hit = self.base_cache.as_ref().map_or(false, |c| {
                c.crop == job.crop
                    && c.out_w == job.output.w
                    && c.out_h == job.output.h
                    && c.fit == job.fit
            });
            let base = if base_hit {
                self.base_cache.as_ref().unwrap().img.clone()
            } else {
                match compose::prepare_base(&job) {
                    Ok(b) => {
                        self.base_cache = Some(BaseCache {
                            crop: job.crop,
                            out_w: job.output.w,
                            out_h: job.output.h,
                            fit: job.fit,
                            img: b.clone(),
                        });
                        b
                    }
                    Err(e) => {
                        self.error = Some(format!("Render gagal: {e}"));
                        return;
                    }
                }
            };
            let filtered = compose::apply_filters_and_bars(base, &job);
            self.filtered_cache = Some(FilteredCache {
                crop: job.crop,
                out_w: job.output.w,
                out_h: job.output.h,
                fit: job.fit,
                filters: job.filters,
                canvas: job.canvas,
                img: filtered.clone(),
            });
            filtered
        };

        // Layer 2: overlays (stickers, text, censors) — always re-run, since
        // they're what's being edited, but now on top of the cached filtered base.
        match compose::draw_overlays(filtered, &job) {
            Ok(img) => {
                let size = [img.width() as usize, img.height() as usize];
                let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                self.texture = Some(ctx.load_texture("preview", color, egui::TextureOptions::LINEAR));
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Render gagal: {e}")),
        }
    }

    fn export(&mut self) {
        let Some(job) = self.current_job() else {
            self.error = Some("Muat foto dulu sebelum export.".into());
            return;
        };
        match compose::render(&job).and_then(|img| compose::encode_png(&img)) {
            Ok(png) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .set_file_name("screenie.png")
                    .save_file()
                {
                    if let Err(e) = std::fs::write(path, png) {
                        self.error = Some(format!("Gagal simpan: {e}"));
                    } else {
                        self.error = None;
                    }
                }
            }
            Err(e) => self.error = Some(format!("Render gagal: {e}")),
        }
    }
}

/// Color swatches for the palette (name, RRGGBB).
const PALETTE: &[(&str, &str)] = &[
    ("Putih", "FFFFFF"),
    ("Merah", "E74C3C"),
    ("Hijau", "2ECC71"),
    ("Biru", "3498DB"),
    ("Kuning", "F1C40F"),
    ("Oranye", "E67E22"),
    ("Ungu", "9B59B6"),
    ("Toska", "1ABC9C"),
    ("Pink", "FF79C6"),
    ("Abu", "95A5A6"),
];

fn hex_to_color32(hex: &str) -> egui::Color32 {
    let v = u32::from_str_radix(hex, 16).unwrap_or(0xFF_FFFF);
    egui::Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

/// Largest centered crop of the given ratio (None = whole photo), source px.
fn centered_crop(pw: u32, ph: u32, ratio: Option<f32>) -> CropRect {
    let (pw, ph) = (pw as f64, ph as f64);
    match ratio {
        None => CropRect { x: 0.0, y: 0.0, w: pw, h: ph },
        Some(r) => {
            let r = r as f64;
            let mut w = pw;
            let mut h = w / r;
            if h > ph {
                h = ph;
                w = h * r;
            }
            CropRect { x: (pw - w) / 2.0, y: (ph - h) / 2.0, w, h }
        }
    }
}

/// Keep a crop box inside the photo, honoring the aspect ratio if locked.
fn clamp_crop(c: &mut CropRect, pw: u32, ph: u32, ratio: Option<f32>) {
    let (pw, ph) = (pw as f64, ph as f64);
    c.w = c.w.clamp(16.0, pw);
    c.h = c.h.clamp(16.0, ph);
    if let Some(r) = ratio {
        let r = r as f64;
        c.h = c.w / r;
        if c.h > ph {
            c.h = ph;
            c.w = c.h * r;
        }
        if c.w > pw {
            c.w = pw;
            c.h = c.w / r;
        }
    }
    c.x = c.x.clamp(0.0, (pw - c.w).max(0.0));
    c.y = c.y.clamp(0.0, (ph - c.h).max(0.0));
}

/// Decode a base64 image into an egui ColorImage (for the crop-edit view),
/// used when switching photo tabs.
fn decode_color_image(base64: &str) -> Option<egui::ColorImage> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(base64).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    ))
}

fn identity_filters() -> FilterValues {
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

fn anchor_label(a: Anchor) -> &'static str {
    match a {
        Anchor::Free => "Bebas",
        Anchor::KiriAtas => "Kiri Atas",
        Anchor::KiriBawah => "Kiri Bawah",
    }
}

fn bg_label(b: BgMode) -> &'static str {
    match b {
        BgMode::None => "Tidak ada",
        BgMode::Block => "Blok",
        BgMode::Mask => "Mask",
    }
}
