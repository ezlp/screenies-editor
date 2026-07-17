Milestone B — Editor Tool Rail

Summary

This folder captures Milestone B work: introduce a persistent editor tool rail, split editor controls into per-tool panels, relocate Export and Undo/Redo, and add keyboard shortcuts. It includes current status, tasks, and how to continue the work.

Branch: feature/milestone-b-tool-rail

Current status (2026-07-17T19:03:00+07:00):
- Tool enum added; App.active_tool persisted in Settings
- EditorState.active_tool added and synced
- Left 56px tool rail implemented (icons set active tool)
- editor.controls refactored into tool_photo/tool_crop/tool_chatlog/tool_text/tool_fx
- Chatlog parser bug (non-UTF8) fixed

Next recommended steps:
1. Move Export to action bar (if not already) and move Undo/Redo to floating toolbar
2. Add keyboard shortcuts (1..5) to switch tools
3. Add unit/manual smoke tests and update CI if needed
4. Visual QA & screenshots for PR

How to run:
- Build checks run by CI: .github/workflows/build.yml
- Local quick check: cd native && cargo check

Contacts:
- Branch PR: https://github.com/ezlp/screenies-editor/pull/new/feature/milestone-b-tool-rail
