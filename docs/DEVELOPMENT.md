# ScreeniesEditor — Developer Guide

How the codebase works, for anyone (including future-us) touching the code.

> **Workspace note:** the Rust side is a Cargo workspace —
> `core/` (screenies-core: parser + render, shell-free) and `src-tauri/`
> (the shipping app). `cargo test --workspace` covers both.

## Big picture

```
┌────────────── Frontend (TypeScript + Vite) ──────────────┐
│ index.html — all static UI                               │
│ src/ts/                                                  │
│   state.ts        single mutable AppState + onChange bus │
│   canvas.ts       preview renderer + ALL text layout     │
│   chatlog.ts      blocks UI, debounced parse calls       │
│   crop.ts filters.ts textstyle.ts colorpalette.ts        │
│   stickers.ts preset.ts theme.ts settings.ts export.ts   │
│   tauri-bridge.ts THE ONLY file that talks to Rust       │
└──────────────────────────┬────────────────────────────────┘
                    invoke() over Tauri IPC
┌──────────────────────────┴─── Backend (Rust) ─────────────┐
│ main.rs      registers commands + plugins                 │
│ commands.rs  thin #[tauri::command] wrappers (no logic)   │
│ chatlog/     parser: timestamp → autocolor → systag       │
│ render/      export: compose→crop→filters→sticker→text    │
│ config.rs    settings.json   files.rs  dialogs & disk     │
│ fonts.rs     shared fontdb (scanned once)  clipboard.rs   │
└────────────────────────────────────────────────────────────┘
```

## The three contracts (read before changing anything)

1. **Layout is frontend-owned.** `canvas.ts::buildRenderBlocks()` computes
   every text token's absolute x/y (plus BG strips) in output space. The
   preview paints these AND the exporter receives these — Rust never
   re-wraps or re-measures. This is why preview == PNG. If you change
   layout, you change it in exactly one place.
2. **Types are mirrored, field-for-field.** Every payload crossing IPC has
   a TS interface (tauri-bridge.ts) and a Rust struct
   (`serde(rename_all = "camelCase")`). New fields use `#[serde(default)]`
   so old settings/preset files keep loading. CI-adjacent check: the parity
   scripts in the project history compare both sides.
3. **Slow or dialog-opening commands are `async`.** Sync commands run on
   the main thread; a blocking dialog there deadlocks the app (learned in
   v0.7.1). Renders are async for the same reason.

## Rendering pipeline (export, render/compose.rs)

base64 photo → decode → crop (crop.rs) → Lanczos3 resize to the photo area
→ CSS-spec filter math (filters.rs, unit-tested against the spec) → paste
onto a canvas of `output` size filled with `luarColor` → sticker overlays
(sticker.rs, alpha) → text (text.rs: BG strips, outline via radial stamps,
fill; fonts from the shared fontdb).

The preview mirrors this with two stacked canvases: `#image-canvas`
(photo, CSS `filter` on the element — WebKitGTK has no ctx.filter) under
`#preview-canvas` (strips, stickers, text, crop UI, pointer events).

## Conventions

- One module = one concern; `initX()` wires DOM once; state mutations call
  `notify()`; `canvas.draw()` is the single onChange consumer that repaints.
- No logic in commands.rs — it delegates to the owning module.
- Indonesian for user-facing strings, English for code/comments.
- Version lives in THREE files: package.json, tauri.conf.json, Cargo.toml.

## Build & release

- Local: `npm install && npm run tauri dev` (needs Rust + Node 22).
- CI: every push → .github/workflows/build.yml (tsc + vite + cargo test).
- **Release channels:**
  - `vX.Y.Z` tag → **stable** release, becomes "Latest".
  - `vX.Y.Z-beta.N` tag (anything with `-`) → **pre-release** badge,
    never "Latest". Version files should carry the same string.
  - Actions → *Nightly (dev snapshot)* → Run workflow → rolling
    pre-release tagged `nightly`; each run replaces the last, installers
    are stamped `X.Y.Z-nightly.<sha>` so bug reports pin the commit.
    Manual-dispatch only (no cron) — press it when there's something
    worth testing.
- Rust tests: `cargo test` in src-tauri (parser pins real JGRP lines,
  filter math, TOML round-trips, filename sanitizing, compose smoke test).

## Adding a feature — worked example ("add a new filter")

1. state.ts: add the field to `Filters` + default.
2. filters.ts `DEFS`: add key + max → slider auto-wires.
3. canvas.ts `cssFilterString()`: append the CSS function.
4. render/filters.rs: implement the same math per the CSS spec + a test.
5. index.html: copy a `.filter-row`. Done — export parity is automatic
   because both sides read the same `Filters` payload.
