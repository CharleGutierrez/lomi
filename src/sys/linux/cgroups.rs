use std::fs;
use std::path::Path;
use std::io::Write;

/// Dynamic cgroups v2 Omni-Tuning
/// Directly interacts with the Linux kernel's cgroup v2 pseudo-filesystem.
pub fn throttle_background_tasks(limit_pct: u32) -> Result<(), String> {
    let cgroup_path = "/sys/fs/cgroup/lomi";
    
    // Create the cgroup if it doesn't exist
    if !Path::new(cgroup_path).exists() {
        fs::create_dir_all(cgroup_path).map_err(|e| format!("Failed to create cgroup: {}", e))?;
    }

    // cgroups v2 cpu.max format: "$MAX $PERIOD" (default period is 100000)
    // To limit to limit_pct% of a single core: MAX = (limit_pct * 100000) / 100
    let max_quota = (limit_pct * 100000) / 100;
    let cpu_max_val = format!("{} 100000", max_quota);

    let cpu_max_file = format!("{}/cpu.max", cgroup_path);
    
    let mut file = fs::File::create(&cpu_max_file)
        .map_err(|e| format!("Failed to open {}: {}", cpu_max_file, e))?;
        
    file.write_all(cpu_max_val.as_bytes())
        .map_err(|e| format!("Failed to write to cpu.max: {}", e))?;

    println!("✅ [cgroups v2] Background tasks throttled to {}% CPU", limit_pct);
    Ok(())
}
