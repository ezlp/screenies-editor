/**
 * filters.ts — live image filters (Milestone 3b).
 *
 * Five sliders, instant preview exactly like ssrp-editor: the canvas
 * applies the same CSS filter functions the export pipeline (M3c) will
 * reproduce in Rust, so preview == saved PNG. Filters touch ONLY the
 * photo — chat text stays crisp on top.
 */

import { DEFAULT_FILTERS, state, notify } from "./state";
import type { Filters } from "./state";

interface FilterDef {
  key: keyof Filters;
  max: number;
}

const DEFS: FilterDef[] = [
  { key: "brightness", max: 200 },
  { key: "grayscale", max: 100 },
  { key: "sepia", max: 100 },
  { key: "saturate", max: 200 },
  { key: "contrast", max: 200 },
];

export function initFilters(): void {
  for (const def of DEFS) {
    const row = document.querySelector<HTMLElement>(`.filter-row[data-filter="${def.key}"]`);
    if (!row) throw new Error(`Missing filter row for ${def.key}`);

    const slider = row.querySelector<HTMLInputElement>("input[type=range]");
    const value = row.querySelector<HTMLElement>(".filter-val");
    const reset = row.querySelector<HTMLButtonElement>(".filter-reset");
    if (!slider || !value || !reset) throw new Error(`Incomplete filter row: ${def.key}`);

    slider.min = "0";
    slider.max = String(def.max);
    slider.disabled = false;

    const sync = () => {
      const v = state.filters[def.key];
      slider.value = String(v);
      value.textContent = `${v}%`;
      const isDefault = v === DEFAULT_FILTERS[def.key];
      reset.disabled = isDefault;
      row.classList.toggle("filter-active", !isDefault);
    };

    slider.addEventListener("input", () => {
      state.filters[def.key] = Number(slider.value);
      sync();
      notify();
    });

    reset.addEventListener("click", () => {
      state.filters[def.key] = DEFAULT_FILTERS[def.key];
      sync();
      notify();
    });

    sync();
  }
}
