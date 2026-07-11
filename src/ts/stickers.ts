/**
 * stickers.ts — PNG overlays (Milestone 4c).
 *
 * Import objek/stiker/chatlog-PNGs; each gets a card (scale slider +
 * delete) and is draggable directly on the preview, drawn between the
 * photo and the text. Export sends original bytes + final rect to Rust.
 */

import { state, notify } from "./state";
import type { Sticker } from "./state";
import { clampBlocksToOutput } from "./canvas";

const ACCEPTED = ["image/png", "image/webp"];
let seq = 0;
let listEl: HTMLElement;

export function initStickers(): void {
  listEl = mustGet<HTMLElement>("sticker-list");
  const input = mustGet<HTMLInputElement>("sticker-input");
  mustGet<HTMLButtonElement>("btn-add-sticker").addEventListener("click", () => input.click());

  input.addEventListener("change", () => {
    const file = input.files?.[0];
    if (file) loadSticker(file);
    input.value = "";
  });
}

function loadSticker(file: File): void {
  if (!ACCEPTED.includes(file.type)) {
    console.warn(`[screenies-editor] stiker harus PNG/WebP: ${file.name}`);
    return;
  }
  const reader = new FileReader();
  reader.onload = () => {
    const img = new Image();
    img.onload = () => {
      const st: Sticker = {
        id: ++seq,
        name: file.name,
        dataBase64: String(reader.result).split(",")[1] ?? "",
        img,
        x: 24 + 18 * state.stickers.length,
        y: 24 + 18 * state.stickers.length,
        scale: 100,
      };
      state.stickers.push(st);
      listEl.appendChild(buildCard(st));
      notify();
    };
    img.src = String(reader.result);
  };
  reader.readAsDataURL(file);
}

function buildCard(st: Sticker): HTMLElement {
  const card = document.createElement("div");
  card.className = "sticker-card";

  const head = document.createElement("div");
  head.className = "row space-between";
  const name = document.createElement("span");
  name.className = "meta-chip";
  name.textContent = st.name;
  const remove = document.createElement("button");
  remove.className = "btn btn-ghost btn-small";
  remove.textContent = "✕";
  remove.title = "Hapus stiker";
  remove.addEventListener("click", () => {
    state.stickers = state.stickers.filter((s) => s.id !== st.id);
    card.remove();
    notify();
  });
  head.append(name, remove);

  const row = document.createElement("div");
  row.className = "row";
  const label = document.createElement("span");
  label.className = "field-label";
  label.textContent = "Ukuran";
  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = "10";
  slider.max = "300";
  slider.value = String(st.scale);
  slider.className = "slider-flex";
  const val = document.createElement("span");
  val.className = "meta-chip mono";
  val.textContent = "100%";
  slider.addEventListener("input", () => {
    st.scale = Number(slider.value);
    val.textContent = `${st.scale}%`;
    clampBlocksToOutput(); // keep it grabbable if it grew off-canvas
    notify();
  });
  row.append(label, slider, val);

  card.append(head, row);
  return card;
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
