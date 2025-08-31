use log::info;
use std::env;
use std::fs;
use std::process::Command;

/// Detect NVIDIA graphics and apply envs
pub fn setup_nvidia_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    if is_nvidia_system()? {
        env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        info!("NVIDIA graphics detected - applied WebKit compatibility fix");
    }
    Ok(())
}

/// Checks if the system has NVIDIA graphics using multiple detection methods
fn is_nvidia_system() -> Result<bool, Box<dyn std::error::Error>> {
    // Method 1: Check for nvidia kernel modules
    if check_nvidia_modules()? {
        return Ok(true);
    }

    // Method 2: Check for nvidia-smi command
    if check_nvidia_smi() {
        return Ok(true);
    }

    // Method 3: Check /proc/driver/nvidia
    if check_proc_nvidia() {
        return Ok(true);
    }

    // Method 4: Check lspci output
    if check_lspci_nvidia()? {
        return Ok(true);
    }

    Ok(false)
}

fn check_nvidia_modules() -> Result<bool, Box<dyn std::error::Error>> {
    match fs::read_to_string("/proc/modules") {
        Ok(modules) => Ok(modules.lines().any(|line| {
            line.starts_with("nvidia")
                || line.starts_with("nvidia_drm")
                || line.starts_with("nvidia_modeset")
        })),
        Err(_) => Ok(false),
    }
}

fn check_nvidia_smi() -> bool {
    Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
        .map(|output| output.status.success() && !output.stdout.is_empty())
        .unwrap_or(false)
}

fn check_proc_nvidia() -> bool {
    std::path::Path::new("/proc/driver/nvidia").exists()
}

fn check_lspci_nvidia() -> Result<bool, Box<dyn std::error::Error>> {
    match Command::new("lspci").output() {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                Ok(output_str.lines().any(|line| {
                    let line_lower = line.to_lowercase();
                    (line_lower.contains("vga")
                        || line_lower.contains("3d")
                        || line_lower.contains("display"))
                        && line_lower.contains("nvidia")
                }))
            } else {
                Ok(false)
            }
        },
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_setup_nvidia_compatibility() {
        let result = setup_nvidia_compatibility();
        assert!(result.is_ok());

        // Check if the environment variable was set (only if NVIDIA was detected)
        if is_nvidia_system().unwrap_or(false) {
            assert_eq!(env::var("WEBKIT_DISABLE_DMABUF_RENDERER").unwrap(), "1");
        }
    }

    #[test]
    fn test_detection_methods_dont_panic() {
        // Ensure all detection methods handle errors gracefully
        let _ = check_nvidia_modules();
        let _ = check_nvidia_smi();
        let _ = check_proc_nvidia();
        let _ = check_lspci_nvidia();
    }
}
