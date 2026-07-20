<div align="center">

# 🖼️ ScreeniesEditor v4.0.0

**Native Screenshot Roleplay (SSRP) Editor for SA-MP & GTA Roleplay Communities**

Paste Chatlog → Auto Color → Edit & Crop → ImgBB Upload / Storyline Albums → Export Sharp PNG.  
Built 100% in **Native Rust (egui)** — ultra-fast, zero WebView2/Edge, low RAM footprint, smooth on low-spec laptops.

*by Isut Indraputra & DeepMind (Google Antigravity)* · **Bahasa Indonesia:** [README.md](README.md)

</div>

---

## 🌟 Key Features in v4.0.0

### 🗂️ 1. Segmented Dual-Tab Gallery (Sources vs Edits)
* **Source Shots Tab**: Browse raw, unedited in-game screenshots.
* **Finished Edits Tab**: Manage and organize exported SSRP artwork.

### 📚 2. Smart Albums & Storyline Narrative Logs
* Create storyline albums (e.g., *Faction Heist*, *Daily Patrol*, *Business Meeting*).
* Edit custom **album titles & narrative storyline logs**.
* Assign screenshots to albums and toggle **Album Filtering** to focus your gallery view.

### ☁️ 3. ImgBB Cloud Direct Uploader & Inline Link Copy
* Upload finished screenshots to ImgBB cloud storage in the background (non-blocking async).
* Displays raw direct image URLs inline right beneath thumbnails with zero intrusive popups.
* One-click **Copy Link (📋)** button to clipboard.
* User API key configuration in Settings.

### ⚡ 4. Unified Fast-Editor UI & Custom Shortcuts
* **Unified UI Mode**: Consolidates all tool panels (Photo, Chatlog, Text, Crop, Fx) into a single collapsible sidebar for lightning-fast editing.
* **Layout Switcher**: Header toggle button `🗂 Unified UI` / `🔲 Classic UI` to switch layouts instantly.
* **Shortcuts & Keybindings**: Customizable hotkeys mapping table for fast actions (`Open`, `Paste`, `Export`, `Undo`, `Redo`, `Cinematic`).

### 🎨 5. Theme Engine & UI Density Customization
* 7 Built-in Themes (Midnight, Paper, Dark, Light, Cyberpunk, Forest, Slate).
* Custom Accent Color picker.
* UI Density toggle (Cozy vs Compact).
* Full localization support for English & Bahasa Indonesia.

---

## 🔧 Tech Stack & Performance

| Component | Technology |
|---|---|
| **Core Engine** | Pure Rust (`screenies-core`) — chatlog parser, composition pipeline, filters, crop, stickers, font rasterization |
| **Desktop Shell** | `egui` / `eframe` — Native GUI without WebView2 or Edge dependencies |
| **Networking & HTTP** | `ureq` + `rustls` — Pure-Rust TLS HTTP stack without system OpenSSL dependencies |
| **Render Engine** | `image` + `ab_glyph` + `fontdb` — Lanczos resampling, blur/pixelate filters, text stroke rendering |

---

## ⬇️ Download

Download the pre-compiled binaries from **[Releases](../../releases/tag/v4.0.0)**:
* 🪟 **Windows**: `screenies-editor-v4.0.0-windows-x86_64.exe` (Portable Executable)
* 🐧 **Linux**: `screenies-editor-v4.0.0-linux-x86_64` (Raw Binary), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `.AppImage`

## 🗺️ Project Map

```
core/   screenies-core — standalone pure Rust engine (all core logic):
        chatlog/     parser: timestamp → autocolor → systag → preset
        render/      compose → crop → filters (blur/pixelate) → sticker → layout → text
        chatlog_library.rs   folder index & search
        gallery.rs           photo lister & file filter
        fonts.rs             shared fontdb (one-time system font scan)
native/ screenies-native — native egui/eframe desktop shell (pure Rust):
        main.rs             app entry, screen navigation & persistent Settings
        editor.rs           editor state, Classic UI & Unified Fast-Editor UI modes
        gallery.rs          Dual-Tab Gallery, Smart Albums & ImgBB Cloud Uploader
        theme.rs            Theme Engine (7 themes + accent picker + UI density)
        i18n.rs             bilingual localization dictionary (ID / EN)
        chatlog_browser.rs  chatlog search & instant text grabber
examples/presets/   community color presets (.toml) for chatlog parser
docs/               technical guides, schema docs, changelog & migration notes
```

---

## 📖 Developer Documentation

Refer to technical guides in `docs/`:
* **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** — Directory layout & contribution guidelines.
* **[PRESETS.md](docs/PRESETS.md)** — Chatlog parser auto-color preset specification.
* **[CHANGELOG.md](docs/CHANGELOG.md)** — Release history & version notes.

---

<div align="center">Made with ❤️ for the SA-MP roleplay community</div>
