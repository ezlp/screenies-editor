<div align="center">

# 🖼️ ScreeniesEditor

**A Screenshot Roleplay (SSRP) editor for the SA-MP community**

Paste a chatlog → automatic colors → crop → filters → export a sharp PNG.
Fully offline — nothing is uploaded anywhere.

*by Isut Indraputra & Claude (Anthropic)* · **Bahasa Indonesia:** [README.md](README.md)

</div>

---

## 🚧 Status: 2.0 migration (Qt)

We're **migrating from Tauri (WebView2/WebKitGTK) to a Qt shell**, keeping the
same Rust backend, so the app runs on older laptops with no WebView2 at all.
Full plan: [docs/2.0-MIGRATION.md](docs/2.0-MIGRATION.md).

During the migration there is **no runnable 2.0 build yet**. The stable **1.x**
(WebView2) app stays available via **[Releases](../../releases)** and the
`main` branch.

## ✨ Features

Chatlog parser with per-server presets (shareable `.toml` files) ·
resolution presets + crop editor · live filters (photo only, text stays
crisp) · text controls (font, size 8–60px, outline, spacing, color
palette) · per-block backgrounds (block/mask) · PNG/WebP stickers ·
Rust-rendered export (Save to Disk / Copy to Clipboard, up to 4K) ·
undo/redo (Ctrl+Z / Ctrl+Y) · paste photo with Ctrl+V · Indonesian/English
UI · dark & light themes · persistent settings and file-name templates.

## 📥 Download & Install

Grab it from **[Releases](../../releases)** — Windows `-setup.exe`
(64/32-bit), Linux `.deb` / `.AppImage`. Use the release marked
**Latest**; *Pre-release* / *nightly* builds are for testers.
Windows SmartScreen: *More info → Run anyway* (the installer isn't
code-signed).

## 🚀 30-second workflow

1. Drag a SA-MP screenshot into the app (or press **Ctrl+V**)
2. Pick a resolution → frame the crop box → **✓ Done**
3. Paste your chatlog → drag the text into place on the preview
4. Filters/stickers to taste → **Save to Disk (.png)**

## 🔧 Tech & docs

Rust `core` (parser & render pipeline, incl. blur/pixelate) + a Qt (CXX-Qt +
QML) shell *(2.0, in progress)*. The old 1.x Tauri + TypeScript/Vite shell
lives in branch `main` / `v1.*` tags.
Architecture: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) ·
Preset guide: [docs/PRESETS.md](docs/PRESETS.md) ·
Migration: [docs/2.0-MIGRATION.md](docs/2.0-MIGRATION.md) ·
Changelog: [docs/CHANGELOG.md](docs/CHANGELOG.md)

<div align="center">Made with ❤️ for the SA-MP roleplay community</div>
