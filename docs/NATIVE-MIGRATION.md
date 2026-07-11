# Native (egui) Migration Plan — "ScreeniesEditor 2.0"

Why: three problems, one rewrite — (1) old laptops without WebView2,
(2) preview/export drift (two renderers today), (3) the browser version
(egui compiles to WASM). Strategy: **strangler fig** — the native shell
grows next to the Tauri app inside one workspace, sharing one core, and
only replaces it when the parity checklist is green. Nothing is bet on it.

## Structure (Phase 0 — DONE)

```
Cargo workspace
├─ core/       screenies-core: parser, presets, render pipeline, fonts,
│              clipboard. No Tauri, no windowing. THE product.
├─ src-tauri/  Tauri shell: commands, dialogs, settings paths. Ships today.
└─ native/     🧪 egui shell: experimental, never in Releases.
```

Rule: anything that computes → core. Anything that touches the OS shell
(dialogs, config dirs, IPC, windowing) → the shells.

## Phases & gates

| Phase | Deliverable | Gate to proceed |
|---|---|---|
| **0 ✅** | Workspace + core extraction; Tauri app unchanged; CI tests whole workspace | Tauri CI green, installers identical |
| **1 ✅** | native skeleton: open photo, type chatlog, preview **via compose::render** (parity by identity) | `cargo run -p screenies-native` shows correct colored text |
| **2** | **Layout engine into core** — port `canvas.ts buildRenderBlocks` to Rust (wrap, anchors, Luar, BG strips). Tauri app keeps its TS layout until Phase 5 | Rust layout output == TS layout on the 596-line test log (token positions within 0.5px) |
| **3** | native interactions: zoom/pan, drag blocks, crop editor, filter sliders, stickers. Perf: cache the photo layer (crop+resize+filters), re-run only the text pass per keystroke | Feels as responsive as Tauri on a weak laptop |
| **4** | native shell services: settings via `dirs`, export/copy (already core), multi-block UI, presets `.toml`, theming pass | Full feature checklist vs README table |
| **5** | Decision point: (a) ship native as the old-laptop build alongside Tauri, or (b) 2.0 switch. Optional: Tauri frontend adopts core layout via a WASM build of core — killing drift in the CURRENT app too | Community feedback |
| **6** | WASM target: `trunk build` of native → browser version | Runs the demo log in Firefox/Chrome |

## Phase-1 skeleton limits (deliberate)

Single block, one token per colored span, **no word-wrap**, fixed font/
size, no crop/filters UI. The point is the plumbing: core parse → rows →
core compose → texture. Everything missing is Phase 2/3 work.

## Risks & mitigations

- **Text input feel** (egui multiline < browser textarea): acceptable for
  editor use; re-evaluate at Phase 3 with real typing.
- **Aesthetics**: egui won't match our CSS; budget a theming pass (P4) and
  accept "clean tool" over "pretty web app".
- **Old-GPU rendering**: eframe's glow backend targets GL 2-era hardware;
  if reports come in, `wgpu`→software fallback is the escape hatch.
- **Two layout engines during P2–P4**: contained — the parity test in P2's
  gate is the only bridge, and P5(b) deletes one side.

## Working on it

```bash
cargo run -p screenies-native     # run the experiment
cargo test --workspace            # everything still green
```
CI compiles native on every push (build.yml) so it can't rot silently.
