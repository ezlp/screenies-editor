//! gallery.rs — feature 4 (2.0): browse previously exported SSRP photos from a
//! folder. Shell-independent listing logic; the Qt shell wraps it with QImage
//! thumbnails in phase 4. Reuses the last-save-dir the exporter remembers.

use std::fs;
use std::path::{Path, PathBuf};

/// Image extensions the gallery lists.
const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];

/// One gallery item — enough for a thumbnail grid + "open in editor".
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub path: PathBuf,
    pub name: String,
    pub modified: Option<std::time::SystemTime>,
}

/// List image files in a folder (non-recursive), newest first.
pub fn list_folder(dir: &Path) -> std::io::Result<Vec<Item>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || !is_image(&path) {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
        items.push(Item { path, name, modified });
    }
    // Newest first; items without a mtime sink to the bottom.
    items.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(items)
}

/// True when the path has an image extension we display.
pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn only_image_extensions_count() {
        assert!(is_image(Path::new("/x/shot.PNG")));
        assert!(is_image(Path::new("/x/a.jpeg")));
        assert!(!is_image(Path::new("/x/notes.txt")));
        assert!(!is_image(Path::new("/x/no-ext")));
    }
}
