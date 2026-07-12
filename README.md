<div align="center">

# 🖼️ ScreeniesEditor

**Editor Screenshot Roleplay (SSRP) untuk komunitas SA-MP Indonesia**

Tempel chatlog → warna otomatis → crop → filter → export PNG tajam.
Semua offline, tanpa upload ke mana-mana.

*oleh Isut Indraputra & Claude (Anthropic)*

</div>

---

## ✨ Fitur

| | |
|---|---|
| 📋 **Parser chatlog** | Tempel log mentah — timestamp `[HH:MM:SS]` dibuang, `*` jadi ungu /me, `(( ))` jadi abu OOC, `/do ((Nama))`, tag `SERVER:`/`Apapun:` otomatis bold, kode `{RRGGBB}` didukung |
| 🧩 **Preset per server** | Bawaan JGRP / SA-MP Umum / Polos + **Kustom** penuh; bagikan sebagai file `.toml` (Impor/Ekspor) — [panduan](docs/PRESETS.md) & [contoh](examples/presets/) |
| 🖼️ **Resolusi & crop** | 800×600 · 4:3 · 16:9 · 21:9 · 4K · W×H kustom · Bebas — kotak crop dengan ratio lock |
| 🎛️ **Filter live** | Brightness (s.d. 300%) / Grayscale / Sepia / Saturate / Contrast — foto saja, teks tetap tajam |
| ✍️ **Kontrol teks** | Font sistem, ukuran 8–60px, outline 0–10px (auto), jarak baris, palet warna seleksi |
| 🏷️ **Background teks** | BG *blok* (per baris) atau *mask* (selebar gambar) per chatlog, dengan slider Geser BG |
| 🖼️ **Stiker** | Import PNG/WebP, seret di preview, skala 10–300% |
| 💾 **Export Rust** | Save Disk (.png) & Copy ke Clipboard — render penuh sampai 4K, **persis seperti preview** |
| ⚙️ **Nyaman** | Undo/redo (Ctrl+Z/Y), paste foto (Ctrl+V), Bahasa ID/EN, dark/light mode, settings tersimpan, template nama file, folder terakhir diingat |

> **English:** see [README.en.md](README.en.md)

## ⚠️ Catatan Windows: butuh WebView2 (runtime Edge)

Versi saat ini memakai **Microsoft WebView2**. Windows 11 & Windows 10
terbaru sudah punya bawaan; **laptop lama** mungkin belum — installer akan
mengunduhnya otomatis saat pemasangan (butuh internet sekali). Kalau
gagal, pasang manual "WebView2 Runtime (Evergreen)" dari situs Microsoft.

**Kabar baiknya:** kami sedang **migrasi ke teknologi lain** yang lebih
ringan dan tanpa WebView2 sama sekali, supaya laptop lama tetap bisa jalan.

## 📥 Download & Install

Ambil dari **[Releases](../../releases)**: Windows `-setup.exe` (64/32-bit) ·
Linux `.deb` / `.AppImage`. Pakai yang berlabel **Latest**; release berlabel
*Pre-release* / *nightly* adalah build percobaan untuk penguji. Windows SmartScreen: *More info → Run anyway*
(installer belum ditandatangani). Panduan lengkap ada di **Wiki**.

## 🚀 Cara pakai (30 detik)

1. Seret screenshot SA-MP ke app
2. Pilih resolusi → atur kotak crop → **✓ Selesai**
3. Tempel chatlog → atur posisi (seret teks langsung di preview)
4. Filter/stiker sesuai selera → **Save Disk (.png)**

## 🔧 Teknologi & kenapa dipakai

| Teknologi | Untuk apa |
|---|---|
| **[Tauri 2](https://tauri.app)** (Rust) | Shell desktop ringan lintas-OS — installer kecil, bukan Electron |
| **Rust** | Parser chatlog + seluruh pipeline export: cepat & teruji (`cargo test`) |
| **TypeScript + Vite** | UI dan preview canvas dengan type-safety penuh |
| **fontdb** | Baca font sistem — sekali scan, dipakai picker & exporter |
| **image + ab_glyph** | Decode/crop/resize Lanczos, matematika filter sesuai spec CSS, rasterisasi teks + outline |
| **arboard** | Copy PNG langsung ke clipboard |
| **serde (+ TOML/JSON)** | Preset `.toml` & `settings.json` yang forward-compatible |
| **GitHub Actions** | Build & release otomatis Windows/Linux tiap tag |

Arsitektur & panduan kontributor: **[docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)** ·
Riwayat rilis: **[docs/CHANGELOG.md](docs/CHANGELOG.md)**

## 🗺️ Setelah 1.0

Browser chatlog (buka folder log, cari di app), efek per-area
(blur/pixelate untuk sensor nama — bisa kena teks), galeri hasil edit.
Detail di [CHANGELOG](docs/CHANGELOG.md).

---

<div align="center">Dibuat dengan ❤️ untuk komunitas roleplay SA-MP Indonesia</div>
