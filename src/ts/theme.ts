/**
 * theme.ts — dark / light mode.
 *
 * Themes are pure CSS-variable override blocks in theme.css, so adding
 * custom themes later = adding another block + an entry here.
 * The choice persists in the webview's localStorage; M4's settings file
 * (config.rs) will take over as the source of truth.
 */

type ThemeName = "dark" | "light";
const STORAGE_KEY = "screenies-theme";

let button: HTMLButtonElement | null = null;

export function initTheme(): void {
  button = document.getElementById("theme-toggle") as HTMLButtonElement | null;
  if (!button) throw new Error("Missing #theme-toggle");

  apply(loadSaved() ?? "dark");
  button.addEventListener("click", () => {
    apply(current() === "dark" ? "light" : "dark");
  });
}

function current(): ThemeName {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

function apply(theme: ThemeName): void {
  document.documentElement.dataset.theme = theme;
  if (button) {
    button.textContent = theme === "dark" ? "☀" : "🌙";
    button.title = theme === "dark" ? "Ganti ke mode terang" : "Ganti ke mode gelap";
  }
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    /* private mode / storage blocked — theme just won't persist */
  }
}

function loadSaved(): ThemeName | null {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "light" || v === "dark" ? v : null;
  } catch {
    return null;
  }
}
