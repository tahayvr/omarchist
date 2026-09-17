use crate::error::{Error, Result};
use std::process::{Command, Stdio};

pub fn launch_omarchy_update() -> Result<()> {
    Command::new("omarchy-launch-floating-terminal-with-presentation")
        .arg("omarchy-update")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::io("Failed to launch omarchy update", e))?;

    Ok(())
}
