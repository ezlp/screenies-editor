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
import { initHistory } from "./history";
import type { Snapshot } from "./history";
import { loadPhotoFile, rebuildBlocksFrom } from "./chatlog";
import { syncFiltersUI } from "./filters";
import { syncTextStyleUI } from "./textstyle";
import { initI18n, setLang, getLang } from "./i18n";
import { state, notify } from "./state";
import { initStickers, rebuildStickersFrom } from "./stickers";
import { appVersion, isTauri } from "./tauri-bridge";


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
  initStickers();
  initI18n();
  initHistory({ restore: restoreSnapshot });
  initPastePhoto();

  const langBtn = document.getElementById("lang-toggle") as HTMLButtonElement | null;
  if (langBtn) {
    langBtn.addEventListener("click", () => {
      setLang(getLang() === "id" ? "en" : "id");
      scheduleSaveSettings();
    });
  }
  void showVersion();
  setLang(getLang()); // broadcast once so every module shows the loaded language
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


/** Undo/redo: put a snapshot back — state, UI, and reparse. */
async function restoreSnapshot(snap: Snapshot): Promise<void> {
  state.crop = snap.crop ? { ...snap.crop } : null;
  state.cropRatio = snap.cropRatio;
  state.outputSize = snap.outputSize ? { ...snap.outputSize } : null;
  state.filters = { ...snap.filters };
  state.textSize = snap.textSize;
  state.strokeWidth = snap.strokeWidth;
  state.lineGap = snap.lineGap;
  state.bgOffset = snap.bgOffset;

  await rebuildStickersFrom(snap.stickers);
  await rebuildBlocksFrom(snap.blocks); // also reparses + notifies
  syncFiltersUI();
  syncTextStyleUI();
  notify();
}

/** Photo can now arrive via Ctrl+V (clipboard image) or a drop anywhere
 *  on the preview — not just the dropzone. */
function initPastePhoto(): void {
  window.addEventListener("paste", (ev: ClipboardEvent) => {
    const items = ev.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (item.type.startsWith("image/")) {
        const file = item.getAsFile();
        if (file) {
          ev.preventDefault();
          loadPhotoFile(file);
        }
        return;
      }
    }
  });

  const viewport = document.getElementById("viewport");
  if (!viewport) return;
  viewport.addEventListener("dragover", (ev) => ev.preventDefault());
  viewport.addEventListener("drop", (ev) => {
    ev.preventDefault();
    const file = ev.dataTransfer?.files?.[0];
    if (file && file.type.startsWith("image/")) loadPhotoFile(file);
  });
}
