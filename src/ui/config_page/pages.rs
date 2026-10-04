//! The Configuration page's content as data: pages of groups of items.
//! A Hyprland item is one option addressed by its dotted path
//! (`input.touchpad.tap_to_click`); ranges and the values of choices come
//! from `hyprctl descriptions`, and a test checks them against the running
//! compositor. An Omarchy item is a [`Backing`]: read and written through
//! Omarchy's own files and scripts (`system::omarchy_settings`).
use crate::system::omarchy_settings::{Backing, Options, Parse, Read, Write};

/// Where an item's value lives.
pub enum Source {
    /// A Hyprland option by dotted path.
    Hyprland(&'static str),
    /// An Omarchy setting.
    Omarchy(Backing),
    /// An action with no value.
    None,
}

/// Which product a page belongs to; the nav groups pages by it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageGroup {
    Hyprland,
    Omarchy,
}

/// How an item is edited.
pub enum FieldDef {
    Number {
        min: f64,
        max: f64,
        step: f64,
        /// Written as an integer, as Hyprland declares the option.
        integer: bool,
    },
    /// One component of a two-number option (`decoration.shadow.offset`).
    Pair {
        index: usize,
        min: f64,
        max: f64,
        step: f64,
    },
    Switch,
    /// A string option with a fixed set of values.
    Dropdown {
        options: &'static [(&'static str, &'static str)],
    },
    /// An integer option whose values have names.
    Choice {
        options: &'static [(i64, &'static str)],
    },
    KeyboardLayout,
    /// A string setting whose choices are found at runtime.
    DynamicDropdown {
        options: Options,
    },
    /// A button that runs a command; in Omarchy's floating terminal when
    /// the command asks for sudo or confirmation.
    Action {
        button: &'static str,
        argv: &'static [&'static str],
        terminal: bool,
    },
    /// Something that is set up or not: a status read at runtime, a setup
    /// command, and a removal command, both run in the floating terminal.
    Feature {
        status: Read,
        setup: &'static [&'static str],
        remove: Option<&'static [&'static str]>,
    },
}

pub struct ItemDef {
    pub id: &'static str,
    pub source: Source,
    pub label: &'static str,
    pub description: &'static str,
    pub field: FieldDef,
    /// A command that must succeed for the item to be shown
    /// (`omarchy-hw-laptop`).
    pub when: Option<&'static [&'static str]>,
    /// The id of a switch item that, while on, makes this item's value
    /// irrelevant (an Omarchy toggle that overrides it), so the control is
    /// disabled.
    pub disabled_by: Option<&'static str>,
}

impl ItemDef {
    /// The Hyprland option path, for Hyprland items.
    pub fn hyprland_path(&self) -> Option<&'static str> {
        match self.source {
            Source::Hyprland(path) => Some(path),
            _ => None,
        }
    }
}

pub struct GroupDef {
    pub title: &'static str,
    pub items: &'static [ItemDef],
}

pub struct PageDef {
    pub title: &'static str,
    pub description: &'static str,
    pub group: PageGroup,
    pub groups: &'static [GroupDef],
}

/// The Hyprland option older versions overrode directly; the layout now
/// goes through `/etc/vconsole.conf`, which Omarchy's `input.lua` reads.
pub const KEYBOARD_LAYOUT_PATH: &str = "input.kb_layout";

/// The system keyboard layout: read from vconsole, set with `localectl`
/// (which asks for the password), then Hyprland reloads so Omarchy's
/// `input.lua` picks it up.
pub const KEYBOARD_LAYOUT: Backing = Backing {
    read: Read::EnvFile {
        path: "/etc/vconsole.conf",
        key: "XKBLAYOUT",
    },
    write: Write::X11Keymap,
};

/// Every page in nav order: Hyprland first, then Omarchy.
pub fn pages() -> impl Iterator<Item = &'static PageDef> {
    HYPRLAND_PAGES.iter().chain(OMARCHY_PAGES.iter())
}

pub fn page(index: usize) -> Option<&'static PageDef> {
    pages().nth(index)
}

pub fn page_count() -> usize {
    HYPRLAND_PAGES.len() + OMARCHY_PAGES.len()
}

/// Every item of every page.
pub fn items() -> impl Iterator<Item = &'static ItemDef> {
    pages().flat_map(|p| p.groups).flat_map(|g| g.items)
}

macro_rules! int_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                integer: true,
            },
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! float_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                integer: false,
            },
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! pair_item {
    ($id:expr, $path:expr, $index:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Pair {
                index: $index,
                min: $min,
                max: $max,
                step: $step,
            },
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! switch_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Switch,
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! dropdown_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $options:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Dropdown { options: $options },
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! choice_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $options:expr) => {
        ItemDef {
            id: $id,
            source: Source::Hyprland($path),
            label: $label,
            description: $desc,
            field: FieldDef::Choice { options: $options },
            when: None,
            disabled_by: None,
        }
    };
}

const ON_OFF_AUTO: &[(i64, &str)] = &[(0, "Off"), (1, "On"), (2, "Auto")];

pub const HYPRLAND_PAGES: &[PageDef] = &[
    PageDef {
        title: "General",
        group: PageGroup::Hyprland,
        description: "Borders, gaps, layout, and floating windows",
        groups: &[
            GroupDef {
                title: "Window Borders",
                items: &[
                    ItemDef {
                        id: "border-size",
                        source: Source::Hyprland("general.border_size"),
                        label: "Border Size",
                        description: "Size of the border around windows",
                        field: FieldDef::Number {
                            min: 0.0,
                            max: 20.0,
                            step: 1.0,
                            integer: true,
                        },
                        when: None,
                        disabled_by: Some("no-gaps"),
                    },
                    switch_item!(
                        "resize-on-border",
                        "general.resize_on_border",
                        "Resize on Border",
                        "Resize windows by dragging their borders and gaps"
                    ),
                    int_item!(
                        "extend-border-grab-area",
                        "general.extend_border_grab_area",
                        "Border Grab Area",
                        "Extra pixels around the border that resize the window",
                        0.0,
                        100.0,
                        1.0
                    ),
                    switch_item!(
                        "hover-icon-on-border",
                        "general.hover_icon_on_border",
                        "Resize Cursor on Border",
                        "Show a resize cursor when hovering a border"
                    ),
                    switch_item!(
                        "border-part-of-window",
                        "decoration.border_part_of_window",
                        "Border Part of Window",
                        "Treat the border as part of the window"
                    ),
                ],
            },
            GroupDef {
                title: "Gaps",
                items: &[
                    ItemDef {
                        id: "gaps-in",
                        source: Source::Hyprland("general.gaps_in"),
                        label: "Gaps In",
                        description: "Gaps between windows",
                        field: FieldDef::Number {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            integer: true,
                        },
                        when: None,
                        disabled_by: Some("no-gaps"),
                    },
                    ItemDef {
                        id: "gaps-out",
                        source: Source::Hyprland("general.gaps_out"),
                        label: "Gaps Out",
                        description: "Gaps between windows and monitor edges",
                        field: FieldDef::Number {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            integer: true,
                        },
                        when: None,
                        disabled_by: Some("no-gaps"),
                    },
                    int_item!(
                        "float-gaps",
                        "general.float_gaps",
                        "Floating Gaps",
                        "Gaps between floating windows and monitor edges",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "gaps-workspaces",
                        "general.gaps_workspaces",
                        "Gaps Workspaces",
                        "Gaps between workspaces. Stacks with gaps out",
                        0.0,
                        100.0,
                        1.0
                    ),
                    ItemDef {
                        id: "no-gaps",
                        source: Source::Omarchy(Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/window-no-gaps.lua",
                                inverted: false,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-hyprland-toggle", "window-no-gaps", "on"]],
                                off: &[&["omarchy-hyprland-toggle", "window-no-gaps", "off"]],
                            },
                        }),
                        label: "No Gaps",
                        description: "Omarchy's toggle: no gaps, borders, or rounding, overriding the fields above",
                        field: FieldDef::Switch,
                        when: None,
                        disabled_by: None,
                    },
                ],
            },
            GroupDef {
                title: "Layout",
                items: &[
                    dropdown_item!(
                        "layout",
                        "general.layout",
                        "Layout",
                        "How windows are tiled",
                        &[
                            ("dwindle", "Dwindle"),
                            ("master", "Master"),
                            ("scrolling", "Scrolling"),
                            ("monocle", "Monocle"),
                        ]
                    ),
                    switch_item!(
                        "no-focus-fallback",
                        "general.no_focus_fallback",
                        "No Focus Fallback",
                        "Do not fall back to the next window when moving focus fails"
                    ),
                    switch_item!(
                        "allow-tearing",
                        "general.allow_tearing",
                        "Allow Tearing",
                        "Let windows that ask for it tear, for lower latency in games"
                    ),
                ],
            },
            GroupDef {
                title: "Floating Windows",
                items: &[
                    choice_item!(
                        "resize-corner",
                        "general.resize_corner",
                        "Resize Corner",
                        "Corner used when resizing a floating window",
                        &[
                            (0, "Nearest"),
                            (1, "Top left"),
                            (2, "Top right"),
                            (3, "Bottom right"),
                            (4, "Bottom left"),
                        ]
                    ),
                    switch_item!(
                        "snap-enabled",
                        "general.snap.enabled",
                        "Snapping",
                        "Snap floating windows to windows and edges"
                    ),
                    int_item!(
                        "snap-window-gap",
                        "general.snap.window_gap",
                        "Snap Window Gap",
                        "Distance from a window at which snapping starts",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "snap-monitor-gap",
                        "general.snap.monitor_gap",
                        "Snap Monitor Gap",
                        "Distance from a monitor edge at which snapping starts",
                        0.0,
                        100.0,
                        1.0
                    ),
                    switch_item!(
                        "snap-border-overlap",
                        "general.snap.border_overlap",
                        "Snap Borders Overlap",
                        "Leave one border's width between snapped windows"
                    ),
                    switch_item!(
                        "snap-respect-gaps",
                        "general.snap.respect_gaps",
                        "Snap Respects Gaps",
                        "Keep the configured gaps when snapping"
                    ),
                    switch_item!(
                        "modal-parent-blocking",
                        "general.modal_parent_blocking",
                        "Block Parents of Modals",
                        "Make a window inert while one of its dialogs is open"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Appearance",
        group: PageGroup::Hyprland,
        description: "Rounding, opacity, dimming, blur, shadows, and animations",
        groups: &[
            GroupDef {
                title: "Rounding",
                items: &[
                    ItemDef {
                        id: "rounding",
                        source: Source::Hyprland("decoration.rounding"),
                        label: "Rounding",
                        description: "Corner radius in pixels",
                        field: FieldDef::Number {
                            min: 0.0,
                            max: 20.0,
                            step: 1.0,
                            integer: true,
                        },
                        when: None,
                        disabled_by: Some("no-gaps"),
                    },
                    float_item!(
                        "rounding-power",
                        "decoration.rounding_power",
                        "Rounding Power",
                        "Corner shape: 2 is a circle, higher is squarer",
                        2.0,
                        10.0,
                        0.5
                    ),
                ],
            },
            GroupDef {
                title: "Opacity",
                items: &[
                    float_item!(
                        "active-opacity",
                        "decoration.active_opacity",
                        "Active Opacity",
                        "Opacity of the focused window",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "inactive-opacity",
                        "decoration.inactive_opacity",
                        "Inactive Opacity",
                        "Opacity of unfocused windows",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "fullscreen-opacity",
                        "decoration.fullscreen_opacity",
                        "Fullscreen Opacity",
                        "Opacity of fullscreen windows",
                        0.0,
                        1.0,
                        0.05
                    ),
                ],
            },
            GroupDef {
                title: "Dimming",
                items: &[
                    switch_item!(
                        "dim-inactive",
                        "decoration.dim_inactive",
                        "Dim Inactive Windows",
                        "Darken windows that do not have focus"
                    ),
                    float_item!(
                        "dim-strength",
                        "decoration.dim_strength",
                        "Dim Strength",
                        "How much inactive windows are darkened",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "dim-special",
                        "decoration.dim_special",
                        "Dim Behind Special Workspace",
                        "How much the screen darkens behind a special workspace",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "dim-around",
                        "decoration.dim_around",
                        "Dim Around",
                        "How much the dimaround window rule darkens the screen",
                        0.0,
                        1.0,
                        0.05
                    ),
                    switch_item!(
                        "dim-modal",
                        "decoration.dim_modal",
                        "Dim Behind Dialogs",
                        "Darken a window while one of its dialogs is open"
                    ),
                ],
            },
            GroupDef {
                title: "Blur",
                items: &[
                    switch_item!(
                        "blur-enabled",
                        "decoration.blur.enabled",
                        "Blur",
                        "Blur what is behind translucent windows"
                    ),
                    int_item!(
                        "blur-size",
                        "decoration.blur.size",
                        "Blur Size",
                        "Blur distance",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "blur-passes",
                        "decoration.blur.passes",
                        "Blur Passes",
                        "Number of blur passes; more is smoother and slower",
                        0.0,
                        10.0,
                        1.0
                    ),
                    switch_item!(
                        "blur-xray",
                        "decoration.blur.xray",
                        "X-Ray",
                        "Floating windows blur the wallpaper, not the tiled windows below"
                    ),
                    switch_item!(
                        "blur-ignore-opacity",
                        "decoration.blur.ignore_opacity",
                        "Ignore Window Opacity",
                        "Blur at full strength whatever the window's opacity"
                    ),
                    float_item!(
                        "blur-noise",
                        "decoration.blur.noise",
                        "Noise",
                        "Grain added to the blur",
                        0.0,
                        1.0,
                        0.005
                    ),
                    float_item!(
                        "blur-contrast",
                        "decoration.blur.contrast",
                        "Contrast",
                        "Contrast of the blurred area",
                        0.0,
                        2.0,
                        0.05
                    ),
                    float_item!(
                        "blur-brightness",
                        "decoration.blur.brightness",
                        "Brightness",
                        "Brightness of the blurred area",
                        0.0,
                        2.0,
                        0.05
                    ),
                    float_item!(
                        "blur-vibrancy",
                        "decoration.blur.vibrancy",
                        "Vibrancy",
                        "Color saturation of the blurred area",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "blur-vibrancy-darkness",
                        "decoration.blur.vibrancy_darkness",
                        "Vibrancy in Dark Areas",
                        "How strongly vibrancy applies to dark areas",
                        0.0,
                        1.0,
                        0.05
                    ),
                    switch_item!(
                        "blur-special",
                        "decoration.blur.special",
                        "Blur Behind Special Workspace",
                        "Blur the screen behind a special workspace"
                    ),
                    switch_item!(
                        "blur-popups",
                        "decoration.blur.popups",
                        "Blur Popups",
                        "Blur behind menus and other popups"
                    ),
                    float_item!(
                        "blur-popups-ignorealpha",
                        "decoration.blur.popups_ignorealpha",
                        "Popup Blur Threshold",
                        "Popup pixels more transparent than this are not blurred",
                        0.0,
                        1.0,
                        0.05
                    ),
                ],
            },
            GroupDef {
                title: "Shadow",
                items: &[
                    switch_item!(
                        "shadow-enabled",
                        "decoration.shadow.enabled",
                        "Shadows",
                        "Draw a drop shadow under windows"
                    ),
                    int_item!(
                        "shadow-range",
                        "decoration.shadow.range",
                        "Shadow Range",
                        "Shadow size in pixels",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "shadow-render-power",
                        "decoration.shadow.render_power",
                        "Shadow Falloff",
                        "Higher fades the shadow out faster",
                        1.0,
                        4.0,
                        1.0
                    ),
                    switch_item!(
                        "shadow-sharp",
                        "decoration.shadow.sharp",
                        "Sharp Shadow",
                        "A hard-edged shadow instead of a soft one"
                    ),
                    float_item!(
                        "shadow-scale",
                        "decoration.shadow.scale",
                        "Shadow Scale",
                        "Size of the shadow relative to the window",
                        0.0,
                        1.0,
                        0.05
                    ),
                    pair_item!(
                        "shadow-offset-x",
                        "decoration.shadow.offset",
                        0,
                        "Shadow Offset X",
                        "Horizontal shadow offset in pixels",
                        -50.0,
                        50.0,
                        1.0
                    ),
                    pair_item!(
                        "shadow-offset-y",
                        "decoration.shadow.offset",
                        1,
                        "Shadow Offset Y",
                        "Vertical shadow offset in pixels",
                        -50.0,
                        50.0,
                        1.0
                    ),
                ],
            },
            GroupDef {
                title: "Glow",
                items: &[
                    switch_item!(
                        "glow-enabled",
                        "decoration.glow.enabled",
                        "Glow",
                        "Draw an inner glow on windows"
                    ),
                    int_item!(
                        "glow-range",
                        "decoration.glow.range",
                        "Glow Range",
                        "Glow size in pixels",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "glow-render-power",
                        "decoration.glow.render_power",
                        "Glow Falloff",
                        "Higher fades the glow out faster",
                        1.0,
                        4.0,
                        1.0
                    ),
                ],
            },
            GroupDef {
                title: "Animations",
                items: &[
                    switch_item!(
                        "animations-enabled",
                        "animations.enabled",
                        "Animations",
                        "Animate windows, workspaces, and fades"
                    ),
                    switch_item!(
                        "workspace-wraparound",
                        "animations.workspace_wraparound",
                        "Workspace Wraparound",
                        "Slide the other way between the first and last workspace"
                    ),
                    switch_item!(
                        "motion-blur-enabled",
                        "decoration.motion_blur.enabled",
                        "Motion Blur",
                        "Blur windows while they move or resize"
                    ),
                    int_item!(
                        "motion-blur-samples",
                        "decoration.motion_blur.samples",
                        "Motion Blur Samples",
                        "Samples per frame of motion blur",
                        1.0,
                        64.0,
                        1.0
                    ),
                    switch_item!(
                        "animate-manual-resizes",
                        "misc.animate_manual_resizes",
                        "Animate Manual Resizes",
                        "Animate windows resized or moved with a keybind"
                    ),
                    switch_item!(
                        "animate-mouse-windowdragging",
                        "misc.animate_mouse_windowdragging",
                        "Animate Mouse Dragging",
                        "Animate windows dragged with the mouse"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Layouts",
        group: PageGroup::Hyprland,
        description: "Dwindle, master, and scrolling layout behaviour",
        groups: &[
            GroupDef {
                title: "Single Window",
                items: &[
                    ItemDef {
                        id: "single-window-aspect-width",
                        source: Source::Hyprland("layout.single_window_aspect_ratio"),
                        label: "Aspect Ratio Width",
                        description: "Shape a lone window to this ratio; 0 and 0 is off",
                        field: FieldDef::Pair {
                            index: 0,
                            min: 0.0,
                            max: 32.0,
                            step: 1.0,
                        },
                        when: None,
                        disabled_by: Some("square-single-window"),
                    },
                    ItemDef {
                        id: "single-window-aspect-height",
                        source: Source::Hyprland("layout.single_window_aspect_ratio"),
                        label: "Aspect Ratio Height",
                        description: "Shape a lone window to this ratio; 0 and 0 is off",
                        field: FieldDef::Pair {
                            index: 1,
                            min: 0.0,
                            max: 32.0,
                            step: 1.0,
                        },
                        when: None,
                        disabled_by: Some("square-single-window"),
                    },
                    ItemDef {
                        id: "single-window-aspect-tolerance",
                        source: Source::Hyprland("layout.single_window_aspect_ratio_tolerance"),
                        label: "Aspect Ratio Tolerance",
                        description: "Minimum difference before the ratio is applied",
                        field: FieldDef::Number {
                            min: 0.0,
                            max: 1.0,
                            step: 0.05,
                            integer: false,
                        },
                        when: None,
                        disabled_by: Some("square-single-window"),
                    },
                    ItemDef {
                        id: "square-single-window",
                        source: Source::Omarchy(Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/single-window-aspect-ratio.lua",
                                inverted: false,
                            },
                            write: Write::Bool {
                                on: &[&[
                                    "omarchy-hyprland-toggle",
                                    "single-window-aspect-ratio",
                                    "on",
                                ]],
                                off: &[&[
                                    "omarchy-hyprland-toggle",
                                    "single-window-aspect-ratio",
                                    "off",
                                ]],
                            },
                        }),
                        label: "Square Single Window",
                        description: "Omarchy's toggle: a lone window is kept square, overriding the ratio above",
                        field: FieldDef::Switch,
                        when: None,
                        disabled_by: None,
                    },
                ],
            },
            GroupDef {
                title: "Dwindle",
                items: &[
                    choice_item!(
                        "dwindle-force-split",
                        "dwindle.force_split",
                        "Split Side",
                        "Where a new window goes",
                        &[
                            (0, "Follow mouse"),
                            (1, "Left or top"),
                            (2, "Right or bottom")
                        ]
                    ),
                    switch_item!(
                        "dwindle-preserve-split",
                        "dwindle.preserve_split",
                        "Preserve Split",
                        "Keep the split direction when windows close"
                    ),
                    switch_item!(
                        "dwindle-smart-split",
                        "dwindle.smart_split",
                        "Smart Split",
                        "Split by where the mouse is in the window"
                    ),
                    switch_item!(
                        "dwindle-smart-resizing",
                        "dwindle.smart_resizing",
                        "Smart Resizing",
                        "Resize in the direction the mouse is on"
                    ),
                    switch_item!(
                        "dwindle-use-active-for-splits",
                        "dwindle.use_active_for_splits",
                        "Split the Active Window",
                        "Split the focused window rather than the one under the mouse"
                    ),
                    switch_item!(
                        "dwindle-permanent-direction-override",
                        "dwindle.permanent_direction_override",
                        "Keep Preselect Direction",
                        "A preselected direction stays until changed"
                    ),
                    switch_item!(
                        "dwindle-precise-mouse-move",
                        "dwindle.precise_mouse_move",
                        "Precise Mouse Move",
                        "Drop a dragged window by the exact mouse position"
                    ),
                    float_item!(
                        "dwindle-default-split-ratio",
                        "dwindle.default_split_ratio",
                        "Default Split Ratio",
                        "Size of a new window relative to its sibling",
                        0.1,
                        1.9,
                        0.1
                    ),
                    choice_item!(
                        "dwindle-split-bias",
                        "dwindle.split_bias",
                        "Split Ratio Applies To",
                        "Which window the split ratio sizes",
                        &[(0, "Directional"), (1, "Current window")]
                    ),
                    float_item!(
                        "dwindle-split-width-multiplier",
                        "dwindle.split_width_multiplier",
                        "Split Width Multiplier",
                        "Prefer vertical splits above this width ratio",
                        0.1,
                        3.0,
                        0.1
                    ),
                    float_item!(
                        "dwindle-special-scale-factor",
                        "dwindle.special_scale_factor",
                        "Special Workspace Scale",
                        "Size of windows on a special workspace",
                        0.0,
                        1.0,
                        0.05
                    ),
                ],
            },
            GroupDef {
                title: "Master",
                items: &[
                    dropdown_item!(
                        "master-new-status",
                        "master.new_status",
                        "New Windows",
                        "Where a new window goes",
                        &[
                            ("slave", "Stack"),
                            ("master", "Master"),
                            ("inherit", "Inherit")
                        ]
                    ),
                    dropdown_item!(
                        "master-new-on-active",
                        "master.new_on_active",
                        "New Window Position",
                        "Place a new window relative to the focused one",
                        &[
                            ("none", "End of stack"),
                            ("before", "Before focused"),
                            ("after", "After focused")
                        ]
                    ),
                    switch_item!(
                        "master-new-on-top",
                        "master.new_on_top",
                        "New on Top",
                        "Put a new window at the top of the stack"
                    ),
                    dropdown_item!(
                        "master-orientation",
                        "master.orientation",
                        "Master Area",
                        "Side of the screen the master window takes",
                        &[
                            ("left", "Left"),
                            ("right", "Right"),
                            ("top", "Top"),
                            ("bottom", "Bottom"),
                            ("center", "Center"),
                        ]
                    ),
                    float_item!(
                        "master-mfact",
                        "master.mfact",
                        "Master Size",
                        "Share of the screen the master window takes",
                        0.0,
                        1.0,
                        0.05
                    ),
                    int_item!(
                        "master-slave-count-for-center-master",
                        "master.slave_count_for_center_master",
                        "Windows Needed to Center",
                        "Center the master only with at least this many other windows",
                        0.0,
                        10.0,
                        1.0
                    ),
                    dropdown_item!(
                        "master-center-master-fallback",
                        "master.center_master_fallback",
                        "Center Fallback",
                        "Master side when there are too few windows to center",
                        &[
                            ("left", "Left"),
                            ("right", "Right"),
                            ("top", "Top"),
                            ("bottom", "Bottom")
                        ]
                    ),
                    switch_item!(
                        "master-center-ignores-reserved",
                        "master.center_ignores_reserved",
                        "Center Ignores Bars",
                        "Center the master on the whole monitor, ignoring reserved space"
                    ),
                    switch_item!(
                        "master-allow-small-split",
                        "master.allow_small_split",
                        "Allow Small Split",
                        "Allow more than one master window, split horizontally"
                    ),
                    switch_item!(
                        "master-always-keep-position",
                        "master.always_keep_position",
                        "Keep Master Position",
                        "Keep the master area in place with a single window"
                    ),
                    switch_item!(
                        "master-focus-master-on-close",
                        "master.focus_master_on_close",
                        "Focus Master on Close",
                        "Focus the master window when a window closes"
                    ),
                    switch_item!(
                        "master-drop-at-cursor",
                        "master.drop_at_cursor",
                        "Drop at Cursor",
                        "Dragged windows land where the cursor is"
                    ),
                    switch_item!(
                        "master-smart-resizing",
                        "master.smart_resizing",
                        "Smart Resizing",
                        "Resize in the direction the mouse is on"
                    ),
                    float_item!(
                        "master-special-scale-factor",
                        "master.special_scale_factor",
                        "Special Workspace Scale",
                        "Size of windows on a special workspace",
                        0.0,
                        1.0,
                        0.05
                    ),
                ],
            },
            GroupDef {
                title: "Scrolling",
                items: &[
                    float_item!(
                        "scrolling-column-width",
                        "scrolling.column_width",
                        "Column Width",
                        "Default width of a column as a share of the screen",
                        0.1,
                        1.0,
                        0.05
                    ),
                    dropdown_item!(
                        "scrolling-direction",
                        "scrolling.direction",
                        "Direction",
                        "Where new columns appear and the layout scrolls",
                        &[
                            ("right", "Right"),
                            ("left", "Left"),
                            ("down", "Down"),
                            ("up", "Up")
                        ]
                    ),
                    switch_item!(
                        "scrolling-fullscreen-on-one-column",
                        "scrolling.fullscreen_on_one_column",
                        "Single Column Fills Screen",
                        "A lone column spans the whole screen"
                    ),
                    switch_item!(
                        "scrolling-follow-focus",
                        "scrolling.follow_focus",
                        "Follow Focus",
                        "Scroll to bring the focused window into view"
                    ),
                    choice_item!(
                        "scrolling-focus-fit-method",
                        "scrolling.focus_fit_method",
                        "Bring Into View",
                        "How a focused column is scrolled into view",
                        &[(0, "Center"), (1, "Fit")]
                    ),
                    float_item!(
                        "scrolling-follow-min-visible",
                        "scrolling.follow_min_visible",
                        "Minimum Visible",
                        "Share of a focused window that must be visible",
                        0.0,
                        1.0,
                        0.05
                    ),
                    switch_item!(
                        "scrolling-wrap-focus",
                        "scrolling.wrap_focus",
                        "Wrap Focus",
                        "Focus wraps from the last column to the first"
                    ),
                    switch_item!(
                        "scrolling-wrap-swapcol",
                        "scrolling.wrap_swapcol",
                        "Wrap Column Moves",
                        "Moving a column wraps around the ends"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Keyboard",
        group: PageGroup::Hyprland,
        description: "Layout, repeat, and modifiers",
        groups: &[
            GroupDef {
                title: "Layout",
                items: &[
                    ItemDef {
                        id: "kb-layout",
                        source: Source::Omarchy(KEYBOARD_LAYOUT),
                        label: "Keyboard Layout",
                        description: "Set system-wide; Omarchy adds a Latin fallback for non-Latin layouts",
                        field: FieldDef::KeyboardLayout,
                        when: None,
                        disabled_by: None,
                    },
                    switch_item!(
                        "numlock-by-default",
                        "input.numlock_by_default",
                        "Num Lock on Start",
                        "Turn Num Lock on when Hyprland starts"
                    ),
                    switch_item!(
                        "resolve-binds-by-sym",
                        "input.resolve_binds_by_sym",
                        "Keybinds by Symbol",
                        "Match keybinds by the key's symbol in the current layout"
                    ),
                ],
            },
            GroupDef {
                title: "Repeat",
                items: &[
                    int_item!(
                        "repeat-rate",
                        "input.repeat_rate",
                        "Repeat Rate",
                        "Repeats per second while a key is held",
                        0.0,
                        200.0,
                        5.0
                    ),
                    int_item!(
                        "repeat-delay",
                        "input.repeat_delay",
                        "Repeat Delay",
                        "Milliseconds before a held key repeats",
                        0.0,
                        2000.0,
                        50.0
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Mouse",
        group: PageGroup::Hyprland,
        description: "Pointer speed, scrolling, and focus",
        groups: &[
            GroupDef {
                title: "Pointer",
                items: &[
                    float_item!(
                        "sensitivity",
                        "input.sensitivity",
                        "Sensitivity",
                        "Pointer speed, from -1 to 1",
                        -1.0,
                        1.0,
                        0.1
                    ),
                    dropdown_item!(
                        "accel-profile",
                        "input.accel_profile",
                        "Acceleration",
                        "Pointer acceleration profile",
                        &[("", "Default"), ("adaptive", "Adaptive"), ("flat", "Flat")]
                    ),
                    switch_item!(
                        "force-no-accel",
                        "input.force_no_accel",
                        "No Acceleration",
                        "Pass raw pointer movement through"
                    ),
                    switch_item!(
                        "left-handed",
                        "input.left_handed",
                        "Left Handed",
                        "Swap the left and right buttons"
                    ),
                    switch_item!(
                        "middle-click-paste",
                        "misc.middle_click_paste",
                        "Middle Click Paste",
                        "Paste the selection with the middle button"
                    ),
                ],
            },
            GroupDef {
                title: "Scrolling",
                items: &[
                    switch_item!(
                        "natural-scroll",
                        "input.natural_scroll",
                        "Natural Scroll",
                        "Scroll content in the direction of the wheel"
                    ),
                    float_item!(
                        "scroll-factor",
                        "input.scroll_factor",
                        "Scroll Speed",
                        "Multiplier for wheel scrolling",
                        0.0,
                        2.0,
                        0.1
                    ),
                    dropdown_item!(
                        "scroll-method",
                        "input.scroll_method",
                        "Scroll Method",
                        "How scrolling is triggered",
                        &[
                            ("", "Default"),
                            ("2fg", "Two fingers"),
                            ("edge", "Edge"),
                            ("on_button_down", "While a button is held"),
                            ("no_scroll", "No scrolling"),
                        ]
                    ),
                    int_item!(
                        "scroll-button",
                        "input.scroll_button",
                        "Scroll Button",
                        "Button code that scrolls while held; 0 is the default",
                        0.0,
                        300.0,
                        1.0
                    ),
                    switch_item!(
                        "scroll-button-lock",
                        "input.scroll_button_lock",
                        "Scroll Button Lock",
                        "Tap the scroll button once instead of holding it"
                    ),
                    choice_item!(
                        "emulate-discrete-scroll",
                        "input.emulate_discrete_scroll",
                        "Discrete Scroll",
                        "Turn smooth wheel events into steps",
                        &[(0, "Off"), (1, "Non-standard wheels"), (2, "All wheels")]
                    ),
                    choice_item!(
                        "off-window-axis-events",
                        "input.off_window_axis_events",
                        "Scroll Outside Window",
                        "What scrolling next to the focused window does",
                        &[(0, "Ignore"), (1, "Send"), (2, "Clamp"), (3, "Warp")]
                    ),
                ],
            },
            GroupDef {
                title: "Focus",
                items: &[
                    choice_item!(
                        "follow-mouse",
                        "input.follow_mouse",
                        "Focus Follows Mouse",
                        "How moving the pointer changes focus",
                        &[(0, "Off"), (1, "Follow"), (2, "Detached"), (3, "Separate")]
                    ),
                    float_item!(
                        "follow-mouse-threshold",
                        "input.follow_mouse_threshold",
                        "Follow Threshold",
                        "Pixels the pointer must travel before focus follows",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "follow-mouse-shrink",
                        "input.follow_mouse_shrink",
                        "Follow Inset",
                        "Pixels inside a window's edge before it takes focus",
                        0.0,
                        300.0,
                        1.0
                    ),
                    switch_item!(
                        "mouse-refocus",
                        "input.mouse_refocus",
                        "Refocus on Hover",
                        "Hovering a window focuses it without a click"
                    ),
                    choice_item!(
                        "focus-on-close",
                        "input.focus_on_close",
                        "Focus After Close",
                        "Which window gets focus when one closes",
                        &[
                            (0, "Next window"),
                            (1, "Window under cursor"),
                            (2, "Last used")
                        ]
                    ),
                    choice_item!(
                        "float-switch-override-focus",
                        "input.float_switch_override_focus",
                        "Focus Under Cursor When Floating Changes",
                        "Focus the window under the cursor when a window floats or tiles",
                        &[(0, "Off"), (1, "On"), (2, "Also across floating windows")]
                    ),
                    switch_item!(
                        "special-fallthrough",
                        "input.special_fallthrough",
                        "Special Workspace Fallthrough",
                        "Focus windows below a special workspace that only has floating windows"
                    ),
                    switch_item!(
                        "mouse-move-focuses-monitor",
                        "misc.mouse_move_focuses_monitor",
                        "Mouse Focuses Monitor",
                        "Moving the pointer to a monitor focuses it"
                    ),
                    switch_item!(
                        "always-follow-on-dnd",
                        "misc.always_follow_on_dnd",
                        "Follow While Dragging",
                        "Focus follows the pointer during drag and drop"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Touchpad",
        group: PageGroup::Hyprland,
        description: "Tapping, scrolling, and gestures on the touchpad",
        groups: &[
            GroupDef {
                title: "Tapping",
                items: &[
                    switch_item!(
                        "tap-to-click",
                        "input.touchpad.tap_to_click",
                        "Tap to Click",
                        "One, two, or three fingers tap for left, right, or middle click"
                    ),
                    switch_item!(
                        "tap-and-drag",
                        "input.touchpad.tap_and_drag",
                        "Tap and Drag",
                        "Tap, then drag with the same finger"
                    ),
                    choice_item!(
                        "drag-lock",
                        "input.touchpad.drag_lock",
                        "Drag Lock",
                        "Keep dragging after lifting the finger",
                        &[(0, "Off"), (1, "Until timeout"), (2, "Until tap")]
                    ),
                    dropdown_item!(
                        "tap-button-map",
                        "input.touchpad.tap_button_map",
                        "Tap Buttons",
                        "Which buttons two and three fingers tap",
                        &[
                            ("", "Default"),
                            ("lrm", "Left, right, middle"),
                            ("lmr", "Left, middle, right")
                        ]
                    ),
                    switch_item!(
                        "clickfinger-behavior",
                        "input.touchpad.clickfinger_behavior",
                        "Click by Finger Count",
                        "Pressing with one, two, or three fingers is a left, right, or middle click"
                    ),
                    switch_item!(
                        "middle-button-emulation",
                        "input.touchpad.middle_button_emulation",
                        "Middle Button Emulation",
                        "Pressing left and right together is a middle click"
                    ),
                    choice_item!(
                        "drag-3fg",
                        "input.touchpad.drag_3fg",
                        "Drag With Fingers",
                        "Drag windows with three or four fingers",
                        &[(0, "Off"), (1, "Three fingers"), (2, "Four fingers")]
                    ),
                ],
            },
            GroupDef {
                title: "Scrolling",
                items: &[
                    switch_item!(
                        "touchpad-natural-scroll",
                        "input.touchpad.natural_scroll",
                        "Natural Scroll",
                        "Content follows the fingers"
                    ),
                    float_item!(
                        "touchpad-scroll-factor",
                        "input.touchpad.scroll_factor",
                        "Scroll Speed",
                        "Multiplier for touchpad scrolling",
                        0.0,
                        2.0,
                        0.1
                    ),
                    switch_item!(
                        "disable-while-typing",
                        "input.touchpad.disable_while_typing",
                        "Disable While Typing",
                        "Ignore the touchpad while keys are pressed"
                    ),
                    switch_item!(
                        "touchpad-flip-x",
                        "input.touchpad.flip_x",
                        "Flip Horizontal",
                        "Invert horizontal movement"
                    ),
                    switch_item!(
                        "touchpad-flip-y",
                        "input.touchpad.flip_y",
                        "Flip Vertical",
                        "Invert vertical movement"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Groups",
        group: PageGroup::Hyprland,
        description: "Tabbed window groups and their bar",
        groups: &[
            GroupDef {
                title: "Grouping",
                items: &[
                    switch_item!(
                        "auto-group",
                        "group.auto_group",
                        "Auto Group",
                        "New windows join the focused group"
                    ),
                    switch_item!(
                        "insert-after-current",
                        "group.insert_after_current",
                        "Insert After Current",
                        "A new window goes after the current one, not at the end"
                    ),
                    switch_item!(
                        "focus-removed-window",
                        "group.focus_removed_window",
                        "Focus Removed Window",
                        "Focus a window taken out of a group"
                    ),
                    choice_item!(
                        "drag-into-group",
                        "group.drag_into_group",
                        "Drag Into Group",
                        "Whether dragging a window onto a group merges it",
                        &[(0, "Off"), (1, "On"), (2, "Only onto the bar")]
                    ),
                    switch_item!(
                        "merge-groups-on-drag",
                        "group.merge_groups_on_drag",
                        "Merge Groups on Drag",
                        "Dragging a group onto another merges them"
                    ),
                    switch_item!(
                        "merge-groups-on-groupbar",
                        "group.merge_groups_on_groupbar",
                        "Merge on Bar",
                        "Dropping a group on a group bar merges them"
                    ),
                    switch_item!(
                        "merge-floated-into-tiled-on-groupbar",
                        "group.merge_floated_into_tiled_on_groupbar",
                        "Merge Floating on Bar",
                        "Dropping a floating window on a tiled group bar merges it"
                    ),
                    switch_item!(
                        "group-on-movetoworkspace",
                        "group.group_on_movetoworkspace",
                        "Group on Move to Workspace",
                        "Moving a window to a workspace joins that workspace's group"
                    ),
                ],
            },
            GroupDef {
                title: "Group Bar",
                items: &[
                    switch_item!(
                        "groupbar-enabled",
                        "group.groupbar.enabled",
                        "Group Bar",
                        "Show a tab bar above grouped windows"
                    ),
                    switch_item!(
                        "groupbar-disable-when-only",
                        "group.groupbar.disable_when_only",
                        "Hide for One Window",
                        "No bar on a group with a single window"
                    ),
                    switch_item!(
                        "groupbar-render-titles",
                        "group.groupbar.render_titles",
                        "Show Titles",
                        "Window titles on the tabs"
                    ),
                    int_item!(
                        "groupbar-font-size",
                        "group.groupbar.font_size",
                        "Font Size",
                        "Title font size",
                        2.0,
                        64.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-height",
                        "group.groupbar.height",
                        "Height",
                        "Bar height in pixels",
                        1.0,
                        64.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-indicator-height",
                        "group.groupbar.indicator_height",
                        "Indicator Height",
                        "Height of the active-tab indicator",
                        1.0,
                        64.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-indicator-gap",
                        "group.groupbar.indicator_gap",
                        "Indicator Gap",
                        "Gap between the indicator and the title",
                        0.0,
                        64.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-gaps-in",
                        "group.groupbar.gaps_in",
                        "Gaps Between Tabs",
                        "Gap between tabs",
                        0.0,
                        20.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-gaps-out",
                        "group.groupbar.gaps_out",
                        "Gap to Window",
                        "Gap between the bar and the window",
                        0.0,
                        20.0,
                        1.0
                    ),
                    switch_item!(
                        "groupbar-keep-upper-gap",
                        "group.groupbar.keep_upper_gap",
                        "Keep Upper Gap",
                        "Keep a gap above the tabs"
                    ),
                    switch_item!(
                        "groupbar-stacked",
                        "group.groupbar.stacked",
                        "Stacked",
                        "Stack the tabs vertically"
                    ),
                    switch_item!(
                        "groupbar-gradients",
                        "group.groupbar.gradients",
                        "Gradients",
                        "Fill tabs with the group colors"
                    ),
                    int_item!(
                        "groupbar-gradient-rounding",
                        "group.groupbar.gradient_rounding",
                        "Gradient Rounding",
                        "Corner radius of the tab fill",
                        0.0,
                        20.0,
                        1.0
                    ),
                    float_item!(
                        "groupbar-gradient-rounding-power",
                        "group.groupbar.gradient_rounding_power",
                        "Gradient Rounding Power",
                        "Corner shape of the tab fill: 2 is a circle",
                        2.0,
                        10.0,
                        0.5
                    ),
                    switch_item!(
                        "groupbar-gradient-round-only-edges",
                        "group.groupbar.gradient_round_only_edges",
                        "Round Only Outer Tab Corners",
                        "Round the fill at the ends of the bar only"
                    ),
                    int_item!(
                        "groupbar-rounding",
                        "group.groupbar.rounding",
                        "Bar Rounding",
                        "Corner radius of the bar",
                        0.0,
                        20.0,
                        1.0
                    ),
                    float_item!(
                        "groupbar-rounding-power",
                        "group.groupbar.rounding_power",
                        "Bar Rounding Power",
                        "Corner shape of the bar: 2 is a circle",
                        2.0,
                        10.0,
                        0.5
                    ),
                    switch_item!(
                        "groupbar-round-only-edges",
                        "group.groupbar.round_only_edges",
                        "Round Only Outer Bar Corners",
                        "Round the bar at its ends only"
                    ),
                    int_item!(
                        "groupbar-text-offset",
                        "group.groupbar.text_offset",
                        "Text Offset",
                        "Vertical offset of the titles",
                        -20.0,
                        20.0,
                        1.0
                    ),
                    int_item!(
                        "groupbar-text-padding",
                        "group.groupbar.text_padding",
                        "Text Padding",
                        "Horizontal padding around the titles",
                        0.0,
                        22.0,
                        1.0
                    ),
                    switch_item!(
                        "groupbar-blur",
                        "group.groupbar.blur",
                        "Blur Behind Bar",
                        "Blur what is behind the bar"
                    ),
                    switch_item!(
                        "groupbar-scrolling",
                        "group.groupbar.scrolling",
                        "Scroll Changes Tab",
                        "Scrolling on the bar switches tabs"
                    ),
                    switch_item!(
                        "groupbar-middle-click-close",
                        "group.groupbar.middle_click_close",
                        "Middle Click Closes",
                        "Middle clicking a tab closes its window"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Cursor",
        group: PageGroup::Hyprland,
        description: "Hiding, warping, and zooming the pointer",
        groups: &[
            GroupDef {
                title: "Hiding",
                items: &[
                    switch_item!(
                        "hide-on-key-press",
                        "cursor.hide_on_key_press",
                        "Hide While Typing",
                        "Hide the pointer on a key press until it moves"
                    ),
                    switch_item!(
                        "hide-on-touch",
                        "cursor.hide_on_touch",
                        "Hide After Touch",
                        "Hide the pointer after touchscreen input until it moves"
                    ),
                    switch_item!(
                        "hide-on-tablet",
                        "cursor.hide_on_tablet",
                        "Hide After Tablet",
                        "Hide the pointer after tablet input until it moves"
                    ),
                    float_item!(
                        "inactive-timeout",
                        "cursor.inactive_timeout",
                        "Hide When Idle",
                        "Seconds without movement before the pointer hides; 0 never",
                        0.0,
                        20.0,
                        1.0
                    ),
                ],
            },
            GroupDef {
                title: "Warping",
                items: &[
                    choice_item!(
                        "warp-on-change-workspace",
                        "cursor.warp_on_change_workspace",
                        "Warp on Workspace Change",
                        "Move the pointer to the focused window after switching workspace",
                        &[(0, "Off"), (1, "On"), (2, "Always")]
                    ),
                    choice_item!(
                        "warp-on-toggle-special",
                        "cursor.warp_on_toggle_special",
                        "Warp on Special Workspace",
                        "Move the pointer to the focused window when toggling a special workspace",
                        &[(0, "Off"), (1, "On"), (2, "Always")]
                    ),
                    switch_item!(
                        "no-warps",
                        "cursor.no_warps",
                        "Never Warp",
                        "Do not move the pointer automatically"
                    ),
                    switch_item!(
                        "persistent-warps",
                        "cursor.persistent_warps",
                        "Remember Position per Window",
                        "Refocusing a window returns the pointer to where it was"
                    ),
                    switch_item!(
                        "warp-back-after-non-mouse-input",
                        "cursor.warp_back_after_non_mouse_input",
                        "Warp Back After Keyboard",
                        "Return the pointer after keyboard or touch input moved it"
                    ),
                    int_item!(
                        "hotspot-padding",
                        "cursor.hotspot_padding",
                        "Edge Padding",
                        "Pixels kept between the pointer and screen edges",
                        0.0,
                        20.0,
                        1.0
                    ),
                ],
            },
            GroupDef {
                title: "Zoom",
                items: &[
                    float_item!(
                        "zoom-factor",
                        "cursor.zoom_factor",
                        "Zoom",
                        "Magnification around the pointer; 1 is none",
                        1.0,
                        10.0,
                        0.5
                    ),
                    switch_item!(
                        "zoom-rigid",
                        "cursor.zoom_rigid",
                        "Rigid Zoom",
                        "The zoomed view follows the pointer exactly"
                    ),
                    switch_item!(
                        "zoom-detached-camera",
                        "cursor.zoom_detached_camera",
                        "Detached Camera",
                        "The zoomed view does not follow the pointer"
                    ),
                    switch_item!(
                        "zoom-disable-aa",
                        "cursor.zoom_disable_aa",
                        "No Anti-Aliasing",
                        "Sharp pixels when zoomed"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Windows",
        group: PageGroup::Hyprland,
        description: "Focus, workspaces, and fullscreen behaviour",
        groups: &[
            GroupDef {
                title: "Focus",
                items: &[
                    switch_item!(
                        "focus-on-activate",
                        "misc.focus_on_activate",
                        "Focus on Request",
                        "Focus an app that asks for it"
                    ),
                    choice_item!(
                        "on-focus-under-fullscreen",
                        "misc.on_focus_under_fullscreen",
                        "Focus Under Fullscreen",
                        "What happens when a window behind a fullscreen one wants focus",
                        &[(0, "Ignore"), (1, "Take over"), (2, "Leave fullscreen")]
                    ),
                    switch_item!(
                        "exit-window-retains-fullscreen",
                        "misc.exit_window_retains_fullscreen",
                        "Keep Fullscreen on Close",
                        "The next window goes fullscreen when a fullscreen one closes"
                    ),
                    switch_item!(
                        "layers-hog-keyboard-focus",
                        "misc.layers_hog_keyboard_focus",
                        "Panels Keep Focus",
                        "Keyboard-interactive panels keep focus when the mouse moves"
                    ),
                    switch_item!(
                        "size-limits-tiled",
                        "misc.size_limits_tiled",
                        "Size Limits for Tiled Windows",
                        "Apply minimum and maximum size rules to tiled windows"
                    ),
                ],
            },
            GroupDef {
                title: "Workspaces",
                items: &[
                    choice_item!(
                        "initial-workspace-tracking",
                        "misc.initial_workspace_tracking",
                        "Open Where Launched",
                        "Open a window on the workspace it was launched from",
                        &[(0, "Off"), (1, "First window"), (2, "Every window")]
                    ),
                    int_item!(
                        "initial-workspace-token-timeout",
                        "misc.initial_workspace_token_timeout",
                        "Launch Tracking Timeout",
                        "Seconds a launched app has to open on its workspace",
                        1.0,
                        600.0,
                        5.0
                    ),
                    switch_item!(
                        "workspace-back-and-forth",
                        "binds.workspace_back_and_forth",
                        "Back and Forth",
                        "Switching to the current workspace returns to the previous one"
                    ),
                    switch_item!(
                        "allow-workspace-cycles",
                        "binds.allow_workspace_cycles",
                        "Remember Previous Workspace",
                        "Workspaces keep their previous workspace for back and forth"
                    ),
                    choice_item!(
                        "workspace-center-on",
                        "binds.workspace_center_on",
                        "Center Pointer On",
                        "Where the pointer goes when switching workspace",
                        &[(0, "Workspace center"), (1, "Last window")]
                    ),
                    switch_item!(
                        "hide-special-on-workspace-change",
                        "binds.hide_special_on_workspace_change",
                        "Hide Special on Switch",
                        "Switching workspace hides the special workspace"
                    ),
                    switch_item!(
                        "close-special-on-empty",
                        "misc.close_special_on_empty",
                        "Close Empty Special",
                        "Close the special workspace when its last window closes"
                    ),
                ],
            },
            GroupDef {
                title: "Moving Focus",
                items: &[
                    choice_item!(
                        "focus-preferred-method",
                        "binds.focus_preferred_method",
                        "Focus Direction Method",
                        "How the window in a direction is chosen",
                        &[(0, "Nearest"), (1, "Largest overlap")]
                    ),
                    switch_item!(
                        "movefocus-cycles-fullscreen",
                        "binds.movefocus_cycles_fullscreen",
                        "Cycle in Fullscreen",
                        "Moving focus on a fullscreen window cycles fullscreen windows"
                    ),
                    switch_item!(
                        "movefocus-cycles-groupfirst",
                        "binds.movefocus_cycles_groupfirst",
                        "Cycle Group First",
                        "Moving focus inside a group cycles its windows first"
                    ),
                    switch_item!(
                        "window-direction-monitor-fallback",
                        "binds.window_direction_monitor_fallback",
                        "Cross Monitors",
                        "Moving focus past a monitor edge continues on the next monitor"
                    ),
                    switch_item!(
                        "ignore-group-lock",
                        "binds.ignore_group_lock",
                        "Ignore Group Lock",
                        "Group dispatchers work on locked groups too"
                    ),
                    switch_item!(
                        "allow-pin-fullscreen",
                        "binds.allow_pin_fullscreen",
                        "Pinned Fullscreen",
                        "Pinned windows can go fullscreen and stay pinned"
                    ),
                ],
            },
            GroupDef {
                title: "Keybinds and Dragging",
                items: &[
                    int_item!(
                        "drag-threshold",
                        "binds.drag_threshold",
                        "Drag Threshold",
                        "Pixels of movement before a mouse bind drags",
                        0.0,
                        100.0,
                        1.0
                    ),
                    switch_item!(
                        "pass-mouse-when-bound",
                        "binds.pass_mouse_when_bound",
                        "Pass Mouse to Apps",
                        "Mouse binds also reach the app under the pointer"
                    ),
                    int_item!(
                        "scroll-event-delay",
                        "binds.scroll_event_delay",
                        "Scroll Bind Delay",
                        "Milliseconds between scroll events a bind accepts",
                        0.0,
                        2000.0,
                        50.0
                    ),
                    switch_item!(
                        "disable-keybind-grabbing",
                        "binds.disable_keybind_grabbing",
                        "Apps Cannot Grab Keybinds",
                        "Ignore requests from apps to take over keybinds"
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "System",
        group: PageGroup::Hyprland,
        description: "Rendering, displays, sessions, and XWayland",
        groups: &[
            GroupDef {
                title: "Rendering",
                items: &[
                    choice_item!(
                        "vrr",
                        "misc.vrr",
                        "Adaptive Sync",
                        "Variable refresh rate on monitors that support it",
                        &[
                            (0, "Off"),
                            (1, "On"),
                            (2, "Fullscreen only"),
                            (3, "Fullscreen games")
                        ]
                    ),
                    choice_item!(
                        "direct-scanout",
                        "render.direct_scanout",
                        "Direct Scanout",
                        "Show a fullscreen window's buffer directly, skipping compositing",
                        ON_OFF_AUTO
                    ),
                    ItemDef {
                        id: "nvidia-anti-flicker",
                        source: Source::Hyprland("opengl.nvidia_anti_flicker"),
                        label: "NVIDIA Anti-Flicker",
                        description: "Reduce flicker on NVIDIA, at the cost of possible frame drops",
                        field: FieldDef::Switch,
                        when: Some(&["omarchy-hw-nvidia"]),
                        disabled_by: None,
                    },
                ],
            },
            GroupDef {
                title: "Color and HDR",
                items: &[
                    switch_item!(
                        "cm-enabled",
                        "render.cm_enabled",
                        "Color Management",
                        "Color management pipelines; takes effect after a restart"
                    ),
                    choice_item!(
                        "cm-auto-hdr",
                        "render.cm_auto_hdr",
                        "Auto HDR",
                        "Switch to HDR for fullscreen HDR apps",
                        &[(0, "Off"), (1, "HDR"), (2, "HDR from EDID")]
                    ),
                    dropdown_item!(
                        "cm-sdr-eotf",
                        "render.cm_sdr_eotf",
                        "SDR Transfer Function",
                        "How SDR apps are shown on an HDR display",
                        &[
                            ("default", "Default"),
                            ("gamma22", "Gamma 2.2"),
                            ("srgb", "sRGB")
                        ]
                    ),
                    choice_item!(
                        "prefer-hdr",
                        "quirks.prefer_hdr",
                        "Prefer HDR",
                        "Prefer HDR mode",
                        &[(0, "Off"), (1, "On"), (2, "Gamescope only")]
                    ),
                    switch_item!(
                        "send-content-type",
                        "render.send_content_type",
                        "Send Content Type",
                        "Tell monitors what is shown so they can switch profiles"
                    ),
                    choice_item!(
                        "ctm-animation",
                        "render.ctm_animation",
                        "Fade Color Changes",
                        "Fade between color transforms such as night light",
                        ON_OFF_AUTO
                    ),
                ],
            },
            GroupDef {
                title: "Display Wake",
                items: &[
                    switch_item!(
                        "mouse-move-enables-dpms",
                        "misc.mouse_move_enables_dpms",
                        "Mouse Wakes Display",
                        "Moving the mouse turns the display back on"
                    ),
                    switch_item!(
                        "key-press-enables-dpms",
                        "misc.key_press_enables_dpms",
                        "Key Wakes Display",
                        "A key press turns the display back on"
                    ),
                ],
            },
            GroupDef {
                title: "Notices",
                items: &[
                    switch_item!(
                        "enable-anr-dialog",
                        "misc.enable_anr_dialog",
                        "Not Responding Dialog",
                        "Offer to close an app that stops responding"
                    ),
                    int_item!(
                        "anr-missed-pings",
                        "misc.anr_missed_pings",
                        "Missed Pings Before Dialog",
                        "Unanswered pings before the not responding dialog",
                        1.0,
                        20.0,
                        1.0
                    ),
                    switch_item!(
                        "no-donation-nag",
                        "ecosystem.no_donation_nag",
                        "No Donation Reminder",
                        "No twice-yearly donation popup"
                    ),
                ],
            },
            GroupDef {
                title: "XWayland",
                items: &[
                    switch_item!(
                        "xwayland-enabled",
                        "xwayland.enabled",
                        "XWayland",
                        "Run X11 apps"
                    ),
                    switch_item!(
                        "xwayland-force-zero-scaling",
                        "xwayland.force_zero_scaling",
                        "Unscaled X11 Apps",
                        "X11 apps render at scale 1 and scale themselves"
                    ),
                    switch_item!(
                        "xwayland-use-nearest-neighbor",
                        "xwayland.use_nearest_neighbor",
                        "Pixelated Scaling",
                        "Scale X11 apps with nearest-neighbor filtering"
                    ),
                ],
            },
        ],
    },
];

// ── Omarchy ────────────────────────────────────────────────────────────

macro_rules! o_int {
    ($id:expr, $backing:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            source: Source::Omarchy($backing),
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                integer: true,
            },
            when: None,
            disabled_by: None,
        }
    };
}

macro_rules! o_switch {
    ($id:expr, $backing:expr, $label:expr, $desc:expr) => {
        o_switch!($id, $backing, $label, $desc, None)
    };
    ($id:expr, $backing:expr, $label:expr, $desc:expr, $when:expr) => {
        ItemDef {
            id: $id,
            source: Source::Omarchy($backing),
            label: $label,
            description: $desc,
            field: FieldDef::Switch,
            when: $when,
            disabled_by: None,
        }
    };
}

macro_rules! o_dropdown {
    ($id:expr, $backing:expr, $label:expr, $desc:expr, $options:expr) => {
        o_dropdown!($id, $backing, $label, $desc, $options, None)
    };
    ($id:expr, $backing:expr, $label:expr, $desc:expr, $options:expr, $when:expr) => {
        ItemDef {
            id: $id,
            source: Source::Omarchy($backing),
            label: $label,
            description: $desc,
            field: FieldDef::DynamicDropdown { options: $options },
            when: $when,
            disabled_by: None,
        }
    };
}

macro_rules! action {
    ($id:expr, $label:expr, $desc:expr, $button:expr, $argv:expr, $terminal:expr) => {
        action!($id, $label, $desc, $button, $argv, $terminal, None)
    };
    ($id:expr, $label:expr, $desc:expr, $button:expr, $argv:expr, $terminal:expr, $when:expr) => {
        ItemDef {
            id: $id,
            source: Source::None,
            label: $label,
            description: $desc,
            field: FieldDef::Action {
                button: $button,
                argv: $argv,
                terminal: $terminal,
            },
            when: $when,
            disabled_by: None,
        }
    };
}

macro_rules! feature {
    ($id:expr, $label:expr, $desc:expr, $status:expr, $setup:expr, $remove:expr) => {
        feature!($id, $label, $desc, $status, $setup, $remove, None)
    };
    ($id:expr, $label:expr, $desc:expr, $status:expr, $setup:expr, $remove:expr, $when:expr) => {
        ItemDef {
            id: $id,
            source: Source::None,
            label: $label,
            description: $desc,
            field: FieldDef::Feature {
                status: $status,
                setup: $setup,
                remove: $remove,
            },
            when: $when,
            disabled_by: None,
        }
    };
}

const LAPTOP: Option<&[&str]> = Some(&["omarchy-hw-laptop"]);

pub const OMARCHY_PAGES: &[PageDef] = &[
    PageDef {
        title: "Lock & Idle",
        group: PageGroup::Omarchy,
        description: "Screensaver, lock screen, and staying awake",
        groups: &[
            GroupDef {
                title: "Idle",
                items: &[
                    o_int!(
                        "idle-screensaver",
                        Backing {
                            read: Read::ShellJson("idle.screensaver"),
                            write: Write::ShellJson("idle.screensaver"),
                        },
                        "Screensaver After",
                        "Seconds of inactivity before the screensaver starts",
                        30.0,
                        7200.0,
                        30.0
                    ),
                    o_int!(
                        "idle-lock",
                        Backing {
                            read: Read::ShellJson("idle.lock"),
                            write: Write::ShellJson("idle.lock"),
                        },
                        "Lock After",
                        "Seconds of inactivity before the screen locks",
                        30.0,
                        7200.0,
                        30.0
                    ),
                    o_switch!(
                        "stay-awake",
                        Backing {
                            read: Read::Flag {
                                flag: "indicators/stay-awake",
                                inverted: false,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-toggle-idle", "stay-awake"]],
                                off: &[&["omarchy-toggle-idle", "allow-idle"]],
                            },
                        },
                        "Stay Awake",
                        "No screensaver or lock while this is on"
                    ),
                    o_switch!(
                        "screensaver-enabled",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/screensaver-off",
                                inverted: true,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-toggle", "screensaver-off", "off"]],
                                off: &[&["omarchy-toggle", "screensaver-off", "on"]],
                            },
                        },
                        "Screensaver",
                        "Start the screensaver when idle"
                    ),
                ],
            },
            GroupDef {
                title: "System Menu",
                items: &[
                    o_switch!(
                        "suspend-available",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/suspend-off",
                                inverted: true,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-toggle", "suspend-off", "off"]],
                                off: &[&["omarchy-toggle", "suspend-off", "on"]],
                            },
                        },
                        "Show Suspend",
                        "Offer Suspend in the system menu"
                    ),
                    action!(
                        "lock-now",
                        "Lock Screen",
                        "Lock the screen now",
                        "Lock",
                        &["omarchy-system-lock"],
                        false
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Power",
        group: PageGroup::Omarchy,
        description: "Power profiles and the battery",
        groups: &[
            GroupDef {
                title: "Power Profile",
                items: &[
                    o_dropdown!(
                        "profile-ac",
                        Backing {
                            read: Read::StateFile("powerprofiles/ac"),
                            write: Write::Command(&["omarchy-powerprofiles-set", "ac"]),
                        },
                        "On Power",
                        "Profile used while plugged in",
                        Options::Lines(&["omarchy-powerprofiles-list"])
                    ),
                    o_dropdown!(
                        "profile-battery",
                        Backing {
                            read: Read::StateFile("powerprofiles/battery"),
                            write: Write::Command(&["omarchy-powerprofiles-set", "battery"]),
                        },
                        "On Battery",
                        "Profile used on battery",
                        Options::Lines(&["omarchy-powerprofiles-list"]),
                        LAPTOP
                    ),
                ],
            },
            GroupDef {
                title: "Battery",
                items: &[
                    o_switch!(
                        "battery-percentage",
                        Backing {
                            read: Read::ShellWidget {
                                id: "omarchy.power",
                                key: "showPercentage",
                            },
                            write: Write::CommandJson(&[
                                "omarchy",
                                "bar",
                                "set",
                                "omarchy.power",
                                "showPercentage",
                            ]),
                        },
                        "Battery Percentage",
                        "Show the percentage next to the battery in the bar",
                        LAPTOP
                    ),
                    action!(
                        "hybrid-gpu",
                        "Hybrid GPU",
                        "Switch between the integrated and the dedicated GPU; reboots",
                        "Switch…",
                        &["omarchy-toggle-hybrid-gpu"],
                        true,
                        Some(&["omarchy-hw-hybrid-gpu"])
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Notifications",
        group: PageGroup::Omarchy,
        description: "Do not disturb and crash reports",
        groups: &[GroupDef {
            title: "Notifications",
            items: &[
                o_switch!(
                    "do-not-disturb",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-shell", "notifications", "isDnd"],
                            parse: Parse::LineIs("on"),
                        },
                        write: Write::Bool {
                            on: &[&["omarchy-shell", "notifications", "setDnd", "true"]],
                            off: &[&["omarchy-shell", "notifications", "setDnd", "false"]],
                        },
                    },
                    "Do Not Disturb",
                    "Hold notifications instead of showing them"
                ),
                o_switch!(
                    "crash-capture",
                    Backing {
                        read: Read::Flag {
                            flag: "toggles/crash-capture-off",
                            inverted: true,
                        },
                        write: Write::Bool {
                            on: &[
                                &["omarchy-toggle", "crash-capture-off", "off"],
                                &[
                                    "systemctl",
                                    "--user",
                                    "start",
                                    "omarchy-crash-watch.service"
                                ],
                            ],
                            off: &[
                                &["omarchy-toggle", "crash-capture-off", "on"],
                                &["systemctl", "--user", "stop", "omarchy-crash-watch.service"],
                            ],
                        },
                    },
                    "Crash Capture",
                    "Notify when an app crashes and capture a report"
                ),
            ],
        }],
    },
    PageDef {
        title: "Default Apps",
        group: PageGroup::Omarchy,
        description: "The browser, terminal, and editor Omarchy opens",
        groups: &[GroupDef {
            title: "Defaults",
            items: &[
                o_dropdown!(
                    "default-browser",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-default-browser"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-default-browser"]),
                    },
                    "Browser",
                    "Opens links and web apps",
                    Options::Installed(&[
                        ("chromium", "chromium", "Chromium", "chromium"),
                        ("chrome", "chrome", "Chrome", "google-chrome-stable"),
                        ("brave", "brave", "Brave", "brave"),
                        (
                            "brave-origin",
                            "brave-origin",
                            "Brave Origin",
                            "brave-origin"
                        ),
                        ("edge", "edge", "Edge", "microsoft-edge-stable"),
                        ("firefox", "firefox", "Firefox", "firefox"),
                        ("zen", "zen", "Zen", "zen-browser"),
                    ])
                ),
                o_dropdown!(
                    "default-terminal",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-default-terminal"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-default-terminal"]),
                    },
                    "Terminal",
                    "Opens with Super + Return and runs TUIs",
                    Options::Installed(&[
                        ("alacritty", "alacritty", "Alacritty", "alacritty"),
                        ("foot", "foot", "Foot", "foot"),
                        ("ghostty", "ghostty", "Ghostty", "ghostty"),
                        ("kitty", "kitty", "Kitty", "kitty"),
                    ])
                ),
                o_dropdown!(
                    "default-editor",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-default-editor"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-default-editor"]),
                    },
                    "Editor",
                    "Opens config files and text",
                    Options::Installed(&[
                        ("nvim", "nvim", "Neovim", "nvim"),
                        ("code", "code", "VS Code", "code"),
                        ("cursor", "cursor", "Cursor", "cursor"),
                        ("zed", "zeditor", "Zed", "zeditor"),
                        (
                            "sublime_text",
                            "sublime_text",
                            "Sublime Text",
                            "sublime_text"
                        ),
                        ("helix", "helix", "Helix", "helix"),
                        ("vim", "vim", "Vim", "vim"),
                        ("emacs", "emacs", "Emacs", "emacs"),
                    ])
                ),
            ],
        }],
    },
    PageDef {
        title: "Bar",
        group: PageGroup::Omarchy,
        description: "Where the bar sits and how it looks",
        groups: &[GroupDef {
            title: "Bar",
            items: &[
                o_switch!(
                    "bar-visible",
                    Backing {
                        read: Read::Flag {
                            flag: "toggles/bar-off",
                            inverted: true,
                        },
                        write: Write::Bool {
                            on: &[&["omarchy-toggle-bar", "on"]],
                            off: &[&["omarchy-toggle-bar", "off"]],
                        },
                    },
                    "Show Bar",
                    "Hide the bar for a clean screen"
                ),
                o_dropdown!(
                    "bar-position",
                    Backing {
                        read: Read::ShellJson("bar.position"),
                        write: Write::Command(&["omarchy", "bar", "position"]),
                    },
                    "Position",
                    "Edge of the screen the bar sits on",
                    Options::Static(&[
                        ("top", "Top"),
                        ("bottom", "Bottom"),
                        ("left", "Left"),
                        ("right", "Right"),
                    ])
                ),
                o_switch!(
                    "bar-transparent",
                    Backing {
                        read: Read::ShellJson("bar.transparent"),
                        write: Write::Bool {
                            on: &[&["omarchy", "bar", "transparent", "true"]],
                            off: &[&["omarchy", "bar", "transparent", "false"]],
                        },
                    },
                    "Transparent",
                    "See the wallpaper through the bar"
                ),
            ],
        }],
    },
    PageDef {
        title: "Fonts",
        group: PageGroup::Omarchy,
        description: "The monospace font and text size everywhere",
        groups: &[GroupDef {
            title: "Text",
            items: &[
                o_dropdown!(
                    "mono-font",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-font-current"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-font-set"]),
                    },
                    "Monospace Font",
                    "Used by the shell and the terminals; restarts the shell",
                    Options::Lines(&["omarchy-font-list"])
                ),
                o_int!(
                    "text-size",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-display-text-size"],
                            parse: Parse::Int,
                        },
                        write: Write::Command(&["omarchy-display-text-size"]),
                    },
                    "Text Size",
                    "Pixel size for the shell, GTK apps, and terminals",
                    9.0,
                    20.0,
                    1.0
                ),
            ],
        }],
    },
    PageDef {
        title: "Displays",
        group: PageGroup::Omarchy,
        description: "Scale, night light, and the laptop display",
        groups: &[
            GroupDef {
                title: "Scale",
                items: &[o_dropdown!(
                    "monitor-scale",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-hyprland-monitor-scaling"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-hyprland-monitor-scaling"]),
                    },
                    "Scale",
                    "Scale of the focused monitor, kept in monitors.lua",
                    Options::Static(&[
                        ("1", "1×"),
                        ("1.25", "1.25×"),
                        ("1.6", "1.6×"),
                        ("2", "2×"),
                        ("3", "3×"),
                        ("4", "4×"),
                    ])
                )],
            },
            GroupDef {
                title: "Night Light",
                items: &[
                    o_switch!(
                        "night-light",
                        Backing {
                            read: Read::Command {
                                argv: &["omarchy-toggle-nightlight", "--status"],
                                parse: Parse::JsonBool("enabled"),
                            },
                            write: Write::ToggleIfDifferent(&["omarchy-toggle-nightlight"]),
                        },
                        "Night Light",
                        "Warm the screen's colors"
                    ),
                    o_int!(
                        "night-light-temperature",
                        Backing {
                            read: Read::Command {
                                argv: &["omarchy-toggle-nightlight", "--status"],
                                parse: Parse::JsonInt {
                                    field: "temperature",
                                    fallback: 6500,
                                },
                            },
                            // The shell's indicator only re-reads on this
                            // call; `omarchy-toggle-nightlight` makes it too.
                            write: Write::CommandAnd {
                                argv: &["hyprctl", "hyprsunset", "temperature"],
                                then: &[&["omarchy-shell", "-q", "nightlight", "refresh"]],
                            },
                        },
                        "Temperature",
                        "Color temperature in kelvin while the night light is on",
                        1000.0,
                        // At 6000 and above the script counts the light as
                        // off and the switch can no longer turn it on.
                        5900.0,
                        100.0
                    ),
                ],
            },
            GroupDef {
                title: "Laptop Display",
                items: &[
                    o_switch!(
                        "internal-display",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/internal-monitor-disable.lua",
                                inverted: true,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-hyprland-monitor-internal", "on"]],
                                off: &[&["omarchy-hyprland-monitor-internal", "off"]],
                            },
                        },
                        "Laptop Display",
                        "Turn the built-in display off when using external ones",
                        LAPTOP
                    ),
                    o_switch!(
                        "mirror-internal-display",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/internal-monitor-mirror.lua",
                                inverted: false,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-hyprland-monitor-internal-mirror", "on"]],
                                off: &[&["omarchy-hyprland-monitor-internal-mirror", "off"]],
                            },
                        },
                        "Mirror Laptop Display",
                        "Show the built-in display on an external one",
                        LAPTOP
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Devices",
        group: PageGroup::Omarchy,
        description: "Touchpad, touchscreen, and Bluetooth",
        groups: &[
            GroupDef {
                title: "Input",
                items: &[
                    o_switch!(
                        "touchpad-enabled",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/touchpad-disabled-name",
                                inverted: true,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-toggle-touchpad", "on"]],
                                off: &[&["omarchy-toggle-touchpad", "off"]],
                            },
                        },
                        "Touchpad",
                        "Turn the touchpad off, for an external mouse",
                        Some(&["omarchy-hw-touchpad"])
                    ),
                    o_switch!(
                        "touchscreen-enabled",
                        Backing {
                            read: Read::Flag {
                                flag: "toggles/hypr/touchscreen-disabled-name",
                                inverted: true,
                            },
                            write: Write::Bool {
                                on: &[&["omarchy-toggle-touchscreen", "on"]],
                                off: &[&["omarchy-toggle-touchscreen", "off"]],
                            },
                        },
                        "Touchscreen",
                        "Turn touch input off",
                        Some(&["omarchy-hw-touchscreen"])
                    ),
                ],
            },
            GroupDef {
                title: "Bluetooth",
                items: &[o_switch!(
                    "bluetooth-power",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-bluetooth-power", "is-on"],
                            parse: Parse::Succeeds,
                        },
                        write: Write::Bool {
                            on: &[&["omarchy-bluetooth-power", "on"]],
                            off: &[&["omarchy-bluetooth-power", "off"]],
                        },
                    },
                    "Bluetooth",
                    "Remembered across reboots"
                )],
            },
        ],
    },
    PageDef {
        title: "Network",
        group: PageGroup::Omarchy,
        description: "DNS and Wi-Fi",
        groups: &[GroupDef {
            title: "Network",
            items: &[
                o_dropdown!(
                    "dns",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-dns"],
                            parse: Parse::Line,
                        },
                        write: Write::Command(&["omarchy-dns"]),
                    },
                    "DNS",
                    "Resolver for every connection; asks for your password",
                    Options::Static(&[
                        ("DHCP", "From the network (DHCP)"),
                        ("Cloudflare", "Cloudflare"),
                        ("Google", "Google"),
                    ])
                ),
                o_dropdown!(
                    "wifi-band",
                    Backing {
                        read: Read::Command {
                            argv: &["omarchy-network-band"],
                            // `band` is the one in use; `selected` is the pin.
                            parse: Parse::TokenOfLine("selected"),
                        },
                        write: Write::Command(&["omarchy-network-band"]),
                    },
                    "Wi-Fi Band",
                    "Pin the band of the active connection",
                    Options::Static(&[
                        ("auto", "Automatic"),
                        ("2.4", "2.4 GHz"),
                        ("5", "5 GHz"),
                        ("6", "6 GHz"),
                    ])
                ),
            ],
        }],
    },
    PageDef {
        title: "Security",
        group: PageGroup::Omarchy,
        description: "Ways to unlock and log in; each opens a terminal",
        groups: &[GroupDef {
            title: "Authentication",
            items: &[
                feature!(
                    "fingerprint",
                    "Fingerprint",
                    "Unlock, sudo, and polkit with a fingerprint",
                    Read::Command {
                        argv: &["pacman", "-Q", "fprintd"],
                        parse: Parse::Succeeds,
                    },
                    &["omarchy-setup-security-fingerprint"],
                    Some(&["omarchy-remove-security-fingerprint"]),
                    Some(&["omarchy-hw-fingerprint"])
                ),
                feature!(
                    "fido2",
                    "FIDO2 Key",
                    "Sudo and polkit with a hardware security key",
                    Read::Command {
                        argv: &["pacman", "-Q", "pam-u2f"],
                        parse: Parse::Succeeds,
                    },
                    &["omarchy-setup-security-fido2"],
                    Some(&["omarchy-remove-security-fido2"])
                ),
                feature!(
                    "sshd",
                    "SSH Server",
                    "Accept SSH logins with a key; opens the firewall",
                    Read::Command {
                        argv: &["systemctl", "is-enabled", "sshd"],
                        parse: Parse::Succeeds,
                    },
                    &["omarchy-setup-security-sshd"],
                    Some(&["omarchy-remove-security-sshd"])
                ),
                feature!(
                    "sudoless-docker",
                    "Sudoless Docker",
                    "Use Docker without sudo; equivalent to root",
                    Read::Command {
                        argv: &["omarchy-sudo-docker", "--configured"],
                        parse: Parse::Fails,
                    },
                    &["omarchy-setup-security-sudoless-docker"],
                    Some(&["omarchy-remove-security-sudoless-docker"])
                ),
            ],
        }],
    },
    PageDef {
        title: "Updates & Resets",
        group: PageGroup::Omarchy,
        description: "Package channel, firmware, time, and config resets",
        groups: &[
            GroupDef {
                title: "Updates",
                items: &[
                    o_dropdown!(
                        "update-channel",
                        Backing {
                            read: Read::Command {
                                argv: &["omarchy-channel-current"],
                                parse: Parse::Line,
                            },
                            write: Write::Terminal(&["omarchy-channel-set"]),
                        },
                        "Package Channel",
                        "Which Omarchy releases you get; switching updates at once",
                        Options::Static(&[
                            ("stable", "Stable"),
                            ("rc", "Release candidate"),
                            ("edge", "Edge"),
                            ("dev", "Development"),
                        ])
                    ),
                    action!(
                        "firmware-update",
                        "Firmware",
                        "Update device firmware with fwupd",
                        "Update…",
                        &["omarchy-update-firmware"],
                        true
                    ),
                ],
            },
            GroupDef {
                title: "Time",
                items: &[
                    // Picked here; the change itself needs sudo, so it runs in
                    // Omarchy's terminal, as `omarchy-menu-timezone` does.
                    o_dropdown!(
                        "timezone",
                        Backing {
                            read: Read::Command {
                                argv: &["timedatectl", "show", "-p", "Timezone", "--value"],
                                parse: Parse::Line,
                            },
                            write: Write::Terminal(&[
                                "bash",
                                "-c",
                                "sudo timedatectl set-timezone \"$1\" && omarchy-shell -q omarchy.clock refresh",
                                "set-timezone",
                            ]),
                        },
                        "Timezone",
                        "The system timezone",
                        Options::Lines(&["timedatectl", "list-timezones"])
                    ),
                    o_switch!(
                        "time-sync",
                        Backing {
                            read: Read::Command {
                                argv: &["timedatectl", "show", "-p", "NTP", "--value"],
                                parse: Parse::LineIs("yes"),
                            },
                            write: Write::Terminal(&[
                                "bash",
                                "-c",
                                "sudo timedatectl set-ntp \"$1\"",
                                "set-ntp",
                            ]),
                        },
                        "Automatic Time",
                        "Keep the clock in sync over the network"
                    ),
                ],
            },
            GroupDef {
                title: "Reset to Omarchy Defaults",
                items: &[
                    action!(
                        "reset-hyprland",
                        "Hyprland Config",
                        "Overwrite your ~/.config/hypr Lua files with Omarchy's",
                        "Reset…",
                        &["omarchy-refresh-hyprland"],
                        true
                    ),
                    action!(
                        "reset-shell",
                        "Shell",
                        "Reset the bar layout and shell settings",
                        "Reset…",
                        &["omarchy-refresh-shell"],
                        true
                    ),
                    action!(
                        "reset-tmux",
                        "Tmux",
                        "Overwrite your tmux config with Omarchy's",
                        "Reset…",
                        &["omarchy-refresh-tmux"],
                        true
                    ),
                    action!(
                        "reset-plymouth",
                        "Boot Screen",
                        "Reset the Plymouth boot and unlock screen",
                        "Reset…",
                        &["omarchy-refresh-plymouth"],
                        true
                    ),
                ],
            },
        ],
    },
];
