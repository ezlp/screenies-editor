/**
 * canvas.ts — the live preview, now in two modes.
 *
 * RESULT MODE (default): shows the final output — the cropped region of
 * the photo scaled to the chosen resolution, chat text drawn in OUTPUT
 * space. What you see here is exactly what M3c's Rust exporter will save.
 *
 * CROP EDIT MODE: shows the full source photo dimmed, with a bright
 * draggable/resizable crop box (corner handles; ratio-locked unless the
 * resolution is Bebas). Text is hidden while editing.
 *
 * Controls: Ctrl+Scroll zoom · drag = pan / move text / move-resize crop ·
 * double-click text = reset its position · double-click crop box = recenter.
 */

import {
  DEFAULT_TEXT_X,
  DEFAULT_TEXT_Y,
  state,
  onChange,
  notify,
} from "./state";
import type { ChatBlock, CropRect } from "./state";
import type { ColorSpan, ParsedLine } from "./types";

const MARGIN_X = 14;
const MARGIN_Y = 16;
const LINE_GAP = 1.22;
const TEXT_HIT_PAD = 8;
const KEEP_ON_IMAGE = 24;
const MIN_WRAP = 80;

const CROP_MIN = 60;        // smallest crop side, source px
const HANDLE_SCREEN = 12;   // handle hit size in *screen* px

const ZOOM_MIN = 0.05;
const ZOOM_MAX = 8;
const ZOOM_STEP = 1.2;

let canvas: HTMLCanvasElement;        // top layer: text, crop UI, pointer events
let ctx: CanvasRenderingContext2D;
let imageCanvas: HTMLCanvasElement;   // bottom layer: the photo (CSS-filtered)
let imgCtx: CanvasRenderingContext2D;
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

const boundsById = new Map<number, Bounds>();

type Corner = "tl" | "tr" | "bl" | "br";
type DragMode = "none" | "pan" | "text" | "crop-move" | "crop-resize";
let dragMode: DragMode = "none";
let dragBlockId: number | null = null;
let dragCorner: Corner = "br";

export function initCanvas(): void {
  canvas = mustGet<HTMLCanvasElement>("preview-canvas");
  imageCanvas = mustGet<HTMLCanvasElement>("image-canvas");
  viewport = mustGet<HTMLElement>("viewport");
  emptyOverlay = mustGet<HTMLElement>("viewport-empty");
  hudRes = mustGet<HTMLElement>("hud-res");
  hudZoom = mustGet<HTMLElement>("hud-zoom");

  const context = canvas.getContext("2d");
  const imageContext = imageCanvas.getContext("2d");
  if (!context || !imageContext) throw new Error("Canvas 2D context unavailable");
  ctx = context;
  imgCtx = imageContext;

  new ResizeObserver(() => {
    resizeToViewport();
    draw();
  }).observe(viewport);

  bindInteractions();
  onChange(draw);

  resizeToViewport();
  draw();
}

/* ── shared geometry ── */

/** CSS filter string for the photo (never applied to text). */
export function cssFilterString(): string {
  const f = state.filters;
  const parts: string[] = [];
  if (f.brightness !== 100) parts.push(`brightness(${f.brightness}%)`);
  if (f.grayscale !== 0) parts.push(`grayscale(${f.grayscale}%)`);
  if (f.sepia !== 0) parts.push(`sepia(${f.sepia}%)`);
  if (f.saturate !== 100) parts.push(`saturate(${f.saturate}%)`);
  if (f.contrast !== 100) parts.push(`contrast(${f.contrast}%)`);
  return parts.length > 0 ? parts.join(" ") : "none";
}

/** The crop rectangle in source pixels (whole photo when crop is null). */
export function sourceCrop(): CropRect | null {
  const img = state.image;
  if (!img) return null;
  return state.crop ?? { x: 0, y: 0, w: img.width, h: img.height };
}

/** Final output dimensions — what the export will be, and the text space. */
export function outputDims(): { w: number; h: number } | null {
  const crop = sourceCrop();
  if (!crop) return null;
  if (state.outputSize) return { ...state.outputSize };
  return { w: Math.round(crop.w), h: Math.round(crop.h) };
}

/** Zoom so the current subject (output or full photo) fits, centered. */
export function fitImage(): void {
  const subject = state.cropEditing
    ? state.image && { w: state.image.width, h: state.image.height }
    : outputDims();
  if (!subject) return;
  const vw = viewport.clientWidth;
  const vh = viewport.clientHeight;
  const scale = Math.min(vw / subject.w, vh / subject.h) * 0.96;
  state.zoom = clampZoom(scale);
  state.panX = (vw - subject.w * state.zoom) / 2;
  state.panY = (vh - subject.h * state.zoom) / 2;
  notify();
}

export function getBlockBounds(id: number): Bounds | undefined {
  return boundsById.get(id);
}

/** Keep every free block reachable when the output dimensions change. */
export function clampBlocksToOutput(): void {
  const out = outputDims();
  if (!out) return;
  for (const block of state.blocks) {
    if (block.anchor !== "free") continue;
    block.x = clamp(block.x, 0, Math.max(0, out.w - KEEP_ON_IMAGE));
    block.y = clamp(block.y, 0, Math.max(0, out.h - KEEP_ON_IMAGE));
  }
}

/** Largest crop of the given ratio (null = whole photo), centered. */
export function centeredCrop(ratio: number | null): CropRect | null {
  const img = state.image;
  if (!img) return null;
  if (ratio === null) return { x: 0, y: 0, w: img.width, h: img.height };
  let w = img.width;
  let h = w / ratio;
  if (h > img.height) {
    h = img.height;
    w = h * ratio;
  }
  return {
    x: Math.round((img.width - w) / 2),
    y: Math.round((img.height - h) / 2),
    w: Math.round(w),
    h: Math.round(h),
  };
}

/* ── drawing ── */

function resizeToViewport(): void {
  const dpr = window.devicePixelRatio || 1;
  const w = Math.max(1, Math.round(viewport.clientWidth * dpr));
  const h = Math.max(1, Math.round(viewport.clientHeight * dpr));
  canvas.width = w;
  canvas.height = h;
  imageCanvas.width = w;
  imageCanvas.height = h;
}

function draw(): void {
  const dpr = window.devicePixelRatio || 1;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  imgCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
  imgCtx.clearRect(0, 0, imageCanvas.width, imageCanvas.height);

  const img = state.image;
  emptyOverlay.classList.toggle("hidden", img !== null);

  if (!img) {
    boundsById.clear();
    imageCanvas.style.filter = "none";
    hudRes.textContent = "RES —";
    hudZoom.textContent = "ZOOM —";
    return;
  }

  // Filters apply to the photo layer as a CSS element filter — this works
  // on every webview, unlike ctx.filter (missing on Linux WebKitGTK).
  imageCanvas.style.filter = cssFilterString();

  for (const c2 of [ctx, imgCtx]) {
    c2.save();
    c2.translate(state.panX, state.panY);
    c2.scale(state.zoom, state.zoom);
    c2.imageSmoothingEnabled = state.zoom < 3;
  }

  if (state.cropEditing) {
    drawCropEditor(img);
  } else {
    drawResult(img);
  }

  ctx.restore();
  imgCtx.restore();

  const out = outputDims();
  hudRes.textContent = state.cropEditing
    ? `SUMBER ${img.width}×${img.height}`
    : out
      ? `RES ${out.w}×${out.h}`
      : "RES —";
  hudZoom.textContent = `ZOOM ${(state.zoom * 100).toFixed(0)}%`;
}

/** RESULT MODE: crop region → output size, text on top in output space. */
function drawResult(img: HTMLImageElement): void {
  const crop = sourceCrop();
  const out = outputDims();
  if (!crop || !out) return;

  imgCtx.drawImage(img, crop.x, crop.y, crop.w, crop.h, 0, 0, out.w, out.h);
  drawBlocks(out.w); // text layer on top — never filtered, by construction
}

/** CROP EDIT MODE: full photo, dim outside the box, handles on corners. */
function drawCropEditor(img: HTMLImageElement): void {
  boundsById.clear(); // no text hit-testing while editing
  imgCtx.drawImage(img, 0, 0); // filtered via the layer's CSS filter

  const crop = sourceCrop();
  if (!crop) return;

  // Dim everything OUTSIDE the crop box (4 strips on the UI layer),
  // so the selected region stays bright without re-drawing the photo.
  ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
  ctx.fillRect(0, 0, img.width, crop.y); // top
  ctx.fillRect(0, crop.y + crop.h, img.width, img.height - crop.y - crop.h); // bottom
  ctx.fillRect(0, crop.y, crop.x, crop.h); // left
  ctx.fillRect(crop.x + crop.w, crop.y, img.width - crop.x - crop.w, crop.h); // right

  const lw = 2 / state.zoom; // constant on screen
  ctx.lineWidth = lw;
  ctx.strokeStyle = "#c2a2da";
  ctx.strokeRect(crop.x, crop.y, crop.w, crop.h);

  const hs = HANDLE_SCREEN / state.zoom;
  ctx.fillStyle = "#c2a2da";
  for (const [hx, hy] of cropHandlePoints(crop)) {
    ctx.fillRect(hx - hs / 2, hy - hs / 2, hs, hs);
  }
}

function cropHandlePoints(c: CropRect): Array<[number, number]> {
  return [
    [c.x, c.y],
    [c.x + c.w, c.y],
    [c.x, c.y + c.h],
    [c.x + c.w, c.y + c.h],
  ];
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

function drawBlocks(outputWidth: number): void {
  boundsById.clear();

  const size = state.textSize;
  const stroke = Math.max(2, Math.round(size / 9));

  ctx.textBaseline = "top";
  ctx.lineJoin = "round";
  ctx.lineWidth = stroke;
  ctx.strokeStyle = "#000000";

  const advance = size * LINE_GAP;

  for (const block of state.blocks) {
    if (block.lines.length === 0) continue;

    const wrapWidth = wrapWidthFor(block, outputWidth);
    const layout = layoutLines(block.lines, size, wrapWidth, advance);
    const origin = blockOrigin(block, layout);

    let y = origin.y;
    for (const row of layout.rows) {
      let x = origin.x;
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
      w: Math.max(layout.width, size),
      h: layout.height,
    });
  }
}

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
          if (isSpace) continue;
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

function wrapWidthFor(block: ChatBlock, outputWidth: number): number {
  if (block.anchor === "free") {
    return Math.max(MIN_WRAP, outputWidth - MARGIN_X - block.x);
  }
  return Math.max(MIN_WRAP, outputWidth - MARGIN_X * 2);
}

function blockOrigin(
  block: ChatBlock,
  layout: BlockLayout,
): { x: number; y: number } {
  const out = outputDims() ?? { w: 0, h: 0 };
  switch (block.anchor) {
    case "free":
      return { x: block.x, y: block.y };
    case "kiri-atas":
      return { x: MARGIN_X, y: MARGIN_Y };
    case "kiri-bawah":
      return { x: MARGIN_X, y: out.h - MARGIN_Y - layout.height };
  }
}

function spanFont(span: ColorSpan, size: number): string {
  const weight = span.bold ? "900" : "bold";
  return `${weight} ${size}px "${state.fontFamily}", Arial, sans-serif`;
}

/* ── interactions ── */

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

    if (state.cropEditing) {
      const corner = hitCropHandle(ev.offsetX, ev.offsetY);
      if (corner) {
        dragMode = "crop-resize";
        dragCorner = corner;
      } else if (hitInsideCrop(ev.offsetX, ev.offsetY)) {
        dragMode = "crop-move";
      } else {
        dragMode = "pan";
      }
    } else {
      dragBlockId = hitFreeBlock(ev.offsetX, ev.offsetY);
      dragMode = dragBlockId !== null ? "text" : "pan";
    }

    lastX = ev.clientX;
    lastY = ev.clientY;
    canvas.classList.toggle("panning", dragMode === "pan");
    canvas.classList.toggle(
      "dragging-text",
      dragMode === "text" || dragMode === "crop-move" || dragMode === "crop-resize",
    );
    canvas.setPointerCapture(ev.pointerId);
  });

  canvas.addEventListener("pointermove", (ev) => {
    if (dragMode === "none") {
      const over = state.cropEditing
        ? hitCropHandle(ev.offsetX, ev.offsetY) !== null ||
          hitInsideCrop(ev.offsetX, ev.offsetY)
        : hitFreeBlock(ev.offsetX, ev.offsetY) !== null;
      canvas.classList.toggle("over-text", over);
      return;
    }

    const dx = ev.clientX - lastX;
    const dy = ev.clientY - lastY;
    lastX = ev.clientX;
    lastY = ev.clientY;

    if (dragMode === "pan") {
      state.panX += dx;
      state.panY += dy;
    } else if (dragMode === "text" && state.image && dragBlockId !== null) {
      const block = state.blocks.find((b) => b.id === dragBlockId);
      const out = outputDims();
      if (block && out) {
        const w = boundsById.get(block.id)?.w ?? 0;
        block.x = clamp(
          block.x + dx / state.zoom,
          -(Math.max(w, KEEP_ON_IMAGE) - KEEP_ON_IMAGE),
          out.w - KEEP_ON_IMAGE,
        );
        block.y = clamp(block.y + dy / state.zoom, 0, out.h - KEEP_ON_IMAGE);
      }
    } else if (dragMode === "crop-move" && state.image && state.crop) {
      const img = state.image;
      const c = state.crop;
      c.x = clamp(c.x + dx / state.zoom, 0, img.width - c.w);
      c.y = clamp(c.y + dy / state.zoom, 0, img.height - c.h);
    } else if (dragMode === "crop-resize" && state.image && state.crop) {
      resizeCrop(dx / state.zoom, dy / state.zoom);
    }
    notify();
  });

  const endDrag = (ev: PointerEvent) => {
    if (dragMode === "none") return;
    const wasCrop = dragMode === "crop-move" || dragMode === "crop-resize";
    dragMode = "none";
    dragBlockId = null;
    canvas.classList.remove("panning", "dragging-text");
    canvas.releasePointerCapture(ev.pointerId);
    if (wasCrop) roundCrop();
  };
  canvas.addEventListener("pointerup", endDrag);
  canvas.addEventListener("pointercancel", endDrag);

  canvas.addEventListener("dblclick", (ev) => {
    if (state.cropEditing) {
      // Recenter the crop box at max size for the current ratio.
      const c = centeredCrop(state.cropRatio);
      if (c) {
        state.crop = c;
        notify();
      }
      return;
    }
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

/** Resize from the dragged corner; opposite corner stays anchored. */
function resizeCrop(dx: number, dy: number): void {
  const img = state.image;
  const c = state.crop;
  if (!img || !c) return;

  // Anchor = the corner opposite to the one being dragged.
  const anchorX = dragCorner === "tl" || dragCorner === "bl" ? c.x + c.w : c.x;
  const anchorY = dragCorner === "tl" || dragCorner === "tr" ? c.y + c.h : c.y;
  const moveX = dragCorner === "tl" || dragCorner === "bl" ? c.x + dx : c.x + c.w + dx;
  const moveY = dragCorner === "tl" || dragCorner === "tr" ? c.y + dy : c.y + c.h + dy;

  let w = Math.abs(moveX - anchorX);
  let h = Math.abs(moveY - anchorY);

  if (state.cropRatio !== null) {
    // Lock to ratio: follow the dominant axis of the drag.
    if (w / state.cropRatio >= h) h = w / state.cropRatio;
    else w = h * state.cropRatio;
  }

  w = Math.max(CROP_MIN, w);
  h = Math.max(state.cropRatio ? w / state.cropRatio : CROP_MIN, CROP_MIN);

  let x = Math.min(anchorX, anchorX + (moveX >= anchorX ? w : -w));
  let y = Math.min(anchorY, anchorY + (moveY >= anchorY ? h : -h));

  // Clamp inside the photo, shrinking if the ratio demands it.
  if (x < 0) { w += x; x = 0; }
  if (y < 0) { h += y; y = 0; }
  if (x + w > img.width) w = img.width - x;
  if (y + h > img.height) h = img.height - y;
  if (state.cropRatio !== null) {
    if (w / state.cropRatio > h) w = h * state.cropRatio;
    else h = w / state.cropRatio;
  }

  c.x = x;
  c.y = y;
  c.w = Math.max(CROP_MIN, w);
  c.h = Math.max(CROP_MIN, h);
}

function roundCrop(): void {
  const c = state.crop;
  if (!c) return;
  c.x = Math.round(c.x);
  c.y = Math.round(c.y);
  c.w = Math.round(c.w);
  c.h = Math.round(c.h);
  notify();
}

/* ── hit tests ── */

function toImageSpace(cx: number, cy: number): { x: number; y: number } {
  return { x: (cx - state.panX) / state.zoom, y: (cy - state.panY) / state.zoom };
}

function hitCropHandle(cx: number, cy: number): Corner | null {
  const c = state.crop ?? sourceCrop();
  if (!c) return null;
  const p = toImageSpace(cx, cy);
  const r = HANDLE_SCREEN / state.zoom;
  const corners: Array<[Corner, number, number]> = [
    ["tl", c.x, c.y],
    ["tr", c.x + c.w, c.y],
    ["bl", c.x, c.y + c.h],
    ["br", c.x + c.w, c.y + c.h],
  ];
  for (const [name, hx, hy] of corners) {
    if (Math.abs(p.x - hx) <= r && Math.abs(p.y - hy) <= r) return name;
  }
  return null;
}

function hitInsideCrop(cx: number, cy: number): boolean {
  const c = state.crop ?? sourceCrop();
  if (!c) return false;
  const p = toImageSpace(cx, cy);
  return p.x >= c.x && p.x <= c.x + c.w && p.y >= c.y && p.y <= c.y + c.h;
}

function hitFreeBlock(cx: number, cy: number): number | null {
  if (!state.image) return null;
  const p = toImageSpace(cx, cy);

  for (let i = state.blocks.length - 1; i >= 0; i--) {
    const block = state.blocks[i];
    if (block.anchor !== "free") continue;
    const b = boundsById.get(block.id);
    if (!b) continue;
    if (
      p.x >= b.x - TEXT_HIT_PAD &&
      p.x <= b.x + b.w + TEXT_HIT_PAD &&
      p.y >= b.y - TEXT_HIT_PAD &&
      p.y <= b.y + b.h + TEXT_HIT_PAD
    ) {
      return block.id;
    }
  }
  return null;
}

/* ── zoom helpers ── */

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

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
