# Membuat Preset Sendiri (.toml) — ScreeniesEditor

> Halaman ini siap dicopy ke GitHub Wiki. Untuk komunitas: kalau kamu sudah
> membuat preset untuk server kamu, bagikan file `.toml`-nya di wiki ini
> supaya pemain lain tinggal **Impor**.

Preset menentukan bagaimana ScreeniesEditor membaca dan mewarnai chatlog
server kamu — semua aturannya adalah data, bukan kode. Satu preset = satu
file `.toml` yang bisa dibagikan.

## Cara tercepat (tanpa nulis file)

1. Buka app → panel kiri → **Preset format** → pilih **Kustom…**
2. Atur toggle & warna sampai chatlog kamu terlihat benar di preview
3. Klik **Ekspor .toml** → simpan, misalnya `preset-serverku.toml`
4. Bagikan file itu — teman kamu tinggal klik **Impor .toml**

Selesai. Sisanya di halaman ini untuk yang mau menulis / mengedit file
langsung.

## Anatomi file preset

Buat file teks berakhiran `.toml`, isi seperti ini:

```toml
# Preset untuk Server Saya — dibuat oleh Isut
name = "Server Saya"

# ── Pembersihan ──
stripTimestamps = true      # buang [HH:MM:SS] di awal baris
hexCodes = true             # baca kode warna {RRGGBB} / {rrggbb}

# ── Aturan pewarnaan otomatis ──
mePrefix = true             # baris diawali *  → warna /me
oocWrap = true              # baris diawali (( → warna OOC
doSuffix = false            # baris berakhiran ((Nama)) → warna /me (khas JGRP)
systemTags = true           # SERVER:, VEHICLE:, Apapun:  → tag jadi bold

# ── Channel radio yang dianggap ucapan ──
radioChannels = ["phone", "walkie"]

# ── Warna (hex, pakai tanda #) ──
colorMe = "#C2A2DA"
colorOoc = "#9C9C9C"
colorDefault = "#FFFFFF"
```

Baris berawalan `#` adalah komentar — bebas dipakai untuk catatan.

## Referensi field

| Field | Tipe | Default | Artinya |
|---|---|---|---|
| `name` | teks | `"JGRP (Jogjagamers)"` | Nama yang tampil |
| `stripTimestamps` | true/false | `true` | Hapus `[HH:MM:SS]` |
| `hexCodes` | true/false | `true` | Parse `{RRGGBB}` (huruf besar/kecil sama saja) |
| `mePrefix` | true/false | `true` | `*` di awal → warna /me |
| `oocWrap` | true/false | `true` | `((` di awal → warna OOC |
| `doSuffix` | true/false | `true` | Berakhiran `((Nama))` → warna /me |
| `systemTags` | true/false | `true` | `Apapun:` di awal → tag **bold** |
| `radioChannels` | daftar teks | `["phone", "walkie"]` | `[channel]:` dianggap ucapan |
| `colorMe` | hex | `"#C2A2DA"` | Warna /me dan /do |
| `colorOoc` | hex | `"#9C9C9C"` | Warna OOC |
| `colorDefault` | hex | `"#FFFFFF"` | Warna dasar semua teks lain |

**Field yang tidak ditulis otomatis memakai default** — jadi preset lama
tetap jalan di versi app yang lebih baru, dan file minimal ini pun sah:

```toml
name = "Cuma ganti warna me"
colorMe = "#FF9DE2"
```

Yang **selalu aktif** (tidak perlu diatur): deteksi `says:` / `shouts:`
termasuk semua varian kurung seperti `says [low]:`, `says [radio]:`.

## Resep per jenis server

**Server yang `/do`-nya diawali `*` (bukan berakhiran `((Nama))`)**
→ `doSuffix = false`. Baris `*` sudah tertangkap `mePrefix`.

**Server dengan radio departemen** → tambah channelnya:
`radioChannels = ["phone", "walkie", "dep", "gov", "r"]`

**Server dengan warna /me berbeda** → ganti `colorMe`, contoh MTA-style
pink: `colorMe = "#FF66CC"`.

**Format yang belum cocok sama sekali** → matikan semua aturan
(`mePrefix/oocWrap/doSuffix/systemTags = false`) lalu warnai manual dengan
kode `{RRGGBB}` langsung di teks chatlog.

## Kalau impor gagal

App menolak file yang TOML-nya rusak dan menunjukkan pesan errornya di
console (Ctrl+Shift+I). Penyebab paling umum: lupa tanda kutip pada teks
(`name = Server Saya` ❌ → `name = "Server Saya"` ✅) atau koma di dalam
daftar (`["phone" "walkie"]` ❌ → `["phone", "walkie"]` ✅).

## Berbagi di wiki

Konvensi yang disarankan untuk halaman wiki komunitas:

1. Satu halaman per server: **Preset: NamaServer**
2. Tempel isi `.toml` dalam code block + screenshot hasilnya
3. Tulis tanggal & versi app saat preset diuji
