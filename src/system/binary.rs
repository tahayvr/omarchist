//! Which `omarchist` the files Omarchist writes for other programs should
//! run: the bar widget's `command` file, flow launcher entries, startup
//! hooks. `current_exe()` is right for an installed package and for a
//! build run from `target/`; two cases need care.
use std::path::PathBuf;

/// The binary to write into launcher entries and hooks.
///
/// After a package upgrade while the app runs, `/proc/self/exe` reads
/// `/usr/bin/omarchist (deleted)`, which no launcher could run; and a
/// development build under `target/` is stale tomorrow, so the installed
/// `omarchist` on `PATH` wins when there is one.
pub fn omarchist_binary() -> String {
    let exe = std::env::current_exe().ok();
    let on_path = path_binary();
    match exe {
        Some(exe) if !is_deleted(&exe) && (!is_dev_build(&exe) || on_path.is_none()) => {
            exe.to_string_lossy().to_string()
        }
        _ => on_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "omarchist".to_string()),
    }
}

fn is_deleted(exe: &std::path::Path) -> bool {
    exe.to_string_lossy().ends_with(" (deleted)") || !exe.is_file()
}

fn is_dev_build(exe: &std::path::Path) -> bool {
    exe.components().any(|c| c.as_os_str() == "target")
}

fn path_binary() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join("omarchist"))
        .find(|candidate| candidate.is_file())
}

/// `argument` quoted for a shell (`sh -c`, a hook script).
pub fn shell_quote(argument: &str) -> String {
    if !argument.is_empty()
        && argument
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+@%".contains(c))
    {
        argument.to_string()
    } else {
        format!("'{}'", argument.replace('\'', "'\\''"))
    }
}

/// `argument` quoted for a desktop entry's `Exec` line: double quotes, with
/// the characters the Desktop Entry spec reserves escaped.
pub fn desktop_exec_quote(argument: &str) -> String {
    if !argument.is_empty()
        && argument
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=+@".contains(c))
    {
        return argument.to_string();
    }
    let mut out = String::from("\"");
    for c in argument.chars() {
        if matches!(c, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }
        // A field code to the desktop entry, unless doubled.
        if c == '%' {
            out.push('%');
        }
        out.push(c);
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoting() {
        assert_eq!(shell_quote("/usr/bin/omarchist"), "/usr/bin/omarchist");
        assert_eq!(shell_quote("/home/a b/omarchist"), "'/home/a b/omarchist'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(
            desktop_exec_quote("/usr/bin/omarchist"),
            "/usr/bin/omarchist"
        );
        assert_eq!(desktop_exec_quote("/home/a b/x"), "\"/home/a b/x\"");
        assert_eq!(desktop_exec_quote("a\"b$c"), "\"a\\\"b\\$c\"");
    }

    #[test]
    fn the_binary_is_an_existing_file_or_the_bare_name() {
        let binary = omarchist_binary();
        assert!(binary == "omarchist" || std::path::Path::new(&binary).is_file());
        assert!(!binary.ends_with("(deleted)"));
    }
}
