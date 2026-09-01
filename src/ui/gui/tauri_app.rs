/// Tauri Desktop GUI Integration
/// NOTE: Tauri requires a full webview runtime and the `tauri` crate which is not
/// included in this project's dependencies. This module redirects to the real
/// Slint native GUI instead.
pub fn launch_tauri_app() -> Result<(), String> {
    println!("⚠️  Tauri webview is not configured in this build.");
    println!("   Tauri requires the `tauri` crate and a webview runtime bundle.");
    println!("   Redirecting to the native Slint GUI instead...\n");

    // Delegate to the real, working native GUI
    crate::ui::gui::slint_app::launch_slint_app()
}
