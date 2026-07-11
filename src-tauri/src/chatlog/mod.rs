//! chatlog — text processing pipeline.
//!
//! Pipeline per line (Milestone 2, complete):
//!   1. `timestamp::strip`  — remove `[HH:MM:SS]` prefixes
//!   2. `parser::strip_hex` — plain text for classification
//!   3. `autocolor::classify` — line type + base color (`*` ungu, `(( ))`
//!      abu-abu, says/shouts/[phone]/[walkie], /do suffix, system tags)
//!   4. `parser::split_hex_spans` — `{RRGGBB}` colors (case-insensitive)
//!   5. system lines get their `TAG:` prefix bolded
//!
//! ⚠ RULE: `ColorSpan`, `LineType` and `ParsedLine` MUST stay in sync with
//! `src/ts/types.ts` (serde renames fields to camelCase). If you change one
//! side, change the other in the same commit.

pub mod autocolor;
pub mod parser;
pub mod preset;
pub mod systag;
pub mod timestamp;

use preset::ParsePreset;

use serde::Serialize;

/// One colored run of text inside a line.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ColorSpan {
    pub text: String,
    /// Hex color like "#FFFFFF".
    pub color: String,
    /// Heavier weight — used for system-tag prefixes like "VEHICLE: ".
    pub bold: bool,
}

/// Kind of chat line — drives auto-coloring and the "RP only" filter.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LineType {
    Normal,
    Me,
    Do,
    Ooc,
    Says,
    Shouts,
    /// Tagged game message (`SERVER:`, `VEHICLE:`, `AdmCmd:` …).
    System,
}

/// A fully parsed chatlog line, ready to draw.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedLine {
    pub spans: Vec<ColorSpan>,
    pub line_type: LineType,
}

/// SA-MP chat white — the default for uncolored, unclassified text.
pub const CHAT_WHITE: &str = "#FFFFFF";

/// Full parse pipeline over a pasted chatlog, driven by the preset.
pub fn parse(text: &str, preset: &ParsePreset) -> Vec<ParsedLine> {
    let ctx = autocolor::RuleCtx::new(preset);
    text.lines()
        .map(|line| {
            if preset.strip_timestamps {
                timestamp::strip(line)
            } else {
                line
            }
        })
        .map(|line| line.trim_end())
        .filter(|line| !plain_of(line, preset).trim().is_empty())
        .map(|line| parse_line(line, preset, &ctx))
        .collect()
}

/// The classification text: hex-stripped when hex parsing is on.
fn plain_of(line: &str, preset: &ParsePreset) -> String {
    if preset.hex_codes {
        parser::strip_hex(line)
    } else {
        line.to_string()
    }
}

fn parse_line(line: &str, preset: &ParsePreset, ctx: &autocolor::RuleCtx) -> ParsedLine {
    let plain = plain_of(line, preset);
    let line_type = autocolor::classify(&plain, ctx);

    let base_color: &str = match line_type {
        LineType::Me | LineType::Do => &preset.color_me,
        LineType::Ooc => &preset.color_ooc,
        _ => &preset.color_default,
    };

    let mut spans = if preset.hex_codes && parser::has_hex(line) {
        parser::split_hex_spans(line, base_color)
    } else {
        vec![ColorSpan {
            text: line.to_string(),
            color: base_color.to_uppercase(),
            bold: false,
        }]
    };

    if line_type == LineType::System {
        if let Some((tag, _rest)) = systag::split(plain.trim_start()) {
            bold_tag_prefix(&mut spans, tag);
        }
    }

    ParsedLine { spans, line_type }
}

/// Make the leading `TAG:` (plus one following space, if any) a bold span.
/// If hex codes chopped the tag oddly, we skip bolding rather than guess.
fn bold_tag_prefix(spans: &mut Vec<ColorSpan>, tag: &str) {
    let Some(first) = spans.first_mut() else { return };

    let lead = first.text.len() - first.text.trim_start().len();
    let prefix = format!("{tag}:");
    if !first.text[lead..].starts_with(&prefix) {
        return;
    }

    let mut split_at = lead + prefix.len();
    if first.text[split_at..].starts_with(' ') {
        split_at += 1; // keep "TAG: " together in the bold span
    }

    let rest = first.text.split_off(split_at);
    first.bold = true;
    if !rest.is_empty() {
        let color = first.color.clone();
        spans.insert(
            1,
            ColorSpan {
                text: rest,
                color,
                bold: false,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_jgrp(text: &str) -> Vec<ParsedLine> {
        parse(text, &preset::jgrp())
    }

    #[test]
    fn strips_timestamps_and_empty_lines() {
        let input = "[12:34:56] Budi_Santoso says: halo\n\n[1:02:03] * Budi menoleh.\n   \n";
        let out = parse(input);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].spans[0].text, "Budi_Santoso says: halo");
        assert_eq!(out[0].line_type, LineType::Says);
        assert_eq!(out[1].line_type, LineType::Me);
        assert_eq!(out[1].spans[0].color, preset::jgrp().color_me);
    }

    #[test]
    fn me_and_ooc_get_their_colors() {
        let out = parse_jgrp("* Daniel Ochoa showing Id Card\n(( PM from [59] Rene: halo ))");
        assert_eq!(out[0].line_type, LineType::Me);
        assert_eq!(out[0].spans[0].color, preset::jgrp().color_me);
        assert_eq!(out[1].line_type, LineType::Ooc);
        assert_eq!(out[1].spans[0].color, preset::jgrp().color_ooc);
    }

    #[test]
    fn do_suffix_is_purple() {
        let out = parse_jgrp("relax ((Oswald Heisenberg))");
        assert_eq!(out[0].line_type, LineType::Do);
        assert_eq!(out[0].spans[0].color, preset::jgrp().color_me);
    }

    #[test]
    fn plain_system_tag_bold_prefix() {
        let out = parse_jgrp("[00:11:09] VEHICLE: Handbrakes disengaged");
        assert_eq!(out[0].line_type, LineType::System);
        assert_eq!(out[0].spans[0].text, "VEHICLE: ");
        assert!(out[0].spans[0].bold);
        assert_eq!(out[0].spans[1].text, "Handbrakes disengaged");
        assert!(!out[0].spans[1].bold);
    }

    #[test]
    fn hex_prefixed_system_line_from_real_log() {
        // Line 8 of the JGRP fixture — the M1 caveat, now unlocked.
        let out = parse_jgrp("[23:56:18] {00ffff}SERVER:{ffffff} Selamat datang {ffff00}Coco_Aguilar{ffffff}.");
        let line = &out[0];
        assert_eq!(line.line_type, LineType::System);
        assert_eq!(line.spans[0].text, "SERVER:");
        assert!(line.spans[0].bold);
        assert_eq!(line.spans[0].color, "#00FFFF");
        assert_eq!(line.spans[1].text, " Selamat datang ");
        assert_eq!(line.spans[1].color, "#FFFFFF");
        assert_eq!(line.spans[2].color, "#FFFF00");
        assert!(line.spans.iter().skip(1).all(|s| !s.bold));
    }

    #[test]
    fn hex_colors_on_speech_keep_says_type() {
        let out = parse_jgrp("Budi says: aku suka {FF0000}merah");
        assert_eq!(out[0].line_type, LineType::Says);
        assert_eq!(out[0].spans[0].color, CHAT_WHITE);
        assert_eq!(out[0].spans[1].color, "#FF0000");
    }

    #[test]
    fn code_only_lines_are_dropped() {
        assert!(parse_jgrp("{FFFFFF}{00FF00}").is_empty());
    }

    #[test]
    fn preset_toggles_change_output() {
        let mut p = preset::jgrp();
        p.system_tags = false;
        let out = parse("VEHICLE: Handbrakes disengaged", &p);
        assert_eq!(out[0].line_type, LineType::Normal);
        assert_eq!(out[0].spans.len(), 1);
        assert!(!out[0].spans[0].bold);

        p = preset::jgrp();
        p.strip_timestamps = false;
        let out = parse("[00:11:09] halo", &p);
        assert!(out[0].spans[0].text.starts_with("[00:11:09]"));
    }

    #[test]
    fn custom_colors_are_used() {
        let mut p = preset::jgrp();
        p.color_me = "#FF00FF".into();
        let out = parse("* aksi kustom", &p);
        assert_eq!(out[0].spans[0].color, "#FF00FF");
    }

    #[test]
    fn serializes_camel_case_for_typescript() {
        let line = ParsedLine {
            spans: vec![ColorSpan {
                text: "x".into(),
                color: CHAT_WHITE.into(),
                bold: false,
            }],
            line_type: LineType::System,
        };
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("\"lineType\":\"system\""), "json was: {json}");
        assert!(json.contains("\"bold\":false"), "json was: {json}");
    }
}
