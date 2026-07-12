/**
 * i18n.ts — Indonesian / English UI (v1.1).
 *
 * Selector-driven: DICT maps CSS selectors → {id,en}, so index.html needs
 * no markup changes and this file is the single place a translation
 * lives. Dynamic strings (dropdown options, button states) use t(key);
 * modules that build DOM listen for the "i18n-changed" event to relabel.
 * The choice persists via settings.json.
 */

export type Lang = "id" | "en";
let lang: Lang = "id";

/** Static UI: selector → translations. Applied on init and every switch. */
const STATIC: Array<[string, { id: string; en: string }]> = [
  ['label[for="preset-select"]', { id: "Preset format", en: "Format preset" }],
  ["#import-preset", { id: "Impor .toml", en: "Import .toml" }],
  ["#export-preset", { id: "Ekspor .toml", en: "Export .toml" }],
  ['label[for="text-size"]', { id: "Ukuran teks", en: "Text size" }],
  ['label[for="stroke-size"]', { id: "Outline", en: "Outline" }],
  ['label[for="line-gap"]', { id: "Jarak baris", en: "Line spacing" }],
  ['label[for="bg-offset"]', { id: "Geser BG", en: "BG offset" }],
  ['label[for="palette-color"]', { id: "Palet", en: "Palette" }],
  ["#palette-apply", { id: "Terapkan ke seleksi", en: "Apply to selection" }],
  ["#btn-add-sticker", { id: "+ Import Stiker (PNG)", en: "+ Import Sticker (PNG)" }],
  ['label[for="file-name-template"]', { id: "Nama file", en: "File name" }],
  ["#btn-copy", { id: "Copy ke Clipboard", en: "Copy to Clipboard" }],
  ["#btn-crop-reset", { id: "Full", en: "Full" }],
  ["#btn-custom-apply", { id: "Set", en: "Set" }],
];

/** Dynamic strings requested by modules via t(). */
const T: Record<string, { id: string; en: string }> = {
  anchorFree: { id: "Bebas (seret)", en: "Free (drag)" },
  anchorTopLeft: { id: "Kiri Atas", en: "Top Left" },
  anchorBottomLeft: { id: "Kiri Bawah", en: "Bottom Left" },
  bgNone: { id: "BG: tanpa", en: "BG: none" },
  bgBlock: { id: "BG: blok", en: "BG: block" },
  bgMask: { id: "BG: mask", en: "BG: mask" },
  addChatlog: { id: "+ Tambah Chatlog", en: "+ Add Chatlog" },
  cropEdit: { id: "Atur Area Crop", en: "Adjust Crop Area" },
  cropDone: { id: "✓ Selesai", en: "✓ Done" },
  saveIdle: { id: "Save Disk (.png)", en: "Save to Disk (.png)" },
  saving: { id: "Merender…", en: "Rendering…" },
  copyIdle: { id: "Copy ke Clipboard", en: "Copy to Clipboard" },
  copying: { id: "Menyalin…", en: "Copying…" },
  copied: { id: "Disalin ✓", en: "Copied ✓" },
  failed: { id: "Gagal — lihat console", en: "Failed — see console" },
  lines: { id: "baris", en: "lines" },
};

export function t(key: keyof typeof T): string {
  return T[key][lang];
}

export function getLang(): Lang {
  return lang;
}

export function setLang(next: Lang): void {
  lang = next;
  applyStatic();
  const btn = document.getElementById("lang-toggle");
  if (btn) btn.textContent = lang === "id" ? "EN" : "ID"; // shows the OTHER one
  window.dispatchEvent(new CustomEvent("i18n-changed"));
}

export function initI18n(): void {
  applyStatic();
  const btn = document.getElementById("lang-toggle");
  if (btn) btn.textContent = lang === "id" ? "EN" : "ID";
}

function applyStatic(): void {
  for (const [selector, tr] of STATIC) {
    const el = document.querySelector<HTMLElement>(selector);
    if (el) el.textContent = tr[lang];
  }
}
