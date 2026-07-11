//! ScreeniesEditor Native — 🧪 Phase-1 skeleton (docs/NATIVE-MIGRATION.md).
//!
//! What it proves: open a photo, type a chatlog, and the preview you see
//! is literally `screenies_core::render::compose::render()` — the exact
//! production export pipeline. No preview/export drift is possible.
//!
//! Known Phase-1 limits (by plan, not by accident): single chatlog block,
//! naive per-span layout without word-wrap, no crop/filter/sticker UI.
//! Phase 2 ports the layout engine into core so both shells share it.

use ab_glyph::{Font, ScaleFont};
use base64::Engine;
use eframe::egui;
use screenies_core::chatlog::{self, preset};
use screenies_core::render::{
    compose, CropRect, ExportBlock, ExportRow, ExportToken, FilterValues, RenderJob, Size,
};

const TEXT_SIZE: f32 = 22.0;
const MARGIN: f32 = 14.0;
const LINE_GAP: f32 = 1.22;

fn main() -> eframe::Result {
    eframe::run_native(
        "ScreeniesEditor Native (eksperimen)",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Default)]
struct App {
    photo_b64: Option<String>,
    photo_size: (u32, u32),
    chat: String,
    dirty: bool,
    preview: Option<egui::TextureHandle>,
    error: Option<String>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("controls").min_width(280.0).show(ctx, |ui| {
            ui.heading("ScreeniesEditor 🧪");
            ui.label("Shell egui — pipeline export asli sebagai preview.");
            ui.separator();

            if ui.button("📂 Buka Foto…").clicked() {
                self.open_photo();
            }
            ui.add_space(8.0);
            ui.label("Chatlog (preset JGRP):");
            if ui
                .add(egui::TextEdit::multiline(&mut self.chat).desired_rows(10).code_editor())
                .changed()
            {
                self.dirty = true;
            }
            if let Some(err) = &self.error {
                ui.colored_label(egui::Color32::LIGHT_RED, err);
            }
        });

        if self.dirty && self.photo_b64.is_some() {
            self.dirty = false;
            self.rerender(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| match &self.preview {
            Some(tex) => {
                let avail = ui.available_size();
                let size = tex.size_vec2();
                let scale = (avail.x / size.x).min(avail.y / size.y).min(1.0);
                ui.centered_and_justified(|ui| ui.image((tex.id(), size * scale)));
            }
            None => {
                ui.centered_and_justified(|ui| ui.label("Buka foto untuk mulai."));
            }
        });
    }
}

impl App {
    fn open_photo(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Gambar", &["png", "jpg", "jpeg", "webp", "bmp"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read(&path) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => {
                    self.photo_size = (img.width(), img.height());
                    self.photo_b64 =
                        Some(base64::engine::general_purpose::STANDARD.encode(&bytes));
                    self.dirty = true;
                    self.error = None;
                }
                Err(e) => self.error = Some(format!("Decode gagal: {e}")),
            },
            Err(e) => self.error = Some(format!("Baca file gagal: {e}")),
        }
    }

    /// Parse (core) → naive layout (Phase-1 local) → compose::render (core).
    fn rerender(&mut self, ctx: &egui::Context) {
        let Some(b64) = &self.photo_b64 else { return };
        let (w, h) = self.photo_size;

        let lines = chatlog::parse(&self.chat, &preset::jgrp());

        let rows = naive_rows(&lines);
        let job = RenderJob {
            image_base64: b64.clone(),
            crop: CropRect { x: 0.0, y: 0.0, w: w as f64, h: h as f64 },
            output: Size { w, h },
            stickers: vec![],
            filters: FilterValues {
                brightness: 100.0,
                grayscale: 0.0,
                sepia: 0.0,
                saturate: 100.0,
                contrast: 100.0,
            },
            font_family: "Arial".into(),
            text_size: TEXT_SIZE,
            stroke_width: 3.0,
            blocks: vec![ExportBlock { rows }],
        };

        match compose::render(&job) {
            Ok(img) => {
                let color = egui::ColorImage::from_rgba_unmultiplied(
                    [img.width() as usize, img.height() as usize],
                    img.as_raw(),
                );
                self.preview =
                    Some(ctx.load_texture("preview", color, egui::TextureOptions::LINEAR));
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Render: {e:?}")),
        }
    }
}

/// Phase-1 layout: one token per colored span, advance measured with the
/// same font family the renderer uses. No wrapping — Phase 2 moves the
/// real layout engine (buildRenderBlocks) into screenies-core.
fn naive_rows(lines: &[chatlog::ParsedLine]) -> Vec<ExportRow> {
    let advance_px = measure_setup();
    let mut rows = Vec::new();
    let mut y = MARGIN;
    for line in lines {
        let mut tokens = Vec::new();
        let mut x = MARGIN;
        for span in &line.spans {
            let width = advance_px(&span.text, span.bold);
            tokens.push(ExportToken {
                text: span.text.clone(),
                x,
                color: span.color.clone(),
                bold: span.bold,
            });
            x += width;
        }
        rows.push(ExportRow { y, tokens, bg: None });
        y += TEXT_SIZE * LINE_GAP;
    }
    rows
}

/// Tiny measurer over the shared font DB (bold Arial-ish, like the shell).
fn measure_setup() -> impl Fn(&str, bool) -> f32 {
    let db = screenies_core::fonts::database();
    let q = fontdb::Query {
        families: &[fontdb::Family::Name("Arial"), fontdb::Family::SansSerif],
        weight: fontdb::Weight::BOLD,
        ..Default::default()
    };
    let face = q_bytes(db, &q);
    move |text: &str, _bold: bool| {
        let Some((bytes, index)) = &face else {
            return text.len() as f32 * TEXT_SIZE * 0.55; // crude fallback
        };
        let Ok(font) = ab_glyph::FontRef::try_from_slice_and_index(bytes, *index) else {
            return text.len() as f32 * TEXT_SIZE * 0.55;
        };
        let scaled = font.as_scaled(ab_glyph::PxScale::from(TEXT_SIZE));
        text.chars().map(|c| scaled.h_advance(scaled.glyph_id(c))).sum()
    }
}

fn q_bytes(db: &fontdb::Database, q: &fontdb::Query) -> Option<(Vec<u8>, u32)> {
    let id = db.query(q)?;
    db.with_face_data(id, |data, index| (data.to_vec(), index))
}
