//! Atomic file replacement for every state file Omarchist owns.
//!
//! A plain `fs::write` truncates first, so a crash, a full disk, or a reader
//! that runs in between (Omarchy staging a theme, the app's own scan) sees an
//! empty or half-written file. Writing next to the target and renaming over
//! it makes the replacement one step.

use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// Writes `contents` to `path` by writing a temporary sibling and renaming it
/// over the target. Follows a symlinked target to its real file and keeps the
/// target's permissions. `what` names the file in errors.
pub fn write_atomic(path: &Path, contents: impl AsRef<[u8]>, what: &str) -> Result<()> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let dir = target
        .parent()
        .ok_or(Error::UnknownDirectory("file parent"))?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(Error::UnknownDirectory("file name"))?;
    // A dotfile: `omarchy-theme-set` copies theme folders with `*`, which
    // skips it, so a stray temp file never ends up in a staged theme.
    let tmp = dir.join(format!(".{name}.tmp"));
    fs::write(&tmp, contents).map_err(|e| Error::io(format!("Failed to write {what}"), e))?;
    if let Ok(metadata) = fs::metadata(&target) {
        let _ = fs::set_permissions(&tmp, metadata.permissions());
    }
    fs::rename(&tmp, &target).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        Error::io(format!("Failed to replace {what}"), e)
    })
}

#[cfg(test)]
mod tests {
    use super::write_atomic;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn replaces_the_file_and_leaves_no_temp_behind() {
        let dir = tempdir();
        let path = dir.join("state.json");
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

        write_atomic(&path, "new", "state file").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "no temp file left");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn creates_a_missing_file() {
        let dir = tempdir();
        let path = dir.join("new.toml");
        write_atomic(&path, "a = 1\n", "toml").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "a = 1\n");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn writes_through_a_symlink() {
        let dir = tempdir();
        let real = dir.join("real.json");
        let link = dir.join("link.json");
        fs::write(&real, "{}").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();

        write_atomic(&link, "{\"a\":1}", "json").unwrap();

        assert!(fs::symlink_metadata(&link).unwrap().is_symlink());
        assert_eq!(fs::read_to_string(&real).unwrap(), "{\"a\":1}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_directory_is_an_error_not_a_panic() {
        let dir = tempdir();
        let err = write_atomic(&dir.join("nope").join("x"), "x", "x").unwrap_err();
        assert!(err.to_string().contains("Failed to write x"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    fn tempdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "omarchist-fs-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
