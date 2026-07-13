<div align="center">

# 🖼️ ScreeniesEditor 2.0

**A Screenshot Roleplay (SSRP) editor for the SA-MP community**

Paste a chatlog → automatic colors → filters → export a sharp PNG.
Fully offline — nothing is uploaded anywhere.

*by Isut Indraputra & Claude (Anthropic)* · **Bahasa Indonesia:** [README.md](README.md)

</div>

---

## 🚧 Status: 2.0 preview (native, no webview)

2.0 is a **native Rust (egui)** app — **no WebView2/Edge**, so it runs on old
laptops with a small binary. Same Rust engine (`core`) as before. Currently in
**preview/alpha**.

- **Try the preview:** grab it from **[Releases](../../releases)**
  (`native-preview-*`) — Windows `.exe`, Linux `.deb` / `.rpm` / raw binary.
- The old **1.x** (Tauri/WebView2) build lives on branch `main` and `v1.*` tags.

## ✨ Features (preview)

Chatlog parser with per-server presets (JGRP/Umum/Polos, `{RRGGBB}` colors,
auto-color for `*`/`(( ))`/`/do`/system tags) · load photo · filters
(brightness/contrast/grayscale/sepia/saturate) · **2.0 effects: blur & pixelate**
(censor names/plates) · text controls (font, 8–60px, outline, spacing) ·
per-block background (block/mask) · anchor (free/top-left/bottom-left) ·
live preview + PNG export rendered by `core` (**preview == PNG**).

Not yet in the preview (next phases): crop editor, stickers, color palette,
undo/redo, multi-block, i18n, settings, **Chatlog Parser** (search a log
folder), and the **Gallery** of edited shots.

## 🔧 Tech

Rust `core` (parser + render/export pipeline, unit-tested) + an **egui/eframe**
native shell (pure Rust, no C++/webview) + `image`/`ab_glyph` for
decode/crop/resize/filters (incl. blur/pixelate) and text rasterization.

Architecture: [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) ·
2.0 plan: [docs/2.0-MIGRATION.md](docs/2.0-MIGRATION.md) ·
Presets: [docs/PRESETS.md](docs/PRESETS.md) ·
Changelog: [docs/CHANGELOG.md](docs/CHANGELOG.md)

<div align="center">Made with ❤️ for the SA-MP roleplay community</div>
