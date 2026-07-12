// screenies-qt — 2.0 shell entry point.
//
// Boots a Qt6 application and loads the QML landing menu. The Rust backend
// (bridge/) exposes screenies-core to QML as QObjects.
//
// Phase-0 scaffold: navigates a stub UI. The editor/parser/gallery pages are
// filled in during phases 2–4 (see docs/2.0-MIGRATION.md).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridge;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

fn main() {
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        // The QML module URI + file registered in build.rs.
        engine.load(&QUrl::from("qrc:/qt/qml/com/screenies/app/qml/Main.qml"));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
