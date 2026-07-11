/**
 * chatlog.ts — left panel: image upload + the chatlog block list.
 *
 * Each block is a card: its own textarea, a position dropdown
 * (Bebas / 4 corners) and a remove button. "+ Tambah Chatlog" adds more.
 *
 * Flow per block: paste → debounce → Rust `parse_chatlog` (strips
 * timestamps, bolds system tags) → block.lines → canvas redraws.
 */

import { parseChatlog } from "./tauri-bridge";
import { DEFAULT_TEXT_X, DEFAULT_TEXT_Y, state, notify } from "./state";
import type { Anchor, ChatBlock } from "./state";
import { fitImage, getBlockBounds } from "./canvas";
import { onImageLoaded } from "./crop";
import { autoTextSize } from "./textstyle";

const DEBOUNCE_MS = 150;
const ACCEPTED = ["image/png", "image/jpeg", "image/webp", "image/bmp"];
/** Vertical offset between newly added free blocks so they don't stack. */
const NEW_BLOCK_STEP = 70;

const ANCHOR_OPTIONS: Array<{ value: Anchor; label: string }> = [
  { value: "free", label: "Bebas (seret)" },
  { value: "kiri-atas", label: "Kiri Atas" },
  { value: "kiri-bawah", label: "Kiri Bawah" },
];

let blockSeq = 0;
let listEl: HTMLElement;
/** Per-block "N baris" hints, so preset changes can refresh them. */
const statusById = new Map<number, HTMLElement>();

/* ── image upload (unchanged behavior) ── */

export function initUpload(): void {
  const dropzone = mustGet<HTMLElement>("dropzone");
  const fileInput = mustGet<HTMLInputElement>("file-input");
  const meta = mustGet<HTMLElement>("image-meta");
  const metaName = mustGet<HTMLElement>("meta-name");
  const metaRes = mustGet<HTMLElement>("meta-res");
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

  function loadImageFile(file: File): void {
    if (!ACCEPTED.includes(file.type)) {
      console.warn(`[screenies-editor] Ignored non-image file: ${file.name} (${file.type})`);
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const img = new Image();
      img.onload = () => {
        state.image = img;
        state.imageName = file.name;
        metaName.textContent = file.name;
        metaRes.textContent = `${img.width}×${img.height}`;
        meta.hidden = false;
        onImageLoaded();         // new photo → crop back to full frame
        autoTextSize(img.width); // low-res photo → smaller text, high-res → bigger
        fitImage();
      };
      img.src = String(reader.result);
    };
    reader.readAsDataURL(file);
  }
}

/* ── chatlog blocks ── */

export function initChatlog(): void {
  listEl = mustGet<HTMLElement>("chatlog-list");
  const addBtn = mustGet<HTMLButtonElement>("btn-add-chatlog");
  addBtn.addEventListener("click", () => addBlock());
  if (state.blocks.length === 0) addBlock(); // start with one block
}

function addBlock(): void {
  const block: ChatBlock = {
    id: ++blockSeq,
    rawText: "",
    lines: [],
    anchor: "free",
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
}

function removeBlock(id: number, card: HTMLElement): void {
  statusById.delete(id);
  state.blocks = state.blocks.filter((b) => b.id !== id);
  card.remove();
  refreshTitles();
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
  for (const opt of ANCHOR_OPTIONS) {
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
    notify();
  });

  const remove = document.createElement("button");
  remove.className = "btn btn-ghost btn-small chat-block-remove";
  remove.textContent = "✕";
  remove.title = "Hapus chatlog ini";
  remove.addEventListener("click", () => removeBlock(block.id, card));

  head.append(title, select, remove);

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

  let timer: number | undefined;
  let requestSeq = 0;
  textarea.addEventListener("input", () => {
    window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      const seq = ++requestSeq;
      block.rawText = textarea.value;
      try {
        const lines = await parseChatlog(textarea.value, state.preset);
        if (seq !== requestSeq) return; // a newer request finished later
        block.lines = lines;
        status.textContent = `${lines.length} baris`;
        notify();
      } catch (err) {
        status.textContent = "gagal memproses — lihat console";
        console.error("[screenies-editor] parse_chatlog failed:", err);
      }
    }, DEBOUNCE_MS);
  });

  card.append(head, textarea, status);
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
