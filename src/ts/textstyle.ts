/**
 * textstyle.ts — text styling controls.
 *
 * ACTIVE: size slider + auto-size from image resolution, and the FONT
 * PICKER — populated from the real installed system fonts via Rust
 * (`list_fonts`), with a safe fallback list for browser dev mode.
 *
 * Readability rule: auto-size never goes below AUTO_MIN, so low-res
 * screenshots keep legible text (the slider can still go lower manually).
 *
 * MILESTONE 4 leftovers: stroke width & line-spacing controls.
 */

import { listFonts } from "./tauri-bridge";
import { state, notify } from "./state";

const SIZE_MAX = 60;
/** Auto-size floor — below this, chat text stops being readable. */
const AUTO_MIN = 16;
/** Auto rule: image width / this = comfortable SSRP text size (800 → ~27px). */
const AUTO_DIVISOR = 30;

/** Shown when Rust font enumeration is unavailable (browser dev mode). */
const FALLBACK_FONTS = [
  "Arial",
  "Verdana",
  "Tahoma",
  "Trebuchet MS",
  "Segoe UI",
  "Georgia",
  "Times New Roman",
  "Courier New",
  "Impact",
];

let slider: HTMLInputElement | null = null;
let valueLabel: HTMLElement | null = null;
let fontSelect: HTMLSelectElement | null = null;

export function initTextStyle(): void {
  slider = document.getElementById("text-size") as HTMLInputElement | null;
  valueLabel = document.getElementById("text-size-val");
  fontSelect = document.getElementById("font-select") as HTMLSelectElement | null;
  if (!slider || !valueLabel || !fontSelect) {
    throw new Error("Missing #text-size / #text-size-val / #font-select");
  }

  slider.addEventListener("input", () => {
    if (!slider) return;
    state.textSize = Number(slider.value);
    syncLabel();
    notify();
  });

  fontSelect.addEventListener("change", () => {
    if (!fontSelect) return;
    state.fontFamily = fontSelect.value;
    notify();
  });

  syncControls();
  void populateFonts();
}

/** Called on image load: pick a size that fits this resolution. */
export function autoTextSize(imageWidth: number): void {
  const size = Math.min(SIZE_MAX, Math.max(AUTO_MIN, Math.round(imageWidth / AUTO_DIVISOR)));
  state.textSize = size;
  syncControls();
  notify();
}

async function populateFonts(): Promise<void> {
  if (!fontSelect) return;

  let families: string[] = [];
  try {
    families = await listFonts();
  } catch (err) {
    console.error("[screenies-editor] list_fonts failed:", err);
  }
  if (families.length === 0) families = FALLBACK_FONTS;

  fontSelect.innerHTML = "";
  for (const family of families) {
    const option = document.createElement("option");
    option.value = family;
    option.textContent = family;
    fontSelect.appendChild(option);
  }

  // Prefer the current choice, else Arial, else the first family.
  const preferred =
    families.find((f) => f === state.fontFamily) ??
    families.find((f) => f.toLowerCase() === "arial") ??
    families[0];
  state.fontFamily = preferred;
  fontSelect.value = preferred;
  notify();
}

function syncControls(): void {
  if (slider) slider.value = String(state.textSize);
  syncLabel();
}

function syncLabel(): void {
  if (valueLabel) valueLabel.textContent = `${state.textSize}px`;
}
