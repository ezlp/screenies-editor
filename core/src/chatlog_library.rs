//! chatlog_library.rs — feature 2 (2.0): open a FOLDER of chatlog logs, index
//! every line, and search it in-app (hover/search). Shell-independent: the Qt
//! shell wraps this as a QObject; it stays unit-tested here.
//!
//! Indexing (`index_text`) is split from filesystem scanning (`load_folder`)
//! so the index/search logic is testable without touching disk. Each indexed
//! line keeps its source file + 1-based line number for provenance on hover.

use std::fs;
use std::path::{Path, PathBuf};

/// One matchable chatlog line.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub file: String,
    pub line_no: usize,
    pub text: String,
}

/// A search hit — an entry plus the byte offset where the query matched.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub file: String,
    pub line_no: usize,
    pub text: String,
    pub match_at: usize,
}

/// Chatlog file extensions we index.
const LOG_EXTS: &[&str] = &["txt", "log"];

#[derive(Debug, Default)]
pub struct ChatlogLibrary {
    entries: Vec<Entry>,
}

impl ChatlogLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Index one file's contents. `file` is the display name kept on each entry.
    /// Blank lines are skipped (they can't be searched or hovered usefully).
    pub fn index_text(&mut self, file: &str, contents: &str) {
        for (i, line) in contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            self.entries.push(Entry {
                file: file.to_string(),
                line_no: i + 1,
                text: line.to_string(),
            });
        }
    }

    /// Scan a folder (non-recursive) and index every `.txt`/`.log` file.
    /// Returns the number of files indexed.
    pub fn load_folder(&mut self, dir: &Path) -> std::io::Result<usize> {
        let mut files = 0;
        for entry in fs::read_dir(dir)? {
            let path: PathBuf = entry?.path();
            if !path.is_file() {
                continue;
            }
            let is_log = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| LOG_EXTS.contains(&e.to_lowercase().as_str()))
                .unwrap_or(false);
            if !is_log {
                continue;
            }
            if let Ok(contents) = fs::read_to_string(&path) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                self.index_text(&name, &contents);
                files += 1;
            }
        }
        Ok(files)
    }

    /// Case-insensitive substring search over every indexed line.
    /// Empty query returns nothing (the UI shows the full list separately).
    pub fn search(&self, query: &str) -> Vec<Hit> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        self.entries
            .iter()
            .filter_map(|e| {
                e.text.to_lowercase().find(&q).map(|at| Hit {
                    file: e.file.clone(),
                    line_no: e.line_no,
                    text: e.text.clone(),
                    match_at: at,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lib() -> ChatlogLibrary {
        let mut l = ChatlogLibrary::new();
        l.index_text(
            "monday.txt",
            "[12:00:00] Budi_Santoso says: halo semua\n\n[12:00:05] * Budi menoleh\n",
        );
        l.index_text("tuesday.log", "[09:15:00] Ani_Wijaya says: pagi bang\n");
        l
    }

    #[test]
    fn blank_lines_are_not_indexed() {
        assert_eq!(lib().len(), 3); // 4 lines, one blank
    }

    #[test]
    fn search_is_case_insensitive_and_keeps_provenance() {
        let hits = lib().search("BUDI");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].file, "monday.txt");
        assert_eq!(hits[0].line_no, 1);
    }

    #[test]
    fn empty_query_returns_nothing() {
        assert!(lib().search("   ").is_empty());
    }

    #[test]
    fn reports_match_offset() {
        let hits = lib().search("says");
        assert!(hits.iter().all(|h| h.text[h.match_at..].starts_with("says")));
    }
}
