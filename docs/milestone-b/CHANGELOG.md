Milestone B — Change Log

2026-07-17
- Added Tool enum and App.active_tool (persisted in Settings)
- Added EditorState.active_tool and default initialization
- Implemented left tool rail UI (icons for Photo, Crop, Chatlog, Text, Fx)
- Refactored editor.controls into tool_photo/tool_crop/tool_chatlog/tool_text/tool_fx
- Fixed chatlog browser to handle non-UTF8 files (lossy read)
- Moved Export UI into a fixed action bar at the bottom of the controls panel
- Added floating Undo/Redo toolbar overlaying preview (top-right)

Commits (branch feature/milestone-b-tool-rail):
- feat(milestone-b): add Tool enum, active_tool persistence and left tool rail UI
- refactor(milestone-b): split editor.controls into tool-specific panels
- fix(chatlog): handle non-UTF8 log files with lossy read

Files changed:
- native/src/main.rs
- native/src/editor.rs
- native/src/chatlog_browser.rs

