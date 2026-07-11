/**
 * export.ts — "Save Disk" and "Copy ke Clipboard" (Milestone 3c).
 *
 * Assembles a RenderJob from live state — source photo, crop, output size,
 * filters, font, and the SAME laid-out rows the preview just painted — and
 * hands it to Rust for a pixel-exact full-resolution render.
 */

import { effectiveStroke, state, onChange } from "./state";
import { buildRenderBlocks, outputDims, sourceCrop } from "./canvas";
import { copyPng, exportPng } from "./tauri-bridge";
import { scheduleSaveSettings } from "./settings";
import type { RenderJobPayload } from "./tauri-bridge";

let saveBtn: HTMLButtonElement;
let copyBtn: HTMLButtonElement;
let busy = false;

export function initExport(): void {
  saveBtn = mustGet<HTMLButtonElement>("btn-save");
  copyBtn = mustGet<HTMLButtonElement>("btn-copy");

  const template = mustGet<HTMLInputElement>("file-name-template");
  template.value = state.fileNameTemplate;
  template.addEventListener("input", () => {
    state.fileNameTemplate = template.value;
    scheduleSaveSettings();
  });

  saveBtn.addEventListener("click", () => void run("save"));
  copyBtn.addEventListener("click", () => void run("copy"));

  onChange(syncEnabled);
  syncEnabled();
}

function syncEnabled(): void {
  if (busy) return;
  const ready = state.image !== null && !state.cropEditing;
  saveBtn.disabled = !ready;
  copyBtn.disabled = !ready;
}

async function run(kind: "save" | "copy"): Promise<void> {
  const job = buildJob();
  if (!job || busy) return;

  const btn = kind === "save" ? saveBtn : copyBtn;
  const idle = kind === "save" ? "Save Disk (.png)" : "Copy ke Clipboard";
  busy = true;
  saveBtn.disabled = true;
  copyBtn.disabled = true;
  btn.textContent = kind === "save" ? "Merender…" : "Menyalin…";

  try {
    if (kind === "save") {
      await exportPng(job, expandFileName()); // false = dialog cancelled — fine
      btn.textContent = idle;
    } else {
      await copyPng(job);
      btn.textContent = "Disalin ✓";
      window.setTimeout(() => (btn.textContent = idle), 1400);
    }
  } catch (err) {
    console.error(`[screenies-editor] ${kind} failed:`, err);
    btn.textContent = "Gagal — lihat console";
    window.setTimeout(() => (btn.textContent = idle), 2200);
  } finally {
    busy = false;
    syncEnabled();
  }
}

function buildJob(): RenderJobPayload | null {
  const img = state.image;
  const crop = sourceCrop();
  const out = outputDims();
  if (!img || !crop || !out) return null;

  // The photo was loaded from a data URL — everything after the comma is
  // the original file's base64, exactly what Rust's decoder wants.
  const comma = img.src.indexOf(",");
  if (!img.src.startsWith("data:") || comma < 0) {
    console.error("[screenies-editor] photo is not a data URL — cannot export");
    return null;
  }

  return {
    imageBase64: img.src.slice(comma + 1),
    crop: { x: crop.x, y: crop.y, w: crop.w, h: crop.h },
    output: { w: out.w, h: out.h },
    filters: { ...state.filters },
    fontFamily: state.fontFamily,
    textSize: state.textSize,
    strokeWidth: effectiveStroke(),
    blocks: buildRenderBlocks(out.w)
      .filter((b) => b.rows.length > 0)
      .map((b) => ({ rows: b.rows })),
  };
}

/** Expand {placeholder}s in the template. Indonesian + English aliases. */
function expandFileName(): string {
  const now = new Date();
  const p2 = (n: number) => String(n).padStart(2, "0");
  const date = `${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}`;
  const time = `${p2(now.getHours())}-${p2(now.getMinutes())}-${p2(now.getSeconds())}`;
  const out = outputDims();
  const res = out ? `${out.w}x${out.h}` : "";
  const foto = state.imageName.replace(/\.[^.]+$/, "");

  const values: Record<string, string> = {
    tanggal: date, date,
    jam: time, time,
    res,
    foto, name: foto,
  };

  const raw = state.fileNameTemplate.trim() || "screenie-{tanggal}-{jam}";
  const expanded = raw.replace(/\{(\w+)\}/g, (whole, key: string) => values[key] ?? whole);
  // Light client-side sanitize; Rust does the authoritative pass.
  return expanded.replace(/[<>:"/\\|?*]/g, "-");
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
