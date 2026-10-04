use std::fs;
use std::path::Path;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp"];

fn is_image(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// The image the Themes page shows for a theme, chosen the way Omarchy's
/// own `omarchy-theme-switcher` chooses it: a `preview.*` in the theme
/// root, else the first wallpaper (by name) in `backgrounds/`. Other images
/// in the root (the Plymouth logo, say) are never used.
pub fn find_preview_image(theme_dir: &Path) -> Option<String> {
    let mut root: Vec<_> = fs::read_dir(theme_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            is_image(path)
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|stem| stem.eq_ignore_ascii_case("preview"))
        })
        .collect();
    root.sort();
    if let Some(preview) = root.into_iter().next() {
        return Some(preview.to_string_lossy().to_string());
    }

    let mut backgrounds: Vec<_> = fs::read_dir(theme_dir.join("backgrounds"))
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| is_image(path))
        .collect();
    backgrounds.sort();
    backgrounds
        .into_iter()
        .next()
        .map(|path| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::find_preview_image;
    use std::fs;

    #[test]
    fn preview_then_first_background_never_other_root_images() {
        let dir = std::env::temp_dir().join(format!("omarchist-preview-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("backgrounds")).unwrap();
        fs::write(dir.join("unlock.png"), "x").unwrap();
        fs::write(dir.join("preview-unlock.png"), "x").unwrap();
        assert_eq!(find_preview_image(&dir), None, "logos are not previews");

        fs::write(dir.join("backgrounds/2-b.jpg"), "x").unwrap();
        fs::write(dir.join("backgrounds/1-a.png"), "x").unwrap();
        assert!(
            find_preview_image(&dir)
                .unwrap()
                .ends_with("backgrounds/1-a.png")
        );

        fs::write(dir.join("Preview.JPG"), "x").unwrap();
        assert!(find_preview_image(&dir).unwrap().ends_with("Preview.JPG"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
