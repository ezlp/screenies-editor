<div align="center">

# 🖼️ ScreeniesEditor 2.0

**Editor Screenshot Roleplay (SSRP) untuk komunitas SA-MP Indonesia**

Tempel chatlog → warna otomatis → filter → export PNG tajam.
Semua offline, tanpa upload ke mana-mana.

*oleh Isut Indraputra & Claude (Anthropic)* · **English:** [README.en.md](README.en.md)

</div>

---

## 🚧 Status: 2.0 preview (native, tanpa webview)

Versi 2.0 adalah aplikasi **native Rust (egui)** — **tanpa WebView2/Edge**,
biar jalan di laptop lama, binary kecil. Backend Rust (`core`) sama seperti
sebelumnya. Saat ini **tahap preview/alpha**.

- **Coba preview:** ambil dari **[Releases](../../releases)** (`native-preview-*`)
  — Windows `.exe`, Linux `.deb` / `.rpm` / binary.
- Versi **1.x** (berbasis Tauri/WebView2) yang lama ada di branch `main`
  dan tag `v1.*`.

## ✨ Fitur (preview)

| | |
|---|---|
| 📋 **Parser chatlog** | Timestamp `[HH:MM:SS]` dibuang, `*` /me ungu, `(( ))` OOC abu, `/do`, tag `SERVER:` bold, kode `{RRGGBB}` — preset per server (JGRP/Umum/Polos) |
| 🖼️ **Foto** | Muat gambar (PNG/JPG/WebP/BMP), preview langsung |
| 🎛️ **Filter** | Brightness / Contrast / Grayscale / Sepia / Saturate |
| ✨ **Efek 2.0** | **Blur** & **Pixelate** (sensor nama/plat) |
| ✍️ **Teks** | Font, ukuran 8–60px, outline (auto), jarak baris |
| 🏷️ **Background teks** | Blok (per baris) / Mask (selebar) · posisi Bebas / Kiri Atas / Kiri Bawah |
| 💾 **Export** | Render penuh via `core` — **preview = PNG** (parity by identity) |

Belum masuk preview (fase berikutnya): crop editor, stiker, palet warna,
undo/redo, multi-chatlog, i18n, simpan setelan, **Chatlog Parser** (cari di
folder log), **Gallery** hasil edit.

## 🔧 Teknologi

| | |
|---|---|
| **Rust** | Seluruh logika di crate `core` — parser + pipeline render/export, teruji (`cargo test`) |
| **egui / eframe** | Shell desktop native, pure-Rust, tanpa C++/webview |
| **image + ab_glyph** | Decode/crop/resize Lanczos, filter (incl. blur/pixelate), rasterisasi teks + outline |
| **fontdb** · **arboard** | Font sistem · copy PNG ke clipboard |

Arsitektur: **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** ·
Rencana 2.0: **[docs/2.0-MIGRATION.md](docs/2.0-MIGRATION.md)** ·
Preset: **[docs/PRESETS.md](docs/PRESETS.md)** ·
Riwayat: **[docs/CHANGELOG.md](docs/CHANGELOG.md)**

---

<div align="center">Dibuat dengan ❤️ untuk komunitas roleplay SA-MP Indonesia</div>
