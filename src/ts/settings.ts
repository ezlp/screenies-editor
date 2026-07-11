/**
 * settings.ts — persistence glue (Milestone 4a).
 *
 * Boot: pull settings.json from Rust and apply it to state + theme.
 * After: any module calls scheduleSaveSettings() and a debounced write
 * captures theme + font + the full active preset (Kustom included).
 */

import { loadSettings, saveSettings } from "./tauri-bridge";
import { state } from "./state";
import { currentTheme, setTheme } from "./theme";

const SAVE_DEBOUNCE_MS = 400;
let timer: number | undefined;

/** Call once, before module inits, so they sync UI from restored state. */
export async function applyLoadedSettings(): Promise<void> {
  try {
    const s = await loadSettings();
    if (!s) return;
    state.fontFamily = s.fontFamily;
    state.preset = s.preset;
    setTheme(s.theme === "light" ? "light" : "dark");
  } catch (err) {
    console.error("[screenies-editor] load_settings failed:", err);
  }
}

/** Debounced persist — safe to call on every settings-ish change. */
export function scheduleSaveSettings(): void {
  window.clearTimeout(timer);
  timer = window.setTimeout(() => void persist(), SAVE_DEBOUNCE_MS);
}

async function persist(): Promise<void> {
  try {
    await saveSettings({
      theme: currentTheme(),
      fontFamily: state.fontFamily,
      preset: state.preset,
    });
  } catch (err) {
    console.error("[screenies-editor] save_settings failed:", err);
  }
}
