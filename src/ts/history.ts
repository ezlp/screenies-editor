/**
 * history.ts — undo / redo (v1.1).
 *
 * Snapshot-based: modules call commit() at DISCRETE moments (drag end,
 * slider release, text parsed, add/remove) — never per-pixel — so the
 * stack stays small and each undo step feels intentional. The photo
 * itself is not part of history; everything on top of it is.
 *
 * Shortcuts: Ctrl+Z undo · Ctrl+Y / Ctrl+Shift+Z redo (ignored while a
 * text field is focused, so the browser's native textarea undo wins).
 */

import { state } from "./state";
import type { Anchor, BgMode, CropRect, Filters } from "./state";

export interface Snapshot {
  blocks: Array<{ rawText: string; anchor: Anchor; bgMode: BgMode; x: number; y: number }>;
  stickers: Array<{ name: string; src: string; base64: string; x: number; y: number; scale: number }>;
  crop: CropRect | null;
  cropRatio: number | null;
  outputSize: { w: number; h: number } | null;
  filters: Filters;
  textSize: number;
  strokeWidth: number | null;
  lineGap: number;
  bgOffset: number;
}

interface Hooks {
  /** Rebuild UI + state from a snapshot (async: images decode, text reparses). */
  restore: (snap: Snapshot) => Promise<void>;
}

const MAX_STEPS = 50;
let past: Snapshot[] = [];
let future: Snapshot[] = [];
let hooks: Hooks | null = null;
let restoring = false;
let undoBtn: HTMLButtonElement;
let redoBtn: HTMLButtonElement;

export function initHistory(h: Hooks): void {
  hooks = h;
  undoBtn = mustGet<HTMLButtonElement>("btn-undo");
  redoBtn = mustGet<HTMLButtonElement>("btn-redo");
  undoBtn.addEventListener("click", () => void undo());
  redoBtn.addEventListener("click", () => void redo());

  window.addEventListener("keydown", (ev) => {
    if (!(ev.ctrlKey || ev.metaKey)) return;
    const el = document.activeElement;
    const typing = el instanceof HTMLTextAreaElement ||
      (el instanceof HTMLInputElement && (el.type === "text" || el.type === "number"));
    if (typing) return; // let the field's native undo handle it

    const k = ev.key.toLowerCase();
    if (k === "z" && !ev.shiftKey) { ev.preventDefault(); void undo(); }
    else if (k === "y" || (k === "z" && ev.shiftKey)) { ev.preventDefault(); void redo(); }
  });

  past = [capture()]; // baseline
  syncButtons();
}

/** Record the current state as a new step (call at discrete moments). */
export function commit(): void {
  if (restoring) return;
  const snap = capture();
  const last = past[past.length - 1];
  if (last && JSON.stringify(last) === JSON.stringify(snap)) return; // no-op
  past.push(snap);
  if (past.length > MAX_STEPS) past.shift();
  future = [];
  syncButtons();
}

async function undo(): Promise<void> {
  if (past.length < 2 || !hooks || restoring) return;
  future.push(past.pop()!);
  await applyGuarded(past[past.length - 1]);
}

async function redo(): Promise<void> {
  const snap = future.pop();
  if (!snap || !hooks || restoring) return;
  past.push(snap);
  await applyGuarded(snap);
}

async function applyGuarded(snap: Snapshot): Promise<void> {
  restoring = true;
  try {
    await hooks!.restore(snap);
  } finally {
    restoring = false;
    syncButtons();
  }
}

function capture(): Snapshot {
  return {
    blocks: state.blocks.map((b) => ({
      rawText: b.rawText, anchor: b.anchor, bgMode: b.bgMode, x: b.x, y: b.y,
    })),
    stickers: state.stickers.map((s) => ({
      name: s.name, src: s.img.src, base64: s.dataBase64, x: s.x, y: s.y, scale: s.scale,
    })),
    crop: state.crop ? { ...state.crop } : null,
    cropRatio: state.cropRatio,
    outputSize: state.outputSize ? { ...state.outputSize } : null,
    filters: { ...state.filters },
    textSize: state.textSize,
    strokeWidth: state.strokeWidth,
    lineGap: state.lineGap,
    bgOffset: state.bgOffset,
  };
}

function syncButtons(): void {
  undoBtn.disabled = past.length < 2;
  redoBtn.disabled = future.length === 0;
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
