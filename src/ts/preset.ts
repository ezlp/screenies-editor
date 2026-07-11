/**
 * preset.ts — the "Preset format" UI.
 *
 * Dropdown of built-in presets from Rust (`list_presets`) plus a Kustom
 * entry that exposes every rule: toggles, colors, radio channels.
 * Any change re-parses every chatlog block through Rust with the new rules.
 *
 * Preset JSON is documented in docs/PRESETS.md (wiki material) — M4 adds
 * saving/loading custom presets as files.
 */

import { exportPresetToml, importPresetToml, listPresets } from "./tauri-bridge";
import { DEFAULT_PRESET, state, notify } from "./state";
import type { ParsePreset } from "./types";
import { reparseAllBlocks } from "./chatlog";
import { scheduleSaveSettings } from "./settings";

const CUSTOM_VALUE = "__custom__";

let select: HTMLSelectElement;
let customBox: HTMLElement;
let builtins: ParsePreset[] = [];

/* Kustom controls */
let ckTimestamps: HTMLInputElement;
let ckMe: HTMLInputElement;
let ckOoc: HTMLInputElement;
let ckDo: HTMLInputElement;
let ckSystag: HTMLInputElement;
let colorMe: HTMLInputElement;
let colorOoc: HTMLInputElement;
let radioChannels: HTMLInputElement;

export function initPreset(): void {
  select = mustGet<HTMLSelectElement>("preset-select");
  customBox = mustGet<HTMLElement>("preset-custom");
  ckTimestamps = mustGet<HTMLInputElement>("ck-timestamps");
  ckMe = mustGet<HTMLInputElement>("ck-me");
  ckOoc = mustGet<HTMLInputElement>("ck-ooc");
  ckDo = mustGet<HTMLInputElement>("ck-do");
  ckSystag = mustGet<HTMLInputElement>("ck-systag");
  colorMe = mustGet<HTMLInputElement>("color-me");
  colorOoc = mustGet<HTMLInputElement>("color-ooc");
  radioChannels = mustGet<HTMLInputElement>("radio-channels");

  select.addEventListener("change", onSelect);

  mustGet<HTMLButtonElement>("import-preset").addEventListener("click", async () => {
    try {
      const imported = await importPresetToml();
      if (!imported) return; // dialog cancelled
      state.preset = imported;
      if (!state.preset.name) state.preset.name = "Kustom";
      select.value = CUSTOM_VALUE;
      customBox.hidden = false;
      syncCustomControls();
      scheduleSaveSettings();
      await reparseAllBlocks();
    } catch (err) {
      console.error("[screenies-editor] import preset failed:", err);
    }
  });

  mustGet<HTMLButtonElement>("export-preset").addEventListener("click", async () => {
    try {
      await exportPresetToml(state.preset);
    } catch (err) {
      console.error("[screenies-editor] export preset failed:", err);
    }
  });
  for (const el of [ckTimestamps, ckMe, ckOoc, ckDo, ckSystag, colorMe, colorOoc]) {
    el.addEventListener("change", onCustomEdit);
  }
  radioChannels.addEventListener("input", debounce(onCustomEdit, 300));

  void populate();
}

async function populate(): Promise<void> {
  try {
    builtins = await listPresets();
  } catch (err) {
    console.error("[screenies-editor] list_presets failed:", err);
  }
  if (builtins.length === 0) builtins = [structuredClone(DEFAULT_PRESET)];

  select.innerHTML = "";
  builtins.forEach((p, i) => {
    const o = document.createElement("option");
    o.value = String(i);
    o.textContent = p.name;
    select.appendChild(o);
  });
  const custom = document.createElement("option");
  custom.value = CUSTOM_VALUE;
  custom.textContent = "Kustom…";
  select.appendChild(custom);

  // Restore whatever settings.ts loaded: a matching built-in selects
  // itself; anything else (a saved Kustom) opens the custom controls.
  const idx = builtins.findIndex((b) => b.name === state.preset.name);
  if (idx >= 0) {
    state.preset = structuredClone(builtins[idx]);
    select.value = String(idx);
    customBox.hidden = true;
  } else {
    select.value = CUSTOM_VALUE;
    customBox.hidden = false;
  }
  syncCustomControls();
  await reparseAllBlocks();
}

function onSelect(): void {
  if (select.value === CUSTOM_VALUE) {
    // Seed Kustom from whatever was active, then show the controls.
    state.preset = { ...structuredClone(state.preset), name: "Kustom" };
    customBox.hidden = false;
    syncCustomControls();
  } else {
    const idx = Number(select.value);
    state.preset = structuredClone(builtins[idx] ?? builtins[0]);
    customBox.hidden = true;
  }
  scheduleSaveSettings();
  void reparseAllBlocks();
}

function onCustomEdit(): void {
  const p = state.preset;
  p.name = "Kustom";
  p.stripTimestamps = ckTimestamps.checked;
  p.mePrefix = ckMe.checked;
  p.oocWrap = ckOoc.checked;
  p.doSuffix = ckDo.checked;
  p.systemTags = ckSystag.checked;
  p.colorMe = colorMe.value.toUpperCase();
  p.colorOoc = colorOoc.value.toUpperCase();
  p.radioChannels = radioChannels.value
    .split(",")
    .map((c) => c.trim())
    .filter((c) => c.length > 0);
  scheduleSaveSettings();
  notify();
  void reparseAllBlocks();
}

function syncCustomControls(): void {
  const p = state.preset;
  ckTimestamps.checked = p.stripTimestamps;
  ckMe.checked = p.mePrefix;
  ckOoc.checked = p.oocWrap;
  ckDo.checked = p.doSuffix;
  ckSystag.checked = p.systemTags;
  colorMe.value = p.colorMe;
  colorOoc.value = p.colorOoc;
  radioChannels.value = p.radioChannels.join(", ");
}

function debounce(fn: () => void, ms: number): () => void {
  let timer: number | undefined;
  return () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(fn, ms);
  };
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
