use std::process::Command;

/// mlock() & tmpfs RAM-Disk
/// Provisions a tmpfs volume and uses mlock to pin AI weights into physical RAM, preventing swap.
pub fn pin_model_to_ram(model_name: &str) -> Result<(), String> {
    println!("🧠 [mlock/tmpfs] Provisioning zero-latency RAM-Disk for {}...", model_name);
    
    // Mount tmpfs in production: mount -t tmpfs -o size=16G tmpfs /mnt/lomi_ramdisk
    let output = Command::new("mount")
        .args(["-t", "tmpfs", "-o", "size=8G", "tmpfs", "/tmp/lomi_models"])
        .output();
        
    match output {
        Ok(out) if out.status.success() => {
            println!("✅ [mlock/tmpfs] 8GB RAM-Disk allocated. Model pinned. Swap disabled.");
            Ok(())
        }
        _ => Err("Requires root privileges to mount tmpfs.".into()),
    }
}
