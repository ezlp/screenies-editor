/**
 * chatlog.ts — left panel: image upload + the chatlog block list.
 *
 * Each block is a card: its own textarea, a position dropdown
 * (Bebas / 4 corners) and a remove button. "+ Tambah Chatlog" adds more.
 *
 * Flow per block: paste → debounce → Rust `parse_chatlog` (strips
 * timestamps, bolds system tags) → block.lines → canvas redraws.
 */

import { parseChatlog, readDroppedImage } from "./tauri-bridge";
import { DEFAULT_TEXT_X, DEFAULT_TEXT_Y, state, notify } from "./state";
import type { Anchor, BgMode, ChatBlock } from "./state";
import { fitImage, getBlockBounds } from "./canvas";
import { onImageLoaded } from "./crop";
import { commit } from "./history";
import { t } from "./i18n";
import { autoTextSize } from "./textstyle";

const DEBOUNCE_MS = 150;
const ACCEPTED = ["image/png", "image/jpeg", "image/webp", "image/bmp"];
/** Vertical offset between newly added free blocks so they don't stack. */
const NEW_BLOCK_STEP = 70;

function anchorOptions(): Array<{ value: Anchor; label: string }> {
  return [
    { value: "free", label: t("anchorFree") },
    { value: "kiri-atas", label: t("anchorTopLeft") },
    { value: "kiri-bawah", label: t("anchorBottomLeft") },
  ];
}

function bgOptions(): Array<{ value: BgMode; label: string }> {
  return [
    { value: "none", label: t("bgNone") },
    { value: "block", label: t("bgBlock") },
    { value: "mask", label: t("bgMask") },
  ];
}

let blockSeq = 0;
let listEl: HTMLElement;
/** Per-block "N baris" hints, so preset changes can refresh them. */
const statusById = new Map<number, HTMLElement>();

/** The chatlog textarea the user last focused — the palette's target. */
let activeArea: { blockId: number; textarea: HTMLTextAreaElement } | null = null;

export function getActiveChatArea(): HTMLTextAreaElement | null {
  return activeArea?.textarea ?? null;
}

/** Language switch: retranslate every card's dropdown options in place. */
function relabelSelects(): void {
  const container = document.getElementById("chatlog-list");
  if (!container) return;
  for (const card of Array.from(container.children)) {
    const selects = card.querySelectorAll("select");
    const sets = [anchorOptions(), bgOptions()];
    selects.forEach((sel, i) => {
      const opts = sets[i];
      if (!opts) return;
      Array.from(sel.options).forEach((o, j) => {
        if (opts[j]) o.textContent = opts[j].label;
      });
    });
  }
}

/** Set by initChatlog; lets paste/drop handlers reuse the photo loader. */
let imageLoader: ((file: File) => void) | null = null;

/** Meta strip (name + resolution) — set in initUpload, reused by every loader. */
let metaEl: HTMLElement | null = null;
let metaNameEl: HTMLElement | null = null;
let metaResEl: HTMLElement | null = null;

export function loadPhotoFile(file: File): void {
  imageLoader?.(file);
}

/** Apply a decoded image from any source (File reader, clipboard, or a
 *  Tauri-dropped path resolved to a data: URL). Single place that mutates
 *  state.image so preview/crop/text-size all react identically. */
function loadImageFromSrc(src: string, name: string): void {
  const img = new Image();
  img.onload = () => {
    state.image = img;
    state.imageName = name;
    if (metaNameEl) metaNameEl.textContent = name;
    if (metaResEl) metaResEl.textContent = `${img.width}×${img.height}`;
    if (metaEl) metaEl.hidden = false;
    onImageLoaded();         // new photo → crop back to full frame
    autoTextSize(img.width); // low-res photo → smaller text, high-res → bigger
    fitImage();
  };
  img.src = src;
}

/** OS drag-drop (Tauri) hands us a file PATH, not a File — read it via Rust. */
export async function loadPhotoFromPath(path: string): Promise<void> {
  try {
    const dataUrl = await readDroppedImage(path);
    if (!dataUrl) return;
    const name = path.split(/[\\/]/).pop() ?? "dropped";
    loadImageFromSrc(dataUrl, name);
  } catch (err) {
    console.error("[screenies-editor] read_dropped_image failed:", err);
  }
}

/** Undo/redo: rebuild all block cards + state from plain snapshot data. */
export async function rebuildBlocksFrom(
  snaps: Array<{ rawText: string; anchor: Anchor; bgMode: BgMode; x: number; y: number }>,
): Promise<void> {
  const container = mustGet<HTMLElement>("chatlog-list");
  container.innerHTML = "";
  statusById.clear();
  activeArea = null;
  state.blocks = [];
  for (const s of snaps) {
    const block = addBlock();
    block.rawText = s.rawText;
    block.anchor = s.anchor;
    block.bgMode = s.bgMode;
    block.x = s.x;
    block.y = s.y;
    const card = container.lastElementChild;
    const ta = card?.querySelector("textarea");
    if (ta) ta.value = s.rawText;
    const selects = card?.querySelectorAll("select");
    if (selects && selects[0]) (selects[0] as HTMLSelectElement).value = s.anchor;
    if (selects && selects[1]) (selects[1] as HTMLSelectElement).value = s.bgMode;
  }
  await reparseAllBlocks();
}

/* ── image upload (unchanged behavior) ── */

export function initUpload(): void {
  const dropzone = mustGet<HTMLElement>("dropzone");
  const fileInput = mustGet<HTMLInputElement>("file-input");
  const meta = mustGet<HTMLElement>("image-meta");
  metaEl = meta;
  metaNameEl = mustGet<HTMLElement>("meta-name");
  metaResEl = mustGet<HTMLElement>("meta-res");
  const clearBtn = mustGet<HTMLButtonElement>("btn-clear-image");

  const openPicker = () => fileInput.click();
  dropzone.addEventListener("click", openPicker);
  dropzone.addEventListener("keydown", (ev) => {
    if (ev.key === "Enter" || ev.key === " ") {
      ev.preventDefault();
      openPicker();
    }
  });

  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (file) loadImageFile(file);
    fileInput.value = ""; // allow re-selecting the same file
  });

  // Drag & drop onto the dropzone AND the whole window (like both old sites).
  for (const target of [dropzone, document.body]) {
    target.addEventListener("dragover", (ev) => {
      ev.preventDefault();
      dropzone.classList.add("dragover");
    });
    target.addEventListener("dragleave", () => dropzone.classList.remove("dragover"));
    target.addEventListener("drop", (ev) => {
      ev.preventDefault();
      dropzone.classList.remove("dragover");
      const file = ev.dataTransfer?.files?.[0];
      if (file) loadImageFile(file);
    });
  }

  clearBtn.addEventListener("click", () => {
    state.image = null;
    state.imageName = "";
    meta.hidden = true;
    notify();
  });

  imageLoader = loadImageFile;
  function loadImageFile(file: File): void {
    if (!ACCEPTED.includes(file.type)) {
      console.warn(`[screenies-editor] Ignored non-image file: ${file.name} (${file.type})`);
      return;
    }
    const reader = new FileReader();
    reader.onload = () => loadImageFromSrc(String(reader.result), file.name);
    reader.readAsDataURL(file);
  }
}

/* ── chatlog blocks ── */

export function initChatlog(): void {
  listEl = mustGet<HTMLElement>("chatlog-list");
  const addBtn = mustGet<HTMLButtonElement>("btn-add-chatlog");
  addBtn.textContent = t("addChatlog");
  window.addEventListener("i18n-changed", () => {
    addBtn.textContent = t("addChatlog");
    relabelSelects();
  });

  addBtn.addEventListener("click", () => {
    addBlock();
    commit();
  });
  if (state.blocks.length === 0) addBlock(); // start with one block
}

function addBlock(): ChatBlock {
  const block: ChatBlock = {
    id: ++blockSeq,
    rawText: "",
    lines: [],
    anchor: "free",
    bgMode: "none",
    x: DEFAULT_TEXT_X,
    y: DEFAULT_TEXT_Y + NEW_BLOCK_STEP * state.blocks.length,
  };
  if (state.image) {
    block.y = Math.min(block.y, Math.max(0, state.image.height - 40));
  }
  state.blocks.push(block);
  listEl.appendChild(buildCard(block));
  refreshTitles();
  notify();
  return block;
}

function removeBlock(id: number, card: HTMLElement): void {
  statusById.delete(id);
  if (activeArea?.blockId === id) activeArea = null;
  state.blocks = state.blocks.filter((b) => b.id !== id);
  card.remove();
  refreshTitles();
  commit();
  notify();
}

function buildCard(block: ChatBlock): HTMLElement {
  const card = document.createElement("div");
  card.className = "chat-block";

  // ── header: title · position select · remove ──
  const head = document.createElement("div");
  head.className = "row chat-block-head";

  const title = document.createElement("span");
  title.className = "chat-block-title";

  const select = document.createElement("select");
  select.className = "select";
  select.title = "Posisi chatlog";
  for (const opt of anchorOptions()) {
    const o = document.createElement("option");
    o.value = opt.value;
    o.textContent = opt.label;
    select.appendChild(o);
  }
  select.value = block.anchor;
  select.addEventListener("change", () => {
    const next = select.value as Anchor;
    if (next === "free") {
      // Start dragging from where the block currently sits, no jump.
      const b = getBlockBounds(block.id);
      block.x = b ? b.x : DEFAULT_TEXT_X;
      block.y = b ? b.y : DEFAULT_TEXT_Y;
    }
    block.anchor = next;
    commit();
    notify();
  });

  const bgSelect = document.createElement("select");
  bgSelect.className = "select select-compact";
  bgSelect.title = "Background di belakang teks";
  for (const opt of bgOptions()) {
    const o = document.createElement("option");
    o.value = opt.value;
    o.textContent = opt.label;
    bgSelect.appendChild(o);
  }
  bgSelect.value = block.bgMode;
  bgSelect.addEventListener("change", () => {
    block.bgMode = bgSelect.value as BgMode;
    commit();
    notify();
  });

  const remove = document.createElement("button");
  remove.className = "btn btn-ghost btn-small chat-block-remove";
  remove.textContent = "✕";
  remove.title = "Hapus chatlog ini";
  remove.addEventListener("click", () => removeBlock(block.id, card));

  head.append(title, remove);

  const controls = document.createElement("div");
  controls.className = "chat-controls";
  controls.append(select, bgSelect);

  // ── textarea ──
  const textarea = document.createElement("textarea");
  textarea.className = "input textarea mono chat-block-text";
  textarea.rows = 6;
  textarea.spellcheck = false;
  textarea.placeholder =
    "[12:34:56] Budi_Santoso says: contoh chat\n[12:34:57] * Budi menoleh ke belakang.\n\nTimestamp [00:00:00] otomatis dihapus.";

  const status = document.createElement("span");
  status.className = "hint";
  status.textContent = "0 baris";
  statusById.set(block.id, status);

  textarea.addEventListener("focus", () => {
    activeArea = { blockId: block.id, textarea };
  });

  let timer: number | undefined;
  let requestSeq = 0;
  textarea.addEventListener("input", () => {
    // Sync rawText SYNCHRONOUSLY so any snapshot captured during the debounce
    // window (e.g. a filter/drag commit from another module) records the real
    // text — otherwise undo can restore a phantom "empty chatlog" step.
    block.rawText = textarea.value;
    window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      const seq = ++requestSeq;
      try {
        const lines = await parseChatlog(textarea.value, state.preset);
        if (seq !== requestSeq) return; // a newer request finished later
        block.lines = lines;
        status.textContent = `${lines.length} ${t("lines")}`;
        commit();
        notify();
      } catch (err) {
        status.textContent = "gagal memproses — lihat console";
        console.error("[screenies-editor] parse_chatlog failed:", err);
      }
    }, DEBOUNCE_MS);
  });

  card.append(head, controls, textarea, status);
  return card;
}

/** Re-run the Rust parser on every block — used when the preset changes. */
export async function reparseAllBlocks(): Promise<void> {
  for (const block of state.blocks) {
    if (block.rawText.trim().length === 0) {
      block.lines = [];
      continue;
    }
    try {
      block.lines = await parseChatlog(block.rawText, state.preset);
      const status = statusById.get(block.id);
      if (status) status.textContent = `${block.lines.length} baris`;
    } catch (err) {
      console.error("[screenies-editor] reparse failed:", err);
    }
  }
  notify();
}

/** Renumber "Chatlog N" titles and disable ✕ when only one block remains. */
function refreshTitles(): void {
  const cards = listEl.querySelectorAll<HTMLElement>(".chat-block");
  cards.forEach((card, i) => {
    const title = card.querySelector<HTMLElement>(".chat-block-title");
    if (title) title.textContent = `Chatlog ${i + 1}`;
  });
  const removes = listEl.querySelectorAll<HTMLButtonElement>(".chat-block-remove");
  removes.forEach((btn) => {
    btn.disabled = state.blocks.length <= 1;
  });
}

function mustGet<T extends HTMLElement>(id: string): T {
  const el = document.getElementById(id);
  if (!el) throw new Error(`Missing element #${id}`);
  return el as T;
}
