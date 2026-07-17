Milestone B — TODO (tracking)

- [x] Create branch feature/milestone-b-tool-rail
- [x] Add Tool enum + App.active_tool (persist in Settings)
- [x] Add EditorState.active_tool and mirror App <-> Editor
- [x] Implement left tool rail UI (56px) and set active_tool on click
- [x] Refactor editor.controls into tool_* panels (photo/crop/chatlog/text/fx)
- [x] Fix chatlog non-UTF8 read bug

- [x] Move Export to action bar (or keep in Photo panel and add dedicated action bar)
- [x] Move Undo/Redo to floating preview toolbar
- [x] Add keyboard shortcuts (1..5) for tool switching
- [x] Add unit smoke tests for each tool panel
- [ ] Complete visual QA, capture screenshots, and publish the PR description (see QA.md and PR-DESCRIPTION.md)
- [ ] Merge to main and tag v3.0.0-beta when ready

Notes:
- Keep commits small and focused per task.
- Update this file as tasks progress.
