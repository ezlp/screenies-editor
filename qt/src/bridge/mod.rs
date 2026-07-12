// bridge/ — the Rust side exposed to QML via CXX-Qt.
//
// app.rs holds the CXX-Qt QObject(s). The new-feature *logic* lives in
// screenies-core (shell-independent, unit-tested there):
//   - chatlog search:  screenies_core::chatlog_library
//   - gallery listing:  screenies_core::gallery
// Phases 3–4 add thin QObject wrappers here that call into those modules.
// See docs/2.0-MIGRATION.md.

pub mod app;
