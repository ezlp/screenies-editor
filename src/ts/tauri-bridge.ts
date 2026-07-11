/**
 * tauri-bridge.ts — the ONLY file allowed to call into Rust.
 *
 * Every backend call gets a small typed wrapper here so the rest of the
 * frontend never touches raw `invoke` strings.
 *
 * When the UI is opened in a plain browser (`npm run dev` without Tauri),
 * a minimal TS fallback keeps the preview usable — real parsing always
 * happens in Rust inside the actual app.
 */

import { invoke } from "@tauri-apps/api/core";
import type { ParsedLine, ParsePreset } from "./types";
import type { RenderRow } from "./canvas";

/** True when running inside the Tauri shell (vs a plain browser tab). */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/** Rust: chatlog text → parsed lines, using the given parsing preset. */
export async function parseChatlog(text: string, preset: ParsePreset): Promise<ParsedLine[]> {
  if (!isTauri()) return browserFallbackParse(text);
  return invoke<ParsedLine[]>("parse_chatlog", { text, preset });
}

/** Rust: built-in parsing presets. Empty in browser dev. */
export async function listPresets(): Promise<ParsePreset[]> {
  if (!isTauri()) return [];
  return invoke<ParsePreset[]>("list_presets");
}

/** Rust: open-file dialog → preset from a .toml file (null = cancelled). */
export async function importPresetToml(): Promise<ParsePreset | null> {
  if (!isTauri()) return null;
  return invoke<ParsePreset | null>("import_preset_toml");
}

/** Rust: save-file dialog → write preset as .toml (false = cancelled). */
export async function exportPresetToml(preset: ParsePreset): Promise<boolean> {
  if (!isTauri()) return false;
  return invoke<boolean>("export_preset_toml", { preset });
}

/** Everything Rust needs for one export — mirrors render/mod.rs RenderJob. */
export interface RenderJobPayload {
  imageBase64: string;
  crop: { x: number; y: number; w: number; h: number };
  output: { w: number; h: number };
  filters: { brightness: number; grayscale: number; sepia: number; saturate: number; contrast: number };
  fontFamily: string;
  textSize: number;
  strokeWidth: number;
  blocks: Array<{ rows: RenderRow[] }>;
}

/** Rust: render the job and save it via a native dialog (false = cancel). */
export async function exportPng(job: RenderJobPayload, fileName: string): Promise<boolean> {
  if (!isTauri()) return false;
  return invoke<boolean>("export_png", { job, fileName });
}

/** Rust: render the job and put it on the system clipboard. */
export async function copyPng(job: RenderJobPayload): Promise<void> {
  if (!isTauri()) return;
  return invoke<void>("copy_png", { job });
}

/** Persisted settings — mirrors config.rs AppSettings. */
export interface AppSettings {
  theme: string;
  fontFamily: string;
  preset: ParsePreset;
  fileNameTemplate: string;
}

/** Rust: saved settings, or null on first run / browser dev. */
export async function loadSettings(): Promise<AppSettings | null> {
  if (!isTauri()) return null;
  return invoke<AppSettings | null>("load_settings");
}

/** Rust: persist settings to settings.json. */
export async function saveSettings(settings: AppSettings): Promise<void> {
  if (!isTauri()) return;
  return invoke<void>("save_settings", { settings });
}

/** Rust: installed system font families (sorted). Empty in browser dev. */
export async function listFonts(): Promise<string[]> {
  if (!isTauri()) return [];
  return invoke<string[]>("list_fonts");
}

/** Rust: app version from Cargo.toml (proves the bridge is alive). */
export async function appVersion(): Promise<string> {
  if (!isTauri()) return "dev (browser)";
  return invoke<string>("app_version");
}

/* ── Browser-only fallback (never used inside the real app) ── */

const TIMESTAMP_RE = /^\s*(?:\[\d{1,2}:\d{2}(?::\d{2})?\]\s*)+/;
const SYSTAG_RE = /^([A-Za-z][A-Za-z0-9_/-]{0,19}):\s+(.+)$/;

function browserFallbackParse(text: string): ParsedLine[] {
  console.warn("[screenies-editor] Tauri not detected — using browser fallback parser.");
  return text
    .split(/\r?\n/)
    .map((line) => line.replace(TIMESTAMP_RE, "").trimEnd())
    .filter((line) => line.length > 0)
    .map((line): ParsedLine => {
      const tag = SYSTAG_RE.exec(line);
      if (tag) {
        return {
          spans: [
            { text: `${tag[1]}: `, color: "#FFFFFF", bold: true },
            { text: tag[2], color: "#FFFFFF", bold: false },
          ],
          lineType: "system",
        };
      }
      return {
        spans: [{ text: line, color: "#FFFFFF", bold: false }],
        lineType: "normal",
      };
    });
}
