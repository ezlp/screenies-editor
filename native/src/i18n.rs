// i18n.rs — minimal ID/EN localization. Strings are keyed by their Indonesian
// text, so the ID path is identity and only the EN translations live here.
// Unknown keys fall through to the Indonesian text.

#[derive(Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Lang {
    #[default]
    Id,
    En,
}



/// Translate `id` (an Indonesian source string) for `lang`.
pub fn t(lang: Lang, id: &'static str) -> &'static str {
    if lang == Lang::Id {
        return id;
    }
    match id {
        // menu / nav
        "Screenshot Roleplay toolkit — komunitas SA-MP" => "Screenshot Roleplay toolkit — SA-MP community",
        "Menu Utama" => "Main Menu",
        "Hasil Edit Terbaru" => "Recent Edits",
        "Crop · chatlog · filter · export" => "Crop · chatlog · filters · export",
        "Buka folder chatlog · cari di aplikasi" => "Open a chatlog folder · search in-app",
        "Jelajahi foto SSRP hasil edit" => "Browse your edited SSRP photos",
        "Bahasa · tema · ukuran ruang edit" => "Language · theme · editing space size",
        // settings screen
        "Pengaturan" => "Settings",
        "Ukuran ruang edit" => "Editing space size",
        "Perbesar/perkecil seluruh tampilan aplikasi." => "Scale the whole app UI up or down.",
        "☀  Mode terang" => "☀  Light mode",
        "🌙  Mode gelap" => "🌙  Dark mode",
        "Ganti tema" => "Toggle theme",
        // editor — photo / crop
        "📂  Muat Foto" => "📂  Load Photo",
        "Belum ada foto." => "No photo yet.",
        "Crop / Resolusi" => "Crop / Resolution",
        "Bebas" => "Free",
        "✏ Edit crop" => "✏ Edit crop",
        "✓ Selesai crop" => "✓ Finish crop",
        "Seret kotak untuk framing · sisi/pojok untuk resize · klik “✓ Selesai crop”" => "Drag the box to frame · any side/corner to resize · click “✓ Finish crop”",
        "Muat foto dulu." => "Load a photo first.",
        "Muat foto untuk mulai mengedit." => "Load a photo to start editing.",
        // editor — cinematic mode
        "Mode" => "Mode",
        "🎬 Sinema" => "🎬 Cinematic",
        "Tinggi bar %" => "Bar height %",
        "Warna bar" => "Bar color",
        "Posisi bar" => "Bar position",
        "Keduanya" => "Both",
        "Atas" => "Top",
        "Bawah" => "Bottom",
        "Bar sinema digambar di dalam foto (pilih posisi)." => "Cinematic bars drawn inside the photo (pick a side).",
        // editor — crop fit / crop toggle
        "Potong" => "Crop",
        "Muat penuh" => "Fit",
        "Muat: simpan seluruh gambar + bar (ukuran tetap)." => "Fit: keep the whole image + bars (fixed sizes).",
        // editor — chatlog
        "Chatlog:" => "Chatlog:",
        "Tambah chatlog" => "Add chatlog",
        "Chatlog dari folder" => "Chatlog from folder",
        "Tambah foto" => "Add photo",
        "Tutup foto ini" => "Close this photo",
        "🗑 Hapus chatlog ini" => "🗑 Delete this chatlog",
        // editor — text
        "Teks" => "Text",
        "Font" => "Font",
        "Ukuran" => "Size",
        "Jarak baris %" => "Line spacing %",
        "Outline otomatis" => "Auto outline",
        "Outline px" => "Outline px",
        // palette
        "Palet warna" => "Color palette",
        "Pilih teks di chatlog, klik warna untuk membungkus {RRGGBB}." => "Select chatlog text, click a color to wrap it in {RRGGBB}.",
        "Terapkan kustom" => "Apply custom",
        "✓ ada teks terpilih" => "✓ text selected",
        "(pilih teks di chatlog dulu)" => "(select chatlog text first)",
        // palette swatch names
        "Putih" => "White",
        "Merah" => "Red",
        "Hijau" => "Green",
        "Biru" => "Blue",
        "Kuning" => "Yellow",
        "Oranye" => "Orange",
        "Ungu" => "Purple",
        "Toska" => "Teal",
        "Abu" => "Gray",
        // position / background
        "Posisi" => "Position",
        "Kiri Atas" => "Top Left",
        "Kiri Bawah" => "Bottom Left",
        "Tidak ada" => "None",
        "Blok" => "Block",
        // filters / effects
        "Efek (2.0)" => "Effects (2.0)",
        "Blur (px)" => "Blur (px)",
        "Pixelate (blok px)" => "Pixelate (block px)",
        // censor
        "Sensor area (blur/pixelate lokal)" => "Censor area (local blur/pixelate)",
        "Klik kotak di preview untuk pilih · seret badan untuk geser · seret pojok untuk resize." => "Click a box in the preview to select · drag body to move · drag corner to resize.",
        "Blur radius (px)" => "Blur radius (px)",
        "Blok (px)" => "Block (px)",
        "🗑 Hapus area" => "🗑 Delete area",
        // stickers
        "Stiker" => "Sticker",
        "+ Tambah stiker" => "+ Add sticker",
        "Klik stiker di preview untuk pilih · seret untuk geser · pojok untuk resize." => "Click a sticker in the preview to select · drag to move · corner to resize.",
        "Lebar (px)" => "Width (px)",
        "🗑 Hapus stiker" => "🗑 Delete sticker",
        // export
        "💾  Export PNG" => "💾  Export PNG",
        "Muat foto dulu sebelum export." => "Load a photo before exporting.",
        // chatlog parser
        "📂  Buka folder chatlog" => "📂  Open chatlog folder",
        "Pilih folder berisi file .txt / .log" => "Pick a folder with .txt / .log files",
        "cari nama / kata di semua chatlog…" => "search a name / word across all chatlogs…",
        "Ketik untuk mencari." => "Type to search.",
        "klik untuk salin" => "click to copy",
        // gallery
        "📂  Buka folder" => "📂  Open folder",
        "Pilih folder berisi foto hasil edit." => "Pick a folder with your edited photos.",
        "Pilih gambar dari daftar." => "Pick an image from the list.",
        "✏  Buka di editor" => "✏  Open in editor",
        // settings screen — appearance (theme engine, Milestone A)
        "Penampilan" => "Appearance",
        "Tema" => "Theme",
        "Accent color" => "Accent color",
        "Kepadatan UI" => "UI Density",
        "Nyaman" => "Cozy",
        "Kompak" => "Compact",
        "Default edit" => "Editing Defaults",
        "Bahasa" => "Language",
        "Tentang" => "About",
        "GitHub" => "GitHub",
        // Clean translations (emoji-free keys)
        "Muat Foto" => "Load Photo",
        "Edit crop" => "Edit crop",
        "Selesai crop" => "Finish crop",
        "Sinema" => "Cinematic",
        "Hapus chatlog ini" => "Delete this chatlog",
        "Hapus area" => "Delete area",
        "Hapus stiker" => "Delete sticker",
        "Buka folder" => "Open folder",
        "Buka di editor" => "Open in editor",
        "Transparansi" => "Transparency",
        "Pilih skema warna dasar untuk seluruh antarmuka aplikasi." => "Choose the base color scheme for the entire application interface.",
        "Sesuaikan warna penekanan untuk tombol, seleksi, dan tautan." => "Customize the highlight color for buttons, selections, and links.",
        "Sesuaikan jarak antar elemen UI (Nyaman atau Kompak)." => "Adjust the spacing between UI elements (Cozy or Compact).",
        "Skala perbesaran antarmuka editor dan teks." => "Scale the zoom level of the editor interface and text.",
        "Ganti bahasa pelokalan aplikasi." => "Change the localization language of the application.",
        "Source Shots" => "Tangkapan Mentah",
        "Finished Edits" => "Hasil Edit",
        "Pilih folder berisi screenshot mentah." => "Choose the folder containing raw screenshots.",
        "Buat Album Baru" => "Create New Album",
        "Album Baru" => "New Album",
        "Hapus Album" => "Delete Album",
        "Judul Album" => "Album Title",
        "Deskripsi Cerita" => "Storyline Description",
        "Pilih atau buat album untuk mulai menyusun cerita." => "Select or create an album to start writing your story.",
        "Filter berdasarkan album ini" => "Filter by this album",
        "Di dalam album" => "In Album",
        "Salin tautan" => "Copy link",
        "Konfigurasi API Key ImgBB di Pengaturan untuk mengunggah" => "Configure ImgBB API Key in Settings to upload",
        _ => id,
    }
}
