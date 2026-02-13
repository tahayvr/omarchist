use smol::unblock;
use std::process::Command;

pub async fn apply_theme(dir: String) -> Result<(), String> {
    unblock(move || {
        let output = Command::new("omarchy-theme-set")
            .arg(&dir)
            .output()
            .map_err(|e| format!("Failed to execute omarchy-theme-set: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to apply theme '{dir}': {stderr}"));
        }

        Ok(())
    })
    .await
}
