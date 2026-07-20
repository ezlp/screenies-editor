<div align="center">

# 🖼️ ScreeniesEditor v4.0.0

**Editor Screenshot Roleplay (SSRP) Native untuk Komunitas SA-MP & GTA Roleplay**

Tempel chatlog → Warna Otomatis → Edit & Crop → Unggah ImgBB / Album Cerita → Export PNG Tajam.  
Murni berbasis **Native Rust (egui)** — super cepat, tanpa WebView2/Edge, hemat RAM & berjalan mulus di laptop spek rendah.

*Oleh Isut Indraputra & DeepMind (Google Antigravity)* · **English Version:** [README.md](README.md)

</div>

---

## 🌟 Fitur Utama v4.0.0

### 🗂️ 1. Galeri Dua Tab (Tangkapan Mentah vs Hasil Edit)
* **Tab Tangkapan Mentah (Source Shots)**: Jelajahi screenshot game yang belum diedit.
* **Tab Hasil Edit (Finished Edits)**: Tempat menyimpan dan mengelola karya SSRP yang sudah diekspor.

### 📚 2. Smart Albums & Log Deskripsi Narasi
* Buat album khusus cerita roleplay (misal: *Faction Heist*, *Daily Patrol*, *Business Meeting*).
* Tambahkan **judul album & deskripsi narasi cerita** secara interaktif.
* Tandai gambar untuk dimasukkan ke dalam album dan gunakan fitur **Filter Album** untuk menyaring tampilan galeri.

### ☁️ 3. Cloud Uploader ImgBB & Salin Tautan Langsung
* Unggah hasil foto langsung ke layanan cloud ImgBB di latar belakang (*async non-blocking*).
* Menampilkan URL langsung (*raw image URL*) tepat di bawah thumbnail gambar tanpa pop-up mengganggu.
* Tombol **Salin Tautan (📋)** satu-klik ke clipboard.
* Integrasi API Key ImgBB di menu Pengaturan.

### ⚡ 4. Unified Fast-Editor UI & Pengaturan Shortcuts
* **Unified UI Mode**: Menggabungkan seluruh panel alat (Foto, Chatlog, Teks, Potong, Efek) ke dalam satu halaman lipat serbaguna untuk pengeditan super cepat.
* **Layout Switcher**: Tombol cepat `🗂 Unified UI` / `🔲 Classic UI` di bagian header editor untuk berpindah tata letak secara instant.
* **Pintasan Kibor (Shortcuts)**: Tabel pengaturan hotkey kustom untuk tindakan cepat (`Open`, `Paste`, `Export`, `Undo`, `Redo`, `Cinematic`).

### 🎨 5. Theme Engine & Kepadatan Antarmuka
* 7 Pilihan Tema bawaan (Midnight, Paper, Dark, Light, Cyberpunk, Forest, Slate).
* Kustomisasi warna penekanan (*Accent Color Picker*).
* Pilihan Kepadatan UI (Nyaman vs Kompak).
* Bahasa Indonesia & English full localization support.

---

## 🔧 Teknologi & Performa

| Komponen | Teknologi |
|---|---|
| **Core Engine** | Pure Rust (`screenies-core`) — parser chatlog, pipeline composition, filter, crop, sticker, font rasterization |
| **Desktop Shell** | `egui` / `eframe` — GUI native tanpa WebView2 / Edge |
| **Network & HTTP** | `ureq` + `rustls` — Pure-Rust TLS HTTP client tanpa dependensi OpenSSL sistem |
| **Render Engine** | `image` + `ab_glyph` + `fontdb` — Lanczos resampling, blur/pixelate filter, text stroke generation |

---

## ⬇️ Unduh Aplikasi

Dapatkan rilis binary terbaru dari halaman **[Releases](../../releases/tag/v4.0.0)**:
* 🪟 **Windows**: `screenies-editor-v4.0.0-windows-x86_64.exe` (Portable Binary)
* 🐧 **Linux**: `screenies-editor-v4.0.0-linux-x86_64` (Raw Executable), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), `.AppImage`

## 🗺️ Peta Proyek (Project Map)

```
core/   screenies-core — engine utama murni Rust (seluruh logika inti):
        chatlog/     parser: timestamp → autocolor → systag → preset
        render/      compose → crop → filters (blur/pixelate) → sticker → layout → text
        chatlog_library.rs   indeks & pencarian log folder
        gallery.rs           pencacah file & filter galeri
        fonts.rs             shared fontdb (pemindaian font sistem sekali)
native/ screenies-native — antarmuka desktop native egui/eframe (Pure Rust):
        main.rs             entri aplikasi, navigasi layar & penyimpanan Settings
        editor.rs           status editor, mode Classic UI & Unified Fast-Editor UI
        gallery.rs          Galeri Dua Tab, Smart Albums & ImgBB Cloud Uploader
        theme.rs            Theme Engine (7 tema + accent picker + kepadatan UI)
        i18n.rs             kamus pelokalan dua bahasa (ID / EN)
        chatlog_browser.rs  pencari chatlog & pengambil teks instan
examples/presets/   preset warna komunal (.toml) untuk parser chatlog
docs/               panduan teknis, dokumentasi skema, changelog & migrasi
```

---

## 📖 Panduan Pengembangan

Lihat dokumentasi teknis di folder `docs/`:
* **[DEVELOPMENT.md](docs/DEVELOPMENT.md)** — Struktur direktori & panduan kontribusi.
* **[PRESETS.md](docs/PRESETS.md)** — Skema preset warna parser chatlog.
* **[CHANGELOG.md](docs/CHANGELOG.md)** — Catatan perubahan antar versi rilis.

---

<div align="center">Dibuat dengan ❤️ untuk komunitas roleplay SA-MP Indonesia</div>
