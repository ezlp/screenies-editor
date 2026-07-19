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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_paths: Vec<String>,
}

#[derive(Default)]
pub struct GalleryState {
    pub active_tab: GalleryTab,
    pub source_folder: Option<PathBuf>,
    pub source_items: Vec<Item>,
    pub finished_folder: Option<PathBuf>,
    pub finished_items: Vec<Item>,
    pub albums: Vec<Album>,
    pub selected_album_id: Option<String>,
    pub filter_by_album: bool,
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

    fn get_filtered_indices(&self) -> Vec<usize> {
        let items = match self.active_tab {
            GalleryTab::Sources => &self.source_items,
            GalleryTab::Edits => &self.finished_items,
        };
        if self.active_tab == GalleryTab::Edits && self.filter_by_album {
            if let Some(album_id) = &self.selected_album_id {
                if let Some(album) = self.albums.iter().find(|a| &a.id == album_id) {
                    return (0..items.len())
                        .filter(|&idx| {
                            let path_str = items[idx].path.to_string_lossy().into_owned();
                            album.image_paths.contains(&path_str)
                        })
                        .collect();
                }
            }
        }
        (0..items.len()).collect()
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

        if self.active_tab == GalleryTab::Edits {
            ui.horizontal(|ui| {
                // Left Column: Album Side Panel
                ui.allocate_ui_with_layout(
                    egui::vec2(240.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.albums_panel(ui);
                    },
                );

                ui.separator();

                // Right Column: Grid Area
                ui.vertical(|ui| {
                    self.grid_panel(ui);
                });
            });
        } else {
            self.grid_panel(ui);
        }

        // Popup preview overlay (open while an item is selected).
        self.preview_window(ui.ctx());
    }

    fn albums_panel(&mut self, ui: &mut egui::Ui) {
        let title_smart_albums = self.t("Smart Albums").to_string();
        let btn_create_album = format!("➕ {}", self.t("Buat Album Baru"));
        let default_album_title = self.t("Album Baru").to_string();
        let tooltip_delete = self.t("Hapus Album").to_string();

        ui.label(egui::RichText::new(title_smart_albums).strong().size(16.0));
        ui.add_space(4.0);

        if ui.button(btn_create_album).clicked() {
            let id = format!("album_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());
            self.albums.push(Album {
                id: id.clone(),
                title: default_album_title,
                description: String::new(),
                image_paths: Vec::new(),
            });
            self.selected_album_id = Some(id);
        }
        ui.add_space(8.0);

        // Pre-clone list of albums to avoid borrow checker overlap
        let album_summaries: Vec<(usize, String, String)> = self.albums
            .iter()
            .enumerate()
            .map(|(idx, a)| (idx, a.id.clone(), a.title.clone()))
            .collect();

        // Scrollable list of albums
        egui::ScrollArea::vertical()
            .id_salt("albums_scroll")
            .max_height(160.0)
            .show(ui, |ui| {
                let mut to_delete = None;
                for (idx, id, title) in album_summaries {
                    let is_selected = Some(&id) == self.selected_album_id.as_ref();
                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_selected, &title).clicked() {
                            self.selected_album_id = Some(id.clone());
                        }
                        if is_selected {
                            if ui.small_button("🗑").on_hover_text(&tooltip_delete).clicked() {
                                to_delete = Some(idx);
                            }
                        }
                    });
                }
                if let Some(idx) = to_delete {
                    self.albums.remove(idx);
                    self.selected_album_id = None;
                }
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Pre-evaluate translation strings for selected album details
        let label_judul = self.t("Judul Album").to_string();
        let label_deskripsi = self.t("Deskripsi Cerita").to_string();
        let label_filter = self.t("Filter berdasarkan album ini").to_string();
        let placeholder_select = self.t("Pilih atau buat album untuk mulai menyusun cerita.").to_string();

        // Details of selected album
        if let Some(album_id) = &self.selected_album_id {
            if let Some(album) = self.albums.iter_mut().find(|a| &a.id == album_id) {
                ui.label(label_judul);
                ui.text_edit_singleline(&mut album.title);
                ui.add_space(6.0);

                ui.label(label_deskripsi);
                ui.text_edit_multiline(&mut album.description);
                ui.add_space(6.0);

                ui.checkbox(&mut self.filter_by_album, label_filter);
                ui.add_space(4.0);
                ui.small(format!("{} gambar", album.image_paths.len()));
            }
        } else {
            ui.weak(placeholder_select);
        }
    }

    fn grid_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button(format!("{} {}", icons::ICON_FOLDER, self.t("Buka folder"))).clicked() {
                self.open_folder();
            }
            let active_folder = match self.active_tab {
                GalleryTab::Sources => &self.source_folder,
                GalleryTab::Edits => &self.finished_folder,
            };
            let indices = self.get_filtered_indices();
            if let Some(f) = active_folder {
                ui.small(format!("{} · {} gambar", f.display(), indices.len()));
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

        let indices = self.get_filtered_indices();

        if indices.is_empty() {
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
    }

    /// The thumbnail grid: vertical scroll, wraps to fill the width.
    fn grid(&mut self, ui: &mut egui::Ui) {
        let mut decoded = 0usize;
        let mut clicked: Option<usize> = None;
        let indices = self.get_filtered_indices();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for &i in &indices {
                        let path = match self.active_tab {
                            GalleryTab::Sources => self.source_items[i].path.clone(),
                            GalleryTab::Edits => self.finished_items[i].path.clone(),
                        };
                        // Decode a few uncached thumbnails per frame.
                        if !self.thumbs.contains_key(&path) && decoded < THUMBS_PER_FRAME {
                            self.load_thumb(ui.ctx(), &path);
                            decoded += 1;
                        }

                        ui.vertical(|ui| {
                            if self.thumb_cell(ui, i, &path) {
                                clicked = Some(i);
                            }

                            if self.active_tab == GalleryTab::Edits {
                                if let Some(album_id) = &self.selected_album_id {
                                    let path_str = path.to_string_lossy().into_owned();
                                    let mut in_album = false;
                                    if let Some(album) = self.albums.iter().find(|a| &a.id == album_id) {
                                        in_album = album.image_paths.contains(&path_str);
                                    }

                                    let mut check_val = in_album;
                                    if ui.checkbox(&mut check_val, self.t("Di dalam album")).changed() {
                                        if let Some(album) = self.albums.iter_mut().find(|a| &a.id == album_id) {
                                            if check_val {
                                                if !album.image_paths.contains(&path_str) {
                                                    album.image_paths.push(path_str);
                                                }
                                            } else {
                                                album.image_paths.retain(|p| p != &path_str);
                                            }
                                        }
                                    }
                                }
                            }
                        });
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
