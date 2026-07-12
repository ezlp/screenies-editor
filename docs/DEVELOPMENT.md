# ScreeniesEditor — Developer Guide

How the codebase works, for anyone (including future-us) touching the code.

> **2.0 migration in progress.** The app is moving from a Tauri (webview +
> TypeScript) shell to a **Qt (CXX-Qt + QML)** shell, keeping the Rust engine.
> The Tauri stack and its TypeScript frontend have been removed. See
> [2.0-MIGRATION.md](2.0-MIGRATION.md) for the plan and current phase.

## Layout

```
core/   screenies-core — the shell-independent engine (all the real logic):
        chatlog/   parser: timestamp → autocolor → systag
        render/    export pipeline: compose→crop→filters→sticker→text
                   (filters.rs also has the 2.0 blur/pixelate effects)
        chatlog_library.rs   folder index + search   (2.0 feature)
        gallery.rs           exported-photo listing   (2.0 feature)
        fonts.rs   shared fontdb (scanned once)   clipboard.rs   error.rs
qt/     screenies-qt — the 2.0 Qt shell (CXX-Qt + QML). Consumes core.
        Excluded from the default workspace (needs Qt6). Build:
        `cargo run -p screenies-qt`.
examples/presets/   community parsing presets (.toml) the parser accepts.
```

`cargo test --workspace` builds and tests `core` (fast, no Qt/JS needed).

## Enduring contracts (read before changing the engine)

1. **`core` is shell-independent.** No windowing, no dialogs, no IPC. Anything
   that computes lives here, tested once, reused by any shell (Tauri before,
   Qt now, WASM later). The Qt shell is a thin adapter over it.
2. **Types crossing a shell boundary are mirrored, field-for-field**, and new
   fields use `#[serde(default)]` so old settings/preset/RenderJob files keep
   loading. (Example: the 2.0 `blur`/`pixelate` filter fields defaulted in so
   1.x export payloads still parse.)
3. **Layout drives export.** The renderer positions every text token/row
   absolutely in output space; whoever draws the preview must use the SAME
   positions so preview == exported PNG. In 1.x the TS canvas owned this; in
   2.0 the Qt preview must adopt `compose::render` (parity by identity).

## Rendering pipeline (`core/src/render/compose.rs`)

base64 photo → decode → crop (crop.rs) → Lanczos3 resize to the photo area →
CSS-spec color filters + blur/pixelate passes (filters.rs, unit-tested) →
paste onto the `output`-sized canvas → sticker overlays (sticker.rs, alpha) →
text (text.rs: BG strips, outline, fill; fonts from the shared fontdb).

## Conventions

- One module = one concern. No I/O or shell calls in `core`.
- Indonesian for user-facing strings, English for code/comments.
- Every render/effect change ships with a unit test in the same file.

## Build

- Engine: `cargo test --workspace` (this is what CI runs).
- Qt shell: `cargo run -p screenies-qt` — needs a local Qt6 + C++ toolchain.
  Packaging/CI for the Qt app arrives in migration phase 6.
