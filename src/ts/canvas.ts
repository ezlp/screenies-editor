/**
 * canvas.ts — the live preview.
 *
 * Draws the screenshot plus every chatlog block in *image space*, so text
 * scales naturally with zoom exactly as it will in the final export.
 *
 * Controls: Ctrl+Scroll = zoom to cursor · drag image = pan ·
 *           drag a "Bebas" block = move it (anchored blocks are locked) ·
 *           double-click a Bebas block = reset it to the corner ·
 *           FIT / + / − buttons.
 */

import {
  DEFAULT_TEXT_X,
  DEFAULT_TEXT_Y,
  state,
  onChange,
  notify,
} from "./state";
import type { ChatBlock } from "./state";
import type { ColorSpan, ParsedLine } from "./types";

/* SSRP text look — size comes from state.textSize (auto-scales to image,
   adjustable with the "Ukuran teks" slider). Font/stroke controls: M4. */
const MARGIN_X = 14;      // side margins for anchored blocks & wrap limit
const MARGIN_Y = 16;      // top/bottom margins for anchored blocks
const LINE_GAP = 1.22;
const TEXT_HIT_PAD = 8;   // grab area around a free block, image px
const KEEP_ON_IMAGE = 24; // px of a free block that must stay inside the image
const MIN_WRAP = 80;      // never wrap narrower than this, image px

const ZOOM_MIN = 0.05;
const ZOOM_MAX = 8;
const ZOOM_STEP = 1.2;

let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;
let viewport: HTMLElement;
let emptyOverlay: HTMLElement;
let hudRes: HTMLElement;
let hudZoom: HTMLElement;

interface Bounds {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** Drawn bounds per block id (image px) — refreshed on every draw. */
const boundsById = new Map<number, Bounds>();

type DragMode = "none" | "pan" | "text";
let dragMode: DragMode = "none";
let dragBlockId: number | null = null;

export function initCanvas(): void {
  canvas = mustGet<HTMLCanvasElement>("preview-canvas");
  viewport = mustGet<HTMLElement>("viewport");
  emptyOverlay = mustGet<HTMLElement>("viewport-empty");
  hudRes = mustGet<HTMLElement>("hud-res");
  hudZoom = mustGet<HTMLElement>("hud-zoom");

  const context = canvas.getContext("2d");
  if (!context) throw new Error("Canvas 2D context unavailable");
  ctx = context;

  new ResizeObserver(() => {
    resizeToViewport();
    draw();
  }).observe(viewport);

  bindInteractions();
  onChange(draw);

  resizeToViewport();
  draw();
}

/** Zoom so the whole image fits the viewport, centered. */
export function fitImage(): void {
  const img = state.image;
  if (!img) return;
  const vw = viewport.clientWidth;
  const vh = viewport.clientHeight;
  const scale = Math.min(vw / img.width, vh / img.height) * 0.96;
  state.zoom = clampZoom(scale);
  state.panX = (vw - img.width * state.zoom) / 2;
  state.panY = (vh - img.height * state.zoom) / 2;
  notify();
}

/** Last drawn bounds of a block, if it has been drawn (used by the panel UI). */
export function getBlockBounds(id: number): Bounds | undefined {
  return boundsById.get(id);
}

/** Keep every free block reachable when a new image (size) arrives. */
export function clampBlocksToImage(img: HTMLImageElement): void {
  for (const block of state.blocks) {
    if (block.anchor !== "free") continue;
    block.x = clamp(block.x, 0, Math.max(0, img.width - KEEP_ON_IMAGE));
    block.y = clamp(block.y, 0, Math.max(0, img.height - KEEP_ON_IMAGE));
  }
}

/* ── drawing ── */

function resizeToViewport(): void {
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(viewport.clientWidth * dpr));
  canvas.height = Math.max(1, Math.round(viewport.clientHeight * dpr));
}

function draw(): void {
  const dpr = window.devicePixelRatio || 1;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);

  const img = state.image;
  emptyOverlay.classList.toggle("hidden", img !== null);

  if (!img) {
    boundsById.clear();
    hudRes.textContent = "RES —";
    hudZoom.textContent = "ZOOM —";
    return;
  }

  ctx.save();
  ctx.translate(state.panX, state.panY);
  ctx.scale(state.zoom, state.zoom);

  ctx.imageSmoothingEnabled = state.zoom < 3; // crisp pixels when zoomed far in
  ctx.drawImage(img, 0, 0);
  drawBlocks(img);

  ctx.restore();

  hudRes.textContent = `RES ${img.width}×${img.height}`;
  hudZoom.textContent = `ZOOM ${(state.zoom * 100).toFixed(0)}%`;
}

interface Token {
  text: string;
  font: string;
  color: string;
  width: number;
}
interface Row {
  tokens: Token[];
  width: number;
}
interface BlockLayout {
  rows: Row[];
  width: number;
  height: number;
}

function drawBlocks(img: HTMLImageElement): void {
  boundsById.clear();

  const size = state.textSize;
  const stroke = Math.max(2, Math.round(size / 9)); // outline scales with size

  ctx.textBaseline = "top";
  ctx.lineJoin = "round";
  ctx.lineWidth = stroke;
  ctx.strokeStyle = "#000000";

  const advance = size * LINE_GAP;

  for (const block of state.blocks) {
    // "Hanya RP" hides system-tagged lines (SERVER:, VEHICLE:, AdmCmd:, …).
    const lines = state.rpOnly
      ? block.lines.filter((l) => l.lineType !== "system")
      : block.lines;
    if (lines.length === 0) continue;

    const wrapWidth = wrapWidthFor(block, img);
    const layout = layoutLines(lines, size, wrapWidth, advance);
    const origin = blockOrigin(block, layout, img);
    const rightEdge = origin.x + layout.width;

    let y = origin.y;
    for (const row of layout.rows) {
      let x = origin.align === "right" ? rightEdge - row.width : origin.x;
      for (const token of row.tokens) {
        ctx.font = token.font;
        ctx.strokeText(token.text, x, y);
        ctx.fillStyle = token.color;
        ctx.fillText(token.text, x, y);
        x += token.width;
      }
      y += advance;
    }

    boundsById.set(block.id, {
      x: origin.x,
      y: origin.y,
      w: Math.max(layout.width, size), // never a zero-width grab target
      h: layout.height,
    });
  }
}

/** Word-wrap all lines into positioned rows (pass 1: measure only). */
function layoutLines(
  lines: ParsedLine[],
  size: number,
  wrapWidth: number,
  advance: number,
): BlockLayout {
  const rows: Row[] = [];
  let blockWidth = 0;

  for (const line of lines) {
    let row: Row = { tokens: [], width: 0 };

    for (const span of line.spans) {
      const font = spanFont(span, size);
      ctx.font = font;

      for (const raw of span.text.split(/(\s+)/)) {
        if (raw.length === 0) continue;
        const isSpace = /^\s+$/.test(raw);
        const width = ctx.measureText(raw).width;

        if (row.width + width > wrapWidth && row.width > 0) {
          rows.push(row);
          if (row.width > blockWidth) blockWidth = row.width;
          row = { tokens: [], width: 0 };
          if (isSpace) continue; // no leading space on a wrapped row
        }
        if (!isSpace || row.width > 0) {
          row.tokens.push({ text: raw, font, color: span.color, width });
          row.width += width;
        }
      }
    }

    rows.push(row);
    if (row.width > blockWidth) blockWidth = row.width;
  }

  const height = rows.length > 0 ? rows.length * advance - (advance - size) : 0;
  return { rows, width: blockWidth, height };
}

function wrapWidthFor(block: ChatBlock, img: HTMLImageElement): number {
  if (block.anchor === "free") {
    return Math.max(MIN_WRAP, img.width - MARGIN_X - block.x);
  }
  return Math.max(MIN_WRAP, img.width - MARGIN_X * 2);
}

function blockOrigin(
  block: ChatBlock,
  layout: BlockLayout,
  img: HTMLImageElement,
): { x: number; y: number; align: "left" | "right" } {
  switch (block.anchor) {
    case "free":
      return { x: block.x, y: block.y, align: "left" };
    case "kiri-atas":
      return { x: MARGIN_X, y: MARGIN_Y, align: "left" };
    case "kanan-atas":
      return {
        x: img.width - MARGIN_X - layout.width,
        y: MARGIN_Y,
        align: "right",
      };
    case "kiri-bawah":
      return {
        x: MARGIN_X,
        y: img.height - MARGIN_Y - layout.height,
        align: "left",
      };
    case "kanan-bawah":
      return {
        x: img.width - MARGIN_X - layout.width,
        y: img.height - MARGIN_Y - layout.height,
        align: "right",
      };
  }
}

/** System-tag prefixes render heavier (900) than regular chat (700). */
function spanFont(span: ColorSpan, size: number): string {
  const weight = span.bold ? "900" : "bold";
  return `${weight} ${size}px "${state.fontFamily}", Arial, sans-serif`;
}

/* ── interactions: zoom, pan, free block drag ── */

function bindInteractions(): void {
  canvas.addEventListener(
    "wheel",
    (ev) => {
      if (!ev.ctrlKey || !state.image) return;
      ev.preventDefault();
      const factor = ev.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
      zoomAround(ev.offsetX, ev.offsetY, factor);
    },
    { passive: false },
  );

  let lastX = 0;
  let lastY = 0;

  canvas.addEventListener("pointerdown", (ev) => {
    if (!state.image) return;
    dragBlockId = hitFreeBlock(ev.offsetX, ev.offsetY);
    dragMode = dragBlockId !== null ? "text" : "pan";
    lastX = ev.clientX;
    lastY = ev.clientY;
    canvas.classList.toggle("panning", dragMode === "pan");
    canvas.classList.toggle("dragging-text", dragMode === "text");
    canvas.setPointerCapture(ev.pointerId);
  });

  canvas.addEventListener("pointermove", (ev) => {
    if (dragMode === "none") {
      // Hover feedback: move-cursor over grabbable (Bebas) blocks only.
      canvas.classList.toggle(
        "over-text",
        hitFreeBlock(ev.offsetX, ev.offsetY) !== null,
      );
      return;
    }

    const dx = ev.clientX - lastX;
    const dy = ev.clientY - lastY;
    lastX = ev.clientX;
    lastY = ev.clientY;

    if (dragMode === "pan") {
      state.panX += dx;
      state.panY += dy;
    } else if (state.image && dragBlockId !== null) {
      const block = state.blocks.find((b) => b.id === dragBlockId);
      if (block) {
        // Free mode: move in image space so zoom doesn't distort the drag.
        const img = state.image;
        const w = boundsById.get(block.id)?.w ?? 0;
        block.x = clamp(
          block.x + dx / state.zoom,
          -(Math.max(w, KEEP_ON_IMAGE) - KEEP_ON_IMAGE),
          img.width - KEEP_ON_IMAGE,
        );
        block.y = clamp(block.y + dy / state.zoom, 0, img.height - KEEP_ON_IMAGE);
      }
    }
    notify();
  });

  const endDrag = (ev: PointerEvent) => {
    if (dragMode === "none") return;
    dragMode = "none";
    dragBlockId = null;
    canvas.classList.remove("panning", "dragging-text");
    canvas.releasePointerCapture(ev.pointerId);
  };
  canvas.addEventListener("pointerup", endDrag);
  canvas.addEventListener("pointercancel", endDrag);

  // Double-click a Bebas block → snap it back to the default corner.
  canvas.addEventListener("dblclick", (ev) => {
    const id = hitFreeBlock(ev.offsetX, ev.offsetY);
    if (id === null) return;
    const block = state.blocks.find((b) => b.id === id);
    if (!block) return;
    block.x = DEFAULT_TEXT_X;
    block.y = DEFAULT_TEXT_Y;
    notify();
  });

  mustGet<HTMLButtonElement>("btn-fit").addEventListener("click", fitImage);
  mustGet<HTMLButtonElement>("btn-zoom-in").addEventListener("click", () =>
    zoomAround(viewport.clientWidth / 2, viewport.clientHeight / 2, ZOOM_STEP),
  );
  mustGet<HTMLButtonElement>("btn-zoom-out").addEventListener("click", () =>
    zoomAround(viewport.clientWidth / 2, viewport.clientHeight / 2, 1 / ZOOM_STEP),
  );
}

/** Topmost draggable (anchor "free") block under this canvas point, or null. */
function hitFreeBlock(cx: number, cy: number): number | null {
  if (!state.image) return null;
  const ix = (cx - state.panX) / state.zoom;
  const iy = (cy - state.panY) / state.zoom;

  // Later blocks draw on top, so hit-test in reverse order.
  for (let i = state.blocks.length - 1; i >= 0; i--) {
    const block = state.blocks[i];
    if (block.anchor !== "free") continue;
    const b = boundsById.get(block.id);
    if (!b) continue;
    if (
      ix >= b.x - TEXT_HIT_PAD &&
      ix <= b.x + b.w + TEXT_HIT_PAD &&
      iy >= b.y - TEXT_HIT_PAD &&
      iy <= b.y + b.h + TEXT_HIT_PAD
    ) {
      return block.id;
    }
  }
  return null;
}

/** Zoom keeping the point under (cx, cy) fixed on screen. */
function zoomAround(cx: number, cy: number, factor: number): void {
  if (!state.image) return;
  const next = clampZoom(state.zoom * factor);
  const applied = next / state.zoom;
  state.panX = cx - (cx - state.panX) * applied;
  state.panY = cy - (cy - state.panY) * applied;
  state.zoom = next;
  notify();
}

function clampZoom(z: number): number {
  return Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, z));
}

function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v));
}

/* ── util ── */

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
