/**
 * main.ts — app entry. Imports styles, boots every module.
 */

import "../styles/theme.css";
import "../styles/main.css";
import "../styles/panels.css";

import { initCanvas } from "./canvas";
import { initUpload, initChatlog } from "./chatlog";
import { initTextStyle } from "./textstyle";
import { initPreset } from "./preset";
import { initTheme } from "./theme";
import { applyLoadedSettings, scheduleSaveSettings } from "./settings";
import { initCrop } from "./crop";
import { initFilters } from "./filters";
import { initExport } from "./export";
import { initColorPalette } from "./colorpalette";
import { appVersion, isTauri } from "./tauri-bridge";

/* Later-milestone modules — imported so tsc type-checks them from day one. */
import "./zones";
import "./backgrounds";
import "./stickers";
import "./shortcuts";

window.addEventListener("DOMContentLoaded", () => {
  void boot();
});

async function boot(): Promise<void> {
  await applyLoadedSettings(); // restore theme/font/preset BEFORE inits sync UI
  initTheme(scheduleSaveSettings);
  initCanvas();
  initUpload();
  initChatlog();
  initTextStyle();
  initPreset();
  initCrop();
  initFilters();
  initExport();
  initColorPalette();
  void showVersion();
}

async function showVersion(): Promise<void> {
  const badge = document.getElementById("version-badge");
  if (!badge) return;
  try {
    badge.textContent = `v${await appVersion()}`;
    if (!isTauri()) badge.textContent += " · browser";
  } catch (err) {
    badge.textContent = "v?";
    console.error("[screenies-editor] app_version failed:", err);
  }
}
