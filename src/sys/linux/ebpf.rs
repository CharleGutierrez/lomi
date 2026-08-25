use aya::Bpf;
use aya::programs::{Xdp, XdpFlags};
use std::path::Path;

/// eBPF / XDP Zero-Copy Proxy Acceleration
/// Attaches an XDP program to intercept network packets at the NIC driver level
pub fn init_xdp_proxy(interface: &str) -> Result<(), String> {
    println!("🛡️ [eBPF/XDP] Attaching program to interface: {}", interface);
    
    // In production, this path points to the compiled eBPF bytecode (.o or .elf)
    let bpf_path = Path::new("/var/lib/lomi/lomi-xdp.bpf.o");
    
    if !bpf_path.exists() {
        println!("⚠️ [eBPF/XDP] Bytecode not found at {:?}. Skipping actual load.", bpf_path);
        return Ok(());
    }

    // Load the eBPF program from disk
    let mut bpf = Bpf::load_file(bpf_path)
        .map_err(|e| format!("Failed to load eBPF ELF: {}", e))?;

    // Extract the XDP program block by name
    let program: &mut Xdp = bpf.program_mut("xdp_lomi_router")
        .unwrap()
        .try_into()
        .map_err(|_| "Failed to cast eBPF program to XDP")?;

    // Attach to the specified network interface with default flags
    program.load().map_err(|e| format!("Failed to load XDP program into kernel: {}", e))?;
    program.attach(interface, XdpFlags::default())
        .map_err(|e| format!("Failed to attach to interface {}: {}", interface, e))?;

    println!("✅ [eBPF/XDP] Zero-copy routing active. AI traffic bypassing Linux network stack.");
    Ok(())
}
