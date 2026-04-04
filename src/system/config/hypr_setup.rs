use std::fs;
use std::path::PathBuf;

const SOURCE_COMMENT: &str = "# Added by Omarchist";
const SOURCE_DIRECTIVE: &str = "source = ~/.config/omarchist/hyprland/*";
const HYPR_CONFIG_PATH: &str = ".config/hypr/hyprland.conf";

pub fn ensure_hypr_source() -> Result<bool, String> {
    let hypr_config_path = get_hypr_config_path()?;

    // Check if hyprland.conf exists
    if !hypr_config_path.exists() {
        return Err(format!(
            "Hyprland config not found at: {}",
            hypr_config_path.display()
        ));
    }

    // Read the config file
    let content = fs::read_to_string(&hypr_config_path)
        .map_err(|e| format!("Failed to read hyprland.conf: {}", e))?;

    // Check if source directive already exists
    if content.contains(SOURCE_DIRECTIVE) {
        return Ok(false);
    }

    // Append source directive with comment to the end
    let new_content = format!(
        "{}\n\n{}\n{}\n",
        content.trim_end(),
        SOURCE_COMMENT,
        SOURCE_DIRECTIVE
    );

    fs::write(&hypr_config_path, new_content)
        .map_err(|e| format!("Failed to write hyprland.conf: {}", e))?;

    println!("Added omarchist source directive to hyprland.conf");

    Ok(true)
}

fn get_hypr_config_path() -> Result<PathBuf, String> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;

    Ok(home_dir.join(HYPR_CONFIG_PATH))
}
