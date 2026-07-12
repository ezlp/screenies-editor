<div align="center">

# 🖼️ ScreeniesEditor

**A Screenshot Roleplay (SSRP) editor for the SA-MP community**

Paste a chatlog → automatic colors → crop → filters → export a sharp PNG.
Fully offline — nothing is uploaded anywhere.

*by Isut Indraputra & Claude (Anthropic)* · **Bahasa Indonesia:** [README.md](README.md)

</div>

---

## ⚠️ Windows note: WebView2 (Edge runtime) required

The current version runs on **Microsoft WebView2**. Windows 11 and
up-to-date Windows 10 ship it; **older laptops** may not — the installer
downloads it automatically during setup (needs internet once). If that
fails, install "WebView2 Runtime (Evergreen)" from Microsoft manually.

**Good news:** we are **migrating to a lighter technology** with no
WebView2 at all — so older machines will be supported.

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

Tauri 2 + Rust core (parser & render pipeline) + TypeScript/Vite UI.
Architecture: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) ·
Preset guide: [docs/PRESETS.md](docs/PRESETS.md) ·
Changelog: [docs/CHANGELOG.md](docs/CHANGELOG.md)

<div align="center">Made with ❤️ for the SA-MP roleplay community</div>
