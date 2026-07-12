// app.rs — the root CXX-Qt QObject. Proves the Rust↔QML bridge is alive
// (QML shows the version) and will grow the editor/parser/gallery commands.
//
// ⚠️ Phase-0 scaffold: CXX-Qt 0.7 syntax, not yet build-verified locally.

#[cxx_qt::bridge]
pub mod qobject {
    extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    unsafe extern "RustQt" {
        /// Exposed to QML as `AppBackend` in the `com.screenies.app` module.
        #[qobject]
        #[qml_element]
        type AppBackend = super::AppBackendRust;

        /// App version, straight from Cargo.toml — the "is the bridge alive?"
        /// probe, mirroring the Tauri `app_version` command.
        #[qinvokable]
        fn app_version(self: &AppBackend) -> QString;
    }
}

use cxx_qt_lib::QString;

/// Backing Rust struct for the AppBackend QObject.
#[derive(Default)]
pub struct AppBackendRust;

impl qobject::AppBackend {
    pub fn app_version(&self) -> QString {
        QString::from(env!("CARGO_PKG_VERSION"))
    }
}
