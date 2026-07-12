//! layout.rs — text layout: wrap lines and assign every token an absolute
//! x/y in output space, producing the `ExportBlock` structures the renderer
//! (text.rs) consumes.
//!
//! In 1.x this lived in the TypeScript canvas (`buildRenderBlocks`), and the
//! export received the pre-computed positions. 2.0 moves it into core so the
//! Qt preview and the PNG export share ONE layout — preview == PNG by identity
//! rather than by two implementations agreeing. Constants match the 1.x canvas.
//!
//! Testability: the algorithm is generic over a `Measure` (word → width). The
//! real measurer (text.rs `GlyphMeasure`) sums ab_glyph advances — the SAME
//! advances the renderer pens glyphs with. Tests use a deterministic mock so
//! the wrapping/positioning math is verifiable without any font installed.

use super::{BgRect, ExportBlock, ExportRow, ExportToken};
use crate::chatlog::ParsedLine;

// Match src/ts/canvas.ts exactly.
const MARGIN_X: f32 = 14.0;
const MARGIN_Y: f32 = 16.0;
const MIN_WRAP: f32 = 80.0;

/// Where a block's text is pinned. "Free" = draggable at (x, y).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Free,
    KiriAtas,
    KiriBawah,
}

/// Dark strip behind the text, per block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgMode {
    None,
    Block,
    Mask,
}

/// One chatlog block to lay out.
pub struct LayoutBlock {
    pub lines: Vec<ParsedLine>,
    pub anchor: Anchor,
    pub bg_mode: BgMode,
    /// Free-anchor origin (ignored for the pinned anchors).
    pub x: f32,
    pub y: f32,
}

/// Shared layout parameters for a render pass.
pub struct LayoutParams {
    pub text_size: f32,
    /// Line spacing as a percent of text size (122 = the classic SSRP look).
    pub line_gap: f32,
    pub bg_offset: f32,
    pub output_w: f32,
    pub output_h: f32,
}

/// Bounding box of a laid-out block, output px — for UI hit-testing (dragging).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Width provider: the advance width of `text` at the layout's text size.
pub trait Measure {
    fn width(&self, text: &str, bold: bool) -> f32;
}

/// A laid-out block: what the renderer draws, plus its bounds for the UI.
pub struct Laid {
    pub block: ExportBlock,
    pub bounds: Option<Bounds>,
}

/// Lay out every block against the same params/measurer.
pub fn layout_blocks<M: Measure>(blocks: &[LayoutBlock], p: &LayoutParams, m: &M) -> Vec<Laid> {
    blocks.iter().map(|b| layout_block(b, p, m)).collect()
}

/// Lay out one block into positioned rows.
pub fn layout_block<M: Measure>(b: &LayoutBlock, p: &LayoutParams, m: &M) -> Laid {
    let size = p.text_size;
    let advance = size * (p.line_gap / 100.0);

    if b.lines.is_empty() {
        return Laid { block: ExportBlock { rows: Vec::new() }, bounds: None };
    }

    let wrap = wrap_width_for(b, p.output_w);
    let (rows, block_w, block_h) = wrap_lines(&b.lines, advance, size, wrap, m);
    let (ox, oy) = block_origin(b, p, block_h);

    let pad_top = (advance - size) / 2.0;
    // Glyphs sit low in the em box — nudge the BG strip down ~8% + user offset.
    let bg_shift = (size * 0.08).round() + p.bg_offset;

    let mut export_rows = Vec::with_capacity(rows.len());
    let mut y = oy;
    for row in &rows {
        let mut tokens = Vec::new();
        let mut x = ox;
        for t in &row.tokens {
            // Whitespace advances the pen but is never drawn.
            if !t.text.trim().is_empty() {
                tokens.push(ExportToken {
                    text: t.text.clone(),
                    x,
                    color: t.color.clone(),
                    bold: t.bold,
                });
            }
            x += t.width;
        }

        let bg = if row.width > 0.0 {
            match b.bg_mode {
                BgMode::Block => Some(BgRect {
                    x: ox - 6.0,
                    y: y - pad_top + bg_shift,
                    w: row.width + 12.0,
                    h: advance,
                }),
                BgMode::Mask => Some(BgRect {
                    x: 0.0,
                    y: y - pad_top + bg_shift,
                    w: p.output_w,
                    h: advance,
                }),
                BgMode::None => None,
            }
        } else {
            None
        };

        export_rows.push(ExportRow { y, tokens, bg });
        y += advance;
    }

    Laid {
        block: ExportBlock { rows: export_rows },
        bounds: Some(Bounds { x: ox, y: oy, w: block_w.max(size), h: block_h }),
    }
}

struct Tok {
    text: String,
    color: String,
    bold: bool,
    width: f32,
}
struct WrapRow {
    tokens: Vec<Tok>,
    width: f32,
}

/// Greedy word-wrap, faithful to canvas.ts `layoutLines`: split each span on
/// whitespace runs (spaces kept as their own zero-drawn tokens), break when a
/// word would overflow, and swallow the leading space after a break.
fn wrap_lines<M: Measure>(
    lines: &[ParsedLine],
    advance: f32,
    size: f32,
    wrap: f32,
    m: &M,
) -> (Vec<WrapRow>, f32, f32) {
    let mut rows: Vec<WrapRow> = Vec::new();
    let mut block_w = 0.0f32;

    for line in lines {
        let mut row = WrapRow { tokens: Vec::new(), width: 0.0 };
        for span in &line.spans {
            for raw in split_ws(&span.text) {
                if raw.is_empty() {
                    continue;
                }
                let is_space = raw.trim().is_empty();
                let width = m.width(&raw, span.bold);

                if row.width + width > wrap && row.width > 0.0 {
                    if row.width > block_w {
                        block_w = row.width;
                    }
                    rows.push(row);
                    row = WrapRow { tokens: Vec::new(), width: 0.0 };
                    if is_space {
                        continue;
                    }
                }
                if !is_space || row.width > 0.0 {
                    row.tokens.push(Tok {
                        text: raw,
                        color: span.color.clone(),
                        bold: span.bold,
                        width,
                    });
                    row.width += width;
                }
            }
        }
        if row.width > block_w {
            block_w = row.width;
        }
        rows.push(row);
    }

    let height = if rows.is_empty() {
        0.0
    } else {
        rows.len() as f32 * advance - (advance - size)
    };
    (rows, block_w, height)
}

/// Split on whitespace runs, keeping the runs (like JS `split(/(\s+)/)`):
/// "a  b" → ["a", "  ", "b"].
fn split_ws(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut cur_is_space: Option<bool> = None;
    for ch in s.chars() {
        let sp = ch.is_whitespace();
        if cur_is_space == Some(sp) {
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(ch);
            cur_is_space = Some(sp);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn wrap_width_for(b: &LayoutBlock, output_w: f32) -> f32 {
    match b.anchor {
        Anchor::Free => (output_w - MARGIN_X - b.x).max(MIN_WRAP),
        _ => (output_w - MARGIN_X * 2.0).max(MIN_WRAP),
    }
}

fn block_origin(b: &LayoutBlock, p: &LayoutParams, block_h: f32) -> (f32, f32) {
    match b.anchor {
        Anchor::Free => (b.x, b.y),
        Anchor::KiriAtas => (MARGIN_X, MARGIN_Y),
        Anchor::KiriBawah => (MARGIN_X, p.output_h - MARGIN_Y - block_h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chatlog::{ColorSpan, LineType, ParsedLine};

    /// Deterministic measurer: every char is 10px wide (spaces included),
    /// bold ignored — so wrapping/positioning math is exact and font-free.
    struct Mock;
    impl Measure for Mock {
        fn width(&self, text: &str, _bold: bool) -> f32 {
            text.chars().count() as f32 * 10.0
        }
    }

    fn line(text: &str) -> ParsedLine {
        ParsedLine {
            spans: vec![ColorSpan { text: text.into(), color: "#FFFFFF".into(), bold: false }],
            line_type: LineType::Normal,
        }
    }

    fn params() -> LayoutParams {
        LayoutParams { text_size: 20.0, line_gap: 122.0, bg_offset: 0.0, output_w: 800.0, output_h: 600.0 }
    }

    #[test]
    fn free_block_positions_tokens_left_to_right() {
        let b = LayoutBlock { lines: vec![line("ab cd")], anchor: Anchor::Free, bg_mode: BgMode::None, x: 100.0, y: 50.0 };
        let laid = layout_block(&b, &params(), &Mock);
        let rows = &laid.block.rows;
        assert_eq!(rows.len(), 1);
        let toks = &rows[0].tokens;
        assert_eq!(toks.len(), 2); // "ab" and "cd"; the space is not drawn
        assert_eq!(toks[0].text, "ab");
        assert_eq!(toks[0].x, 100.0); // origin.x
        assert_eq!(rows[0].y, 50.0); // origin.y
        // "cd" starts after "ab"(20) + " "(10) = x 130.
        assert_eq!(toks[1].text, "cd");
        assert_eq!(toks[1].x, 130.0);
    }

    #[test]
    fn pinned_anchors_use_the_margin_origin() {
        let atas = LayoutBlock { lines: vec![line("hi")], anchor: Anchor::KiriAtas, bg_mode: BgMode::None, x: 999.0, y: 999.0 };
        let laid = layout_block(&atas, &params(), &Mock);
        assert_eq!(laid.block.rows[0].tokens[0].x, MARGIN_X);
        assert_eq!(laid.block.rows[0].y, MARGIN_Y);

        // Bottom-left: origin.y = output_h - MARGIN_Y - height. One row →
        // height = advance - (advance - size) = size = 20. So y = 600-16-20.
        let bawah = LayoutBlock { lines: vec![line("hi")], anchor: Anchor::KiriBawah, bg_mode: BgMode::None, x: 0.0, y: 0.0 };
        let laid = layout_block(&bawah, &params(), &Mock);
        // height ≈ size (20) for one row; y ≈ 600 - 16 - 20. Float-derived,
        // so compare with a tolerance rather than exact bits.
        assert!((laid.block.rows[0].y - (600.0 - MARGIN_Y - 20.0)).abs() < 0.05);
    }

    #[test]
    fn long_line_wraps_at_the_wrap_width() {
        // Narrow output so wrap floors at MIN_WRAP (80). Two 80px words +
        // a 10px space overflow → the space is swallowed, second word wraps.
        let mut p = params();
        p.output_w = 108.0; // wrap = max(80, 108 - 28) = 80
        let b = LayoutBlock { lines: vec![line("aaaaaaaa bbbbbbbb")], anchor: Anchor::KiriAtas, bg_mode: BgMode::None, x: 0.0, y: 0.0 };
        let laid = layout_block(&b, &p, &Mock);
        assert_eq!(laid.block.rows.len(), 2);
        assert_eq!(laid.block.rows[1].tokens[0].text, "bbbbbbbb");
        assert_eq!(laid.block.rows[1].tokens[0].x, MARGIN_X); // fresh row at origin
    }

    #[test]
    fn empty_block_has_no_rows_or_bounds() {
        let b = LayoutBlock { lines: vec![], anchor: Anchor::Free, bg_mode: BgMode::None, x: 0.0, y: 0.0 };
        let laid = layout_block(&b, &params(), &Mock);
        assert!(laid.block.rows.is_empty());
        assert!(laid.bounds.is_none());
    }

    #[test]
    fn block_bg_strip_wraps_the_row() {
        let b = LayoutBlock { lines: vec![line("hi")], anchor: Anchor::Free, bg_mode: BgMode::Block, x: 100.0, y: 50.0 };
        let laid = layout_block(&b, &params(), &Mock);
        let bg = laid.block.rows[0].bg.expect("block bg present");
        assert_eq!(bg.x, 100.0 - 6.0); // origin.x - 6
        assert_eq!(bg.w, 20.0 + 12.0); // row width ("hi"=20) + 12
    }

    #[test]
    fn mask_bg_spans_full_output_width() {
        let b = LayoutBlock { lines: vec![line("hi")], anchor: Anchor::Free, bg_mode: BgMode::Mask, x: 100.0, y: 50.0 };
        let laid = layout_block(&b, &params(), &Mock);
        let bg = laid.block.rows[0].bg.expect("mask bg present");
        assert_eq!(bg.x, 0.0);
        assert_eq!(bg.w, 800.0); // output_w
    }
}
