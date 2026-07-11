/**
 * shortcuts.ts — quick-text templates (Milestone 4c).
 *
 * ssrphelper's shortcuts, persisted by Rust as templates.json in the app
 * config dir — survives cache clears, shareable. A chip inserts its text
 * at the cursor of the last-focused chatlog box.
 */

import { listTemplates, saveTemplates } from "./tauri-bridge";
import type { QuickText } from "./tauri-bridge";
import { getActiveChatArea } from "./chatlog";

let items: QuickText[] = [];
let chipRow: HTMLElement;

export function initShortcuts(): void {
  chipRow = mustGet<HTMLElement>("template-chips");
  const labelIn = mustGet<HTMLInputElement>("template-label");
  const textIn = mustGet<HTMLInputElement>("template-text");

  mustGet<HTMLButtonElement>("btn-add-template").addEventListener("click", () => {
    const label = labelIn.value.trim();
    const text = textIn.value;
    if (!label || !text) return;
    items.push({ label, text });
    labelIn.value = "";
    textIn.value = "";
    renderChips();
    void persist();
  });

  void (async () => {
    try {
      items = await listTemplates();
    } catch (err) {
      console.error("[screenies-editor] list_templates failed:", err);
    }
    renderChips();
  })();
}

function renderChips(): void {
  chipRow.innerHTML = "";
  for (const [i, item] of items.entries()) {
    const chip = document.createElement("button");
    chip.className = "btn btn-small template-chip";
    chip.textContent = item.label;
    chip.title = `${item.text}\n(klik kanan untuk hapus)`;
    chip.addEventListener("click", () => insert(item.text));
    chip.addEventListener("contextmenu", (ev) => {
      ev.preventDefault();
      items.splice(i, 1);
      renderChips();
      void persist();
    });
    chipRow.appendChild(chip);
  }
  if (items.length === 0) {
    const hint = document.createElement("span");
    hint.className = "hint";
    hint.textContent = "Belum ada template.";
    chipRow.appendChild(hint);
  }
}

function insert(text: string): void {
  const area = getActiveChatArea();
  if (!area) {
    console.warn("[screenies-editor] template: klik dulu kotak chatlog tujuan");
    return;
  }
  const start = area.selectionStart ?? area.value.length;
  const end = area.selectionEnd ?? start;
  area.value = area.value.slice(0, start) + text + area.value.slice(end);
  const cursor = start + text.length;
  area.focus();
  area.setSelectionRange(cursor, cursor);
  area.dispatchEvent(new Event("input", { bubbles: true }));
}

async function persist(): Promise<void> {
  try {
    await saveTemplates(items);
  } catch (err) {
    console.error("[screenies-editor] save_templates failed:", err);
  }
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
