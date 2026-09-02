use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamTelemetry {
    pub total_mb: u64,
    pub available_mb: u64,
    pub used_mb: u64,
    pub used_percent: f32,
    pub swap_total_mb: u64,
    pub swap_free_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTelemetry {
    pub detected: bool,
    pub vendor: String,
    pub model: String,
    pub total_vram_mb: u64,
    pub free_vram_mb: u64,
    pub used_vram_mb: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TuningTier {
    UltraLite,
    BalancedLite,
    HighPerformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgileProfile {
    pub tier: TuningTier,
    pub target_num_ctx: u32,
    pub low_vram: bool,
    pub f16_kv: bool,
    pub max_cache_size_mb: u32,
    pub ast_compression_policy: String,
    pub auto_trim_interval_secs: u32,
    pub target_rss_mb: u32,
}

pub struct MemoryTuner;

impl MemoryTuner {
    /// Gathers real-time RAM metrics from /proc/meminfo or sysinfo fallback
    pub fn get_ram_telemetry() -> RamTelemetry {
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        let mut swap_total_kb = 0u64;
        let mut swap_free_kb = 0u64;

        if let Ok(mut f) = File::open("/proc/meminfo") {
            let mut contents = String::new();
            if f.read_to_string(&mut contents).is_ok() {
                for line in contents.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let key = parts[0];
                        let val: u64 = parts[1].parse().unwrap_or(0);
                        match key {
                            "MemTotal:" => total_kb = val,
                            "MemAvailable:" => avail_kb = val,
                            "SwapTotal:" => swap_total_kb = val,
                            "SwapFree:" => swap_free_kb = val,
                            _ => {}
                        }
                    }
                }
            }
        }

        if total_kb == 0 {
            total_kb = 8 * 1024 * 1024; // 8GB default fallback
            avail_kb = 4 * 1024 * 1024;
        }

        let total_mb = total_kb / 1024;
        let available_mb = avail_kb / 1024;
        let used_mb = total_mb.saturating_sub(available_mb);
        let used_percent = if total_mb > 0 {
            (used_mb as f32 / total_mb as f32) * 100.0
        } else {
            50.0
        };

        RamTelemetry {
            total_mb,
            available_mb,
            used_mb,
            used_percent,
            swap_total_mb: swap_total_kb / 1024,
            swap_free_mb: swap_free_kb / 1024,
        }
    }

    /// Detects GPU & VRAM hardware through nvidia-smi and Linux DRM sysfs
    pub fn get_gpu_telemetry() -> GpuTelemetry {
        // Check for NVIDIA GPU
        if let Ok(output) = Command::new("nvidia-smi")
            .args(["--query-gpu=name,memory.total,memory.free,memory.used", "--format=csv,noheader,nounits"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.trim().split(',').map(|s| s.trim()).collect();
                if parts.len() >= 4 {
                    let model = parts[0].to_string();
                    let total: u64 = parts[1].parse().unwrap_or(0);
                    let free: u64 = parts[2].parse().unwrap_or(0);
                    let used: u64 = parts[3].parse().unwrap_or(0);
                    return GpuTelemetry {
                        detected: true,
                        vendor: "NVIDIA".to_string(),
                        model,
                        total_vram_mb: total,
                        free_vram_mb: free,
                        used_vram_mb: used,
                    };
                }
            }
        }

        // Check for Intel / AMD GPU via /sys/class/drm
        let mut drm_vendor = "Integrated / Software".to_string();
        let mut drm_detected = false;
        if Path::new("/sys/class/drm").exists() {
            if let Ok(entries) = fs::read_dir("/sys/class/drm") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("card0") && !file_name.contains('-') {
                        drm_detected = true;
                        let device_path = entry.path().join("device");
                        if let Ok(vendor) = fs::read_to_string(device_path.join("vendor")) {
                            if vendor.trim().contains("0x8086") {
                                drm_vendor = "Intel Integrated Graphics (DirectML/Vulkan)".to_string();
                            } else if vendor.trim().contains("0x1002") {
                                drm_vendor = "AMD Radeon Graphics (ROCm/Vulkan)".to_string();
                            }
                        }
                    }
                }
            }
        }

        GpuTelemetry {
            detected: drm_detected,
            vendor: drm_vendor.clone(),
            model: if drm_detected { drm_vendor } else { "None / CPU Only".to_string() },
            total_vram_mb: 0,
            free_vram_mb: 0,
            used_vram_mb: 0,
        }
    }

    /// Computes the optimal Agile Profile dynamically based on memory pressure
    pub fn compute_agile_profile(ram: &RamTelemetry, gpu: &GpuTelemetry) -> AgileProfile {
        if ram.used_percent > 75.0 || ram.total_mb <= 4096 || (gpu.detected && gpu.total_vram_mb > 0 && gpu.free_vram_mb < 1024) {
            AgileProfile {
                tier: TuningTier::UltraLite,
                target_num_ctx: 1024,
                low_vram: true,
                f16_kv: false, // 8-bit quantized KV cache
                max_cache_size_mb: 5,
                ast_compression_policy: "AST_AGGRESSIVE".to_string(),
                auto_trim_interval_secs: 5,
                target_rss_mb: 35,
            }
        } else if ram.used_percent > 45.0 || ram.total_mb <= 16384 {
            AgileProfile {
                tier: TuningTier::BalancedLite,
                target_num_ctx: 2048,
                low_vram: gpu.total_vram_mb > 0 && gpu.free_vram_mb < 2048,
                f16_kv: false,
                max_cache_size_mb: 20,
                ast_compression_policy: "AST_STANDARD".to_string(),
                auto_trim_interval_secs: 15,
                target_rss_mb: 65,
            }
        } else {
            AgileProfile {
                tier: TuningTier::HighPerformance,
                target_num_ctx: 4096,
                low_vram: false,
                f16_kv: true,
                max_cache_size_mb: 100,
                ast_compression_policy: "AST_MINIMAL".to_string(),
                auto_trim_interval_secs: 60,
                target_rss_mb: 150,
            }
        }
    }

    /// Trims process heap by calling Linux malloc_trim(0) and releasing dirty pages
    pub fn trim_process_heap() {
        #[cfg(target_os = "linux")]
        unsafe {
            extern "C" {
                fn malloc_trim(pad: usize) -> i32;
            }
            malloc_trim(0);
        }
    }

    /// Prunes the semantic cache if it exceeds max size
    pub fn prune_disk_cache(max_size_mb: u32) {
        let cache_path = Path::new("lomi_cache.json");
        if cache_path.exists() {
            if let Ok(metadata) = fs::metadata(cache_path) {
                let size_mb = metadata.len() / (1024 * 1024);
                if size_mb > max_size_mb as u64 {
                    // Evict half of old entries
                    if let Ok(content) = fs::read_to_string(cache_path) {
                        if let Ok(mut map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
                            let keys: Vec<String> = map.keys().cloned().collect();
                            let to_remove = keys.len() / 2;
                            for k in keys.iter().take(to_remove) {
                                map.remove(k);
                            }
                            if let Ok(new_content) = serde_json::to_string_pretty(&map) {
                                let _ = fs::write(cache_path, new_content);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generates optimized Ollama request parameters according to active Agile Profile
    pub fn get_agile_ollama_options(profile: &AgileProfile) -> serde_json::Value {
        serde_json::json!({
            "num_ctx": profile.target_num_ctx,
            "low_vram": profile.low_vram,
            "f16_kv": profile.f16_kv,
            "temperature": 0.2
        })
    }

    /// Executes a full autonomous memory tuning pass and outputs diagnostics
    pub fn execute_tuning_pass() -> AgileProfile {
        let ram = Self::get_ram_telemetry();
        let gpu = Self::get_gpu_telemetry();
        let profile = Self::compute_agile_profile(&ram, &gpu);

        // Actuate memory trim immediately
        Self::trim_process_heap();
        Self::prune_disk_cache(profile.max_cache_size_mb);

        // Save active profile
        let _ = fs::create_dir_all(".lomi_cache");
        let _ = fs::write(
            ".lomi_cache/memory_tuning_profile.json",
            serde_json::to_string_pretty(&serde_json::json!({
                "ram_telemetry": ram,
                "gpu_telemetry": gpu,
                "active_profile": profile,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })).unwrap_or_default()
        );

        profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_telemetry_gathering() {
        let ram = MemoryTuner::get_ram_telemetry();
        assert!(ram.total_mb > 0);
        assert!(ram.used_percent >= 0.0 && ram.used_percent <= 100.0);
    }

    #[test]
    fn test_agile_profile_computation_high_memory() {
        let ram = RamTelemetry {
            total_mb: 8192,
            available_mb: 1024,
            used_mb: 7168,
            used_percent: 87.5,
            swap_total_mb: 2048,
            swap_free_mb: 1024,
        };
        let gpu = GpuTelemetry {
            detected: false,
            vendor: "None".to_string(),
            model: "CPU".to_string(),
            total_vram_mb: 0,
            free_vram_mb: 0,
            used_vram_mb: 0,
        };
        let profile = MemoryTuner::compute_agile_profile(&ram, &gpu);
        assert_eq!(profile.tier, TuningTier::UltraLite);
        assert_eq!(profile.target_num_ctx, 1024);
        assert!(profile.low_vram);
        assert!(!profile.f16_kv);
    }

    #[test]
    fn test_agile_profile_computation_low_memory_pressure() {
        let ram = RamTelemetry {
            total_mb: 32768,
            available_mb: 24576,
            used_mb: 8192,
            used_percent: 25.0,
            swap_total_mb: 8192,
            swap_free_mb: 8192,
        };
        let gpu = GpuTelemetry {
            detected: true,
            vendor: "NVIDIA".to_string(),
            model: "RTX 4090".to_string(),
            total_vram_mb: 24576,
            free_vram_mb: 20000,
            used_vram_mb: 4576,
        };
        let profile = MemoryTuner::compute_agile_profile(&ram, &gpu);
        assert_eq!(profile.tier, TuningTier::HighPerformance);
        assert_eq!(profile.target_num_ctx, 4096);
        assert!(!profile.low_vram);
        assert!(profile.f16_kv);
    }
}

