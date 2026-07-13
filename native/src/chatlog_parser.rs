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
    pub lang: crate::i18n::Lang,
}

impl ChatlogParserState {
    fn t(&self, s: &'static str) -> &'static str {
        crate::i18n::t(self.lang, s)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let open_lbl = self.t("📂  Buka folder chatlog");
        let empty_lbl = self.t("Pilih folder berisi file .txt / .log");
        ui.horizontal(|ui| {
            if ui.button(open_lbl).clicked() {
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
                ui.small(empty_lbl);
            }
        });

        if let Some(err) = &self.error {
            ui.colored_label(egui::Color32::from_rgb(220, 90, 90), err);
        }

        ui.separator();
        let search_hint = self.t("cari nama / kata di semua chatlog…");
        ui.horizontal(|ui| {
            ui.label("🔍");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(search_hint)
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
        let copy_hint = self.t("klik untuk salin");
        let type_hint = self.t("Ketik untuk mencari.");
        egui::ScrollArea::vertical().show(ui, |ui| {
            for hit in self.results.iter().take(shown) {
                let resp = ui
                    .add(egui::Label::new(&hit.text).sense(egui::Sense::click()))
                    .on_hover_text(format!("{}:{} — {}", hit.file, hit.line_no, copy_hint));
                if resp.clicked() {
                    ui.output_mut(|o| o.copied_text = hit.text.clone());
                }
                ui.small(format!("{}:{}", hit.file, hit.line_no));
                ui.separator();
            }
            if self.folder.is_some() && self.query.trim().is_empty() {
                ui.weak(type_hint);
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
