# Milestone B: contextual editor tool rail

## Summary

Milestone B reorganizes the native editor controls into a persistent tool rail
and five focused panels: Photo, Crop, Chatlog, Text, and FX. The preview stays
in place while the controls change, reducing the amount of scrolling needed to
edit a screenshot.

It also moves Export into a persistent action bar, places Undo/Redo over the
preview, adds `1`–`5` tool shortcuts, and persists the active tool.

## Changes

- Add persisted `Tool` state and a 56px left tool rail.
- Split the editor controls into tool-specific panel methods.
- Add keyboard shortcuts `1`–`5` (disabled while editing text).
- Keep Export in the bottom action bar and Undo/Redo in the preview toolbar.
- Handle non-UTF-8 chatlogs with lossy decoding.
- Add headless egui smoke coverage for every tool panel.

## Screenshots

Add the three screenshots listed in [QA.md](QA.md) here before opening the PR.

## Verification

- `cargo test --manifest-path native/Cargo.toml`
- Complete [QA.md](QA.md), including restart/persistence and non-UTF-8 log checks.

