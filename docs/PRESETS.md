# Preset Format — ScreeniesEditor

> Halaman ini siap dicopy ke GitHub Wiki.

ScreeniesEditor tidak menghardcode format satu server. Semua aturan parsing
chatlog didefinisikan oleh sebuah **preset** — dan kamu bisa mengaturnya
sendiri lewat dropdown **Preset format → Kustom…** di panel kiri.

## Preset bawaan

| Preset | Untuk siapa | Catatan |
|---|---|---|
| **JGRP (Jogjagamers)** | Pemain JGRP | Semua aturan aktif, termasuk deteksi `/do` yang berakhiran `((Nama))` |
| **SA-MP Umum** | Server SA-MP RP lain | Sama seperti JGRP tapi tanpa deteksi `/do ((Nama))` (khas JGRP) |
| **Polos (tanpa auto-warna)** | Format yang belum didukung | Hanya hapus timestamp + kode warna `{RRGGBB}`; tanpa pewarnaan otomatis |

## Aturan yang bisa diatur (mode Kustom)

| Aturan | Efek |
|---|---|
| Hapus timestamp | Buang `[HH:MM:SS]` di awal baris |
| Warna otomatis `*` | Baris diawali `*` → warna /me (default ungu `#C2A2DA`) |
| Warna otomatis `(( ))` | Baris diawali `((` → warna OOC (default abu `#9C9C9C`) |
| Deteksi `/do ((Nama))` | Baris yang **berakhir** `((Nama))` → warna /me |
| Tag sistem bold | `SERVER:`, `VEHICLE:`, `AdmCmd:`, dan `Apapun:` lainnya → tag jadi **bold** + bisa disembunyikan lewat "Hanya RP" |
| Warna /me & OOC | Ganti warnanya sesuka hati |
| Channel radio | Daftar channel `[...]:` yang dianggap ucapan, pisahkan koma. Contoh: `phone, walkie, dep, gov` |

Aturan yang **selalu aktif** (universal SA-MP): deteksi `says:` /
`shouts:` termasuk varian `says [low]:`, `says [apapun]:`.

## Skema JSON

Preset adalah JSON biasa. Field yang tidak ditulis otomatis memakai nilai
default, jadi preset versi lama tetap jalan di versi app yang lebih baru.

```json
{
  "name": "Server Saya",
  "stripTimestamps": true,
  "hexCodes": true,
  "mePrefix": true,
  "oocWrap": true,
  "doSuffix": false,
  "systemTags": true,
  "radioChannels": ["phone", "walkie", "dep"],
  "colorMe": "#C2A2DA",
  "colorOoc": "#9C9C9C",
  "colorDefault": "#FFFFFF"
}
```

> **Rencana M4:** simpan / muat preset Kustom sebagai file `.json`, supaya
> preset server kamu bisa dibagikan ke teman satu komunitas lewat wiki ini.

## Contoh untuk server lain

Server yang `/do`-nya diawali `*` (bukan berakhiran `((Nama))`):
matikan **Deteksi /do ((Nama))** — baris `*` sudah tertangkap aturan `*`.

Server dengan radio departemen: isi channel radio dengan
`phone, walkie, dep, r, gov` sesuai format server kamu.

Server non-RP / log custom: pakai **Polos**, lalu warnai manual dengan kode
`{RRGGBB}` di teks (palet warna klik-klik menyusul di M4).
