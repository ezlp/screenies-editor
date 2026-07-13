// chatlog_parser.rs — Phase 3 screen: open a FOLDER of chatlog logs, index
// every line (core::chatlog_library), and search it in-app. Click a hit to
// copy the line. The indexing/search logic is unit-tested in core; this is
// just the egui view over it.

use eframe::egui;
use screenies_core::chatlog_library::{ChatlogLibrary, Hit};
use std::path::PathBuf;

/// Cap on rendered rows so a broad query over a huge folder stays snappy.
const MAX_ROWS: usize = 1000;

#[derive(Default)]
pub struct ChatlogParserState {
    library: ChatlogLibrary,
    folder: Option<PathBuf>,
    files: usize,
    query: String,
    results: Vec<Hit>,
    error: Option<String>,
}

impl ChatlogParserState {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("📂  Buka folder chatlog").clicked() {
                self.open_folder();
            }
            if let Some(f) = &self.folder {
                ui.small(format!(
                    "{} · {} file · {} baris",
                    f.display(),
                    self.files,
                    self.library.len()
                ));
            } else {
                ui.small("Pilih folder berisi file .txt / .log");
            }
        });

        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
        }

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("🔍");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("cari nama / kata di semua chatlog…")
                    .desired_width(f32::INFINITY),
            );
            if resp.changed() {
                self.run_search();
            }
        });

        let shown = self.results.len().min(MAX_ROWS);
        ui.small(if self.results.len() > MAX_ROWS {
            format!("{} hasil (menampilkan {})", self.results.len(), MAX_ROWS)
        } else {
            format!("{} hasil", self.results.len())
        });

        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for hit in self.results.iter().take(shown) {
                let resp = ui
                    .add(egui::Label::new(&hit.text).sense(egui::Sense::click()))
                    .on_hover_text(format!("{}:{} — klik untuk salin", hit.file, hit.line_no));
                if resp.clicked() {
                    ui.output_mut(|o| o.copied_text = hit.text.clone());
                }
                ui.small(format!("{}:{}", hit.file, hit.line_no));
                ui.separator();
            }
            if self.folder.is_some() && self.query.trim().is_empty() {
                ui.weak("Ketik untuk mencari.");
            }
        });
    }

    fn open_folder(&mut self) {
        let Some(dir) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        let mut lib = ChatlogLibrary::new();
        match lib.load_folder(&dir) {
            Ok(files) => {
                self.library = lib;
                self.files = files;
                self.folder = Some(dir);
                self.error = None;
                self.run_search();
            }
            Err(e) => self.error = Some(format!("Gagal baca folder: {e}")),
        }
    }

    fn run_search(&mut self) {
        self.results = self.library.search(&self.query);
    }
}
