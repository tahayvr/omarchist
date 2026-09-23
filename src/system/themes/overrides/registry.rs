/// The Theme Designer tab an override is edited on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Desktop,
    Terminals,
    Editors,
    Apps,
    AiTools,
}

impl Category {
    pub fn all() -> [Category; 5] {
        [
            Category::Desktop,
            Category::Terminals,
            Category::Editors,
            Category::Apps,
            Category::AiTools,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Category::Desktop => "Desktop",
            Category::Terminals => "Terminals",
            Category::Editors => "Editors",
            Category::Apps => "Apps",
            Category::AiTools => "AI Tools",
        }
    }
}

/// The file's syntax, for highlighting and validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Toml,
    Json,
    Yaml,
    Lua,
    Css,
    Ini,
    Plain,
}

impl Format {
    /// The gpui-component highlighter language, if it has one.
    pub fn language(&self) -> Option<&'static str> {
        match self {
            Format::Toml => Some("toml"),
            Format::Json => Some("json"),
            Format::Yaml => Some("yaml"),
            Format::Lua => Some("lua"),
            Format::Css => Some("css"),
            Format::Ini | Format::Plain => None,
        }
    }
}

/// How the Theme Designer edits the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorKind {
    /// Typed fields specific to the file.
    Form,
    /// Fields built from the section's keys in `shell.toml.tpl`.
    ShellSection,
    /// A picker for every hex color in the file, with a source view.
    ColorMap,
    /// The file as text.
    Source,
}

/// Where the starting content of a new override comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seed {
    /// Rendered from the named `*.tpl`.
    Template(&'static str),
    /// One `[section]` of the rendered `shell.toml.tpl`.
    ShellSection(&'static str),
    /// Fixed text; Omarchy has no template for the file.
    Fixed(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub struct OverrideSpec {
    /// File name inside the theme folder; also the override's id.
    pub file: &'static str,
    pub app: &'static str,
    pub description: &'static str,
    pub category: Category,
    pub format: Format,
    pub editor: EditorKind,
    pub seed: Seed,
    /// Programs that consume the file. Empty when it is always relevant.
    pub binaries: &'static [&'static str],
}

/// Files `omarchy-theme-set` refuses to stage from a theme cloned from a git
/// repo, besides every `*.lua` (`INSTALLED_THEME_DENIED`).
const GIT_DENIED: &[&str] = &[
    "alacritty.toml",
    "foot.ini",
    "ghostty.conf",
    "kitty.conf",
    "vscode.json",
];

impl OverrideSpec {
    /// Whether Omarchy ignores this file in a theme installed from a git repo.
    pub fn git_restricted(&self) -> bool {
        self.file.ends_with(".lua") || GIT_DENIED.contains(&self.file)
    }

    /// The `shell.toml` section this file replaces, if it is a section override.
    pub fn shell_section(&self) -> Option<&'static str> {
        match self.seed {
            Seed::ShellSection(section) => Some(section),
            _ => None,
        }
    }
}

pub fn find(file: &str) -> Option<&'static OverrideSpec> {
    OVERRIDES.iter().find(|spec| spec.file == file)
}

pub fn in_category(category: Category) -> impl Iterator<Item = &'static OverrideSpec> {
    OVERRIDES
        .iter()
        .filter(move |spec| spec.category == category)
}

macro_rules! shell_section {
    ($file:literal, $section:literal, $app:literal, $description:literal) => {
        OverrideSpec {
            file: $file,
            app: $app,
            description: $description,
            category: Category::Desktop,
            format: Format::Toml,
            editor: EditorKind::ShellSection,
            seed: Seed::ShellSection($section),
            binaries: &[],
        }
    };
}

pub const OVERRIDES: &[OverrideSpec] = &[
    shell_section!(
        "shell.bar.toml",
        "bar",
        "Bar",
        "The bar's background, text, and size."
    ),
    shell_section!(
        "shell.notifications.toml",
        "notifications",
        "Notifications",
        "Notification cards and their countdown."
    ),
    shell_section!(
        "shell.launcher.toml",
        "launcher",
        "Launcher",
        "The app launcher overlay."
    ),
    shell_section!(
        "shell.menu.toml",
        "menu",
        "Menus",
        "Menus, the clipboard, and the emoji picker."
    ),
    shell_section!(
        "shell.popups.toml",
        "popups",
        "Popups",
        "Bar flyouts, the OSD, and popup cards."
    ),
    shell_section!(
        "shell.tooltip.toml",
        "tooltip",
        "Tooltips",
        "Hover tooltips across the shell."
    ),
    shell_section!(
        "shell.polkit.toml",
        "polkit",
        "Password Prompt",
        "The polkit and sudo authentication dialog."
    ),
    shell_section!(
        "shell.lock.toml",
        "lock",
        "Lock Screen",
        "The lock screen's password field."
    ),
    shell_section!(
        "shell.image-picker.toml",
        "image-picker",
        "Image Picker",
        "The carousel for backgrounds and themes."
    ),
    shell_section!(
        "shell.controls.toml",
        "controls",
        "Controls",
        "Buttons, dropdowns, and tab strips in shell panels."
    ),
    shell_section!(
        "shell.hyprland.toml",
        "hyprland",
        "Shell Borders",
        "The border colors other shell surfaces refer to."
    ),
    shell_section!(
        "shell.spacing.toml",
        "spacing",
        "Spacing",
        "Margins, gaps, and padding across the shell."
    ),
    shell_section!(
        "shell.font.toml",
        "font",
        "Font Sizes",
        "The shell's type scale."
    ),
    OverrideSpec {
        file: "hyprland.lua",
        app: "Hyprland",
        description: "Window border colors, plus any extra Hyprland settings for this theme.",
        category: Category::Desktop,
        format: Format::Lua,
        editor: EditorKind::Source,
        seed: Seed::Template("hyprland.lua.tpl"),
        binaries: &[],
    },
    OverrideSpec {
        file: "gum_env.lua",
        app: "Terminal Menus",
        description: "Colors of Omarchy's terminal menus and prompts (gum).",
        category: Category::Desktop,
        format: Format::Lua,
        editor: EditorKind::Source,
        seed: Seed::Template("gum_env.lua.tpl"),
        binaries: &[],
    },
    OverrideSpec {
        file: "hyprland-preview-share-picker.css",
        app: "Screen Share Picker",
        description: "The window and screen picker shown when an app starts sharing.",
        category: Category::Desktop,
        format: Format::Css,
        editor: EditorKind::Source,
        seed: Seed::Template("hyprland-preview-share-picker.css.tpl"),
        binaries: &["hyprland-preview-share-picker"],
    },
    OverrideSpec {
        file: "icons.theme",
        app: "Icons",
        description: "The Yaru icon color for GTK apps and the file manager.",
        category: Category::Desktop,
        format: Format::Plain,
        editor: EditorKind::Form,
        seed: Seed::Fixed("Yaru-blue\n"),
        binaries: &[],
    },
    OverrideSpec {
        file: "keyboard.rgb",
        app: "Keyboard Lighting",
        description: "The backlight color of supported RGB keyboards.",
        category: Category::Desktop,
        format: Format::Plain,
        editor: EditorKind::Form,
        seed: Seed::Template("keyboard.rgb.tpl"),
        binaries: &["asusctl", "qmk_hid"],
    },
    OverrideSpec {
        file: "alacritty.toml",
        app: "Alacritty",
        description: "Alacritty's colors.",
        category: Category::Terminals,
        format: Format::Toml,
        editor: EditorKind::Source,
        seed: Seed::Template("alacritty.toml.tpl"),
        binaries: &["alacritty"],
    },
    OverrideSpec {
        file: "kitty.conf",
        app: "Kitty",
        description: "Kitty's colors.",
        category: Category::Terminals,
        format: Format::Ini,
        editor: EditorKind::Source,
        seed: Seed::Template("kitty.conf.tpl"),
        binaries: &["kitty"],
    },
    OverrideSpec {
        file: "ghostty.conf",
        app: "Ghostty",
        description: "Ghostty's colors.",
        category: Category::Terminals,
        format: Format::Ini,
        editor: EditorKind::Source,
        seed: Seed::Template("ghostty.conf.tpl"),
        binaries: &["ghostty"],
    },
    OverrideSpec {
        file: "foot.ini",
        app: "Foot",
        description: "Foot's colors.",
        category: Category::Terminals,
        format: Format::Ini,
        editor: EditorKind::Source,
        seed: Seed::Template("foot.ini.tpl"),
        binaries: &["foot"],
    },
    OverrideSpec {
        file: "neovim.lua",
        app: "Neovim",
        description: "The colorscheme LazyVim loads, or a plugin that provides one.",
        category: Category::Editors,
        format: Format::Lua,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("neovim.lua.tpl"),
        binaries: &["nvim"],
    },
    OverrideSpec {
        file: "vscode.json",
        app: "VS Code Extension",
        description: "A Marketplace theme VS Code, VSCodium, and Cursor install and switch to.",
        category: Category::Editors,
        format: Format::Json,
        editor: EditorKind::Form,
        seed: Seed::Fixed("{\n  \"name\": \"\",\n  \"extension\": \"\"\n}\n"),
        binaries: &["code", "code-insiders", "codium", "cursor"],
    },
    OverrideSpec {
        file: "vscode-theme.json",
        app: "VS Code Theme",
        description: "The color theme Omarchy installs into VS Code, VSCodium, and Cursor.",
        category: Category::Editors,
        format: Format::Json,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("vscode-theme.json.tpl"),
        binaries: &["code", "code-insiders", "codium", "cursor"],
    },
    OverrideSpec {
        file: "helix.toml",
        app: "Helix",
        description: "Helix's syntax and UI colors.",
        category: Category::Editors,
        format: Format::Toml,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("helix.toml.tpl"),
        binaries: &["helix", "hx"],
    },
    OverrideSpec {
        file: "obsidian.css",
        app: "Obsidian",
        description: "The Omarchy theme copied into every Obsidian vault.",
        category: Category::Editors,
        format: Format::Css,
        editor: EditorKind::Source,
        seed: Seed::Template("obsidian.css.tpl"),
        binaries: &["obsidian"],
    },
    OverrideSpec {
        file: "btop.theme",
        app: "btop",
        description: "btop's boxes, graphs, and text.",
        category: Category::Apps,
        format: Format::Ini,
        editor: EditorKind::Form,
        seed: Seed::Template("btop.theme.tpl"),
        binaries: &["btop"],
    },
    OverrideSpec {
        file: "chromium.theme",
        app: "Browsers",
        description: "The toolbar color of Chromium, Chrome, Edge, and Brave.",
        category: Category::Apps,
        format: Format::Plain,
        editor: EditorKind::Form,
        seed: Seed::Template("chromium.theme.tpl"),
        binaries: &[
            "chromium",
            "google-chrome-stable",
            "microsoft-edge-stable",
            "brave",
        ],
    },
    OverrideSpec {
        file: "claude.json",
        app: "Claude Code",
        description: "Claude Code's custom Omarchy theme.",
        category: Category::AiTools,
        format: Format::Json,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("claude.json.tpl"),
        binaries: &["claude"],
    },
    OverrideSpec {
        file: "pi.json",
        app: "Pi",
        description: "The Pi coding agent's theme.",
        category: Category::AiTools,
        format: Format::Json,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("pi.json.tpl"),
        binaries: &["pi"],
    },
    OverrideSpec {
        file: "hermes.yaml",
        app: "Hermes",
        description: "The Hermes skin for its desktop app, TUI, and CLI.",
        category: Category::AiTools,
        format: Format::Yaml,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("hermes.yaml.tpl"),
        binaries: &["hermes"],
    },
    OverrideSpec {
        file: "t3code.json",
        app: "T3 Code",
        description: "T3 Code's Omarchy theme.",
        category: Category::AiTools,
        format: Format::Json,
        editor: EditorKind::ColorMap,
        seed: Seed::Template("t3code.json.tpl"),
        binaries: &["t3"],
    },
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;

    use super::{OVERRIDES, Seed};
    use crate::system::omarchy_paths::themed_templates_dir;

    #[test]
    fn files_are_unique() {
        let mut seen = HashSet::new();
        for spec in OVERRIDES {
            assert!(seen.insert(spec.file), "duplicate override {}", spec.file);
        }
    }

    #[test]
    fn section_files_are_named_after_their_section() {
        for spec in OVERRIDES {
            if let Seed::ShellSection(section) = spec.seed {
                assert_eq!(spec.file, format!("shell.{section}.toml"));
            }
        }
    }

    #[test]
    fn templates_generate_the_file_they_override() {
        for spec in OVERRIDES {
            if let Seed::Template(template) = spec.seed {
                assert_eq!(template, format!("{}.tpl", spec.file), "{}", spec.file);
            }
        }
    }

    // A template Omarchy adds must get an entry here, or the Theme Designer
    // silently offers no way to override it.
    #[test]
    fn every_installed_template_has_an_override() {
        let Ok(entries) = fs::read_dir(themed_templates_dir()) else {
            return;
        };
        let covered: HashSet<&str> = OVERRIDES
            .iter()
            .filter_map(|spec| match spec.seed {
                Seed::Template(template) => Some(template),
                Seed::ShellSection(_) => Some("shell.toml.tpl"),
                Seed::Fixed(_) => None,
            })
            .collect();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".tpl") {
                assert!(covered.contains(name.as_str()), "no override for {name}");
            }
        }
    }
}
