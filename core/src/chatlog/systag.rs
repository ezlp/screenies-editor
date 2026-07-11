//! systag.rs — detect system/game message tags.
//!
//! JGRP-style logs tag system lines right after the timestamp:
//! `SERVER:`, `VEHICLE:`, `ENGINEINFO:`, `AdmCmd:`, `Reason:`, `KEGUNAAN:` …
//! A real log showed ~30 distinct tags, so we match the *pattern* instead of
//! keeping a list — any `Word:` prefix counts. That way tag #31 (or another
//! server's tags) works without a code change.
//!
//! RP lines never match: "Park Areu says:" has a space before the colon,
//! "* Name action" starts with `*`, "(( PM ... ))" starts with `(`.
//!
//! M1 limitation: lines where a hex code precedes the tag
//! (`{00ffff}SERVER:{ffffff} …`) don't match yet — Milestone 2 strips hex
//! codes first, which unlocks bold for those too.

use regex::Regex;
use std::sync::OnceLock;

/// `TAG:` at line start: letter first, then up to 19 more of [A-Za-z0-9_/-],
/// a colon, and at least one whitespace before the message.
fn tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^([A-Za-z][A-Za-z0-9_/-]{0,19}):\s+(.+)$").expect("valid systag regex")
    })
}

/// If `line` is a system-tagged message, return `(tag, rest)`.
pub fn split(line: &str) -> Option<(&str, &str)> {
    let caps = tag_re().captures(line)?;
    let tag = caps.get(1)?;
    let rest = caps.get(2)?;
    Some((
        &line[tag.start()..tag.end()],
        &line[rest.start()..rest.end()],
    ))
}

#[cfg(test)]
mod tests {
    use super::split;

    #[test]
    fn matches_common_tags() {
        assert_eq!(split("VEHICLE: Seatbelts ON"), Some(("VEHICLE", "Seatbelts ON")));
        assert_eq!(split("ERROR: No unpaid hospital bills!"), Some(("ERROR", "No unpaid hospital bills!")));
        assert_eq!(split("ENGINEINFO: Mesin masih mati"), Some(("ENGINEINFO", "Mesin masih mati")));
        assert_eq!(split("KEGUNAAN: /sellplant [wheat]"), Some(("KEGUNAAN", "/sellplant [wheat]")));
    }

    #[test]
    fn matches_mixed_case_tags() {
        assert_eq!(
            split("AdmCmd: Chester_Calloway has been warned"),
            Some(("AdmCmd", "Chester_Calloway has been warned"))
        );
        assert_eq!(split("Reason: AFK in public space."), Some(("Reason", "AFK in public space.")));
    }

    #[test]
    fn ignores_roleplay_lines() {
        assert_eq!(split("Park Areu says: Kenapa lagi?"), None);
        assert_eq!(split("Coco Aguilar says [low]: Pak"), None);
        assert_eq!(split("* Daniel Ochoa showing Id Card"), None);
        assert_eq!(split("(( PM from [59] Rene: halo ))"), None);
    }

    #[test]
    fn ignores_hex_prefixed_tags_for_now() {
        // split() itself only sees plain text — mod.rs strips hex codes first,
        // so hex-prefixed tags ARE detected by the full pipeline (see mod tests).
        assert_eq!(split("{00ffff}SERVER:{ffffff} Selamat datang"), None);
    }
}
