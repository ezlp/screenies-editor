# ScreeniesEditor

Desktop **Screenshot Roleplay (SSRP) editor** for the SA-MP community —
the merger of [ssrp-editor](https://ssrp-editor.netlify.app/) and
[ssrphelper](https://ssrphelper.netlify.app/ssrphelper.html) as a native app.

Built with **Tauri v2** (Rust backend) + **TypeScript / Vite** (frontend).
Windows + Linux. You do **not** need Rust or Node installed — GitHub Actions
compiles everything and hands you the installers.

---

## 1 · One-time setup (~15 minutes)

1. Install **Git**: <https://git-scm.com/downloads> (default options are fine)
2. Create a GitHub account: <https://github.com/signup>
3. Create a **new empty repository** named `screenies-editor`
   (no README, no .gitignore — this folder already has them)

## 2 · Push this project

Open a terminal (**Git Bash** on Windows) inside this folder, then —
replace `YOUR_USERNAME` with your GitHub username:

```bash
git init
git add .
git commit -m "Milestone 1: skeleton — upload, preview, timestamp removal"
git branch -M main
git remote add origin https://github.com/YOUR_USERNAME/screenies-editor.git
git push -u origin main
```

Pushing triggers **Build Check** (Actions tab) — it type-checks the
TypeScript and runs the Rust unit tests. Wait for the green ✓ (~5–10 min
the first time).

## 3 · Build the installers (first release)

```bash
git tag v0.1.0
git push --tags
```

This triggers the **Release** workflow. ~15–20 minutes later, open your
repo's **Releases** page (right sidebar) and download:

| OS | File |
|---|---|
| Windows | `ScreeniesEditor_0.3.1_x64_en-US.msi` (or the `.exe`) |
| Linux | `.AppImage` (run directly) or `.deb` |

> **Windows SmartScreen:** the app is not code-signed, so Windows shows
> "Windows protected your PC" on first run. Click **More info → Run anyway**.
> This is normal for unsigned open-source apps.

## 4 · What Milestone 1 can do

- Open a native window (dark UI, SA-MP purple accent)
- Upload a screenshot — click or **drag & drop**
- **Ctrl + Scroll** zoom · drag to pan · **FIT** / **+** / **−** buttons
- Paste a chatlog → **Rust strips `[00:00:00]` timestamps** live
- Lines draw on the preview in SSRP style (white, black stroke) —
  **text size auto-scales to the photo's resolution**, adjustable with the
  *Ukuran teks* slider
- System/game messages (`VEHICLE:`, `ERROR:`, `AdmCmd:` — any `Word:` prefix,
  pattern-based so all ~30 JGRP tags and future ones work) render the tag
  **bold** automatically
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

---

**ScreeniesEditor** — dibuat oleh **Isut Indraputra**, dibangun bersama Claude (Anthropic).
