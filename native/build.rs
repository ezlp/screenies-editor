// build.rs — embed the app icon into the Windows .exe (shows in Explorer /
// taskbar). No-op on non-Windows targets.

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        // Best-effort: don't fail the whole build if the resource compiler
        // (rc.exe / llvm-rc) isn't available.
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon embed skipped: {e}");
        }
    }
}
