use std::fs;
use std::path::Path;

/// Returns the absolute file path to the image
pub fn find_preview_image(theme_dir: &Path) -> Option<String> {
    // First, check for preview.png specifically (most common)
    let preview_path = theme_dir.join("preview.png");
    if preview_path.exists() {
        return Some(preview_path.to_string_lossy().to_string());
    }

    // Fallback: look for any image file
    let entries = fs::read_dir(theme_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension().and_then(|e| e.to_str())
            && matches!(
                ext.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg"
            )
        {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}
