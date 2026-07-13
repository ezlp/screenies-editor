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
use screenies_core::chatlog::{self, preset::ParsePreset};
use screenies_core::render::compose;
use screenies_core::render::layout::{self, Anchor, BgMode, LayoutBlock, LayoutParams};
use screenies_core::render::text::GlyphMeasure;
use screenies_core::render::{CensorKind, CensorRegion, CropRect, FilterValues, RenderJob, Size};

/// A loaded photo: base64 for the render pipeline + its pixel dimensions.
struct Photo {
    base64: String,
    w: u32,
    h: u32,
}

pub struct EditorState {
    preset: ParsePreset,
    photo: Option<Photo>,
    chatlog_text: String,

    // Text controls.
    font_family: String,
    text_size: f32,
    line_gap: f32,
    stroke_auto: bool,
    stroke_width: f32,

    // Placement + background.
    anchor: Anchor,
    bg_mode: BgMode,
    block_x: f32,
    block_y: f32,

    filters: FilterValues,

    // Local censor boxes (blur/pixelate a region), placed + resized on the preview.
    censors: Vec<CensorRegion>,
    selected_censor: Option<usize>,

    // Live preview: re-rendered only when `dirty`.
    dirty: bool,
    texture: Option<egui::TextureHandle>,
    error: Option<String>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            preset: chatlog::preset::jgrp(),
            photo: None,
            chatlog_text: String::new(),
            font_family: "Verdana".into(),
            text_size: 27.0,
            line_gap: 122.0,
            stroke_auto: true,
            stroke_width: 3.0,
            anchor: Anchor::KiriAtas,
            bg_mode: BgMode::None,
            block_x: 40.0,
            block_y: 40.0,
            filters: identity_filters(),
            censors: Vec::new(),
            selected_censor: None,
            dirty: false,
            texture: None,
            error: None,
        }
    }
}

impl EditorState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(320.0)
            .show_inside(ui, |ui| self.controls(ui));

        egui::CentralPanel::default().show_inside(ui, |ui| self.preview(ui));

        // Re-render after the UI ran, so this frame's edits are included.
        if self.dirty {
            self.refresh(ui.ctx());
        }
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(6.0);
            if ui.button("📂  Muat Foto").clicked() {
                self.pick_photo();
            }
            if let Some(p) = &self.photo {
                ui.small(format!("{}×{} px", p.w, p.h));
            } else {
                ui.small("Belum ada foto.");
            }

            ui.separator();
            ui.label("Chatlog");
            if ui
                .add(
                    egui::TextEdit::multiline(&mut self.chatlog_text)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .hint_text("[12:34:56] Budi_Santoso says: contoh chat"),
                )
                .changed()
            {
                self.dirty = true;
            }

            ui.separator();
            ui.label("Teks");
            ui.horizontal(|ui| {
                ui.label("Font");
                if ui.text_edit_singleline(&mut self.font_family).changed() {
                    self.dirty = true;
                }
            });
            if ui
                .add(egui::Slider::new(&mut self.text_size, 8.0..=60.0).text("Ukuran"))
                .changed()
            {
                self.dirty = true;
            }
            if ui
                .add(egui::Slider::new(&mut self.line_gap, 80.0..=200.0).text("Jarak baris %"))
                .changed()
            {
                self.dirty = true;
            }
            if ui.checkbox(&mut self.stroke_auto, "Outline otomatis").changed() {
                self.dirty = true;
            }
            if !self.stroke_auto
                && ui
                    .add(egui::Slider::new(&mut self.stroke_width, 0.0..=10.0).text("Outline px"))
                    .changed()
            {
                self.dirty = true;
            }

            ui.separator();
            ui.label("Posisi & Background");
            self.combo_anchor(ui);
            self.combo_bg(ui);

            ui.separator();
            ui.collapsing("Filter", |ui| {
                self.filter_slider(ui, "Brightness", 0.0..=300.0, |f| &mut f.brightness);
                self.filter_slider(ui, "Contrast", 0.0..=200.0, |f| &mut f.contrast);
                self.filter_slider(ui, "Grayscale", 0.0..=100.0, |f| &mut f.grayscale);
                self.filter_slider(ui, "Sepia", 0.0..=100.0, |f| &mut f.sepia);
                self.filter_slider(ui, "Saturate", 0.0..=300.0, |f| &mut f.saturate);
            });

            ui.collapsing("Efek (2.0)", |ui| {
                // Global neighborhood effects — new in 2.0 (core render::filters).
                self.filter_slider(ui, "Blur (px)", 0.0..=20.0, |f| &mut f.blur);
                self.filter_slider(ui, "Pixelate (blok px)", 0.0..=64.0, |f| &mut f.pixelate);
            });

            ui.collapsing("Sensor area (blur/pixelate lokal)", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("+ Blur").clicked() {
                        self.add_censor(CensorKind::Blur);
                    }
                    if ui.button("+ Pixelate").clicked() {
                        self.add_censor(CensorKind::Pixelate);
                    }
                });
                ui.small("Klik kotak di preview untuk pilih · seret badan untuk geser · seret pojok untuk resize.");

                if let Some(i) = self.selected_censor {
                    if i < self.censors.len() {
                        let kind = self.censors[i].kind;
                        let label = match kind {
                            CensorKind::Blur => "Blur radius (px)",
                            CensorKind::Pixelate => "Blok (px)",
                        };
                        let mut strength = self.censors[i].strength;
                        if ui.add(egui::Slider::new(&mut strength, 1.0..=64.0).text(label)).changed() {
                            self.censors[i].strength = strength;
                            self.dirty = true;
                        }
                        if ui.button("🗑 Hapus area").clicked() {
                            self.censors.remove(i);
                            self.selected_censor = None;
                            self.dirty = true;
                        }
                    }
                }
                ui.small(format!("{} area sensor", self.censors.len()));
            });

            ui.separator();
            if ui.button("💾  Export PNG").clicked() {
                self.export();
            }
            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
            }
        });
    }

    fn combo_anchor(&mut self, ui: &mut egui::Ui) {
        let prev = self.anchor;
        egui::ComboBox::from_label("Posisi")
            .selected_text(anchor_label(self.anchor))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.anchor, Anchor::Free, "Bebas");
                ui.selectable_value(&mut self.anchor, Anchor::KiriAtas, "Kiri Atas");
                ui.selectable_value(&mut self.anchor, Anchor::KiriBawah, "Kiri Bawah");
            });
        if self.anchor != prev {
            self.dirty = true;
        }
        if self.anchor == Anchor::Free {
            let a = ui.add(egui::Slider::new(&mut self.block_x, 0.0..=2000.0).text("X")).changed();
            let b = ui.add(egui::Slider::new(&mut self.block_y, 0.0..=2000.0).text("Y")).changed();
            if a || b {
                self.dirty = true;
            }
        }
    }

    fn combo_bg(&mut self, ui: &mut egui::Ui) {
        let prev = self.bg_mode;
        egui::ComboBox::from_label("Background")
            .selected_text(bg_label(self.bg_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.bg_mode, BgMode::None, "Tidak ada");
                ui.selectable_value(&mut self.bg_mode, BgMode::Block, "Blok");
                ui.selectable_value(&mut self.bg_mode, BgMode::Mask, "Mask");
            });
        if self.bg_mode != prev {
            self.dirty = true;
        }
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
        let Some(tex) = self.texture.as_ref() else {
            ui.centered_and_justified(|ui| {
                ui.label("Muat foto untuk mulai mengedit.");
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
                egui::Color32::from_rgb(120, 200, 255)
            } else {
                egui::Color32::from_rgb(210, 210, 210)
            };
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(if selected { 2.0 } else { 1.0 }, color));
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
    }

    fn pick_photo(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Gambar", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        {
            match std::fs::read(&path) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        self.photo = Some(Photo {
                            base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
                            w: img.width(),
                            h: img.height(),
                        });
                        self.error = None;
                        self.dirty = true;
                    }
                    Err(e) => self.error = Some(format!("Gagal decode gambar: {e}")),
                },
                Err(e) => self.error = Some(format!("Gagal baca file: {e}")),
            }
        }
    }

    /// Add a censor box centered on the photo (sized relative to it).
    fn add_censor(&mut self, kind: CensorKind) {
        let (ow, oh) = match &self.photo {
            Some(p) => (p.w as f32, p.h as f32),
            None => (400.0, 300.0),
        };
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
        let output = Size { w: photo.w, h: photo.h };

        let blocks = if self.chatlog_text.trim().is_empty() {
            Vec::new()
        } else if let Ok(measure) = GlyphMeasure::new(&self.font_family, self.text_size) {
            let params = LayoutParams {
                text_size: self.text_size,
                line_gap: self.line_gap,
                bg_offset: 0.0,
                output_w: photo.w as f32,
                output_h: photo.h as f32,
            };
            let block = LayoutBlock {
                lines: chatlog::parse(&self.chatlog_text, &self.preset),
                anchor: self.anchor,
                bg_mode: self.bg_mode,
                x: self.block_x,
                y: self.block_y,
            };
            layout::layout_blocks(&[block], &params, &measure)
                .into_iter()
                .map(|l| l.block)
                .collect()
        } else {
            Vec::new()
        };

        Some(RenderJob {
            image_base64: photo.base64.clone(),
            crop: CropRect { x: 0.0, y: 0.0, w: photo.w as f64, h: photo.h as f64 },
            output,
            stickers: Vec::new(),
            filters: self.filters,
            censors: self.censors.clone(),
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
        match compose::render(&job) {
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
