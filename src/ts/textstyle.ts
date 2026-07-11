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
import { effectiveStroke, state, notify } from "./state";
import { scheduleSaveSettings } from "./settings";
import { commit } from "./history";

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
let strokeSlider: HTMLInputElement | null = null;
let strokeVal: HTMLElement | null = null;
let gapSlider: HTMLInputElement | null = null;
let gapVal: HTMLElement | null = null;

/** Undo/redo: reflect restored sizes on the sliders. */
export function syncTextStyleUI(): void {
  syncControls();
  const bgOffset = document.getElementById("bg-offset") as HTMLInputElement | null;
  const bgOffsetVal = document.getElementById("bg-offset-val");
  if (bgOffset) bgOffset.value = String(state.bgOffset);
  if (bgOffsetVal) bgOffsetVal.textContent = `${state.bgOffset}px`;
}

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
    scheduleSaveSettings();
    notify();
  });

  strokeSlider = document.getElementById("stroke-size") as HTMLInputElement | null;
  strokeVal = document.getElementById("stroke-val");
  gapSlider = document.getElementById("line-gap") as HTMLInputElement | null;
  gapVal = document.getElementById("line-gap-val");
  const strokeAuto = document.getElementById("stroke-auto");
  const gapReset = document.getElementById("line-gap-reset");
  if (!strokeSlider || !strokeVal || !gapSlider || !gapVal || !strokeAuto || !gapReset) {
    throw new Error("Missing stroke/line-gap controls");
  }

  strokeSlider.addEventListener("input", () => {
    state.strokeWidth = Number(strokeSlider!.value);
    syncStroke();
    notify();
  });
  strokeSlider.addEventListener("change", commit);
  strokeAuto.addEventListener("click", () => {
    state.strokeWidth = null; // back to auto (scales with text size)
    syncStroke();
    commit();
    notify();
  });

  gapSlider.addEventListener("input", () => {
    state.lineGap = Number(gapSlider!.value);
    syncGap();
    notify();
  });
  gapSlider.addEventListener("change", commit);
  gapReset.addEventListener("click", () => {
    state.lineGap = 122;
    syncGap();
    commit();
    notify();
  });

  const bgOffset = document.getElementById("bg-offset") as HTMLInputElement | null;
  const bgOffsetVal = document.getElementById("bg-offset-val");
  if (!bgOffset || !bgOffsetVal) throw new Error("Missing BG controls");

  bgOffset.addEventListener("input", () => {
    state.bgOffset = Number(bgOffset.value);
    bgOffsetVal.textContent = `${state.bgOffset}px`;
    notify();
  });
  bgOffset.addEventListener("change", commit);
  syncControls();
  void populateFonts();
}

function syncStroke(): void {
  if (!strokeSlider || !strokeVal) return;
  strokeSlider.value = String(effectiveStroke());
  strokeVal.textContent = state.strokeWidth === null ? `auto (${effectiveStroke()}px)` : `${state.strokeWidth}px`;
}

function syncGap(): void {
  if (!gapSlider || !gapVal) return;
  gapSlider.value = String(state.lineGap);
  gapVal.textContent = `${state.lineGap}%`;
}

/** Called on image load: pick a size that fits this resolution. */
export function autoTextSize(imageWidth: number): void {
  const size = Math.min(SIZE_MAX, Math.max(AUTO_MIN, Math.round(imageWidth / AUTO_DIVISOR)));
  state.textSize = size;
  syncControls();
  syncStroke(); // auto stroke follows text size
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
  syncStroke();
  syncGap();
}

function syncLabel(): void {
  if (valueLabel) valueLabel.textContent = `${state.textSize}px`;
}
