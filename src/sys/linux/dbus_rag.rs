use std::process::Command;

/// System-Wide D-Bus & journalctl RAG
/// Tails systemd journalctl to provide context for AI prompt injection.
pub fn query_system_logs(service_name: &str) -> Result<Vec<String>, String> {
    println!("🔍 [D-Bus/journalctl] Extracting local system context for '{}'...", service_name);
    
    let output = Command::new("journalctl")
        .args(["-u", service_name, "-n", "20", "--no-pager", "--output=cat"])
        .output()
        .map_err(|e| format!("Failed to execute journalctl: {}", e))?;

    if !output.status.success() {
        return Err(format!("journalctl returned non-zero status"));
    }

    let logs = String::from_utf8_lossy(&output.stdout);
    let log_lines: Vec<String> = logs
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();
        
    println!("✅ [D-Bus/journalctl] Captured {} lines of system RAG context", log_lines.len());
    Ok(log_lines)
}
