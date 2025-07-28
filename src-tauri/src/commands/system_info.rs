use std::env;
use tauri::command;

#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub app_version: String,
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
}

#[command]
pub fn get_system_info() -> SystemInfo {
    let app_version = env!("CARGO_PKG_VERSION").to_string();

    let os_name = if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        "Unknown".to_string()
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "64-Bit".to_string()
    } else if cfg!(target_arch = "x86") {
        "32-Bit".to_string()
    } else if cfg!(target_arch = "aarch64") {
        "ARM64".to_string()
    } else {
        env::consts::ARCH.to_string()
    };

    // Versuche Windows-Version zu ermitteln
    let os_version = if cfg!(target_os = "windows") {
        get_windows_version()
    } else {
        "Unknown".to_string()
    };

    SystemInfo {
        app_version,
        os_name,
        os_version,
        arch,
    }
}

#[cfg(target_os = "windows")]
fn get_windows_version() -> String {
    use std::process::Command;

    // Versuche Windows-Version über wmic zu ermitteln
    if let Ok(output) = Command::new("wmic")
        .args(["os", "get", "Caption", "/value"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.starts_with("Caption=") {
                    let version = line.replace("Caption=", "").trim().to_string();
                    if !version.is_empty() {
                        return version;
                    }
                }
            }
        }
    }

    // Fallback
    "Windows".to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_windows_version() -> String {
    "Unknown".to_string()
}
