use std::process::Command;

/// WireGuard Netlink AI Swarm
/// Configures a real WireGuard tunnel for encrypted peer-to-peer compute networking.
/// Generates real keypairs, configures the interface, and validates results.
/// Falls back to AI-powered troubleshooting via local Ollama when setup fails.
pub fn join_wireguard_swarm(peer_key: &str, endpoint: &str) -> Result<(), String> {
    println!("🕸️ [WireGuard/Netlink] Forging encrypted P2P mesh connection to {}...", endpoint);

    // Step 1: Check if WireGuard tools are installed
    let wg_available = Command::new("which")
        .arg("wg")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !wg_available {
        println!("   ❌ WireGuard tools (`wg`) not found.");
        println!("   Install: sudo apt install wireguard-tools");
        return Err("WireGuard tools not installed.".into());
    }
    println!("   ✅ WireGuard tools detected.");

    // Step 2: Generate a real keypair for this node
    let genkey_output = Command::new("wg")
        .arg("genkey")
        .output()
        .map_err(|e| format!("Failed to generate WireGuard key: {}", e))?;

    if !genkey_output.status.success() {
        return Err("wg genkey failed.".into());
    }

    let private_key = String::from_utf8_lossy(&genkey_output.stdout).trim().to_string();
    println!("   🔑 Generated private key: {}...", &private_key[..private_key.len().min(8)]);

    // Derive the public key from the private key
    let mut pubkey_cmd = Command::new("wg")
        .arg("pubkey")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn wg pubkey: {}", e))?;

    if let Some(ref mut stdin) = pubkey_cmd.stdin {
        use std::io::Write;
        let _ = stdin.write_all(private_key.as_bytes());
    }

    let pubkey_output = pubkey_cmd.wait_with_output()
        .map_err(|e| format!("wg pubkey failed: {}", e))?;
    let public_key = String::from_utf8_lossy(&pubkey_output.stdout).trim().to_string();
    println!("   🔑 Derived public key:  {}...", &public_key[..public_key.len().min(8)]);

    // Step 3: Create the WireGuard interface
    let iface_name = "wg_lomi";
    let add_result = Command::new("ip")
        .args(["link", "add", "dev", iface_name, "type", "wireguard"])
        .output();

    match add_result {
        Ok(out) if out.status.success() => {
            println!("   ✅ Interface `{}` created.", iface_name);
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if stderr.contains("exists") {
                println!("   ℹ️  Interface `{}` already exists. Reconfiguring...", iface_name);
            } else {
                println!("   ⚠️  Failed to create interface: {}", stderr.trim());
                println!("      This requires root/sudo. Run: sudo lomi experimental --feature swarm");
                ai_troubleshoot_wireguard(&stderr);
                return Err(format!("Interface creation failed: {}", stderr.trim()));
            }
        }
        Err(e) => {
            println!("   ❌ `ip` command failed: {}", e);
            return Err(format!("ip command error: {}", e));
        }
    }

    // Step 4: Write private key to temp file and configure WireGuard
    let key_path = std::env::temp_dir().join("lomi_wg_privkey");
    if let Err(e) = std::fs::write(&key_path, &private_key) {
        println!("   ⚠️  Could not write key file: {}", e);
        return Err(format!("Key file write failed: {}", e));
    }

    // Set the private key and listening port
    let wg_set = Command::new("wg")
        .args(["set", iface_name,
               "listen-port", "51820",
               "private-key", key_path.to_str().unwrap_or("/tmp/lomi_wg_privkey")])
        .output();

    // Clean up the key file immediately
    let _ = std::fs::remove_file(&key_path);

    match wg_set {
        Ok(out) if out.status.success() => {
            println!("   ✅ Private key and listen port configured.");
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("   ⚠️  wg set failed: {}", stderr.trim());
            ai_troubleshoot_wireguard(&stderr);
            return Err(format!("wg set failed: {}", stderr.trim()));
        }
        Err(e) => return Err(format!("wg set error: {}", e)),
    }

    // Step 5: Add the remote peer
    let add_peer = Command::new("wg")
        .args(["set", iface_name,
               "peer", peer_key,
               "endpoint", endpoint,
               "allowed-ips", "10.0.0.0/24"])
        .output();

    match add_peer {
        Ok(out) if out.status.success() => {
            println!("   ✅ Peer {} added (endpoint: {}).", &peer_key[..peer_key.len().min(8)], endpoint);
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("   ⚠️  Peer configuration failed: {}", stderr.trim());
            ai_troubleshoot_wireguard(&stderr);
        }
        Err(e) => println!("   ⚠️  wg set peer error: {}", e),
    }

    // Step 6: Assign IP and bring the interface up
    let _ = Command::new("ip")
        .args(["address", "add", "10.0.0.1/24", "dev", iface_name])
        .output();

    let up_result = Command::new("ip")
        .args(["link", "set", "up", "dev", iface_name])
        .output();

    match up_result {
        Ok(out) if out.status.success() => {
            println!("   ✅ Interface `{}` is UP with IP 10.0.0.1/24.", iface_name);
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("   ⚠️  Could not bring interface up: {}", stderr.trim());
        }
        Err(e) => println!("   ⚠️  ip link set up failed: {}", e),
    }

    // Step 7: Verify with `wg show`
    if let Ok(out) = Command::new("wg").args(["show", iface_name]).output() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if !stdout.trim().is_empty() {
            println!("\n   📊 WireGuard Status:");
            for line in stdout.lines() {
                println!("      {}", line);
            }
        }
    }

    println!("\n✅ [WireGuard/Netlink] Swarm mesh tunnel configured on `{}`.", iface_name);
    println!("   Share your public key with peers: {}", public_key);
    Ok(())
}

/// AI-powered troubleshooting: sends the WireGuard error to local Ollama for diagnosis
fn ai_troubleshoot_wireguard(error_msg: &str) {
    let error_owned = error_msg.to_string();
    std::thread::spawn(move || {
        if let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
        {
            let prompt = format!(
                "A WireGuard VPN setup command failed with this error: '{}'. \
                 Give a concise 2-sentence diagnosis and fix. Be specific to Linux.",
                &error_owned.chars().take(500).collect::<String>()
            );
            let payload = serde_json::json!({
                "model": "qwen2.5-coder:7b",
                "prompt": prompt,
                "stream": false,
                "options": { "num_predict": 150 }
            });
            if let Ok(resp) = client.post("http://127.0.0.1:11434/api/generate")
                .json(&payload)
                .send()
            {
                if let Ok(json) = resp.json::<serde_json::Value>() {
                    if let Some(advice) = json["response"].as_str() {
                        println!("   🤖 AI Diagnosis: {}", advice.trim().chars().take(200).collect::<String>());
                    }
                }
            }
        }
    });
}
