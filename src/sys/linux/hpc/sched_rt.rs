use std::process::Command;

/// SCHED_FIFO Hard Real-Time (PREEMPT_RT)
/// Elevates Lomi's Tuner engine to run with Hard Real-Time priority (1000Hz jitter-free).
/// Validates the result and reports actual success or failure.
pub fn elevate_to_rtos() -> Result<(), String> {
    println!("⏱️ [PREEMPT_RT] Elevating Omni-Tuner to SCHED_FIFO Hard Real-Time priority...");

    let pid = std::process::id();

    // Check if chrt is available
    let chrt_available = Command::new("which")
        .arg("chrt")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !chrt_available {
        println!("   ❌ `chrt` command not found. Install: sudo apt install util-linux");
        return Err("chrt not available".into());
    }

    // Attempt SCHED_FIFO with priority 99
    let result = Command::new("chrt")
        .args(["-f", "-p", "99", &pid.to_string()])
        .output();

    match result {
        Ok(out) if out.status.success() => {
            println!("   ✅ [PREEMPT_RT] PID {} elevated to SCHED_FIFO priority 99.", pid);

            // Verify by reading back the current scheduling policy
            if let Ok(verify) = Command::new("chrt").args(["-p", &pid.to_string()]).output() {
                let stdout = String::from_utf8_lossy(&verify.stdout);
                for line in stdout.lines() {
                    println!("      {}", line.trim());
                }
            }
            Ok(())
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("   ⚠️  SCHED_FIFO elevation failed: {}", stderr.trim());
            println!("      Requires root/CAP_SYS_NICE. Run: sudo lomi experimental --feature hpc");

            // Fallback: try a lower priority that might succeed without root
            println!("   Attempting fallback: renice to -10...");
            let renice = Command::new("renice")
                .args(["-n", "-10", "-p", &pid.to_string()])
                .output();
            match renice {
                Ok(r) if r.status.success() => {
                    println!("   ✅ Fallback: Process priority elevated via renice.");
                    Ok(())
                }
                _ => {
                    println!("   ❌ Both SCHED_FIFO and renice failed. Running at default priority.");
                    Err(format!("RT elevation failed: {}", stderr.trim()))
                }
            }
        }
        Err(e) => {
            println!("   ❌ chrt command execution error: {}", e);
            Err(format!("chrt error: {}", e))
        }
    }
}
