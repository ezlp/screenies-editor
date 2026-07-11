//! parser.rs — split lines into colored spans by SA-MP hex codes.
//!
//! Codes look like `{FF0000}` and are case-insensitive in real logs
//! (`{ffff00}` appears constantly on JGRP). Everything after a code takes
//! that color until the next code. Invalid codes (`{GGGGGG}`, `{FFF}`)
//! are not codes — they stay as literal text.

use super::ColorSpan;
use regex::Regex;
use std::sync::OnceLock;

fn hex_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{([0-9A-Fa-f]{6})\}").expect("valid hex regex"))
}

/// Does this line contain at least one `{RRGGBB}` code?
pub fn has_hex(line: &str) -> bool {
    hex_re().is_match(line)
}

/// The line with every `{RRGGBB}` code removed — the "plain" text used for
/// classification (auto-color, system-tag detection, RP filter).
pub fn strip_hex(line: &str) -> String {
    hex_re().replace_all(line, "").into_owned()
}

/// Split into colored spans. Text before the first code uses `default_color`
/// (whatever auto-color decided for this line). Colors are normalized to
/// uppercase `#RRGGBB`.
pub fn split_hex_spans(line: &str, default_color: &str) -> Vec<ColorSpan> {
    let mut spans: Vec<ColorSpan> = Vec::new();
    let mut color = default_color.to_uppercase();
    let mut last = 0;

    for caps in hex_re().captures_iter(line) {
        let m = caps.get(0).expect("whole match");
        let text = &line[last..m.start()];
        if !text.is_empty() {
            spans.push(ColorSpan {
                text: text.to_string(),
                color: color.clone(),
                bold: false,
            });
        }
        color = format!("#{}", caps[1].to_uppercase());
        last = m.end();
    }

    let tail = &line[last..];
    if !tail.is_empty() {
        spans.push(ColorSpan {
            text: tail.to_string(),
            color,
            bold: false,
        });
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_by_codes_case_insensitive() {
        let spans = split_hex_spans("halo {FF0000}merah {ffff00}kuning", "#FFFFFF");
        assert_eq!(spans.len(), 3);
        assert_eq!((spans[0].text.as_str(), spans[0].color.as_str()), ("halo ", "#FFFFFF"));
        assert_eq!((spans[1].text.as_str(), spans[1].color.as_str()), ("merah ", "#FF0000"));
        assert_eq!((spans[2].text.as_str(), spans[2].color.as_str()), ("kuning", "#FFFF00"));
    }

    #[test]
    fn leading_code_means_no_default_span() {
        let spans = split_hex_spans("{00ffff}SERVER: halo", "#FFFFFF");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].color, "#00FFFF");
        assert_eq!(spans[0].text, "SERVER: halo");
    }

    #[test]
    fn invalid_codes_stay_literal() {
        let spans = split_hex_spans("x {GGGGGG} y {FFF} z", "#FFFFFF");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "x {GGGGGG} y {FFF} z");
    }

    #[test]
    fn strip_hex_removes_all_codes() {
        assert_eq!(
            strip_hex("{00ffff}SERVER:{ffffff} Selamat datang {ffff00}Coco_Aguilar{ffffff}."),
            "SERVER: Selamat datang Coco_Aguilar."
        );
        assert!(!has_hex("polos saja"));
        assert!(has_hex("{FFFFFF}x"));
    }
}
