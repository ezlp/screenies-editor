# Changelog (ringkas per rilis)

- **Stable 1.1 fixes:** **undo no longer drops the chatlog** — block text is
  now synced to state synchronously on every keystroke, so a snapshot taken
  mid-edit (e.g. by a filter/drag commit) can never capture an empty chatlog
  and make it vanish on undo. **Default font is now Verdana** (Tahoma/Arial
  fallback) for crisper text at small SSRP sizes instead of the blurry Arial
  default. **Experimental egui `native/` shell removed** — the workspace is
  now just `core` + `src-tauri`; the WebView2-free migration continues but is
  no longer carried as dead code. Root `Cargo.toml` fixed to be a proper
  workspace manifest (CI `cargo test --workspace` was failing).
- **Pre-1.0 fixes (v0.15.0):** **Luar redesigned** — fixed resolutions stay
  EXACTLY as chosen: the strip is carved from inside, the photo shrinks into
  the remaining area, the crop box re-locks to it automatically, and the
  strip color is pickable (persisted; strip capped at 40% of the output).
  **BG strips** now sit on the glyphs (auto shift + a "Geser BG" fine-tune
  slider). **Template section removed** (unused).
- **M4c (v0.14.0) — the last feature milestone:** **Stickers** (import PNG/
  WebP, drag on the preview, per-sticker scale, alpha-blended in export);
  **background modes** per block — *blok* behind each row, *mask* full-width,
  and the ssrp-editor special **Luar (bawah foto)**: text in a black area
  appended below the photo so the screenshot stays 100% clean; **quick-text
  templates** persisted as templates.json (click chip = insert at cursor);
  and the save dialog now **remembers your last folder**.
- **Revisions (v0.13.0):** brightness now reaches **300%** (night shots),
  text size slider goes down to **8px** with an auto-outline that thins at
  small sizes so tiny text stays readable, and **custom save-file naming** —
  a persisted template with `{tanggal} {jam} {res} {foto}` placeholders.
- **Text styling + palette (v0.12.0, M4b):** *Outline* slider (0–10, or Auto
  that follows text size — 0 disables the stroke entirely, in export too),
  *Jarak baris* spacing slider, and the ssrp-editor **palette**: select text
  in a chatlog box → pick a color / quick swatch → `{RRGGBB}` codes wrap the
  selection. Fixes: Enter applies custom resolution; export no longer
  re-allocates stroke offsets per glyph.
- **Settings memory (v0.11.0, M4a):** theme, font, and your full preset
  (Kustom included) persist in `settings.json` and restore on launch.
  Exports & the font picker got faster — the system font scan now runs once
  per session instead of on every call. Dead "Zona & Warna" placeholder
  removed (its features shipped in v0.2–v0.5).
- **Export (v0.10.0, M3c):** *Save Disk (.png)* and *Copy ke Clipboard* are
  LIVE. Rust re-renders everything at full resolution — decode → crop →
  Lanczos resize → CSS-spec filter math → your font with a real outline —
  and the frontend ships its exact text layout, so the PNG matches the
  preview by construction. Stickers & background modes move to M4.
- **Live filters (v0.9.0, M3b):** Brightness / Grayscale / Sepia / Saturate /
  Contrast sliders with instant preview and per-filter reset — only the photo
  is filtered, chat text stays crisp. Dark-mode dropdown lists fixed
  (`color-scheme`). v0.9.1: filters render via a layered photo canvas so
  they work on Linux WebKitGTK too (ctx.filter is Chromium-only).
- **Resolution changer (v0.8.0, M3a):** pick 800×600 / 4:3 / 16:9 / 21:9 /
  4K / custom W×H / Bebas — a crop box drops on the photo (drag to frame,
  corner handles resize, ratio-locked unless Bebas, double-click recenters).
  The preview then shows the exact output; text lives in output space, so
  M3c's exporter will match it pixel-for-pixel.
- **Preset files `.toml` (v0.7.0):** *Impor / Ekspor .toml* buttons — tune
  Kustom until your server's log looks right, export, share the file on the
  wiki; friends just import. Step-by-step guide: [`docs/PRESETS.md`](docs/PRESETS.md), ready-made examples in [`examples/presets/`](examples/presets/).
- **Dark & light mode (v0.6.0):** toggle in the top bar; themes are pure
  CSS-variable blocks, so custom themes can be added later.
- **Preset format (v0.5.0):** parsing rules are data, not code — pick
  **JGRP**, **SA-MP Umum**, or **Polos**, or open **Kustom…** to toggle every
  rule and recolor /me & OOC. Schema + examples in
  [`docs/PRESETS.md`](docs/PRESETS.md) — wiki-ready.
- **Font picker:** choose any font installed on the PC — Rust enumerates the
  real system font list (not a hardcoded set). Auto text size now floors at
  16px so low-res screenshots stay readable.
- **Multi-chatlog blocks:** "+ Tambah Chatlog" adds independent blocks, each
  with its own text and position dropdown — **Bebas** (drag it anywhere on the
  photo, double-click to reset) or locked to **Kiri Atas / Kiri Bawah**.
  Positions stick through zoom.
- HUD shows resolution + zoom; version badge proves the TS↔Rust bridge works

Filters, zones, colors, crop, stickers, export = Milestones 2–4
(panels are visible but marked and disabled).

## 5 · If a build fails

Open **Actions** → click the red ✗ run → click the failing step —
the log shows the exact file + line. Copy that log back into the chat
and we fix it together. That's our debugging loop.

## 6 · Optional: run locally with hot-reload (later)

Only when CI round-trips get annoying:

1. Install Node.js ≥ 20: <https://nodejs.org>
2. Install Rust: <https://rustup.rs>
3. Linux only — system deps:
   `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
4. Then:

```bash
npm install
npm run tauri dev     # opens the app; edits hot-reload in ~2 s
```

(`npm run dev` alone opens the UI in a browser with a fallback parser —
handy for CSS work, but the real parsing runs in Rust inside the app.)

## 7 · Project map

```
index.html                 UI layout (3 panels)
src/styles/                theme.css · main.css · panels.css
src/ts/                    frontend logic
  ├─ types.ts              ⚠ mirrors Rust structs — keep in sync
  ├─ tauri-bridge.ts       the ONLY file that calls Rust
  ├─ state.ts              single app state
  ├─ canvas.ts             preview: draw, zoom, pan, FIT
  ├─ chatlog.ts            upload + chatlog input
  └─ (zones, crop, …)      M2–M4 stubs, documented
src-tauri/src/             Rust backend
  ├─ main.rs               entry + module tree
  ├─ commands.rs           thin command layer (no logic — rule)
  ├─ error.rs              one serializable error type
  ├─ chatlog/              ✅ timestamp.rs working · parser/autocolor = M2
  └─ render/, files.rs, …  M3–M4 stubs
.github/workflows/         build.yml (checks) · release.yml (installers)
```

## Roadmap

- [x] **M1** — skeleton: window, upload, preview, timestamp removal, CI
- [x] **M2** — text engine ✅ (v0.4.0): `{RRGGBB}` parsing (case-insensitive),
  auto-color (`*` ungu, `(( ))` abu-abu, `/do` suffix `((Nama))` ungu),
  generic `says [apapun]:` / `[phone]:` / `[walkie]:` variants, bold tags on
  hex-prefixed lines. Manual color palette → M4. Rules are preset-driven since v0.5.0.
- [x] **M3** — image engine, split in three: **M3a crop/resolution ✅ (v0.8.0)** → **M3b live filters ✅ (v0.9.0)** → **M3c Rust export ✅ (v0.10.0)** — stickers & background modes → M4
- [x] **M4** — comfort, split in three: **M4a settings memory + cleanup ✅ (v0.11.0)** → **M4b styling + palette ✅ (v0.12.0)** → **M4c stickers, backgrounds, templates ✅ (v0.14.0)**
- [ ] **v1.0** — polish + release to the community
