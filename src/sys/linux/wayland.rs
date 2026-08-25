use std::process::Command;

/// Wayland Layer-Shell Global Spotlight
/// Hooks into Wofi/Rofi for a global OS-level drop-down AI prompt.
pub fn show_wayland_spotlight() -> Result<(), String> {
    println!("✨ [Wayland] Triggering Global Spotlight Overlay...");
    
    // Try wofi (Wayland) first, fallback to rofi (X11)
    let output = Command::new("wofi")
        .args(["--show", "dmenu", "--prompt", "Lomi AI: Ask anything..."])
        .output()
        .or_else(|_| {
            Command::new("rofi")
                .args(["-dmenu", "-p", "Lomi AI: Ask anything..."])
                .output()
        })
        .map_err(|e| format!("Neither wofi nor rofi installed: {}", e))?;

    let prompt = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !prompt.is_empty() {
        println!("✅ [Wayland] User prompt captured: '{}'", prompt);
    }
    
    Ok(())
}
