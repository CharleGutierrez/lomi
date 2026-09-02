use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonInstallReport {
    pub service_name: String,
    pub service_path: String,
    pub service_content: String,
    pub installed: bool,
    pub status_msg: String,
}

pub struct DaemonInstaller;

impl DaemonInstaller {
    const SERVICE_FILE_NAME: &'static str = "lomi.service";

    /// Generates standard Linux systemd service unit file configuration
    pub fn generate_systemd_service() -> String {
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/local/bin/lomi".to_string());

        format!(
            r#"[Unit]
Description=LOMI: LLM Optimization & Model Improver AGI Proxy Daemon
After=network.target

[Service]
Type=simple
ExecStart={} serve-proxy --port 8109
Restart=always
RestartSec=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
"#,
            exe_path
        )
    }

    /// Generates and attempts installation of systemd unit file
    pub fn install_service() -> DaemonInstallReport {
        let content = Self::generate_systemd_service();
        let target_path = Path::new("/etc/systemd/system").join(Self::SERVICE_FILE_NAME);
        let local_path = Path::new("lomi.service");

        // Write local copy first
        let _ = fs::write(local_path, &content);

        let installed = if target_path.parent().map(|p| p.exists()).unwrap_or(false) {
            fs::write(&target_path, &content).is_ok()
        } else {
            false
        };

        let status_msg = if installed {
            format!("✅ Installed systemd service to {:?}", target_path)
        } else {
            format!("⚠️ Local service file written to {:?}. Run `sudo cp lomi.service /etc/systemd/system/` to complete system installation.", local_path)
        };

        DaemonInstallReport {
            service_name: Self::SERVICE_FILE_NAME.to_string(),
            service_path: if installed { target_path.to_string_lossy().to_string() } else { local_path.to_string_lossy().to_string() },
            service_content: content,
            installed,
            status_msg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemd_service_generation() {
        let content = DaemonInstaller::generate_systemd_service();
        assert!(content.contains("[Unit]"));
        assert!(content.contains("serve-proxy"));
        assert!(content.contains("WantedBy=multi-user.target"));
    }
}
