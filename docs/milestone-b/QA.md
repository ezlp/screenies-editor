# Milestone B — Visual QA checklist

Run this checklist with `cd native && cargo run` before opening the PR.
Capture the referenced screenshots at the default **Midnight** theme and a
desktop-sized window (at least 1180×760).

## Editor tool rail

- [ ] Open the SSRP Editor and confirm a 56px rail appears to the left of the
  controls panel.
- [ ] Select Photo, Crop, Chatlog, Text, and FX from the rail. Each selection
  must replace only the controls panel; the preview must remain visible.
- [ ] Press `1` through `5` while no text field is focused. Confirm the same
  five panels are selected in order.
- [ ] Focus the Chatlog text field and enter `1` through `5`. Confirm they are
  inserted as text rather than changing the active tool.

## Actions and persistence

- [ ] Confirm Export appears only in the fixed bottom action bar.
- [ ] Make a change and confirm Undo/Redo appear only in the floating preview
  toolbar and work with both the buttons and keyboard shortcuts.
- [ ] Choose a non-Photo tool, close the app, and reopen it. Confirm the
  active tool is restored.

## Regression checks

- [ ] Load a photo, crop it, add a chatlog, adjust text, and add a filter.
  Confirm the preview still updates and Export produces a PNG.
- [ ] Open a chatlog folder containing a non-UTF-8 `.log` file. Confirm it
  previews without a UTF-8 decoding error.

## Screenshots for the PR

1. **Tool rail:** Editor with Chatlog selected and the rail visible.
2. **Photo/Crop:** rail plus the Crop panel and preview toolbar.
3. **Actions:** visible bottom Export action bar and floating Undo/Redo toolbar.

