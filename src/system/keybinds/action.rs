//! What a keybind does, expressed as the choices the dialog's builder
//! offers. Every action maps to exactly one dispatcher string, and a
//! dispatcher string maps back to an action when it has the shape the
//! builder would have produced.
use crate::system::keybinds::Dispatcher;

// MARK: Shell words

/// Quotes as Omarchy's `helpers.lua` does: single quotes, with embedded
/// single quotes closed, escaped, and reopened.
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Splits a POSIX shell command line into words, honouring single quotes,
/// double quotes, and backslash escapes. Unterminated quotes run to the end.
pub fn shell_split(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                in_word = true;
                for c in chars.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    current.push(c);
                }
            }
            '"' => {
                in_word = true;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some(e @ ('"' | '\\' | '$' | '`')) => current.push(e),
                            Some(other) => {
                                current.push('\\');
                                current.push(other);
                            }
                            None => current.push('\\'),
                        },
                        _ => current.push(c),
                    }
                }
            }
            '\\' => {
                in_word = true;
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            _ => {
                in_word = true;
                current.push(c);
            }
        }
    }
    if in_word {
        words.push(current);
    }
    words
}

fn join_words(words: &[String]) -> String {
    words
        .iter()
        .map(|w| {
            let plain = !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:=+@%,".contains(c));
            if plain { w.clone() } else { shell_quote(w) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// MARK: Omarchy catalog

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmarchyEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub group: &'static str,
    pub command: &'static str,
}

macro_rules! entries {
    ($($id:literal, $label:literal, $group:literal, $command:literal;)*) => {
        &[$(OmarchyEntry { id: $id, label: $label, group: $group, command: $command },)*]
    };
}

/// Omarchy's own commands, in the order the picker lists them.
pub const OMARCHY_ACTIONS: &[OmarchyEntry] = entries! {
    "terminal", "Terminal", "Apps", "omarchy-launch-terminal";
    "terminal-tmux", "Terminal with tmux", "Apps", "omarchy-launch-terminal-tmux";
    "terminal-herdr", "Terminal with Herdr", "Apps", "omarchy-launch-terminal-herdr";
    "browser", "Browser", "Apps", "omarchy-launch-browser";
    "browser-private", "Browser (private window)", "Apps", "omarchy-launch-browser --private";
    "editor", "Editor", "Apps", "omarchy-launch-editor";
    "nautilus", "File manager", "Apps", "omarchy-launch-nautilus";
    "nautilus-cwd", "File manager (current directory)", "Apps", "omarchy-launch-nautilus-cwd";
    "spotify", "Music (Spotify)", "Apps", "omarchy-launch-spotify";
    "signal", "Signal", "Apps", "omarchy-launch-signal";
    "1password", "1Password", "Apps", "omarchy-launch-1password";
    "docker-tui", "Docker (lazydocker)", "Apps", "omarchy-launch-tui omarchy-launch-docker-tui";
    "activity", "Activity monitor (btop)", "Apps", "omarchy-launch-tui btop";
    "calculator", "Calculator", "Apps", "omacalc";
    "config-editor", "Omarchy config editor", "Apps", "omarchy-launch-config-editor";
    "about", "About Omarchy", "Apps", "omarchy-launch-about";

    "menu", "Omarchy menu", "Menus", "omarchy-menu toggle";
    "menu-apps", "Apps menu", "Menus", "omarchy-menu toggle apps";
    "menu-capture", "Capture menu", "Menus", "omarchy-menu toggle capture";
    "menu-toggle", "Toggle menu", "Menus", "omarchy-menu toggle toggle";
    "menu-hardware", "Hardware menu", "Menus", "omarchy-menu toggle hardware";
    "menu-system", "System menu", "Menus", "omarchy-menu toggle system";
    "menu-background", "Background switcher", "Menus", "omarchy-menu toggle background";
    "menu-theme", "Theme menu", "Menus", "omarchy-menu toggle theme";
    "menu-share", "Share menu", "Menus", "omarchy-menu toggle share";
    "menu-reminder", "Set a reminder", "Menus", "omarchy-menu toggle reminder-set";
    "menu-keybindings", "Keybindings", "Menus", "omarchy-menu-keybindings";
    "menu-tmux-keybindings", "Tmux keybindings", "Menus", "omarchy-menu-tmux-keybindings";
    "menu-herdr-keybindings", "Herdr keybindings", "Menus", "omarchy-menu-herdr-keybindings";

    "panel-clipboard", "Clipboard manager", "Panels", "omarchy-shell shell toggle omarchy.clipboard";
    "panel-emojis", "Emojis", "Panels", "omarchy-shell shell toggle omarchy.emojis";
    "panel-audio", "Audio", "Panels", "omarchy-shell shell toggle omarchy.audio";
    "panel-bluetooth", "Bluetooth", "Panels", "omarchy-shell shell toggle omarchy.bluetooth";
    "panel-monitor", "Display", "Panels", "omarchy-shell shell toggle omarchy.monitor";
    "panel-clock", "Calendar", "Panels", "omarchy-shell shell toggle omarchy.clock";
    "panel-network", "Network", "Panels", "omarchy-shell shell toggle omarchy.network";
    "panel-power", "Power", "Panels", "omarchy-shell shell toggle omarchy.power";

    "screenshot", "Screenshot", "Capture", "omarchy-capture-screenshot";
    "screenrecord", "Screen recording", "Capture", "omarchy-capture-screenrecording --stop-recording || omarchy-menu toggle trigger.capture.screenrecord";
    "color-picker", "Color picker", "Capture", "pkill hyprpicker || hyprpicker -a";
    "ocr", "Extract text from screen (OCR)", "Capture", "omarchy-capture-text";
    "webcam-smaller", "Webcam overlay smaller", "Capture", "omarchy-capture-webcam-resize smaller";
    "webcam-larger", "Webcam overlay larger", "Capture", "omarchy-capture-webcam-resize larger";

    "notif-dismiss", "Dismiss last notification", "Notifications", "omarchy-shell notifications dismissOne";
    "notif-dismiss-all", "Dismiss all notifications", "Notifications", "omarchy-shell notifications dismissAll";
    "notif-invoke", "Open last notification", "Notifications", "omarchy-shell notifications invokeLast";
    "notif-history", "Notification history", "Notifications", "omarchy-shell notifications showHistory";
    "notif-time", "Show the time", "Notifications", "omarchy-notification-time";
    "notif-battery", "Show battery remaining", "Notifications", "omarchy-notification-battery";
    "notif-weather", "Toggle weather", "Notifications", "omarchy-notification-weather";
    "reminders-show", "Show reminders", "Notifications", "omarchy-reminder show";
    "reminders-clear", "Clear reminders", "Notifications", "omarchy-reminder clear";

    "volume-up", "Volume up", "Media", "omarchy-audio-output-volume raise";
    "volume-down", "Volume down", "Media", "omarchy-audio-output-volume lower";
    "volume-mute", "Mute", "Media", "omarchy-audio-output-volume mute-toggle";
    "mic-mute", "Mute microphone", "Media", "omarchy-audio-input-mute";
    "audio-output-switch", "Switch audio output", "Media", "omarchy-audio-output-switch";
    "audio-source-switch", "Switch media source", "Media", "omarchy-audio-source-switch";
    "media-next", "Next track", "Media", "omarchy-shell media next";
    "media-previous", "Previous track", "Media", "omarchy-shell media previous";
    "media-play-pause", "Play / pause", "Media", "omarchy-shell media playPause";
    "brightness-up", "Brightness up", "Media", "omarchy-brightness-display +5%";
    "brightness-down", "Brightness down", "Media", "omarchy-brightness-display 5%-";
    "kbd-brightness-up", "Keyboard brightness up", "Media", "omarchy-brightness-keyboard up";
    "kbd-brightness-down", "Keyboard brightness down", "Media", "omarchy-brightness-keyboard down";
    "kbd-backlight-cycle", "Keyboard backlight cycle", "Media", "omarchy-brightness-keyboard cycle";

    "window-transparency", "Toggle window transparency", "Window", "omarchy-hyprland-window-transparency-toggle";
    "window-gaps", "Toggle window gaps", "Window", "omarchy-hyprland-window-gaps-toggle";
    "window-square", "Toggle single-window square aspect", "Window", "omarchy-hyprland-window-single-square-aspect-toggle";
    "window-tiled-fullscreen", "Toggle tiled full screen", "Window", "omarchy-hyprland-window-tiled-fullscreen-toggle";
    "window-pop", "Pop window out (float and pin)", "Window", "omarchy-hyprland-window-pop";
    "window-width-save", "Save window width", "Window", "omarchy-hyprland-window-width save";
    "window-width-restore", "Restore window width", "Window", "omarchy-hyprland-window-width restore";
    "window-close-all", "Close all windows", "Window", "omarchy-hyprland-window-close-all";
    "workspace-layout", "Toggle workspace layout", "Window", "omarchy-hyprland-workspace-layout-toggle";

    "monitor-scale-up", "Monitor scaling up", "Display", "omarchy-hyprland-monitor-scaling up";
    "monitor-scale-down", "Monitor scaling down", "Display", "omarchy-hyprland-monitor-scaling down";
    "laptop-display", "Toggle laptop display", "Display", "omarchy-hyprland-monitor-internal toggle";
    "laptop-mirror", "Toggle laptop display mirroring", "Display", "omarchy-hyprland-monitor-internal-mirror toggle";
    "touchpad-toggle", "Toggle touchpad", "Display", "omarchy-toggle-touchpad";

    "lock", "Lock the screen", "System", "omarchy-system-lock";
    "screensaver", "Screensaver", "System", "omarchy-launch-screensaver";
    "dictation", "Toggle dictation", "System", "voxtype record toggle";
    "agent", "Agent", "System", "omarchy-agent --pick";
    "transcode", "Transcode", "System", "omarchy-transcode";
    "eject", "Eject media", "System", "eject";
};

pub fn omarchy_entry(id: &str) -> Option<&'static OmarchyEntry> {
    OMARCHY_ACTIONS.iter().find(|e| e.id == id)
}

fn omarchy_entry_for_command(command: &str) -> Option<&'static OmarchyEntry> {
    OMARCHY_ACTIONS.iter().find(|e| e.command == command)
}

// MARK: Window actions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Direction::Left => "Left",
            Direction::Right => "Right",
            Direction::Up => "Up",
            Direction::Down => "Down",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Direction::Left => "l",
            Direction::Right => "r",
            Direction::Up => "u",
            Direction::Down => "d",
        }
    }

    fn parse(code: &str) -> Option<Self> {
        match code {
            "l" => Some(Direction::Left),
            "r" => Some(Direction::Right),
            "u" => Some(Direction::Up),
            "d" => Some(Direction::Down),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceTarget {
    Number(u8),
    Next,
    Previous,
    /// The workspace that was active before the current one.
    Former,
}

impl WorkspaceTarget {
    pub fn label(self) -> String {
        match self {
            WorkspaceTarget::Number(n) => format!("Workspace {n}"),
            WorkspaceTarget::Next => "Next workspace".into(),
            WorkspaceTarget::Previous => "Previous workspace".into(),
            WorkspaceTarget::Former => "Former workspace".into(),
        }
    }

    fn code(self) -> String {
        match self {
            WorkspaceTarget::Number(n) => n.to_string(),
            WorkspaceTarget::Next => "e+1".into(),
            WorkspaceTarget::Previous => "e-1".into(),
            WorkspaceTarget::Former => "previous".into(),
        }
    }

    fn parse(code: &str) -> Option<Self> {
        match code {
            "e+1" | "+1" => Some(WorkspaceTarget::Next),
            "e-1" | "-1" => Some(WorkspaceTarget::Previous),
            "previous" => Some(WorkspaceTarget::Former),
            n => n.parse().ok().map(WorkspaceTarget::Number),
        }
    }
}

/// The builder's list of window actions; each variant is one row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowActionKind {
    Close,
    ToggleFloating,
    Fullscreen,
    Maximize,
    Pseudo,
    ToggleSplit,
    FocusWindow,
    SwapWindow,
    CycleNext,
    CyclePrevious,
    BringToTop,
    SwitchWorkspace,
    MoveToWorkspace,
    ToggleScratchpad,
    MoveToScratchpad,
    FocusNextMonitor,
    FocusPreviousMonitor,
    MoveWorkspaceToMonitor,
    ToggleGroup,
    GroupNext,
    GroupPrevious,
    MoveIntoGroup,
    MoveOutOfGroup,
    Resize,
}

/// Which extra control a window action needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowParam {
    None,
    Direction,
    Workspace,
    /// Workspace plus whether focus follows the window.
    WorkspaceMove,
    Resize,
}

impl WindowActionKind {
    pub const ALL: [WindowActionKind; 24] = [
        WindowActionKind::Close,
        WindowActionKind::ToggleFloating,
        WindowActionKind::Fullscreen,
        WindowActionKind::Maximize,
        WindowActionKind::Pseudo,
        WindowActionKind::ToggleSplit,
        WindowActionKind::FocusWindow,
        WindowActionKind::SwapWindow,
        WindowActionKind::CycleNext,
        WindowActionKind::CyclePrevious,
        WindowActionKind::BringToTop,
        WindowActionKind::Resize,
        WindowActionKind::SwitchWorkspace,
        WindowActionKind::MoveToWorkspace,
        WindowActionKind::ToggleScratchpad,
        WindowActionKind::MoveToScratchpad,
        WindowActionKind::FocusNextMonitor,
        WindowActionKind::FocusPreviousMonitor,
        WindowActionKind::MoveWorkspaceToMonitor,
        WindowActionKind::ToggleGroup,
        WindowActionKind::GroupNext,
        WindowActionKind::GroupPrevious,
        WindowActionKind::MoveIntoGroup,
        WindowActionKind::MoveOutOfGroup,
    ];

    pub fn id(self) -> &'static str {
        match self {
            WindowActionKind::Close => "close",
            WindowActionKind::ToggleFloating => "toggle-floating",
            WindowActionKind::Fullscreen => "fullscreen",
            WindowActionKind::Maximize => "maximize",
            WindowActionKind::Pseudo => "pseudo",
            WindowActionKind::ToggleSplit => "toggle-split",
            WindowActionKind::FocusWindow => "focus-window",
            WindowActionKind::SwapWindow => "swap-window",
            WindowActionKind::CycleNext => "cycle-next",
            WindowActionKind::CyclePrevious => "cycle-previous",
            WindowActionKind::BringToTop => "bring-to-top",
            WindowActionKind::SwitchWorkspace => "switch-workspace",
            WindowActionKind::MoveToWorkspace => "move-to-workspace",
            WindowActionKind::ToggleScratchpad => "toggle-scratchpad",
            WindowActionKind::MoveToScratchpad => "move-to-scratchpad",
            WindowActionKind::FocusNextMonitor => "focus-next-monitor",
            WindowActionKind::FocusPreviousMonitor => "focus-previous-monitor",
            WindowActionKind::MoveWorkspaceToMonitor => "move-workspace-to-monitor",
            WindowActionKind::ToggleGroup => "toggle-group",
            WindowActionKind::GroupNext => "group-next",
            WindowActionKind::GroupPrevious => "group-previous",
            WindowActionKind::MoveIntoGroup => "move-into-group",
            WindowActionKind::MoveOutOfGroup => "move-out-of-group",
            WindowActionKind::Resize => "resize",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|k| k.id() == id)
    }

    pub fn label(self) -> &'static str {
        match self {
            WindowActionKind::Close => "Close window",
            WindowActionKind::ToggleFloating => "Toggle floating",
            WindowActionKind::Fullscreen => "Full screen",
            WindowActionKind::Maximize => "Maximize (full width)",
            WindowActionKind::Pseudo => "Pseudo-tile",
            WindowActionKind::ToggleSplit => "Toggle split direction",
            WindowActionKind::FocusWindow => "Focus window in a direction",
            WindowActionKind::SwapWindow => "Swap window in a direction",
            WindowActionKind::CycleNext => "Focus next window",
            WindowActionKind::CyclePrevious => "Focus previous window",
            WindowActionKind::BringToTop => "Bring window to top",
            WindowActionKind::SwitchWorkspace => "Switch to workspace",
            WindowActionKind::MoveToWorkspace => "Move window to workspace",
            WindowActionKind::ToggleScratchpad => "Toggle scratchpad",
            WindowActionKind::MoveToScratchpad => "Move window to scratchpad",
            WindowActionKind::FocusNextMonitor => "Focus next monitor",
            WindowActionKind::FocusPreviousMonitor => "Focus previous monitor",
            WindowActionKind::MoveWorkspaceToMonitor => "Move workspace to monitor",
            WindowActionKind::ToggleGroup => "Toggle window group",
            WindowActionKind::GroupNext => "Next window in group",
            WindowActionKind::GroupPrevious => "Previous window in group",
            WindowActionKind::MoveIntoGroup => "Move window into group",
            WindowActionKind::MoveOutOfGroup => "Move window out of group",
            WindowActionKind::Resize => "Resize window",
        }
    }

    pub fn group(self) -> &'static str {
        match self {
            WindowActionKind::Close
            | WindowActionKind::ToggleFloating
            | WindowActionKind::Fullscreen
            | WindowActionKind::Maximize
            | WindowActionKind::Pseudo
            | WindowActionKind::ToggleSplit
            | WindowActionKind::FocusWindow
            | WindowActionKind::SwapWindow
            | WindowActionKind::CycleNext
            | WindowActionKind::CyclePrevious
            | WindowActionKind::BringToTop
            | WindowActionKind::Resize => "Window",
            WindowActionKind::SwitchWorkspace
            | WindowActionKind::MoveToWorkspace
            | WindowActionKind::ToggleScratchpad
            | WindowActionKind::MoveToScratchpad => "Workspace",
            WindowActionKind::FocusNextMonitor
            | WindowActionKind::FocusPreviousMonitor
            | WindowActionKind::MoveWorkspaceToMonitor => "Monitor",
            WindowActionKind::ToggleGroup
            | WindowActionKind::GroupNext
            | WindowActionKind::GroupPrevious
            | WindowActionKind::MoveIntoGroup
            | WindowActionKind::MoveOutOfGroup => "Group",
        }
    }

    pub fn param(self) -> WindowParam {
        match self {
            WindowActionKind::FocusWindow
            | WindowActionKind::SwapWindow
            | WindowActionKind::MoveWorkspaceToMonitor
            | WindowActionKind::MoveIntoGroup => WindowParam::Direction,
            WindowActionKind::SwitchWorkspace => WindowParam::Workspace,
            WindowActionKind::MoveToWorkspace => WindowParam::WorkspaceMove,
            WindowActionKind::Resize => WindowParam::Resize,
            _ => WindowParam::None,
        }
    }
}

/// A Hyprland dispatcher with its parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowAction {
    pub kind: WindowActionKind,
    pub direction: Direction,
    pub workspace: WorkspaceTarget,
    /// For `MoveToWorkspace`: focus follows the window.
    pub follow: bool,
    pub dx: i32,
    pub dy: i32,
}

impl WindowAction {
    pub fn new(kind: WindowActionKind) -> Self {
        Self {
            kind,
            direction: Direction::Left,
            workspace: WorkspaceTarget::Number(1),
            follow: true,
            dx: 100,
            dy: 0,
        }
    }

    /// "Focus window left", "Switch to workspace 3", or the kind's label.
    pub fn summary(&self) -> String {
        let direction = self.direction.label().to_lowercase();
        let workspace = self.workspace.label().to_lowercase();
        match self.kind {
            WindowActionKind::FocusWindow => format!("Focus window {direction}"),
            WindowActionKind::SwapWindow => format!("Swap window {direction}"),
            WindowActionKind::MoveIntoGroup => format!("Move window into group {direction}"),
            WindowActionKind::MoveWorkspaceToMonitor => {
                format!("Move workspace to {direction} monitor")
            }
            WindowActionKind::SwitchWorkspace => format!("Switch to {workspace}"),
            WindowActionKind::MoveToWorkspace => format!("Move window to {workspace}"),
            _ => self.kind.label().to_string(),
        }
    }

    /// The `hl.dsp.*` expression, formatted as Omarchy's own bindings are.
    pub fn lua(&self) -> String {
        let d = self.direction.code();
        match self.kind {
            WindowActionKind::Close => "hl.dsp.window.close()".into(),
            WindowActionKind::ToggleFloating => {
                "hl.dsp.window.float({ action = \"toggle\" })".into()
            }
            WindowActionKind::Fullscreen => {
                "hl.dsp.window.fullscreen({ mode = \"fullscreen\" })".into()
            }
            WindowActionKind::Maximize => {
                "hl.dsp.window.fullscreen({ mode = \"maximized\" })".into()
            }
            WindowActionKind::Pseudo => "hl.dsp.window.pseudo()".into(),
            WindowActionKind::ToggleSplit => "hl.dsp.layout(\"togglesplit\")".into(),
            WindowActionKind::FocusWindow => format!("hl.dsp.focus({{ direction = \"{d}\" }})"),
            WindowActionKind::SwapWindow => {
                format!("hl.dsp.window.swap({{ direction = \"{d}\" }})")
            }
            WindowActionKind::CycleNext => "hl.dsp.window.cycle_next()".into(),
            WindowActionKind::CyclePrevious => "hl.dsp.window.cycle_next({ next = false })".into(),
            WindowActionKind::BringToTop => "hl.dsp.window.bring_to_top()".into(),
            WindowActionKind::SwitchWorkspace => {
                format!(
                    "hl.dsp.focus({{ workspace = \"{}\" }})",
                    self.workspace.code()
                )
            }
            WindowActionKind::MoveToWorkspace => {
                let ws = self.workspace.code();
                if self.follow {
                    format!("hl.dsp.window.move({{ workspace = \"{ws}\" }})")
                } else {
                    format!("hl.dsp.window.move({{ workspace = \"{ws}\", follow = false }})")
                }
            }
            WindowActionKind::ToggleScratchpad => {
                "hl.dsp.workspace.toggle_special(\"scratchpad\")".into()
            }
            WindowActionKind::MoveToScratchpad => {
                "hl.dsp.window.move({ workspace = \"special:scratchpad\", follow = false })".into()
            }
            WindowActionKind::FocusNextMonitor => "hl.dsp.focus({ monitor = \"+1\" })".into(),
            WindowActionKind::FocusPreviousMonitor => "hl.dsp.focus({ monitor = \"-1\" })".into(),
            WindowActionKind::MoveWorkspaceToMonitor => {
                format!("hl.dsp.workspace.move({{ monitor = \"{d}\" }})")
            }
            WindowActionKind::ToggleGroup => "hl.dsp.group.toggle()".into(),
            WindowActionKind::GroupNext => "hl.dsp.group.next()".into(),
            WindowActionKind::GroupPrevious => "hl.dsp.group.prev()".into(),
            WindowActionKind::MoveIntoGroup => {
                format!("hl.dsp.window.move({{ into_group = \"{d}\" }})")
            }
            WindowActionKind::MoveOutOfGroup => {
                "hl.dsp.window.move({ out_of_group = true })".into()
            }
            WindowActionKind::Resize => format!(
                "hl.dsp.window.resize({{ x = {}, y = {}, relative = true }})",
                self.dx, self.dy
            ),
        }
    }

    /// Recognises the expressions [`Self::lua`] produces, tolerating
    /// whitespace and key-order differences.
    pub fn parse(expr: &str) -> Option<Self> {
        let (name, args) = split_call(expr)?;
        let table = parse_table(&args);
        let str_arg = |key: &str| table.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
        let literal = args.trim().trim_matches('"').to_string();
        let action = |kind| Some(WindowAction::new(kind));

        match name {
            "window.close" => action(WindowActionKind::Close),
            "window.float" => action(WindowActionKind::ToggleFloating),
            "window.fullscreen" => match str_arg("mode").as_deref() {
                Some("maximized") => action(WindowActionKind::Maximize),
                _ => action(WindowActionKind::Fullscreen),
            },
            "window.pseudo" => action(WindowActionKind::Pseudo),
            "layout" if literal == "togglesplit" => action(WindowActionKind::ToggleSplit),
            "window.cycle_next" => match str_arg("next").as_deref() {
                Some("false") => action(WindowActionKind::CyclePrevious),
                _ => action(WindowActionKind::CycleNext),
            },
            "window.bring_to_top" => action(WindowActionKind::BringToTop),
            "workspace.toggle_special" if literal == "scratchpad" => {
                action(WindowActionKind::ToggleScratchpad)
            }
            "group.toggle" => action(WindowActionKind::ToggleGroup),
            "group.next" => action(WindowActionKind::GroupNext),
            "group.prev" => action(WindowActionKind::GroupPrevious),
            "focus" => {
                if let Some(d) = str_arg("direction") {
                    let mut a = WindowAction::new(WindowActionKind::FocusWindow);
                    a.direction = Direction::parse(&d)?;
                    Some(a)
                } else if let Some(ws) = str_arg("workspace") {
                    let mut a = WindowAction::new(WindowActionKind::SwitchWorkspace);
                    a.workspace = WorkspaceTarget::parse(&ws)?;
                    Some(a)
                } else {
                    match str_arg("monitor").as_deref() {
                        Some("+1") => action(WindowActionKind::FocusNextMonitor),
                        Some("-1") => action(WindowActionKind::FocusPreviousMonitor),
                        _ => None,
                    }
                }
            }
            "window.swap" => {
                let mut a = WindowAction::new(WindowActionKind::SwapWindow);
                a.direction = Direction::parse(&str_arg("direction")?)?;
                Some(a)
            }
            "workspace.move" => {
                let mut a = WindowAction::new(WindowActionKind::MoveWorkspaceToMonitor);
                a.direction = Direction::parse(&str_arg("monitor")?)?;
                Some(a)
            }
            "window.move" => {
                if let Some(d) = str_arg("into_group") {
                    let mut a = WindowAction::new(WindowActionKind::MoveIntoGroup);
                    a.direction = Direction::parse(&d)?;
                    Some(a)
                } else if str_arg("out_of_group").as_deref() == Some("true") {
                    action(WindowActionKind::MoveOutOfGroup)
                } else {
                    let ws = str_arg("workspace")?;
                    let follow = str_arg("follow").as_deref() != Some("false");
                    if ws == "special:scratchpad" && !follow {
                        return action(WindowActionKind::MoveToScratchpad);
                    }
                    let mut a = WindowAction::new(WindowActionKind::MoveToWorkspace);
                    a.workspace = WorkspaceTarget::parse(&ws)?;
                    a.follow = follow;
                    Some(a)
                }
            }
            "window.resize" => {
                if str_arg("relative").as_deref() != Some("true") {
                    return None;
                }
                let mut a = WindowAction::new(WindowActionKind::Resize);
                a.dx = str_arg("x")?.parse().ok()?;
                a.dy = str_arg("y")?.parse().ok()?;
                Some(a)
            }
            _ => None,
        }
    }
}

/// Splits `hl.dsp.<name>(<args>)` into its name and raw argument text.
fn split_call(expr: &str) -> Option<(&str, String)> {
    let expr = expr.trim();
    let rest = expr.strip_prefix("hl.dsp.")?;
    let open = rest.find('(')?;
    let name = &rest[..open];
    let args = rest[open + 1..].strip_suffix(')')?;
    Some((name, args.to_string()))
}

/// Parses `{ key = value, ... }` into pairs; values lose their quotes.
fn parse_table(args: &str) -> Vec<(String, String)> {
    let inner = args.trim();
    let Some(inner) = inner.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return Vec::new();
    };
    inner
        .split(',')
        .filter_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            Some((k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        })
        .collect()
}

// MARK: Action

/// An application chosen in the builder. Everything the launch command
/// needs is carried here so the action stays valid if the entry vanishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLaunch {
    pub exec: String,
    pub wm_class: String,
    pub terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    App {
        app: AppLaunch,
        focus: bool,
    },
    WebApp {
        url: String,
        name: String,
        focus: bool,
    },
    Terminal {
        command: String,
        focus: bool,
    },
    Omarchy(&'static OmarchyEntry),
    Window(WindowAction),
    /// Runs a flow from the Flows page by id.
    Flow(String),
    Command(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    App,
    WebApp,
    Terminal,
    Omarchy,
    Window,
    Flow,
    Command,
}

impl ActionKind {
    pub const ALL: [ActionKind; 7] = [
        ActionKind::App,
        ActionKind::WebApp,
        ActionKind::Terminal,
        ActionKind::Omarchy,
        ActionKind::Window,
        ActionKind::Flow,
        ActionKind::Command,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ActionKind::App => "App",
            ActionKind::WebApp => "Web app",
            ActionKind::Terminal => "Terminal",
            ActionKind::Omarchy => "Omarchy",
            ActionKind::Window => "Window",
            ActionKind::Flow => "Flow",
            ActionKind::Command => "Command",
        }
    }

    /// The Lucide icon under `assets/icons/`.
    pub fn icon_path(self) -> &'static str {
        match self {
            ActionKind::App => "icons/app-window.svg",
            ActionKind::WebApp => "icons/globe.svg",
            ActionKind::Terminal => "icons/square-terminal.svg",
            ActionKind::Omarchy => "icons/sparkles.svg",
            ActionKind::Window => "icons/layout-grid.svg",
            ActionKind::Flow => "icons/workflow.svg",
            ActionKind::Command => "icons/terminal.svg",
        }
    }
}

/// The program a command line starts, without its directory.
pub fn program_name(exec: &str) -> &str {
    let first = exec.split_whitespace().next().unwrap_or(exec);
    first.rsplit('/').next().unwrap_or(first)
}

impl Action {
    pub fn kind(&self) -> ActionKind {
        match self {
            Action::App { .. } => ActionKind::App,
            Action::WebApp { .. } => ActionKind::WebApp,
            Action::Terminal { .. } => ActionKind::Terminal,
            Action::Omarchy(_) => ActionKind::Omarchy,
            Action::Window(_) => ActionKind::Window,
            Action::Flow(_) => ActionKind::Flow,
            Action::Command(_) => ActionKind::Command,
        }
    }

    /// A short description of the action for a bind or a flow step, e.g.
    /// "Focus window left". Apps are named by their program and flows by
    /// their id; callers with the desktop entry or the flow at hand
    /// substitute the friendlier name. `None` for a bare command.
    pub fn summary(&self) -> Option<String> {
        match self {
            Action::App { app, .. } => Some(program_name(&app.exec).to_string()),
            Action::WebApp { url, name, .. } => Some(if name.is_empty() {
                capitalize(&webapp_name(url))
            } else {
                name.clone()
            }),
            Action::Terminal { command, .. } => {
                command.split_whitespace().next().map(str::to_string)
            }
            Action::Omarchy(entry) => Some(entry.label.to_string()),
            Action::Window(action) => Some(action.summary()),
            Action::Flow(id) => Some(format!("Run flow {id}")),
            Action::Command(_) => None,
        }
    }

    pub fn dispatcher(&self) -> Dispatcher {
        match self {
            Action::App { app, focus } => {
                let launch = if app.terminal {
                    format!("omarchy-launch-tui {}", app.exec)
                } else {
                    format!("uwsm-app -- {}", app.exec)
                };
                Dispatcher::Exec(if *focus {
                    format!(
                        "omarchy-launch-or-focus {} {}",
                        shell_quote(&app.wm_class),
                        shell_quote(&launch)
                    )
                } else {
                    launch
                })
            }
            Action::WebApp { url, name, focus } => Dispatcher::Exec(if *focus {
                let name = if name.trim().is_empty() {
                    webapp_name(url)
                } else {
                    name.trim().to_string()
                };
                format!(
                    "omarchy-launch-or-focus-webapp {} {}",
                    shell_quote(&name),
                    shell_quote(url.trim())
                )
            } else {
                format!("omarchy-launch-webapp {}", shell_quote(url.trim()))
            }),
            Action::Terminal { command, focus } => Dispatcher::Exec(format!(
                "{} {}",
                if *focus {
                    "omarchy-launch-or-focus-tui"
                } else {
                    "omarchy-launch-tui"
                },
                command.trim()
            )),
            Action::Omarchy(entry) => Dispatcher::Exec(entry.command.to_string()),
            Action::Window(action) => Dispatcher::Lua(action.lua()),
            Action::Flow(id) => Dispatcher::Exec(crate::system::flows::run_command(id)),
            Action::Command(command) => Dispatcher::Exec(command.trim().to_string()),
        }
    }

    /// Reads a dispatcher back into builder terms. `None` for Lua
    /// expressions the builder cannot express and for Lua functions.
    pub fn from_dispatcher(dispatcher: &Dispatcher) -> Option<Action> {
        match dispatcher {
            Dispatcher::Function => None,
            Dispatcher::Lua(expr) => WindowAction::parse(expr).map(Action::Window),
            Dispatcher::Exec(command) => Some(Self::from_exec(command)),
        }
    }

    fn from_exec(command: &str) -> Action {
        if let Some(id) = crate::system::flows::run_command_id(command) {
            return Action::Flow(id);
        }
        if let Some(entry) = omarchy_entry_for_command(command.trim()) {
            return Action::Omarchy(entry);
        }
        let words = shell_split(command);
        let Some((head, rest)) = words.split_first() else {
            return Action::Command(command.to_string());
        };
        match (head.as_str(), rest) {
            ("omarchy-launch-webapp", [url, ..]) => Action::WebApp {
                url: url.clone(),
                name: String::new(),
                focus: false,
            },
            ("omarchy-launch-or-focus-webapp", [name, url, ..]) => Action::WebApp {
                url: url.clone(),
                name: name.clone(),
                focus: true,
            },
            ("omarchy-launch-tui", cmd) if !cmd.is_empty() => Action::Terminal {
                command: join_words(cmd),
                focus: false,
            },
            ("omarchy-launch-or-focus-tui", cmd) if !cmd.is_empty() => Action::Terminal {
                command: join_words(cmd),
                focus: true,
            },
            ("omarchy-launch-or-focus", [pattern, launch]) => match Self::from_exec(launch) {
                Action::App { mut app, .. } => {
                    app.wm_class = pattern.clone();
                    Action::App { app, focus: true }
                }
                Action::Terminal { command, .. } => Action::Terminal {
                    command,
                    focus: true,
                },
                _ => Action::Command(command.to_string()),
            },
            ("uwsm-app", [dashes, exec @ ..]) if dashes == "--" && !exec.is_empty() => {
                Action::App {
                    app: AppLaunch {
                        exec: join_words(exec),
                        wm_class: exec[0].rsplit('/').next().unwrap_or(&exec[0]).to_string(),
                        terminal: false,
                    },
                    focus: false,
                }
            }
            _ => Action::Command(command.to_string()),
        }
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// A window-title pattern for a URL, used when a web app has no name yet.
pub fn webapp_name(url: &str) -> String {
    let host = url
        .trim()
        .split("://")
        .nth(1)
        .unwrap_or(url.trim())
        .split('/')
        .next()
        .unwrap_or_default();
    let host = host.strip_prefix("www.").unwrap_or(host);
    host.split('.').next().unwrap_or(host).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_words_round_trip() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(
            shell_split("omarchy-launch-or-focus '^obsidian$' 'uwsm-app -- obsidian'"),
            vec![
                "omarchy-launch-or-focus",
                "^obsidian$",
                "uwsm-app -- obsidian"
            ]
        );
        assert_eq!(
            shell_split(r#"a "b c" d\ e 'it'\''s'"#),
            vec!["a", "b c", "d e", "it's"]
        );
    }

    #[test]
    fn flow_actions_round_trip_and_summarize() {
        let action = Action::Flow("morning-start".into());
        let dispatcher = action.dispatcher();
        assert_eq!(
            dispatcher,
            Dispatcher::Exec("omarchist flow run 'morning-start'".into())
        );
        assert_eq!(Action::from_dispatcher(&dispatcher), Some(action.clone()));
        assert_eq!(action.kind(), ActionKind::Flow);
        assert_eq!(action.summary().as_deref(), Some("Run flow morning-start"));

        let mut focus = WindowAction::new(WindowActionKind::FocusWindow);
        focus.direction = Direction::Right;
        assert_eq!(
            Action::Window(focus).summary().as_deref(),
            Some("Focus window right")
        );
        assert_eq!(
            Action::WebApp {
                url: "https://mail.google.com/".into(),
                name: String::new(),
                focus: false
            }
            .summary()
            .as_deref(),
            Some("Mail")
        );
        assert_eq!(Action::Command("ls".into()).summary(), None);
    }

    #[test]
    fn app_actions_emit_omarchy_launch_commands() {
        let app = AppLaunch {
            exec: "obsidian".into(),
            wm_class: "obsidian".into(),
            terminal: false,
        };
        let plain = Action::App {
            app: app.clone(),
            focus: false,
        };
        assert_eq!(
            plain.dispatcher(),
            Dispatcher::Exec("uwsm-app -- obsidian".into())
        );
        let focus = Action::App { app, focus: true };
        assert_eq!(
            focus.dispatcher(),
            Dispatcher::Exec("omarchy-launch-or-focus 'obsidian' 'uwsm-app -- obsidian'".into())
        );
        assert_eq!(Action::from_dispatcher(&focus.dispatcher()), Some(focus));
        assert_eq!(Action::from_dispatcher(&plain.dispatcher()), Some(plain));
    }

    #[test]
    fn webapp_and_terminal_actions_round_trip() {
        let web = Action::WebApp {
            url: "https://chatgpt.com".into(),
            name: String::new(),
            focus: false,
        };
        assert_eq!(
            web.dispatcher(),
            Dispatcher::Exec("omarchy-launch-webapp 'https://chatgpt.com'".into())
        );
        assert_eq!(Action::from_dispatcher(&web.dispatcher()), Some(web));

        let sole = Action::WebApp {
            url: "https://web.whatsapp.com/".into(),
            name: "WhatsApp".into(),
            focus: true,
        };
        assert_eq!(
            sole.dispatcher(),
            Dispatcher::Exec(
                "omarchy-launch-or-focus-webapp 'WhatsApp' 'https://web.whatsapp.com/'".into()
            )
        );
        assert_eq!(Action::from_dispatcher(&sole.dispatcher()), Some(sole));

        let tui = Action::Terminal {
            command: "cliamp".into(),
            focus: true,
        };
        assert_eq!(
            tui.dispatcher(),
            Dispatcher::Exec("omarchy-launch-or-focus-tui cliamp".into())
        );
        assert_eq!(Action::from_dispatcher(&tui.dispatcher()), Some(tui));
    }

    #[test]
    fn omarchy_catalog_matches_default_commands() {
        let d = Dispatcher::Exec("omarchy-shell shell toggle omarchy.clipboard".into());
        assert!(matches!(
            Action::from_dispatcher(&d),
            Some(Action::Omarchy(e)) if e.id == "panel-clipboard"
        ));
        let ids: Vec<_> = OMARCHY_ACTIONS.iter().map(|e| e.id).collect();
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(ids.len(), unique.len(), "catalog ids are unique");
    }

    #[test]
    fn window_actions_round_trip_through_lua() {
        for kind in WindowActionKind::ALL {
            let mut action = WindowAction::new(kind);
            action.direction = Direction::Up;
            action.workspace = WorkspaceTarget::Number(3);
            action.follow = false;
            action.dx = -25;
            action.dy = 10;
            let lua = action.lua();
            assert!(lua.starts_with("hl.dsp."), "{lua}");
            let parsed = WindowAction::parse(&lua).unwrap_or_else(|| panic!("parse {lua}"));
            assert_eq!(parsed.kind, kind, "{lua}");
            assert_eq!(parsed.lua(), lua);
        }
    }

    #[test]
    fn window_actions_parse_omarchy_defaults() {
        let cases = [
            (
                "hl.dsp.focus({ direction = \"l\" })",
                WindowActionKind::FocusWindow,
            ),
            (
                "hl.dsp.focus({ workspace = \"e+1\" })",
                WindowActionKind::SwitchWorkspace,
            ),
            (
                "hl.dsp.focus({ workspace = \"previous\" })",
                WindowActionKind::SwitchWorkspace,
            ),
            (
                "hl.dsp.window.move({ workspace = \"special:scratchpad\", follow = false })",
                WindowActionKind::MoveToScratchpad,
            ),
            (
                "hl.dsp.window.cycle_next({ next = false })",
                WindowActionKind::CyclePrevious,
            ),
            (
                "hl.dsp.window.resize({ x = 0, y = -300, relative = true })",
                WindowActionKind::Resize,
            ),
            (
                "hl.dsp.window.fullscreen({ mode = \"maximized\" })",
                WindowActionKind::Maximize,
            ),
        ];
        for (lua, kind) in cases {
            assert_eq!(
                WindowAction::parse(lua).map(|a| a.kind),
                Some(kind),
                "{lua}"
            );
        }
        assert!(WindowAction::parse("hl.dsp.window.drag()").is_none());
        assert!(WindowAction::parse("hl.dsp.send_key_state({})").is_none());
    }

    #[test]
    fn unknown_commands_fall_back_to_command() {
        assert_eq!(
            Action::from_dispatcher(&Dispatcher::Exec("pkill foo || bar".into())),
            Some(Action::Command("pkill foo || bar".into()))
        );
        assert!(Action::from_dispatcher(&Dispatcher::Function).is_none());
        assert_eq!(webapp_name("https://www.youtube.com/"), "youtube");
    }
}
