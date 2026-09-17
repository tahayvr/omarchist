//! Installed applications, read from freedesktop `.desktop` entries.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopApp {
    /// Desktop file id, e.g. `org.gnome.Nautilus`.
    pub id: String,
    pub name: String,
    /// `Exec=` with field codes removed.
    pub exec: String,
    pub icon: Option<PathBuf>,
    /// Pattern for `omarchy-launch-or-focus`: `StartupWMClass` or the id.
    pub wm_class: String,
    /// `Terminal=true`: the program needs a terminal window.
    pub terminal: bool,
    /// Set for Omarchy web apps (`omarchy-launch-webapp <url>`).
    pub webapp_url: Option<String>,
}

impl DesktopApp {
    pub fn is_webapp(&self) -> bool {
        self.webapp_url.is_some()
    }
}

const ICON_SIZES: &[&str] = &[
    "scalable", "512x512", "256x256", "192x192", "128x128", "96x96", "64x64", "48x48",
];
const ICON_THEMES: &[&str] = &["hicolor", "Yaru", "Papirus", "Adwaita", "breeze"];

fn home() -> Option<PathBuf> {
    dirs::home_dir()
}

/// `$XDG_DATA_HOME`, or `~/.local/share`.
pub fn data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".local/share")))
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = data_home().into_iter().collect();
    let system = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    dirs.extend(
        system
            .split(':')
            .filter(|d| !d.is_empty())
            .map(PathBuf::from),
    );
    dirs
}

/// Every launchable application visible in menus, user entries first,
/// sorted by name. Later directories never override an earlier id.
pub fn installed_apps() -> Vec<DesktopApp> {
    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    for dir in data_dirs() {
        let root = dir.join("applications");
        for (id, path) in desktop_files(&root) {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Some(app) = parse_desktop_entry(&id, &content)
            {
                apps.push(app);
            }
        }
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    apps
}

fn desktop_files(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mut stack = vec![(String::new(), root.to_path_buf())];
    while let Some((prefix, dir)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if path.is_dir() {
                stack.push((format!("{prefix}{file_name}-"), path));
            } else if let Some(stem) = file_name.strip_suffix(".desktop") {
                out.push((format!("{prefix}{stem}"), path));
            }
        }
    }
    out.sort();
    out
}

/// Parses the `[Desktop Entry]` group. Returns `None` for entries that
/// should not appear in a launcher.
pub fn parse_desktop_entry(id: &str, content: &str) -> Option<DesktopApp> {
    let mut in_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut wm_class = None;
    let mut terminal = false;
    let mut hidden = false;
    let mut is_application = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if in_entry {
                break;
            }
            in_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "Type" => is_application = value == "Application",
            "Name" => name = Some(value.to_string()),
            "Exec" => exec = Some(strip_field_codes(value)),
            "Icon" => icon = Some(value.to_string()),
            "StartupWMClass" => wm_class = Some(value.to_string()),
            "Terminal" => terminal = value == "true",
            "NoDisplay" | "Hidden" if value == "true" => hidden = true,
            _ => {}
        }
    }

    if !is_application || hidden {
        return None;
    }
    let name = name?;
    let exec = exec.filter(|e| !e.is_empty())?;
    let webapp_url = exec
        .strip_prefix("omarchy-launch-webapp ")
        .and_then(|rest| {
            crate::system::keybinds::action::shell_split(rest)
                .into_iter()
                .next()
        });
    let wm_class = wm_class
        .filter(|c| !c.is_empty() && !c.starts_with("@@"))
        .unwrap_or_else(|| id.to_string());

    Some(DesktopApp {
        id: id.to_string(),
        name,
        exec,
        icon: icon.and_then(|i| resolve_icon(&i)),
        wm_class,
        terminal,
        webapp_url,
    })
}

/// Removes the `%f`-style field codes an `Exec=` line may carry.
pub fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        if let Some('%') = chars.next() {
            out.push('%');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Finds an icon file for a desktop entry's `Icon=` value.
pub fn resolve_icon(icon: &str) -> Option<PathBuf> {
    if icon.starts_with('/') {
        let path = PathBuf::from(icon);
        return path.is_file().then_some(path);
    }
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(data) = data_home() {
        candidates.push(data.join("applications/icons"));
        for theme in ICON_THEMES {
            for size in ICON_SIZES {
                candidates.push(data.join(format!("icons/{theme}/{size}/apps")));
            }
        }
    }
    for base in ["/usr/share/icons", "/usr/local/share/icons"] {
        for theme in ICON_THEMES {
            for size in ICON_SIZES {
                candidates.push(PathBuf::from(format!("{base}/{theme}/{size}/apps")));
            }
        }
    }
    candidates.push(PathBuf::from("/usr/share/pixmaps"));

    for dir in candidates {
        for ext in ["svg", "png"] {
            let path = dir.join(format!("{icon}.{ext}"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_regular_entry() {
        let app = parse_desktop_entry(
            "chromium",
            "[Desktop Entry]\nType=Application\nName=Chromium\nExec=/usr/bin/chromium %U\nIcon=/nonexistent\nStartupWMClass=@@startup_wm_class\n\n[Desktop Action new-window]\nName=New Window\nExec=/usr/bin/chromium\n",
        )
        .unwrap();
        assert_eq!(app.name, "Chromium");
        assert_eq!(app.exec, "/usr/bin/chromium");
        assert_eq!(app.wm_class, "chromium");
        assert!(!app.terminal);
        assert!(app.webapp_url.is_none());
    }

    #[test]
    fn detects_omarchy_webapps_and_terminal_apps() {
        let web = parse_desktop_entry(
            "Basecamp",
            "[Desktop Entry]\nType=Application\nName=Basecamp\nExec=omarchy-launch-webapp https://launchpad.37signals.com\n",
        )
        .unwrap();
        assert_eq!(
            web.webapp_url.as_deref(),
            Some("https://launchpad.37signals.com")
        );

        let tui = parse_desktop_entry(
            "btop",
            "[Desktop Entry]\nType=Application\nName=btop++\nExec=btop\nTerminal=true\nStartupWMClass=btop\n",
        )
        .unwrap();
        assert!(tui.terminal);
        assert_eq!(tui.wm_class, "btop");
    }

    #[test]
    fn skips_hidden_and_non_application_entries() {
        assert!(
            parse_desktop_entry(
                "x",
                "[Desktop Entry]\nType=Application\nName=X\nExec=x\nNoDisplay=true\n"
            )
            .is_none()
        );
        assert!(
            parse_desktop_entry("y", "[Desktop Entry]\nType=Link\nName=Y\nURL=https://y\n")
                .is_none()
        );
        assert!(parse_desktop_entry("z", "[Desktop Entry]\nType=Application\nName=Z\n").is_none());
    }

    #[test]
    fn strips_field_codes() {
        assert_eq!(strip_field_codes("app %U --flag %f"), "app --flag");
        assert_eq!(strip_field_codes("app 100%% %i"), "app 100%");
    }
}
