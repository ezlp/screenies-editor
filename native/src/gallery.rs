// gallery.rs — Phase 4 screen: browse exported SSRP photos in a folder and
// open one back in the editor. Folder listing is core::gallery (unit-tested);
// this decodes a preview of the selected image on demand.

use eframe::egui;
use screenies_core::gallery::{self, Item};
use std::path::PathBuf;

#[derive(Default)]
pub struct GalleryState {
    folder: Option<PathBuf>,
    items: Vec<Item>,
    selected: Option<usize>,
    /// Cached preview (path it was decoded from + its texture).
    preview: Option<(PathBuf, egui::TextureHandle)>,
    /// Set when the user clicks "open in editor"; the App consumes it.
    pub open_request: Option<PathBuf>,
    error: Option<String>,
}

impl GalleryState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("📂  Buka folder").clicked() {
                self.open_folder();
            }
            if let Some(f) = &self.folder {
                ui.small(format!("{} · {} gambar", f.display(), self.items.len()));
            } else {
                ui.small("Pilih folder berisi foto hasil edit.");
            }
        });
        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
        }
        ui.separator();

        egui::SidePanel::left("gallery-list")
            .resizable(true)
            .default_width(240.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..self.items.len() {
                        if ui
                            .selectable_label(self.selected == Some(i), &self.items[i].name)
                            .clicked()
                        {
                            self.selected = Some(i);
                        }
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let Some(i) = self.selected else {
                ui.centered_and_justified(|ui| ui.label("Pilih gambar dari daftar."));
                return;
            };
            if i >= self.items.len() {
                return;
            }
            ui.horizontal(|ui| {
                ui.strong(self.items[i].name.clone());
                if ui.button("✏  Buka di editor").clicked() {
                    self.open_request = Some(self.items[i].path.clone());
                }
            });
            self.ensure_preview(ui.ctx(), i);
            if let Some((_, tex)) = &self.preview {
                let avail = ui.available_size();
                let sz = tex.size_vec2();
                let scale = (avail.x / sz.x).min(avail.y / sz.y).min(1.0);
                ui.centered_and_justified(|ui| {
                    ui.image(egui::load::SizedTexture::new(tex.id(), sz * scale));
                });
            }
        });
    }

    fn open_folder(&mut self) {
        let Some(dir) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        match gallery::list_folder(&dir) {
            Ok(items) => {
                self.items = items;
                self.folder = Some(dir);
                self.selected = None;
                self.preview = None;
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Gagal baca folder: {e}")),
        }
    }

    /// Decode a (downscaled) preview of item `i`, unless already cached.
    fn ensure_preview(&mut self, ctx: &egui::Context, i: usize) {
        let path = self.items[i].path.clone();
        if self.preview.as_ref().map(|(p, _)| p) == Some(&path) {
            return;
        }
        match image::open(&path) {
            Ok(img) => {
                let rgba = img.thumbnail(1400, 1400).to_rgba8();
                let ci = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                let tex = ctx.load_texture("gallery-preview", ci, egui::TextureOptions::LINEAR);
                self.preview = Some((path, tex));
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Gagal buka gambar: {e}"));
                self.preview = None;
            }
        }
    }
}
