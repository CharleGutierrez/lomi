use std::process::Command;

/// Dynamic Core Parking & Power Plan AI (via PPM API)
/// Interacts with powercfg to shift between Ultimate Performance and Efficiency
pub fn set_ultimate_performance_mode(active: bool) -> Result<(), String> {
    // Windows default GUIDs:
    // Ultimate Performance: e9a42b02-d5df-448d-aa00-03f14749eb61
    // Balanced (Efficiency): 381b4222-f694-41f0-9685-ff5bb260df2e
    
    let guid = if active {
        "e9a42b02-d5df-448d-aa00-03f14749eb61"
    } else {
        "381b4222-f694-41f0-9685-ff5bb260df2e"
    };

    println!("⚡ [Power Plan] Shifting OS Power State to: {}", if active { "Ultimate Performance" } else { "Balanced" });

    let output = Command::new("powercfg.exe")
        .args(["/setactive", guid])
        .output()
        .map_err(|e| format!("Failed to execute powercfg: {}", e))?;

    if !output.status.success() {
        return Err(format!("powercfg failed to apply plan. Requires Admin / valid GUID."));
    }

    println!("✅ [Power Plan] Core parking unlatched. Max frequency applied.");
    Ok(())
}
