// gallery.rs — Phase 4 screen: browse exported SSRP photos as a thumbnail GRID.
// Folder listing is core::gallery (unit-tested). Thumbnails are decoded lazily
// (a few per frame) and cached; clicking a thumbnail opens a popup preview with
// an "open in editor" action.

use eframe::egui;
use screenies_core::gallery::{self, Item};
use crate::icons;
use std::collections::HashMap;
use std::path::PathBuf;

/// Thumbnails decoded per frame — keeps a big folder responsive while it fills.
const THUMBS_PER_FRAME: usize = 6;
/// Thumbnail cell box (px).
const THUMB_W: f32 = 150.0;
const THUMB_H: f32 = 112.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum GalleryTab {
    #[default]
    Sources,
    Edits,
}

#[derive(Default)]
pub struct GalleryState {
    pub active_tab: GalleryTab,
    pub source_folder: Option<PathBuf>,
    pub source_items: Vec<Item>,
    pub finished_folder: Option<PathBuf>,
    pub finished_items: Vec<Item>,
    /// Cached grid thumbnails (None = decode failed, don't retry).
    pub thumbs: HashMap<PathBuf, Option<egui::TextureHandle>>,
    /// Keep track of insertion order to evict old thumbnails (garbage collection).
    thumb_order: Vec<PathBuf>,
    /// Item whose popup preview is open.
    selected: Option<usize>,
    /// Cached popup preview (path it was decoded from + its texture).
    preview: Option<(PathBuf, egui::TextureHandle)>,
    /// Set when the user clicks "open in editor"; the App consumes it.
    pub open_request: Option<PathBuf>,
    error: Option<String>,
    pub lang: crate::i18n::Lang,
}

impl GalleryState {
    fn t(&self, s: &'static str) -> &'static str {
        crate::i18n::t(self.lang, s)
    }

    /// Get the persisted gallery folder path (for Settings).
    pub fn gallery_folder(&self) -> Option<String> {
        self.finished_folder.as_ref().map(|p| p.to_string_lossy().into_owned())
    }

    /// Set the gallery folder path (for Settings load).
    pub fn set_gallery_folder(&mut self, path: Option<String>) {
        self.finished_folder = path.filter(|s| !s.is_empty()).map(PathBuf::from);
        if self.finished_folder.is_some() {
            self.rescan_edits();
        }
    }

    /// Get the persisted source folder path.
    pub fn source_shots_folder(&self) -> Option<String> {
        self.source_folder.as_ref().map(|p| p.to_string_lossy().into_owned())
    }

    /// Set the source folder path.
    pub fn set_source_shots_folder(&mut self, path: Option<String>) {
        self.source_folder = path.filter(|s| !s.is_empty()).map(PathBuf::from);
        if self.source_folder.is_some() {
            self.rescan_sources();
        }
    }

    pub fn rescan(&mut self) {
        match self.active_tab {
            GalleryTab::Sources => self.rescan_sources(),
            GalleryTab::Edits => self.rescan_edits(),
        }
    }

    fn rescan_edits(&mut self) {
        self.finished_items.clear();
        self.selected = None;
        self.preview = None;
        let Some(dir) = &self.finished_folder else { return };
        match gallery::list_folder(dir) {
            Ok(items) => {
                self.finished_items = items;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Gagal baca folder: {e}")),
        }
    }

    fn rescan_sources(&mut self) {
        self.source_items.clear();
        self.selected = None;
        self.preview = None;
        let Some(dir) = &self.source_folder else { return };
        match gallery::list_folder(dir) {
            Ok(items) => {
                self.source_items = items;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Gagal baca folder: {e}")),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Draw Tab Selector
        let label_sources = format!("{} {}", icons::ICON_FOLDER, self.t("Source Shots"));
        let label_edits = format!("{} {}", icons::ICON_IMAGE, self.t("Finished Edits"));
        ui.horizontal(|ui| {
            let resp_sources = ui.selectable_value(&mut self.active_tab, GalleryTab::Sources, label_sources);
            let resp_edits = ui.selectable_value(&mut self.active_tab, GalleryTab::Edits, label_edits);
            if resp_sources.changed() || resp_edits.changed() {
                self.selected = None;
                self.preview = None;
            }
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            if ui.button(format!("{} {}", icons::ICON_FOLDER, self.t("Buka folder"))).clicked() {
                self.open_folder();
            }
            let active_folder = match self.active_tab {
                GalleryTab::Sources => &self.source_folder,
                GalleryTab::Edits => &self.finished_folder,
            };
            let items_len = match self.active_tab {
                GalleryTab::Sources => self.source_items.len(),
                GalleryTab::Edits => self.finished_items.len(),
            };
            if let Some(f) = active_folder {
                ui.small(format!("{} · {} gambar", f.display(), items_len));
            } else {
                let msg = match self.active_tab {
                    GalleryTab::Sources => self.t("Pilih folder berisi screenshot mentah."),
                    GalleryTab::Edits => self.t("Pilih folder berisi foto hasil edit."),
                };
                ui.small(msg);
            }
        });
        if let Some(err) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, err);
        }
        ui.separator();

        let items_len = match self.active_tab {
            GalleryTab::Sources => self.source_items.len(),
            GalleryTab::Edits => self.finished_items.len(),
        };

        if items_len == 0 {
            let msg = match self.active_tab {
                GalleryTab::Sources => self.t("Pilih folder berisi screenshot mentah."),
                GalleryTab::Edits => self.t("Pilih folder berisi foto hasil edit."),
            };
            ui.centered_and_justified(|ui| {
                ui.label(msg);
            });
        } else {
            self.grid(ui);
        }

        // Popup preview overlay (open while an item is selected).
        self.preview_window(ui.ctx());
    }

    /// The thumbnail grid: vertical scroll, wraps to fill the width.
    fn grid(&mut self, ui: &mut egui::Ui) {
        let mut decoded = 0usize;
        let mut clicked: Option<usize> = None;
        let items_len = match self.active_tab {
            GalleryTab::Sources => self.source_items.len(),
            GalleryTab::Edits => self.finished_items.len(),
        };
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..items_len {
                        let path = match self.active_tab {
                            GalleryTab::Sources => self.source_items[i].path.clone(),
                            GalleryTab::Edits => self.finished_items[i].path.clone(),
                        };
                        // Decode a few uncached thumbnails per frame.
                        if !self.thumbs.contains_key(&path) && decoded < THUMBS_PER_FRAME {
                            self.load_thumb(ui.ctx(), &path);
                            decoded += 1;
                        }
                        if self.thumb_cell(ui, i, &path) {
                            clicked = Some(i);
                        }
                    }
                });
            });
        // Still decoding? Ask for another frame so the grid keeps filling.
        if decoded == THUMBS_PER_FRAME {
            ui.ctx().request_repaint();
        }
        if let Some(i) = clicked {
            self.selected = Some(i);
        }
    }

    /// One grid cell: a clickable thumbnail (or a placeholder while it decodes),
    /// with the filename as a tooltip. Returns true when clicked.
    fn thumb_cell(&self, ui: &mut egui::Ui, i: usize, path: &PathBuf) -> bool {
        let name = match self.active_tab {
            GalleryTab::Sources => self.source_items[i].name.clone(),
            GalleryTab::Edits => self.finished_items[i].name.clone(),
        };
        let resp = match self.thumbs.get(path) {
            Some(Some(tex)) => {
                let s = tex.size_vec2();
                let scale = (THUMB_W / s.x).min(THUMB_H / s.y).min(1.0);
                let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), s * scale));
                ui.add(egui::ImageButton::new(img))
            }
            _ => ui.add_sized([THUMB_W, THUMB_H], egui::Button::new("…")),
        };
        resp.on_hover_text(name).clicked()
    }

    fn open_folder(&mut self) {
        let Some(dir) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        match gallery::list_folder(&dir) {
            Ok(items) => {
                match self.active_tab {
                    GalleryTab::Sources => {
                        self.source_items = items;
                        self.source_folder = Some(dir);
                    }
                    GalleryTab::Edits => {
                        self.finished_items = items;
                        self.finished_folder = Some(dir);
                    }
                }
                self.selected = None;
                self.preview = None;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Gagal baca folder: {e}")),
        }
    }

    /// Decode + cache a small grid thumbnail for `path`.
    pub fn load_thumb(&mut self, ctx: &egui::Context, path: &PathBuf) {
        // Enforce garbage collection: cap thumbnail cache to 128 items
        const MAX_THUMBS: usize = 128;
        if !self.thumbs.contains_key(path) {
            self.thumb_order.push(path.clone());
            if self.thumb_order.len() > MAX_THUMBS {
                let oldest = self.thumb_order.remove(0);
                self.thumbs.remove(&oldest);
            }
        }

        let tex = match image::open(path) {
            Ok(img) => {
                let rgba = img.thumbnail(256, 256).to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                Some(ctx.load_texture(
                    format!("thumb:{}", path.display()),
                    ci,
                    egui::TextureOptions::LINEAR,
                ))
            }
            Err(_) => None,
        };
        self.thumbs.insert(path.clone(), tex);
    }

    /// Popup preview window for the selected item (no-op when none selected).
    fn preview_window(&mut self, ctx: &egui::Context) {
        let Some(i) = self.selected else {
            return;
        };
        let items_len = match self.active_tab {
            GalleryTab::Sources => self.source_items.len(),
            GalleryTab::Edits => self.finished_items.len(),
        };
        if i >= items_len {
            self.selected = None;
            return;
        }
        let (name, path) = match self.active_tab {
            GalleryTab::Sources => (self.source_items[i].name.clone(), self.source_items[i].path.clone()),
            GalleryTab::Edits => (self.finished_items[i].name.clone(), self.finished_items[i].path.clone()),
        };
        let open_lbl = format!("{} {}", icons::ICON_PENCIL, self.t("Buka di editor"));
        let mut open = true;
        egui::Window::new(format!("{} {name}", icons::ICON_IMAGE))
            .open(&mut open)
            .collapsible(false)
            .default_width(760.0)
            .default_height(560.0)
            .show(ctx, |ui| {
                if ui.button(open_lbl).clicked() {
                    self.open_request = Some(path.clone());
                }
                ui.separator();
                self.ensure_preview(ctx, &path);
                if let Some((_, tex)) = &self.preview {
                    let avail = ui.available_size();
                    let s = tex.size_vec2();
                    let scale = (avail.x / s.x).min(avail.y / s.y).min(1.0);
                    ui.centered_and_justified(|ui| {
                        ui.image(egui::load::SizedTexture::new(tex.id(), s * scale));
                    });
                } else if let Some(err) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, err);
                }
            });
        // Close on the window ✕, or after jumping to the editor.
        if !open || self.open_request.is_some() {
            self.selected = None;
        }
    }

    /// Decode a larger preview of `path` for the popup, unless already cached.
    fn ensure_preview(&mut self, ctx: &egui::Context, path: &PathBuf) {
        if self.preview.as_ref().map(|(p, _)| p) == Some(path) {
            return;
        }
        match image::open(path) {
            Ok(img) => {
                let rgba = img.thumbnail(1600, 1600).to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                let tex = ctx.load_texture("gallery-preview", ci, egui::TextureOptions::LINEAR);
                self.preview = Some((path.clone(), tex));
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Gagal buka gambar: {e}"));
                self.preview = None;
            }
        }
    }
}
