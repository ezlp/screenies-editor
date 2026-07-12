/**
 * theme.ts — dark / light mode.
 *
 * The choice persists in settings.json via settings.ts (v0.11.0 —
 * replaced the old localStorage stopgap). Themes remain pure CSS-variable
 * override blocks, so custom themes later = another block + entry.
 */

type ThemeName = "dark" | "light";

let button: HTMLButtonElement | null = null;
let onToggled: (() => void) | null = null;

export function initTheme(onToggle?: () => void): void {
  button = document.getElementById("theme-toggle") as HTMLButtonElement | null;
  if (!button) throw new Error("Missing #theme-toggle");
  onToggled = onToggle ?? null;

  syncButton(); // settings may already have applied a theme before init
  button.addEventListener("click", () => {
    setTheme(currentTheme() === "dark" ? "light" : "dark");
    onToggled?.();
  });
}

export function currentTheme(): ThemeName {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

export function setTheme(theme: ThemeName): void {
  document.documentElement.dataset.theme = theme;
  syncButton();
}

function syncButton(): void {
  if (!button) return;
  const theme = currentTheme();
  button.textContent = theme === "dark" ? "☀" : "🌙";
  button.title = theme === "dark" ? "Ganti ke mode terang" : "Ganti ke mode gelap";
}
