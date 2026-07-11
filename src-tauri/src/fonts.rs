//! fonts.rs — enumerate system font families for the picker.
//!
//! Uses `fontdb` to scan the OS font directories, so the dropdown shows
//! every font actually installed on the user's PC — not a hardcoded list.
//! The same family names render correctly in the webview because it is a
//! native browser engine with full system-font access.

use std::collections::HashSet;

/// All installed font family names, deduped, sorted case-insensitively.
pub fn families() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut seen: HashSet<String> = HashSet::new();
    let mut names: Vec<String> = Vec::new();

    for face in db.faces() {
        if let Some((name, _lang)) = face.families.first() {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            if seen.insert(name.to_lowercase()) {
                names.push(name.to_string());
            }
        }
    }

    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    names
}

#[cfg(test)]
mod tests {
    use super::families;

    #[test]
    fn enumerates_without_panicking_and_dedupes() {
        let names = families();
        // CI containers have at least a few fonts; mainly assert no dupes.
        let mut lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        lower.sort();
        lower.dedup();
        assert_eq!(lower.len(), names.len(), "duplicate family names returned");
    }
}
