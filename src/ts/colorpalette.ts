/**
 * colorpalette.ts — select text → pick a color (Milestone 4b).
 *
 * ssrp-editor's palette, native: highlight text in any chatlog box, then
 * click Terapkan (or a quick swatch) and the selection is wrapped in
 * {RRGGBB}…{default} codes — which the M2 engine already renders. With no
 * selection, the code is inserted at the cursor and colors from there on.
 */

import { state } from "./state";
import { getActiveChatArea } from "./chatlog";

const SWATCHES = ["#FFFFFF", "#C2A2DA", "#9C9C9C", "#FF0000", "#FFFF00", "#00FFFF", "#00FF00"];

export function initColorPalette(): void {
  const picker = mustGet<HTMLInputElement>("palette-color");
  const applyBtn = mustGet<HTMLButtonElement>("palette-apply");
  const swatchRow = mustGet<HTMLElement>("palette-swatches");

  applyBtn.addEventListener("click", () => apply(picker.value));

  for (const hex of SWATCHES) {
    const b = document.createElement("button");
    b.className = "swatch";
    b.style.background = hex;
    b.title = hex;
    b.addEventListener("click", () => {
      picker.value = hex;
      apply(hex);
    });
    swatchRow.appendChild(b);
  }
}

function apply(hexWithHash: string): void {
  const area = getActiveChatArea();
  if (!area) {
    console.warn("[screenies-editor] palet: klik dulu kotak chatlog yang mau diwarnai");
    return;
  }

  const hex = hexWithHash.replace("#", "").toUpperCase();
  const restore = state.preset.colorDefault.replace("#", "").toUpperCase();
  const start = area.selectionStart ?? 0;
  const end = area.selectionEnd ?? start;
  const value = area.value;

  let inserted: string;
  let cursor: number;
  if (start === end) {
    inserted = value.slice(0, start) + `{${hex}}` + value.slice(start);
    cursor = start + hex.length + 2;
  } else {
    inserted =
      value.slice(0, start) +
      `{${hex}}` + value.slice(start, end) + `{${restore}}` +
      value.slice(end);
    cursor = end + hex.length + 2 + restore.length + 2;
  }

  area.value = inserted;
  area.focus();
  area.setSelectionRange(cursor, cursor);
  // Reuse the existing debounce → Rust reparse → preview recolors.
  area.dispatchEvent(new Event("input", { bubbles: true }));
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
