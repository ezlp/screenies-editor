//! preset.rs — configurable parsing rules.
//!
//! A `ParsePreset` describes HOW a server formats its chatlog: which
//! auto-color rules apply, what the colors are, which radio channels
//! exist. The frontend sends the active preset with every parse call,
//! so nothing about a specific server is hardcoded in the engine.
//!
//! Presets are plain JSON (serde, camelCase) — shareable between users
//! and documented in docs/PRESETS.md for the project wiki. Unknown or
//! missing fields fall back to defaults (`serde(default)`), so presets
//! written for older app versions keep working.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ParsePreset {
    /// Display name shown in the dropdown.
    pub name: String,

    /// Strip `[HH:MM:SS]` prefixes.
    pub strip_timestamps: bool,
    /// Parse `{RRGGBB}` color codes (case-insensitive).
    pub hex_codes: bool,

    /// `*` at line start → /me color.
    pub me_prefix: bool,
    /// `((` at line start → OOC color.
    pub ooc_wrap: bool,
    /// Line ending in `((Nama))` → /do color (JGRP-style /do output).
    pub do_suffix: bool,
    /// `TAG:` prefixes → System lines with a bold tag (enables "Hanya RP").
    pub system_tags: bool,

    /// Bracket channels treated as speech, e.g. ["phone", "walkie"].
    pub radio_channels: Vec<String>,

    /// Colors (hex like "#C2A2DA").
    pub color_me: String,
    pub color_ooc: String,
    pub color_default: String,
}

impl Default for ParsePreset {
    fn default() -> Self {
        jgrp()
    }
}

/// JGRP / Jogjagamers Reality Project — the fixture this app grew up on.
pub fn jgrp() -> ParsePreset {
    ParsePreset {
        name: "JGRP (Jogjagamers)".into(),
        strip_timestamps: true,
        hex_codes: true,
        me_prefix: true,
        ooc_wrap: true,
        do_suffix: true,
        system_tags: true,
        radio_channels: vec!["phone".into(), "walkie".into()],
        color_me: "#C2A2DA".into(),
        color_ooc: "#9C9C9C".into(),
        color_default: "#FFFFFF".into(),
    }
}

/// Generic SA-MP roleplay server — like JGRP but without the
/// JGRP-specific `/do ((Nama))` suffix heuristic.
pub fn samp_umum() -> ParsePreset {
    ParsePreset {
        name: "SA-MP Umum".into(),
        do_suffix: false,
        ..jgrp()
    }
}

/// No auto-coloring at all — timestamps and hex codes only.
/// For servers whose format the presets don't fit (yet).
pub fn polos() -> ParsePreset {
    ParsePreset {
        name: "Polos (tanpa auto-warna)".into(),
        me_prefix: false,
        ooc_wrap: false,
        do_suffix: false,
        system_tags: false,
        radio_channels: vec![],
        ..jgrp()
    }
}

/// The presets shipped with the app, in dropdown order.
pub fn builtin() -> Vec<ParsePreset> {
    vec![jgrp(), samp_umum(), polos()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_or_partial_json_still_deserializes() {
        // A preset file missing newer fields must not break (wiki-shared files).
        let p: ParsePreset = serde_json::from_str(r#"{ "name": "X", "doSuffix": false }"#).unwrap();
        assert_eq!(p.name, "X");
        assert!(!p.do_suffix);
        assert!(p.me_prefix); // defaulted
        assert_eq!(p.color_me, "#C2A2DA"); // defaulted
    }

    #[test]
    fn builtins_are_distinct() {
        let b = builtin();
        assert_eq!(b.len(), 3);
        assert!(b[0].do_suffix && !b[1].do_suffix && !b[2].me_prefix);
    }
}
