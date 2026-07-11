/**
 * crop.ts — the resolution changer (Milestone 3a).
 *
 * Presets set either a FIXED OUTPUT (800×600, 4K, custom W×H — export at
 * exactly that size) or a RATIO ONLY (16:9, 4:3, 21:9 — export at the
 * crop's own pixels), or FREE (no lock at all). Picking one drops a
 * centered crop box on the photo and opens edit mode; "Selesai" returns
 * to the result preview.
 */

import { state } from "./state";
import { centeredCrop, clampBlocksToOutput, fitImage, outputDims } from "./canvas";
import { autoTextSize } from "./textstyle";
import { commit } from "./history";
import { t } from "./i18n";

interface ResPreset {
  id: string;
  label: string;
  output: { w: number; h: number } | null; // fixed output size
  ratio: number | null;                    // null with null output = free
}

const RES_PRESETS: ResPreset[] = [
  { id: "800x600", label: "800×600", output: { w: 800, h: 600 }, ratio: 800 / 600 },
  { id: "r43", label: "4:3", output: null, ratio: 4 / 3 },
  { id: "r169", label: "16:9", output: null, ratio: 16 / 9 },
  { id: "r219", label: "21:9", output: null, ratio: 21 / 9 },
  { id: "4k", label: "4K", output: { w: 3840, h: 2160 }, ratio: 16 / 9 },
  { id: "free", label: "Bebas", output: null, ratio: null },
];

const CUSTOM_MIN = 50;
const CUSTOM_MAX = 8192;

let grid: HTMLElement;
let editBtn: HTMLButtonElement;
let resetBtn: HTMLButtonElement;
let customW: HTMLInputElement;
let customH: HTMLInputElement;

export function initCrop(): void {
  grid = mustGet<HTMLElement>("res-grid");
  editBtn = mustGet<HTMLButtonElement>("btn-crop-edit");
  resetBtn = mustGet<HTMLButtonElement>("btn-crop-reset");
  customW = mustGet<HTMLInputElement>("custom-w");
  customH = mustGet<HTMLInputElement>("custom-h");

  for (const preset of RES_PRESETS) {
    const btn = document.createElement("button");
    btn.className = "btn btn-small res-btn";
    btn.textContent = preset.label;
    btn.dataset.res = preset.id;
    btn.addEventListener("click", () => applyPreset(preset, btn));
    grid.appendChild(btn);
  }

  mustGet<HTMLButtonElement>("btn-custom-apply").addEventListener("click", applyCustom);
  for (const input of [customW, customH]) {
    input.addEventListener("keydown", (ev) => {
      if (ev.key === "Enter") applyCustom(); // small fix: Enter = Set
    });
  }

  editBtn.addEventListener("click", () => setEditing(!state.cropEditing));
  window.addEventListener("i18n-changed", syncEditButton);
  resetBtn.addEventListener("click", resetToFull);

  syncEditButton();
}

/** Called by chatlog.ts when a new photo loads: back to full frame. */
export function onImageLoaded(): void {
  state.crop = null;
  state.cropRatio = null;
  state.outputSize = null;
  state.cropEditing = false;
  clearActive();
  syncEditButton();
}

function applyPreset(preset: ResPreset, btn: HTMLButtonElement): void {
  state.cropRatio = preset.ratio;
  state.outputSize = preset.output ? { ...preset.output } : null;

  clearActive();
  btn.classList.add("active");

  if (preset.id === "free") {
    // Free: keep the current box if any, just unlock the ratio.
    afterCropChange(false);
    return;
  }

  state.crop = centeredCrop(preset.ratio);
  afterCropChange(true);
}

function applyCustom(): void {
  const w = Math.round(Number(customW.value));
  const h = Math.round(Number(customH.value));
  if (
    !Number.isFinite(w) || !Number.isFinite(h) ||
    w < CUSTOM_MIN || h < CUSTOM_MIN || w > CUSTOM_MAX || h > CUSTOM_MAX
  ) {
    customW.classList.add("input-error");
    customH.classList.add("input-error");
    window.setTimeout(() => {
      customW.classList.remove("input-error");
      customH.classList.remove("input-error");
    }, 900);
    return;
  }

  state.outputSize = { w, h };
  state.cropRatio = w / h;
  state.crop = centeredCrop(state.cropRatio);
  clearActive();
  afterCropChange(true);
}

function resetToFull(): void {
  onImageLoaded();
  afterCropChange(false);
}

/** Shared tail: text size + block clamps follow the new output, re-fit. */
function afterCropChange(enterEdit: boolean): void {
  commit();
  if (enterEdit && state.image) state.cropEditing = true;
  syncEditButton();

  const out = outputDims();
  if (out) {
    autoTextSize(out.w);
    clampBlocksToOutput();
  }
  fitImage(); // also notifies
}

function setEditing(on: boolean): void {
  if (!state.image) return;
  if (on && !state.crop) {
    state.crop = centeredCrop(state.cropRatio);
  }
  state.cropEditing = on;
  syncEditButton();
  if (!on) commit(); // leaving edit = one undo step
  fitImage();
}

function syncEditButton(): void {
  editBtn.textContent = state.cropEditing ? t("cropDone") : t("cropEdit");
  editBtn.classList.toggle("btn-primary", state.cropEditing);
}

function clearActive(): void {
  grid
    .querySelectorAll<HTMLButtonElement>(".res-btn.active")
    .forEach((b) => b.classList.remove("active"));
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
