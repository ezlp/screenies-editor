//! timestamp.rs — remove SA-MP chatlog timestamps.
//!
//! Game chatlogs prefix every line with `[HH:MM:SS]` (sometimes `[H:MM]`).
//! Both old sites strip these automatically on paste; so do we.

use regex::Regex;
use std::sync::OnceLock;

/// One or more `[H:MM]` / `[HH:MM:SS]` stamps at the start of a line,
/// including the whitespace that follows them.
fn timestamp_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\s*(?:\[\d{1,2}:\d{2}(?::\d{2})?\]\s*)+").expect("valid timestamp regex")
    })
}

/// Strip leading timestamp stamp(s) from a single line.
pub fn strip(line: &str) -> &str {
    match timestamp_re().find(line) {
        Some(m) if m.start() == 0 => &line[m.end()..],
        _ => line,
    }
}

#[cfg(test)]
mod tests {
    use super::strip;

    #[test]
    fn strips_standard_stamp() {
        assert_eq!(strip("[12:34:56] halo"), "halo");
    }

    #[test]
    fn strips_short_stamp() {
        assert_eq!(strip("[1:02] halo"), "halo");
    }

    #[test]
    fn strips_double_stamp() {
        assert_eq!(strip("[12:34:56] [12:34:57] halo"), "halo");
    }

    #[test]
    fn strips_with_leading_spaces() {
        assert_eq!(strip("   [00:00:00] halo"), "halo");
    }

    #[test]
    fn leaves_mid_line_stamps_alone() {
        assert_eq!(strip("dia bilang [12:00:00] nanti"), "dia bilang [12:00:00] nanti");
    }

    #[test]
    fn leaves_plain_lines_alone() {
        assert_eq!(strip("halo dunia"), "halo dunia");
    }
}
