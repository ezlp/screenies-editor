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
| 🏷️ **BG & Luar** | BG blok / mask per chatlog + posisi **Luar (bawah foto)**: teks di area warna solid di bawah foto — screenshot bersih |
| 🖼️ **Stiker** | Import PNG/WebP, seret di preview, skala 10–300% |
| 💾 **Export Rust** | Save Disk (.png) & Copy ke Clipboard — render penuh sampai 4K, **persis seperti preview** |
| ⚙️ **Nyaman** | Dark/light mode, settings tersimpan, template nama file `{tanggal} {jam} {res} {foto}`, folder terakhir diingat |

## 📥 Download & Install

Ambil dari **[Releases](../../releases)**: Windows `-setup.exe` (64/32-bit) ·
Linux `.deb` / `.AppImage`. Windows SmartScreen: *More info → Run anyway*
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
