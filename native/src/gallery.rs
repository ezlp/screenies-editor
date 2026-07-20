// gallery.rs — Phase 4 screen: browse exported SSRP photos as a thumbnail GRID.
// Folder listing is core::gallery (unit-tested). Thumbnails are decoded lazily
// (a few per frame) and cached; clicking a thumbnail opens a popup preview with
// an "open in editor" action.

use eframe::egui;
use screenies_core::gallery::{self, Item};
use crate::icons;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadStatus {
    Uploading,
    Success(String),
    Error(String),
}

/// Thumbnails decoded per frame — keeps a big folder responsive while it fills.
const THUMBS_PER_FRAME: usize = 6;
/// Thumbnail grid cell box (px).
const THUMB_W: f32 = 160.0;
const THUMB_H: f32 = 120.0;

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
    pub uploaded_links: HashMap<String, String>,
    pub imgbb_api_key: Option<String>,
    pub upload_status: Arc<Mutex<HashMap<PathBuf, UploadStatus>>>,
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

    #[allow(dead_code)]
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
        ui.horizontal(|ui| {
            // Left Column: Navigation Sidebar & Storyline Albums (260.0 px)
            ui.allocate_ui_with_layout(
                egui::vec2(260.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.sidebar_panel(ui);
                },
            );

            ui.separator();

            // Right Column: Photo Grid Area
            ui.vertical(|ui| {
                self.grid_panel(ui);
            });
        });

        // Popup preview overlay (open while an item is selected).
        self.preview_window(ui.ctx());
    }

    fn sidebar_panel(&mut self, ui: &mut egui::Ui) {
        // Section 1: Folder Selector
        ui.label(egui::RichText::new(self.t("Lokasi Foto")).strong().size(14.0));
        ui.add_space(4.0);

        let label_sources = format!("{} {}", icons::ICON_FOLDER, self.t("Source Shots"));
        let label_edits = format!("{} {}", icons::ICON_IMAGE, self.t("Finished Edits"));

        let resp_sources = ui.selectable_value(&mut self.active_tab, GalleryTab::Sources, label_sources);
        let resp_edits = ui.selectable_value(&mut self.active_tab, GalleryTab::Edits, label_edits);

        if resp_sources.changed() || resp_edits.changed() {
            self.selected = None;
            self.preview = None;
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);

        // Section 2: Smart Albums (for Finished Edits)
        if self.active_tab == GalleryTab::Edits {
            self.albums_panel(ui);
        } else {
            ui.weak(self.t("Beralih ke Finished Edits untuk mengelola Smart Albums cerita."));
        }
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

    /// One grid cell: a clean clickable thumbnail with small status indicators.
    fn thumb_cell(&self, ui: &mut egui::Ui, i: usize, path: &PathBuf) -> bool {
        let name = match self.active_tab {
            GalleryTab::Sources => self.source_items[i].name.clone(),
            GalleryTab::Edits => self.finished_items[i].name.clone(),
        };

        let path_str = path.to_string_lossy().into_owned();
        let is_uploaded = self.uploaded_links.contains_key(&path_str);
        let in_selected_album = if let Some(album_id) = &self.selected_album_id {
            self.albums.iter().find(|a| &a.id == album_id).map_or(false, |a| a.image_paths.contains(&path_str))
        } else {
            false
        };

        let mut clicked = false;
        ui.vertical_centered(|ui| {
            let resp = match self.thumbs.get(path) {
                Some(Some(tex)) => {
                    let s = tex.size_vec2();
                    let scale = (THUMB_W / s.x).min(THUMB_H / s.y).min(1.0);
                    let img = egui::Image::new(egui::load::SizedTexture::new(tex.id(), s * scale));
                    ui.add(egui::ImageButton::new(img))
                }
                _ => ui.add_sized([THUMB_W, THUMB_H], egui::Button::new("…")),
            };

            let mut badge = String::new();
            if is_uploaded {
                badge.push_str("☁ ");
            }
            if in_selected_album {
                badge.push_str("🏷 ");
            }
            let label_text = format!("{badge}{name}");

            ui.add_sized([THUMB_W, 20.0], egui::Label::new(egui::RichText::new(label_text).weak().size(11.0)).truncate());

            if resp.on_hover_text(&name).clicked() {
                clicked = true;
            }
        });

        clicked
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

    /// Spacious Inspector window for the selected item (no-op when none selected).
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
        let path_str = path.to_string_lossy().into_owned();

        // Sync background upload status updates
        let status = {
            let map = self.upload_status.lock().unwrap();
            map.get(&path).cloned()
        };
        if let Some(UploadStatus::Success(ref new_url)) = status {
            self.uploaded_links.insert(path_str.clone(), new_url.clone());
        }

        let open_lbl = format!("{} {}", icons::ICON_PENCIL, self.t("Buka di editor"));
        let mut open = true;

        egui::Window::new(format!("🖼 {name}"))
            .open(&mut open)
            .collapsible(false)
            .default_width(840.0)
            .default_height(580.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Left Column: Image Preview
                    ui.allocate_ui_with_layout(
                        egui::vec2(540.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
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
                        },
                    );

                    ui.separator();

                    // Right Column: Inspector Controls
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&name).strong().size(14.0));
                        ui.small(path.display().to_string());
                        ui.add_space(8.0);

                        if ui.button(open_lbl).clicked() {
                            self.open_request = Some(path.clone());
                        }
                        ui.add_space(8.0);

                        // ImgBB Cloud Uploader Section
                        ui.label(egui::RichText::new("ImgBB Cloud").strong());
                        if let Some(url) = self.uploaded_links.get(&path_str).cloned() {
                            ui.small("Raw Direct URL:");
                            ui.horizontal(|ui| {
                                let mut temp_url = url.clone();
                                ui.add(egui::TextEdit::singleline(&mut temp_url).desired_width(180.0));
                                if ui.small_button("📋").on_hover_text(self.t("Salin tautan")).clicked() {
                                    ui.ctx().copy_text(url);
                                }
                            });
                        } else {
                            match status {
                                Some(UploadStatus::Uploading) => {
                                    ui.horizontal(|ui| {
                                        ui.spinner();
                                        ui.small("Uploading...");
                                    });
                                }
                                Some(UploadStatus::Error(ref err_msg)) => {
                                    let has_key = self.imgbb_api_key.as_ref().map_or(false, |k| !k.is_empty());
                                    if ui.add_enabled(has_key, egui::Button::new("📤 Retry Upload")).on_hover_text(err_msg).clicked() {
                                        if let Some(key) = &self.imgbb_api_key {
                                            perform_upload(key.clone(), path.clone(), self.upload_status.clone());
                                        }
                                    }
                                }
                                _ => {
                                    let has_key = self.imgbb_api_key.as_ref().map_or(false, |k| !k.is_empty());
                                    let tooltip = if has_key {
                                        "Upload screenshot to ImgBB"
                                    } else {
                                        self.t("Konfigurasi API Key ImgBB di Pengaturan untuk mengunggah")
                                    };
                                    if ui.add_enabled(has_key, egui::Button::new("📤 Unggah ke ImgBB")).on_hover_text(tooltip).clicked() {
                                        if let Some(key) = &self.imgbb_api_key {
                                            perform_upload(key.clone(), path.clone(), self.upload_status.clone());
                                        }
                                    }
                                }
                            }
                        }

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Storyline Albums Section
                        if self.active_tab == GalleryTab::Edits && !self.albums.is_empty() {
                            ui.label(egui::RichText::new("Smart Albums").strong());
                            for album in self.albums.iter_mut() {
                                let mut in_album = album.image_paths.contains(&path_str);
                                if ui.checkbox(&mut in_album, &album.title).changed() {
                                    if in_album {
                                        if !album.image_paths.contains(&path_str) {
                                            album.image_paths.push(path_str.clone());
                                        }
                                    } else {
                                        album.image_paths.retain(|p| p != &path_str);
                                    }
                                }
                            }
                        }
                    });
                });
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

fn perform_upload(api_key: String, path: PathBuf, status_map: Arc<Mutex<HashMap<PathBuf, UploadStatus>>>) {
    std::thread::spawn(move || {
        {
            let mut map = status_map.lock().unwrap();
            map.insert(path.clone(), UploadStatus::Uploading);
        }

        let file_bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(e) => {
                let mut map = status_map.lock().unwrap();
                map.insert(path, UploadStatus::Error(format!("Read error: {e}")));
                return;
            }
        };

        use base64::Engine;
        let b64_image = base64::engine::general_purpose::STANDARD.encode(&file_bytes);

        #[derive(serde::Deserialize)]
        struct ImgbbResponse {
            data: ImgbbData,
            success: bool,
        }

        #[derive(serde::Deserialize)]
        struct ImgbbData {
            url: String,
        }

        let url = "https://api.imgbb.com/1/upload";
        let res = ureq::post(url)
            .query("key", &api_key)
            .send_form(&[("image", &b64_image)]);

        match res {
            Ok(resp) => {
                match resp.into_json::<ImgbbResponse>() {
                    Ok(parsed) => {
                        if parsed.success {
                            let mut map = status_map.lock().unwrap();
                            map.insert(path, UploadStatus::Success(parsed.data.url));
                        } else {
                            let mut map = status_map.lock().unwrap();
                            map.insert(path, UploadStatus::Error("Upload failed".to_string()));
                        }
                    }
                    Err(e) => {
                        let mut map = status_map.lock().unwrap();
                        map.insert(path, UploadStatus::Error(format!("JSON parse error: {e}")));
                    }
                }
            }
            Err(e) => {
                let mut map = status_map.lock().unwrap();
                map.insert(path, UploadStatus::Error(format!("HTTP request error: {e}")));
            }
        }
    });
}
