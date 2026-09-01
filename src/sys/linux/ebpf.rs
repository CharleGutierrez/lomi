use std::path::Path;

/// eBPF / XDP Proxy Acceleration
/// Tries to load a compiled XDP program. Falls back to real /proc/net telemetry
/// if the eBPF object file is unavailable or the kernel doesn't support BPF.
pub fn init_xdp_proxy(interface: &str) -> Result<(), String> {
    println!("🛡️ [eBPF/XDP] Initializing network acceleration for interface: {}", interface);

    // Priority 1: Local bpf/ subdirectory (works in-repo without installation)
    let local_bpf = Path::new("src/sys/linux/bpf/lomi-xdp.bpf.o");
    // Priority 2: System install path
    let system_bpf = Path::new("/var/lib/lomi/lomi-xdp.bpf.o");

    let bpf_path = if local_bpf.exists() {
        Some(local_bpf)
    } else if system_bpf.exists() {
        Some(system_bpf)
    } else {
        None
    };

    if let Some(path) = bpf_path {
        println!("   Found eBPF object at {:?}. Attempting kernel load...", path);
        match try_load_xdp(path, interface) {
            Ok(()) => return Ok(()),
            Err(e) => println!("   ⚠️ eBPF load failed ({}). Falling back to /proc telemetry.", e),
        }
    } else {
        println!("   ⚠️ No pre-compiled eBPF object found.");
        println!("   To enable XDP: compile src/sys/linux/bpf/xdp_lomi.c with clang and place at src/sys/linux/bpf/lomi-xdp.bpf.o");
        println!("   Falling back to real kernel /proc/net telemetry...");
    }

    // Real fallback: read actual kernel network stats via /proc/net/tcp
    read_proc_net_telemetry(interface)
}

fn try_load_xdp(bpf_path: &Path, interface: &str) -> Result<(), String> {
    use aya::Bpf;
    use aya::programs::{Xdp, XdpFlags};

    let mut bpf = Bpf::load_file(bpf_path)
        .map_err(|e| format!("Failed to load eBPF ELF: {}", e))?;

    let program = bpf.program_mut("xdp_lomi_router")
        .ok_or("Program 'xdp_lomi_router' not found in eBPF object")?;
    let program: &mut Xdp = program
        .try_into()
        .map_err(|_| "Failed to cast to XDP program")?;

    program.load()
        .map_err(|e| format!("Failed to load XDP into kernel: {}", e))?;
    program.attach(interface, XdpFlags::default())
        .map_err(|e| format!("Failed to attach XDP to {}: {}", interface, e))?;

    println!("✅ [eBPF/XDP] Zero-copy XDP program active on {}!", interface);
    Ok(())
}

/// Real kernel telemetry via /proc/net/tcp
/// Reads actual socket connection data from the kernel — real kernel data, not simulated.
fn read_proc_net_telemetry(interface: &str) -> Result<(), String> {
    println!("📡 [Kernel Telemetry] Reading real network stats from /proc/net/...");

    // Read /proc/net/dev for interface stats
    if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let trimmed = line.trim();
            if trimmed.starts_with(interface) || trimmed.starts_with("lo") || trimmed.starts_with("eth") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 10 {
                    let iface = parts[0].trim_end_matches(':');
                    let rx_bytes: u64 = parts[1].parse().unwrap_or(0);
                    let tx_bytes: u64 = parts[9].parse().unwrap_or(0);
                    println!("   {} → RX: {:.2} MB | TX: {:.2} MB",
                        iface, rx_bytes as f64 / 1_048_576.0, tx_bytes as f64 / 1_048_576.0);
                }
            }
        }
    }

    // Count active TCP connections via /proc/net/tcp
    let tcp_connections = std::fs::read_to_string("/proc/net/tcp")
        .map(|c| c.lines().skip(1).filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0);
    println!("   Active TCP connections (kernel): {}", tcp_connections);

    // Check if eBPF is supported by this kernel
    let bpf_supported = Path::new("/proc/sys/net/core/bpf_jit_enable").exists();
    println!("   BPF JIT compile support: {}", if bpf_supported { "✅ Available" } else { "⚠️ Not found" });

    if bpf_supported {
        if let Ok(val) = std::fs::read_to_string("/proc/sys/net/core/bpf_jit_enable") {
            println!("   BPF JIT status: {}", val.trim());
        }
    }

    println!("✅ [Kernel Telemetry] Real /proc/net stats collected. XDP acceleration requires compiled eBPF object.");
    Ok(())
}
