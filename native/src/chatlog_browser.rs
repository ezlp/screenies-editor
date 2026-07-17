// chatlog_browser.rs — the chatlog-folder feature (replaces the old Chatlog
// Parser screen). Point it at a folder of chatlog .log/.txt files (remembered
// across launches); open one to preview it with SA-MP {RRGGBB} color codes
// rendered as real colors, and copy the raw text to paste into an editor block.

use eframe::egui;
use std::fs;
use std::path::PathBuf;

const EXTS: &[&str] = &["log", "txt"];

#[derive(Default)]
pub struct ChatlogBrowser {
    folder: Option<PathBuf>,
    files: Vec<PathBuf>,
    content: Option<(String, String)>, // (filename, text)
    /// Colored preview of `content`, parsed once when a file loads instead of
    /// rebuilt every frame the popup repaints (scroll/hover). `None` = no file.
    preview_job: Option<egui::text::LayoutJob>,
    open: bool,
    error: Option<String>,
}

impl ChatlogBrowser {
    pub fn open(&mut self) {
        self.open = true;
        if self.folder.is_some() && self.files.is_empty() {
            self.rescan();
        }
    }

    /// Persisted folder path (get/set for Settings).
    pub fn folder_path(&self) -> Option<String> {
        self.folder.as_ref().map(|p| p.to_string_lossy().into_owned())
    }
    pub fn set_folder_path(&mut self, path: Option<String>) {
        self.folder = path.filter(|s| !s.is_empty()).map(PathBuf::from);
        if self.folder.is_some() {
            self.rescan();
        }
    }

    fn pick_folder(&mut self) {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.folder = Some(dir);
            self.content = None;
            self.preview_job = None;
            self.rescan();
        }
    }

    fn rescan(&mut self) {
        self.files.clear();
        let Some(dir) = &self.folder else { return };
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                let is_log = p.is_file()
                    && p.extension()
                        .and_then(|x| x.to_str())
                        .map(|x| EXTS.contains(&x.to_lowercase().as_str()))
                        .unwrap_or(false);
                if is_log {
                    self.files.push(p);
                }
            }
            // Newest first by the dd_mm_yyyy_hh_mm_ss filename; names that don't
            // match that format sink to the bottom (then ordered by name).
            self.files.sort_by(|a, b| {
                let ka = a.file_stem().and_then(|s| s.to_str()).and_then(parse_dt_key);
                let kb = b.file_stem().and_then(|s| s.to_str()).and_then(parse_dt_key);
                match (ka, kb) {
                    (Some(x), Some(y)) => y.cmp(&x), // larger key = newer → first
                    (Some(_), None) => std::cmp::Ordering::Less, // dated above undated
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.file_name().cmp(&b.file_name()),
                }
            });
        }
    }

    fn load(&mut self, path: &PathBuf) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
        match fs::read(path) {
            Ok(bytes) => {
                // Accept non-UTF-8 by doing a lossy conversion so the preview
                // still shows text even when encoding is ambiguous.
                let text = String::from_utf8_lossy(&bytes).into_owned();
                self.preview_job = Some(colored_log(&text)); // parse once, here
                self.content = Some((name, text));
                self.error = None;
            }
            Err(e) => self.error = Some(format!("Gagal baca: {e}")),
        }
    }

    /// Draw the popup window (no-op when closed).
    pub fn window(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }
        let mut open = true;
        egui::Window::new("📂 Chatlog folder")
            .open(&mut open)
            .default_width(620.0)
            .default_height(440.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Pilih folder…").clicked() {
                        self.pick_folder();
                    }
                    if let Some(f) = &self.folder {
                        ui.small(f.display().to_string());
                    } else {
                        ui.small("Pilih folder berisi file .log / .txt");
                    }
                });
                if let Some(err) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, err);
                }
                ui.separator();

                egui::SidePanel::left("chatlog-files")
                    .resizable(true)
                    .default_width(190.0)
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let files = self.files.clone();
                            for p in &files {
                                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                                let sel = self.content.as_ref().map(|(n, _)| n.as_str()) == Some(name);
                                if ui.selectable_label(sel, name).clicked() {
                                    self.load(p);
                                }
                            }
                            if files.is_empty() {
                                ui.weak("(tidak ada file .log)");
                            }
                        });
                    });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if let Some((name, text)) = self.content.clone() {
                        ui.horizontal(|ui| {
                            ui.strong(name);
                            if ui.button("📋 Salin teks").clicked() {
                                ui.output_mut(|o| o.copied_text = text.clone());
                            }
                        });
                        ui.small("Pratinjau warna — kode {RRGGBB} dirender sebagai warna.");
                        // Colored preview on a dark, chat-like background so the
                        // SA-MP colors (white included) stay readable in both themes.
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(24, 26, 32))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        if let Some(job) = &self.preview_job {
                                            // Clone the parsed job (cheap) rather
                                            // than re-parsing every line per frame;
                                            // only max_width varies frame to frame.
                                            let mut job = job.clone();
                                            job.wrap.max_width = ui.available_width();
                                            ui.add(egui::Label::new(job).selectable(true));
                                        }
                                    });
                            });
                    } else {
                        ui.weak("Pilih file .log untuk pratinjau warna (kode {RRGGBB} dirender).");
                    }
                });
            });
        self.open = open;
    }
}

/// Parse a `dd_mm_yyyy_hh_mm_ss` filename stem into a chronological sort key
/// (larger = newer). Returns None when the stem isn't in that exact format, so
/// those files sort to the bottom of the list.
fn parse_dt_key(stem: &str) -> Option<u64> {
    let p: Vec<&str> = stem.split('_').collect();
    if p.len() != 6 {
        return None;
    }
    let n: Vec<u64> = p
        .iter()
        .map(|x| x.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    let (d, mo, y, h, mi, s) = (n[0], n[1], n[2], n[3], n[4], n[5]);
    if !(1..=31).contains(&d) || !(1..=12).contains(&mo) || h > 23 || mi > 59 || s > 59 {
        return None;
    }
    Some(y * 10_000_000_000 + mo * 100_000_000 + d * 1_000_000 + h * 10_000 + mi * 100 + s)
}

/// Build a colored preview of a chatlog: `{RRGGBB}` codes become real colors and
/// the code markers themselves are consumed (same splitter + hex parser the
/// export uses, so colors match). Uncolored text uses SA-MP white, which reads
/// against the dark preview background. Timestamps are left as-is.
fn colored_log(text: &str) -> egui::text::LayoutJob {
    use screenies_core::chatlog::parser;
    use screenies_core::render::text::parse_hex;

    let mut job = egui::text::LayoutJob::default();
    let font = egui::FontId::monospace(13.0);
    for line in text.lines() {
        for span in parser::split_hex_spans(line, "#FFFFFF") {
            let rgb = parse_hex(&span.color);
            job.append(
                &span.text,
                0.0,
                egui::text::TextFormat {
                    font_id: font.clone(),
                    color: egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                    ..Default::default()
                },
            );
        }
        job.append(
            "\n",
            0.0,
            egui::text::TextFormat {
                font_id: font.clone(),
                ..Default::default()
            },
        );
    }
    job
}
