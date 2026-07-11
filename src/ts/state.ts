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

/** One chatlog block — its own text, its own position. */
export interface ChatBlock {
  id: number;
  /** Raw textarea content (what the user pasted). */
  rawText: string;
  /** Parsed lines from Rust (timestamps stripped, tags bolded). */
  lines: ParsedLine[];
  anchor: Anchor;
  /** Origin in image px — only used while anchor === "free". */
  x: number;
  y: number;
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

  /** Selected area of the source photo (null = the whole photo). */
  crop: CropRect | null;
  /** Locked aspect ratio for the crop box (w/h), null = free. */
  cropRatio: number | null;
  /** Fixed output resolution (e.g. 800×600); null = crop's own pixel size. */
  outputSize: { w: number; h: number } | null;
  /** True while the user is adjusting the crop box on the photo. */
  cropEditing: boolean;

  /** Chat text size in image px — auto-scales to image width, user-adjustable. */
  textSize: number;

  /** Font family for all chat text — picked from the installed system fonts. */
  fontFamily: string;


  /** Active parsing rules — sent to Rust with every parse. */
  preset: ParsePreset;

  /** Viewport transform: canvas px per image px, and pan offset in canvas px. */
  zoom: number;
  panX: number;
  panY: number;
}

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
  crop: null,
  cropRatio: null,
  outputSize: null,
  cropEditing: false,
  textSize: 27,
  fontFamily: "Arial",
  preset: structuredClone(DEFAULT_PRESET),
  zoom: 1,
  panX: 0,
  panY: 0,
};

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
