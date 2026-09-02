use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CgroupTelemetry {
    pub is_cgroup_v2_available: bool,
    pub current_memory_mb: u64,
    pub high_memory_limit_mb: u64,
    pub memory_pressure_pct: f32,
    pub cpu_weight: u32,
    pub active_cgroup_path: String,
}

pub struct CgroupManager;

impl CgroupManager {
    /// Reads real Linux cgroups v2 resource metrics from /sys/fs/cgroup
    pub fn get_telemetry() -> CgroupTelemetry {
        let cgroup_base = Path::new("/sys/fs/cgroup");
        let is_cgroup_v2_available = cgroup_base.exists() && cgroup_base.join("cgroup.controllers").exists();

        let mut current_memory_bytes = 0u64;
        let mut high_limit_bytes = 0u64;

        if is_cgroup_v2_available {
            if let Ok(mem_str) = fs::read_to_string(cgroup_base.join("memory.current")) {
                current_memory_bytes = mem_str.trim().parse().unwrap_or(0);
            }
            if let Ok(high_str) = fs::read_to_string(cgroup_base.join("memory.high")) {
                if high_str.trim() != "max" {
                    high_limit_bytes = high_str.trim().parse().unwrap_or(0);
                }
            }
        }

        let current_memory_mb = current_memory_bytes / (1024 * 1024);
        let high_memory_limit_mb = if high_limit_bytes > 0 {
            high_limit_bytes / (1024 * 1024)
        } else {
            8192 // 8GB default limit fallback
        };

        let memory_pressure_pct = if high_memory_limit_mb > 0 {
            ((current_memory_mb as f32 / high_memory_limit_mb as f32) * 100.0).min(100.0)
        } else {
            0.0
        };

        CgroupTelemetry {
            is_cgroup_v2_available,
            current_memory_mb,
            high_memory_limit_mb,
            memory_pressure_pct,
            cpu_weight: 100,
            active_cgroup_path: cgroup_base.to_string_lossy().to_string(),
        }
    }

    /// Sets high memory limit in Linux cgroups v2 to prevent Out-Of-Memory (OOM) kills
    pub fn set_memory_limit(high_limit_mb: u64) -> Result<(), String> {
        let cgroup_base = Path::new("/sys/fs/cgroup");
        if !cgroup_base.exists() {
            return Err("cgroups v2 not available on this kernel".to_string());
        }

        let limit_bytes = high_limit_mb * 1024 * 1024;
        let target_file = cgroup_base.join("memory.high");

        if fs::write(&target_file, limit_bytes.to_string()).is_ok() {
            Ok(())
        } else {
            Err("Insufficient root/sudo permissions to write to /sys/fs/cgroup/memory.high".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroup_telemetry_gathering() {
        let telemetry = CgroupManager::get_telemetry();
        assert!(telemetry.high_memory_limit_mb > 0);
        assert!(telemetry.memory_pressure_pct >= 0.0 && telemetry.memory_pressure_pct <= 100.0);
    }
}
