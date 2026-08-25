use std::process::Command;

/// Local Desktop RAG (Windows Search / Everything IPC hook)
/// Wraps Windows Search / Everything API to instantly grab local context.
pub fn search_local_desktop(query: &str) -> Result<Vec<String>, String> {
    println!("🔍 [Windows RAG] Hooking Desktop Search for: {}", query);
    
    // Fallback to powershell indexing search if Everything CLI (es.exe) is missing
    let ps_script = format!(
        "Get-ChildItem -Path $env:USERPROFILE -Recurse -Filter '*{}*' -ErrorAction SilentlyContinue | Select-Object -First 10 -ExpandProperty FullName", 
        query
    );

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to query Desktop RAG: {}", e))?;

    let paths = String::from_utf8_lossy(&output.stdout);
    let path_lines: Vec<String> = paths
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    println!("✅ [Windows RAG] Retrieved {} local files for AI context.", path_lines.len());
    Ok(path_lines)
}
