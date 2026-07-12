// build.rs — compile the CXX-Qt bridge and register the QML module.
//
// Phase-0 scaffold. Assumes Qt6 is discoverable (qmake on PATH, or the
// QMAKE / CMAKE env the cxx-qt-build docs describe). The QML files under
// qml/ are registered into the `com.screenies.app` module that Main.qml
// and the pages import.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new()
        .qml_module(QmlModule {
            uri: "com.screenies.app",
            rust_files: &["src/bridge/app.rs"],
            qml_files: &[
                "qml/Main.qml",
                "qml/pages/EditorPage.qml",
                "qml/pages/ChatlogParserPage.qml",
                "qml/pages/GalleryPage.qml",
            ],
            ..Default::default()
        })
        .build();
}
