# ScreeniesEditor — Developer Guide

How the codebase works, for anyone (including future-us) touching the code.

> **2.0 is a native egui app.** The old Tauri (WebView2) shell + its TypeScript
> frontend were removed; they live on `main` / `v1.*` tags. See
> [2.0-MIGRATION.md](2.0-MIGRATION.md).

## Layout

```
core/   screenies-core — the shell-independent engine (all the real logic):
        chatlog/     parser: timestamp → autocolor → systag → preset
        render/      compose → crop → filters (incl. blur/pixelate) →
                     sticker (with opacity) → layout (wrap+positioning) → text
        chatlog_library.rs   folder index + search
        gallery.rs           exported-photo listing & file filter
        fonts.rs             shared fontdb (scanned once)
native/ screenies-native — the egui/eframe desktop shell (pure Rust):
        main.rs             app entry, screen router, persistent Settings
        editor.rs           editor state, Classic UI & Unified Fast-Editor UI modes
        gallery.rs          Dual-Tab Gallery (Sources vs Edits), Smart Albums & ImgBB Cloud Uploader
        theme.rs            Theme Engine (7 themes + accent picker + density)
        i18n.rs             bilingual localization dictionary (ID / EN)
        chatlog_browser.rs  chatlog search & instant text grabber
examples/presets/   community parsing presets (.toml)
docs/               technical guides, presets schema, changelog & migration docs
```

`cargo test --workspace` builds/tests `core` (fast; no GUI/JS needed). CI runs
that; `native/` is compiled + packaged by `.github/workflows/native-preview.yml`.

## Enduring contracts

1. **`core` is shell-independent.** No windowing, no dialogs, no I/O beyond what
   it's handed. Anything that computes lives here, tested once, reused by any
   shell (egui now, WASM later). The egui app is a thin view over it.
2. **Layout drives export.** `render::layout` positions every text token/row
   absolutely in output space; the preview and the PNG are the SAME
   `compose::render` call, so preview == export by identity (not by two
   implementations agreeing). The real measurer is `text::GlyphMeasure`, which
   sums the same ab_glyph advances the renderer pens with.
3. **Backward-compatible payloads.** Structs that get (de)serialized use
   `#[serde(default)]` on new fields so old settings/preset files keep loading
   (e.g. the 2.0 `blur`/`pixelate` filter fields).

## Rendering pipeline (`core/src/render/compose.rs`)

base64/loaded photo → decode → crop (crop.rs) → Lanczos3 resize → CSS-spec
color filters + blur/pixelate passes (filters.rs, unit-tested) → sticker
overlays (sticker.rs, alpha) → text (layout.rs positions; text.rs draws BG
strips, outline, fill; fonts from the shared fontdb).

## The egui shell (`native/`)

`main.rs` — landing menu → Editor / Chatlog Parser / Gallery.
`editor.rs` — editor state + UI. On any change it rebuilds a `RenderJob`
(state → `layout::layout_blocks` with `GlyphMeasure` → `compose::render`) and
shows the resulting image as an egui texture; export re-uses the same job +
`encode_png`. That shared call is why the preview matches the saved PNG.

## Conventions

- One module = one concern. No shell/I-O logic in `core`.
- Indonesian for user-facing strings, English for code/comments.
- Every render/effect/layout change ships with a unit test in the same file.

## Build

- Engine: `cargo test --workspace` (CI).
- Native app: `cd native && cargo run` — needs egui GUI deps
  (`libgtk-3-dev libxkbcommon-dev libwayland-dev` + OpenGL on Linux).
- Preview releases (.exe/.deb/.rpm): push a `native-preview-*` tag →
  `native-preview.yml` builds + publishes a GitHub pre-release.
