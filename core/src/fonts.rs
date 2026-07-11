//! fonts.rs — the shared system font database.
//!
//! OPTIMIZATION (v0.11.0): `load_system_fonts()` walks the OS font
//! directories — doing that on every font-list call AND every export was
//! wasted disk work. The database is now built once per app run and
//! shared (fonts installed mid-session appear after a restart).

use std::collections::HashSet;
use std::sync::OnceLock;

static DB: OnceLock<fontdb::Database> = OnceLock::new();

/// The process-wide font database (built on first use).
pub fn database() -> &'static fontdb::Database {
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    })
}

/// All installed font family names, deduped, sorted case-insensitively.
pub fn families() -> Vec<String> {
    let db = database();

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
        let mut lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
        lower.sort();
        lower.dedup();
        assert_eq!(lower.len(), names.len(), "duplicate family names returned");
    }
}
