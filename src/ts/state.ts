/**
 * state.ts — single source of truth for the whole app.
 * Modules mutate `state` through helpers and call `notify()`;
 * the canvas subscribes and redraws.
 */

import type { ParsedLine, ParsePreset } from "./types";

/** Where a chatlog block sits on the photo. */
export type Anchor =
  | "free"        // draggable anywhere (seret di preview)
  | "kiri-atas"
  | "kiri-bawah";

/** Background behind a block's rows. */
export type BgMode = "none" | "block" | "mask";

/** One chatlog block — its own text, its own position. */
export interface ChatBlock {
  id: number;
  /** Raw textarea content (what the user pasted). */
  rawText: string;
  /** Parsed lines from Rust (timestamps stripped, tags bolded). */
  lines: ParsedLine[];
  anchor: Anchor;
  /** Background: none, per-row block, or full-width mask strip. */
  bgMode: BgMode;
  /** Origin in image px — only used while anchor === "free". */
  x: number;
  y: number;
}

/** A PNG overlay (objek/stiker), draggable, drawn under the text. */
export interface Sticker {
  id: number;
  name: string;
  /** Original file bytes (base64) — what the exporter decodes. */
  dataBase64: string;
  img: HTMLImageElement;
  /** Top-left in output px. */
  x: number;
  y: number;
  /** Percent of natural size. */
  scale: number;
}

/** Crop rectangle in source-image pixels. */
export interface CropRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface AppState {
  /** Loaded screenshot (null until the user uploads one). */
  image: HTMLImageElement | null;
  imageName: string;

  /** All chatlog blocks, drawn in order (later = on top). */
  blocks: ChatBlock[];

  /** PNG overlays, drawn between the photo and the text. */
  stickers: Sticker[];

  /** Selected area of the source photo (null = the whole photo). */
  crop: CropRect | null;
  /** Locked aspect ratio for the crop box (w/h), null = free. */
  cropRatio: number | null;
  /** Fixed output resolution (e.g. 800×600); null = crop's own pixel size. */
  outputSize: { w: number; h: number } | null;
  /** True while the user is adjusting the crop box on the photo. */
  cropEditing: boolean;

  /** Live image filters, CSS-filter percentages. Text is never filtered. */
  filters: Filters;

  /** Chat text size in image px — auto-scales to image width, user-adjustable. */
  textSize: number;

  /** Outline thickness in px; null = auto (scales with textSize). */
  strokeWidth: number | null;

  /** Line spacing as percent of textSize (122 = the classic SSRP look). */
  lineGap: number;

  /** Vertical nudge for BG strips, px (fine-tune around the auto shift). */
  bgOffset: number;


  /** Font family for all chat text — picked from the installed system fonts. */
  fontFamily: string;


  /** Active parsing rules — sent to Rust with every parse. */
  preset: ParsePreset;

  /** Save-file name template, e.g. "screenie-{tanggal}-{jam}". */
  fileNameTemplate: string;

  /** Viewport transform: canvas px per image px, and pan offset in canvas px. */
  zoom: number;
  panX: number;
  panY: number;
}

/** Image filter values (percent, per the CSS filter spec). */
export interface Filters {
  brightness: number;
  grayscale: number;
  sepia: number;
  saturate: number;
  contrast: number;
}

export const DEFAULT_FILTERS: Filters = {
  brightness: 100,
  grayscale: 0,
  sepia: 0,
  saturate: 100,
  contrast: 100,
};

/** Default text origin — top-left with a small margin (SSRP convention). */
export const DEFAULT_TEXT_X = 14;
export const DEFAULT_TEXT_Y = 16;

/** Boot preset — replaced by Rust's list on startup (kept in sync manually). */
export const DEFAULT_PRESET: ParsePreset = {
  name: "JGRP (Jogjagamers)",
  stripTimestamps: true,
  hexCodes: true,
  mePrefix: true,
  oocWrap: true,
  doSuffix: true,
  systemTags: true,
  radioChannels: ["phone", "walkie"],
  colorMe: "#C2A2DA",
  colorOoc: "#9C9C9C",
  colorDefault: "#FFFFFF",
};

export const state: AppState = {
  image: null,
  imageName: "",
  blocks: [],
  stickers: [],
  crop: null,
  cropRatio: null,
  outputSize: null,
  cropEditing: false,
  filters: { ...DEFAULT_FILTERS },
  textSize: 27,
  strokeWidth: null,
  lineGap: 122,
  bgOffset: 0,
  fontFamily: "Verdana", // crisper than Arial at small SSRP text sizes
  preset: structuredClone(DEFAULT_PRESET),
  fileNameTemplate: "screenie-{tanggal}-{jam}",
  zoom: 1,
  panX: 0,
  panY: 0,
};

/** The outline width actually used: manual value, or auto from text size. */
export function effectiveStroke(): number {
  if (state.strokeWidth !== null) return state.strokeWidth;
  // Auto: scales with size, but thins to 1px below 14px — a 2px outline
  // drowns small glyphs, which was what made tiny text unreadable.
  const min = state.textSize < 14 ? 1 : 2;
  return Math.max(min, Math.round(state.textSize / 9));
}

type Listener = () => void;
const listeners: Listener[] = [];

/** Subscribe to any state change (canvas uses this to redraw). */
export function onChange(fn: Listener): void {
  listeners.push(fn);
}

/** Broadcast that state changed. */
export function notify(): void {
  for (const fn of listeners) fn();
}
