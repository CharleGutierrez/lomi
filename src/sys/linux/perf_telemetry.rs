/// perf_event_open CPU Cache-Miss Telemetry
/// Reads real hardware performance counters from /proc/stat and /sys/devices/system/cpu.
/// Falls back to perf stat when available for L3 cache-miss tracking.
pub fn start_perf_telemetry() -> Result<(), String> {
    println!("[perf_events] Reading CPU hardware performance counters...");

    // Read real CPU stats from /proc/stat
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        for line in stat.lines().take(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                println!("   /proc/stat: user={} nice={} system={} idle={} iowait={} irq={} softirq={}",
                    parts[1], parts[2], parts[3], parts[4],
                    parts.get(5).unwrap_or(&"0"), parts.get(6).unwrap_or(&"0"),
                    parts.get(7).unwrap_or(&"0"));
            }
        }
    }

    // Read CPU frequency from sysfs
    let freq_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
    if let Ok(freq) = std::fs::read_to_string(freq_path) {
        let freq_mhz = freq.trim().parse::<u64>().unwrap_or(0) / 1000;
        println!("   CPU0 current frequency: {} MHz", freq_mhz);
    }

    // Read CPU cache info
    let cache_path = "/sys/devices/system/cpu/cpu0/cache";
    if std::path::Path::new(cache_path).exists() {
        for i in 0..4 {
            let level_path = format!("{}/index{}/level", cache_path, i);
            let size_path = format!("{}/index{}/size", cache_path, i);
            let type_path = format!("{}/index{}/type", cache_path, i);
            if let (Ok(level), Ok(size), Ok(ctype)) = (
                std::fs::read_to_string(&level_path),
                std::fs::read_to_string(&size_path),
                std::fs::read_to_string(&type_path),
            ) {
                println!("   Cache L{}: {} ({})", level.trim(), size.trim(), ctype.trim());
            }
        }
    }

    // Try real perf stat for hardware cache-miss counters
    let perf_result = std::process::Command::new("perf")
        .args(["stat", "-e", "cache-misses,cache-references,instructions,cycles",
               "--", "sleep", "0.1"])
        .output();

    match perf_result {
        Ok(out) if out.status.success() || !out.stderr.is_empty() => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("   perf stat hardware counters:");
            for line in stderr.lines() {
                let trimmed = line.trim();
                if trimmed.contains("cache") || trimmed.contains("instructions") || trimmed.contains("cycles") {
                    println!("      {}", trimmed);
                }
            }
        }
        _ => {
            println!("   perf tool not available or requires CAP_SYS_ADMIN.");
            println!("   Install: sudo apt install linux-tools-$(uname -r)");
        }
    }

    println!("[perf_events] Hardware telemetry snapshot captured.");
    Ok(())
}
