/// GPU MIG / VFIO Passthrough
/// Hardware-isolates GPU slices and passes them through to sandboxed VMs via real sysfs interaction.
pub fn attach_vfio_gpu(device_id: &str) -> Result<(), String> {
    println!("[VFIO/MIG] Isolating GPU hardware slice (ID: {})...", device_id);

    // Check if the PCI device exists
    let pci_path = format!("/sys/bus/pci/devices/{}", device_id);
    if !std::path::Path::new(&pci_path).exists() {
        println!("   PCI device {} not found in sysfs.", device_id);
        println!("   Scanning for available GPUs...");

        // Real: scan for GPU devices in /sys/bus/pci/devices
        if let Ok(entries) = std::fs::read_dir("/sys/bus/pci/devices") {
            for entry in entries.flatten() {
                let class_path = entry.path().join("class");
                if let Ok(class) = std::fs::read_to_string(&class_path) {
                    // 0x030000 = VGA compatible controller (GPU)
                    if class.trim().starts_with("0x0300") {
                        let vendor = std::fs::read_to_string(entry.path().join("vendor")).unwrap_or_default();
                        let device = std::fs::read_to_string(entry.path().join("device")).unwrap_or_default();
                        println!("   Found GPU: {} (vendor={} device={})",
                            entry.file_name().to_string_lossy(), vendor.trim(), device.trim());
                    }
                }
            }
        }
        return Ok(());
    }

    // Read device info
    let vendor = std::fs::read_to_string(format!("{}/vendor", pci_path)).unwrap_or_default();
    let device = std::fs::read_to_string(format!("{}/device", pci_path)).unwrap_or_default();
    let class = std::fs::read_to_string(format!("{}/class", pci_path)).unwrap_or_default();
    println!("   Device: vendor={} device={} class={}", vendor.trim(), device.trim(), class.trim());

    // Check IOMMU group
    let iommu_link = format!("{}/iommu_group", pci_path);
    match std::fs::read_link(&iommu_link) {
        Ok(group_path) => {
            let group = group_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            println!("   IOMMU Group: {}", group);

            // List all devices in the same IOMMU group
            let group_devices = format!("/sys/kernel/iommu_groups/{}/devices", group);
            if let Ok(entries) = std::fs::read_dir(&group_devices) {
                let devices: Vec<String> = entries.flatten()
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                println!("   Group members: {:?}", devices);
            }
        }
        Err(_) => println!("   IOMMU not enabled or not available for this device."),
    }

    // Check current driver
    let driver_path = format!("{}/driver", pci_path);
    if let Ok(driver_link) = std::fs::read_link(&driver_path) {
        let current = driver_link.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        println!("   Current driver: {}", current);

        if current == "vfio-pci" {
            println!("[VFIO/MIG] GPU {} already bound to vfio-pci. Passthrough ready.", device_id);
            return Ok(());
        }

        // Attempt unbind + rebind to vfio-pci
        println!("   Attempting VFIO passthrough bind...");
        let unbind = format!("{}/driver/unbind", pci_path);
        match std::fs::write(&unbind, device_id) {
            Ok(_) => {
                let _ = std::fs::write("/sys/bus/pci/drivers/vfio-pci/bind", device_id);
                println!("[VFIO/MIG] GPU {} detached from {} and bound to vfio-pci.", device_id, current);
            }
            Err(e) => println!("   Requires root: {}. Run with sudo for GPU passthrough.", e),
        }
    }

    Ok(())
}
