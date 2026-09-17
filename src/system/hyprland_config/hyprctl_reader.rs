use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;

use crate::types::hyprland_config::*;

/// One `j/getoption` record. Hyprland reports each option under a
/// type-specific key: `int`, `float`, `bool`, `str`, and `css` / `custom` /
/// `gradient` for multi-value options such as gaps.
#[derive(Debug, Default, Deserialize)]
struct HyprctlOption {
    #[serde(default)]
    int: Option<i64>,
    #[serde(default)]
    float: Option<f64>,
    #[serde(rename = "bool", default)]
    boolean: Option<bool>,
    #[serde(rename = "str", default)]
    string: Option<String>,
    #[serde(default)]
    custom: Option<String>,
    #[serde(default)]
    css: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HyprctlRecord {
    option: String,
    #[serde(flatten)]
    value: HyprctlOption,
}

/// Every option the config page reads, fetched in one `hyprctl --batch`
/// call; one process per option is slow enough to freeze the window.
const OPTION_KEYS: &[&str] = &[
    "general:border_size",
    "general:gaps_in",
    "general:gaps_out",
    "general:gaps_workspaces",
    "general:layout",
    "general:no_focus_fallback",
    "general:resize_on_border",
    "general:extend_border_grab_area",
    "general:hover_icon_on_border",
    "general:allow_tearing",
    "general:resize_corner",
    "general:modal_parent_blocking",
    "general:snap:enabled",
    "general:snap:window_gap",
    "general:snap:monitor_gap",
    "general:snap:border_overlap",
    "general:snap:respect_gaps",
    "decoration:rounding",
    "decoration:rounding_power",
    "decoration:active_opacity",
    "decoration:inactive_opacity",
    "decoration:fullscreen_opacity",
    "decoration:dim_inactive",
    "decoration:dim_strength",
    "decoration:dim_special",
    "decoration:dim_around",
    "decoration:border_part_of_window",
    "decoration:dim_modal",
    "decoration:blur:enabled",
    "decoration:blur:size",
    "decoration:blur:passes",
    "decoration:blur:ignore_opacity",
    "decoration:blur:new_optimizations",
    "decoration:blur:xray",
    "decoration:blur:noise",
    "decoration:blur:contrast",
    "decoration:blur:brightness",
    "decoration:blur:vibrancy",
    "decoration:blur:vibrancy_darkness",
    "decoration:blur:special",
    "decoration:blur:popups",
    "decoration:blur:popups_ignorealpha",
    "decoration:shadow:enabled",
    "decoration:shadow:range",
    "decoration:shadow:render_power",
    "decoration:shadow:sharp",
    "decoration:shadow:ignore_window",
    "decoration:shadow:offset_x",
    "decoration:shadow:offset_y",
    "decoration:shadow:scale",
    "animations:enabled",
    "animations:workspace_wraparound",
    "input:kb_model",
    "input:kb_layout",
    "input:kb_variant",
    "input:kb_options",
    "input:kb_rules",
    "input:numlock_by_default",
    "input:resolve_binds_by_sym",
    "input:repeat_rate",
    "input:repeat_delay",
    "input:sensitivity",
    "input:accel_profile",
    "input:force_no_accel",
    "input:rotation",
    "input:left_handed",
    "input:scroll_method",
    "input:scroll_button",
    "input:scroll_button_lock",
    "input:scroll_factor",
    "input:natural_scroll",
    "input:follow_mouse",
    "input:follow_mouse_threshold",
    "input:focus_on_close",
    "input:mouse_refocus",
    "input:float_switch_override_focus",
    "input:special_fallthrough",
    "input:off_window_axis_events",
    "input:emulate_discrete_scroll",
    "input:touchpad:disable_while_typing",
    "input:touchpad:natural_scroll",
    "input:touchpad:scroll_factor",
    "input:touchpad:middle_button_emulation",
    "input:touchpad:tap_button_map",
    "input:touchpad:clickfinger_behavior",
    "input:touchpad:tap_to_click",
    "input:touchpad:tap_and_drag",
    "input:touchpad:flip_x",
    "input:touchpad:flip_y",
    "gestures:workspace_swipe_distance",
    "gestures:workspace_swipe_touch",
    "gestures:workspace_swipe_invert",
    "gestures:workspace_swipe_touch_invert",
    "gestures:workspace_swipe_min_speed_to_force",
    "gestures:workspace_swipe_cancel_ratio",
    "gestures:workspace_swipe_create_new",
    "gestures:workspace_swipe_direction_lock",
    "gestures:workspace_swipe_direction_lock_threshold",
    "gestures:workspace_swipe_forever",
    "gestures:workspace_swipe_use_r",
    "misc:disable_hyprland_logo",
    "misc:disable_splash_rendering",
    "misc:font_family",
    "misc:force_default_wallpaper",
    "misc:vfr",
    "misc:vrr",
    "misc:mouse_move_enables_dpms",
    "misc:key_press_enables_dpms",
    "misc:always_follow_on_dnd",
    "misc:layers_hog_keyboard_focus",
    "misc:animate_manual_resizes",
    "misc:animate_mouse_windowdragging",
    "misc:disable_autoreload",
    "misc:enable_swallow",
    "misc:swallow_regex",
    "misc:swallow_exception_regex",
    "misc:focus_on_activate",
    "misc:mouse_move_focuses_monitor",
    "misc:close_special_on_empty",
    "misc:on_focus_under_fullscreen",
    "misc:exit_window_retains_fullscreen",
    "misc:initial_workspace_tracking",
    "misc:middle_click_paste",
    "binds:pass_mouse_when_bound",
    "binds:scroll_event_delay",
    "binds:workspace_back_and_forth",
    "binds:hide_special_on_workspace_change",
    "binds:allow_workspace_cycles",
    "binds:workspace_center_on",
    "binds:focus_preferred_method",
    "binds:ignore_group_lock",
    "binds:movefocus_cycles_fullscreen",
    "binds:movefocus_cycles_groupfirst",
    "binds:disable_keybind_grabbing",
    "binds:window_direction_monitor_fallback",
    "binds:allow_pin_fullscreen",
    "binds:drag_threshold",
    "cursor:invisible",
    "cursor:sync_gsettings_theme",
    "cursor:inactive_timeout",
    "cursor:no_warps",
    "cursor:persistent_warps",
    "cursor:zoom_factor",
    "cursor:zoom_rigid",
    "cursor:enable_hyprcursor",
    "cursor:hide_on_key_press",
    "cursor:hide_on_touch",
    "cursor:hide_on_tablet",
    "cursor:warp_back_after_non_mouse_input",
    "xwayland:enabled",
    "xwayland:use_nearest_neighbor",
    "xwayland:force_zero_scaling",
    "opengl:nvidia_anti_flicker",
    "render:direct_scanout",
    "render:expand_undersized_textures",
    "render:xp_mode",
    "render:cm_enabled",
    "render:new_render_scheduling",
];

struct Options(HashMap<String, HyprctlOption>);

impl Options {
    fn fetch(keys: &[&str]) -> Self {
        let batch = keys
            .iter()
            .map(|k| format!("j/getoption {k}"))
            .collect::<Vec<_>>()
            .join("; ");

        // Unknown options print "no such option" instead of JSON; parse
        // whatever came back rather than failing the whole read.
        match Command::new("hyprctl").args(["--batch", &batch]).output() {
            Ok(output) => Self::parse(&String::from_utf8_lossy(&output.stdout)),
            Err(_) => Self(HashMap::new()),
        }
    }

    fn parse(text: &str) -> Self {
        Self(
            text.lines()
                .filter_map(|line| serde_json::from_str::<HyprctlRecord>(line.trim()).ok())
                .map(|record| (record.option, record.value))
                .collect(),
        )
    }

    fn get(&self, key: &str) -> Option<&HyprctlOption> {
        self.0.get(key)
    }

    fn int(&self, key: &str) -> Option<i64> {
        self.get(key)?.int
    }

    fn boolean(&self, key: &str) -> Option<bool> {
        let opt = self.get(key)?;
        opt.boolean.or(opt.int.map(|v| v != 0))
    }

    fn float(&self, key: &str) -> Option<f64> {
        self.get(key)?.float
    }

    fn string(&self, key: &str) -> Option<String> {
        self.get(key)?.string.clone()
    }

    /// For options like `gaps_in` / `gaps_out` that come back as
    /// `"css": "5 5 5 5"`, take the first token as a single i32.
    fn custom_first_int(&self, key: &str) -> Option<i32> {
        let opt = self.get(key)?;
        if let Some(i) = opt.int {
            return Some(i as i32);
        }
        let raw = opt.css.as_deref().or(opt.custom.as_deref())?;
        raw.split_whitespace().next()?.parse::<i32>().ok()
    }
}

/// Read as many Hyprland settings as possible from the running compositor and
/// return them as a [`HyprlandConfig`].
///
/// Only the fields that can be retrieved are overridden; everything else
/// keeps its `Default` value so we always return a fully-populated struct.
pub fn read_from_hyprctl() -> HyprlandConfig {
    read_from_options(&Options::fetch(OPTION_KEYS))
}

fn read_from_options(opts: &Options) -> HyprlandConfig {
    let mut cfg = HyprlandConfig::default();

    // ── general ──────────────────────────────────────────────────────────────
    if let Some(v) = opts.int("general:border_size") {
        cfg.general.border_size = v as i32;
    }
    if let Some(v) = opts.custom_first_int("general:gaps_in") {
        cfg.general.gaps_in = v;
    }
    if let Some(v) = opts.custom_first_int("general:gaps_out") {
        cfg.general.gaps_out = v;
    }
    if let Some(v) = opts.custom_first_int("general:gaps_workspaces") {
        cfg.general.gaps_workspaces = v;
    }
    if let Some(v) = opts.string("general:layout") {
        cfg.general.layout = v;
    }
    if let Some(v) = opts.boolean("general:no_focus_fallback") {
        cfg.general.no_focus_fallback = v;
    }
    if let Some(v) = opts.boolean("general:resize_on_border") {
        cfg.general.resize_on_border = v;
    }
    if let Some(v) = opts.int("general:extend_border_grab_area") {
        cfg.general.extend_border_grab_area = v as i32;
    }
    if let Some(v) = opts.boolean("general:hover_icon_on_border") {
        cfg.general.hover_icon_on_border = v;
    }
    if let Some(v) = opts.boolean("general:allow_tearing") {
        cfg.general.allow_tearing = v;
    }
    if let Some(v) = opts.int("general:resize_corner") {
        cfg.general.resize_corner = v as i32;
    }
    if let Some(v) = opts.boolean("general:modal_parent_blocking") {
        cfg.general.modal_parent_blocking = v;
    }

    // general:snap
    if let Some(v) = opts.boolean("general:snap:enabled") {
        cfg.general.snap.enabled = v;
    }
    if let Some(v) = opts.int("general:snap:window_gap") {
        cfg.general.snap.window_gap = v as i32;
    }
    if let Some(v) = opts.int("general:snap:monitor_gap") {
        cfg.general.snap.monitor_gap = v as i32;
    }
    if let Some(v) = opts.boolean("general:snap:border_overlap") {
        cfg.general.snap.border_overlap = v;
    }
    if let Some(v) = opts.boolean("general:snap:respect_gaps") {
        cfg.general.snap.respect_gaps = v;
    }

    // ── decoration ───────────────────────────────────────────────────────────
    if let Some(v) = opts.int("decoration:rounding") {
        cfg.decoration.rounding = v as i32;
    }
    if let Some(v) = opts.float("decoration:rounding_power") {
        cfg.decoration.rounding_power = v;
    }
    if let Some(v) = opts.float("decoration:active_opacity") {
        cfg.decoration.active_opacity = v;
    }
    if let Some(v) = opts.float("decoration:inactive_opacity") {
        cfg.decoration.inactive_opacity = v;
    }
    if let Some(v) = opts.float("decoration:fullscreen_opacity") {
        cfg.decoration.fullscreen_opacity = v;
    }
    if let Some(v) = opts.boolean("decoration:dim_inactive") {
        cfg.decoration.dim_inactive = v;
    }
    if let Some(v) = opts.float("decoration:dim_strength") {
        cfg.decoration.dim_strength = v;
    }
    if let Some(v) = opts.float("decoration:dim_special") {
        cfg.decoration.dim_special = v;
    }
    if let Some(v) = opts.float("decoration:dim_around") {
        cfg.decoration.dim_around = v;
    }
    if let Some(v) = opts.boolean("decoration:border_part_of_window") {
        cfg.decoration.border_part_of_window = v;
    }
    if let Some(v) = opts.boolean("decoration:dim_modal") {
        cfg.decoration.dim_modal = v;
    }

    // decoration:blur
    if let Some(v) = opts.boolean("decoration:blur:enabled") {
        cfg.decoration.blur.enabled = v;
    }
    if let Some(v) = opts.int("decoration:blur:size") {
        cfg.decoration.blur.size = v as i32;
    }
    if let Some(v) = opts.int("decoration:blur:passes") {
        cfg.decoration.blur.passes = v as i32;
    }
    if let Some(v) = opts.boolean("decoration:blur:ignore_opacity") {
        cfg.decoration.blur.ignore_opacity = v;
    }
    if let Some(v) = opts.boolean("decoration:blur:new_optimizations") {
        cfg.decoration.blur.new_optimizations = v;
    }
    if let Some(v) = opts.boolean("decoration:blur:xray") {
        cfg.decoration.blur.xray = v;
    }
    if let Some(v) = opts.float("decoration:blur:noise") {
        cfg.decoration.blur.noise = v;
    }
    if let Some(v) = opts.float("decoration:blur:contrast") {
        cfg.decoration.blur.contrast = v;
    }
    if let Some(v) = opts.float("decoration:blur:brightness") {
        cfg.decoration.blur.brightness = v;
    }
    if let Some(v) = opts.float("decoration:blur:vibrancy") {
        cfg.decoration.blur.vibrancy = v;
    }
    if let Some(v) = opts.float("decoration:blur:vibrancy_darkness") {
        cfg.decoration.blur.vibrancy_darkness = v;
    }
    if let Some(v) = opts.boolean("decoration:blur:special") {
        cfg.decoration.blur.special = v;
    }
    if let Some(v) = opts.boolean("decoration:blur:popups") {
        cfg.decoration.blur.popups = v;
    }
    if let Some(v) = opts.float("decoration:blur:popups_ignorealpha") {
        cfg.decoration.blur.popups_ignorealpha = v;
    }

    // decoration:shadow
    if let Some(v) = opts.boolean("decoration:shadow:enabled") {
        cfg.decoration.shadow.enabled = v;
    }
    if let Some(v) = opts.int("decoration:shadow:range") {
        cfg.decoration.shadow.range = v as i32;
    }
    if let Some(v) = opts.int("decoration:shadow:render_power") {
        cfg.decoration.shadow.render_power = v as i32;
    }
    if let Some(v) = opts.boolean("decoration:shadow:sharp") {
        cfg.decoration.shadow.sharp = v;
    }
    if let Some(v) = opts.boolean("decoration:shadow:ignore_window") {
        cfg.decoration.shadow.ignore_window = v;
    }
    if let Some(v) = opts.float("decoration:shadow:offset_x") {
        cfg.decoration.shadow.offset_x = v;
    }
    if let Some(v) = opts.float("decoration:shadow:offset_y") {
        cfg.decoration.shadow.offset_y = v;
    }
    if let Some(v) = opts.float("decoration:shadow:scale") {
        cfg.decoration.shadow.scale = v;
    }

    // ── animations ───────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("animations:enabled") {
        cfg.animations.enabled = v;
    }
    if let Some(v) = opts.boolean("animations:workspace_wraparound") {
        cfg.animations.workspace_wraparound = v;
    }

    // ── input ────────────────────────────────────────────────────────────────
    if let Some(v) = opts.string("input:kb_model") {
        cfg.input.kb_model = v;
    }
    if let Some(v) = opts.string("input:kb_layout") {
        cfg.input.kb_layout = v;
    }
    if let Some(v) = opts.string("input:kb_variant") {
        cfg.input.kb_variant = v;
    }
    if let Some(v) = opts.string("input:kb_options") {
        cfg.input.kb_options = v;
    }
    if let Some(v) = opts.string("input:kb_rules") {
        cfg.input.kb_rules = v;
    }
    if let Some(v) = opts.boolean("input:numlock_by_default") {
        cfg.input.numlock_by_default = v;
    }
    if let Some(v) = opts.boolean("input:resolve_binds_by_sym") {
        cfg.input.resolve_binds_by_sym = v;
    }
    if let Some(v) = opts.int("input:repeat_rate") {
        cfg.input.repeat_rate = v as i32;
    }
    if let Some(v) = opts.int("input:repeat_delay") {
        cfg.input.repeat_delay = v as i32;
    }
    if let Some(v) = opts.float("input:sensitivity") {
        cfg.input.sensitivity = v;
    }
    if let Some(v) = opts.string("input:accel_profile") {
        cfg.input.accel_profile = v;
    }
    if let Some(v) = opts.boolean("input:force_no_accel") {
        cfg.input.force_no_accel = v;
    }
    if let Some(v) = opts.int("input:rotation") {
        cfg.input.rotation = v as i32;
    }
    if let Some(v) = opts.boolean("input:left_handed") {
        cfg.input.left_handed = v;
    }
    if let Some(v) = opts.string("input:scroll_method") {
        cfg.input.scroll_method = v;
    }
    if let Some(v) = opts.int("input:scroll_button") {
        cfg.input.scroll_button = v as i32;
    }
    if let Some(v) = opts.boolean("input:scroll_button_lock") {
        cfg.input.scroll_button_lock = v;
    }
    if let Some(v) = opts.float("input:scroll_factor") {
        cfg.input.scroll_factor = v;
    }
    if let Some(v) = opts.boolean("input:natural_scroll") {
        cfg.input.natural_scroll = v;
    }
    if let Some(v) = opts.int("input:follow_mouse") {
        cfg.input.follow_mouse = v as i32;
    }
    if let Some(v) = opts.float("input:follow_mouse_threshold") {
        cfg.input.follow_mouse_threshold = v;
    }
    if let Some(v) = opts.int("input:focus_on_close") {
        cfg.input.focus_on_close = v as i32;
    }
    if let Some(v) = opts.boolean("input:mouse_refocus") {
        cfg.input.mouse_refocus = v;
    }
    if let Some(v) = opts.int("input:float_switch_override_focus") {
        cfg.input.float_switch_override_focus = v as i32;
    }
    if let Some(v) = opts.boolean("input:special_fallthrough") {
        cfg.input.special_fallthrough = v;
    }
    if let Some(v) = opts.int("input:off_window_axis_events") {
        cfg.input.off_window_axis_events = v as i32;
    }
    if let Some(v) = opts.int("input:emulate_discrete_scroll") {
        cfg.input.emulate_discrete_scroll = v as i32;
    }

    // input:touchpad
    if let Some(v) = opts.boolean("input:touchpad:disable_while_typing") {
        cfg.input.touchpad.disable_while_typing = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:natural_scroll") {
        cfg.input.touchpad.natural_scroll = v;
    }
    if let Some(v) = opts.float("input:touchpad:scroll_factor") {
        cfg.input.touchpad.scroll_factor = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:middle_button_emulation") {
        cfg.input.touchpad.middle_button_emulation = v;
    }
    if let Some(v) = opts.string("input:touchpad:tap_button_map") {
        cfg.input.touchpad.tap_button_map = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:clickfinger_behavior") {
        cfg.input.touchpad.clickfinger_behavior = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:tap_to_click") {
        cfg.input.touchpad.tap_to_click = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:tap_and_drag") {
        cfg.input.touchpad.tap_and_drag = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:flip_x") {
        cfg.input.touchpad.flip_x = v;
    }
    if let Some(v) = opts.boolean("input:touchpad:flip_y") {
        cfg.input.touchpad.flip_y = v;
    }

    // ── gestures ─────────────────────────────────────────────────────────────
    if let Some(v) = opts.int("gestures:workspace_swipe_distance") {
        cfg.gestures.workspace_swipe_distance = v as i32;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_touch") {
        cfg.gestures.workspace_swipe_touch = v;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_invert") {
        cfg.gestures.workspace_swipe_invert = v;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_touch_invert") {
        cfg.gestures.workspace_swipe_touch_invert = v;
    }
    if let Some(v) = opts.int("gestures:workspace_swipe_min_speed_to_force") {
        cfg.gestures.workspace_swipe_min_speed_to_force = v as i32;
    }
    if let Some(v) = opts.float("gestures:workspace_swipe_cancel_ratio") {
        cfg.gestures.workspace_swipe_cancel_ratio = v;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_create_new") {
        cfg.gestures.workspace_swipe_create_new = v;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_direction_lock") {
        cfg.gestures.workspace_swipe_direction_lock = v;
    }
    if let Some(v) = opts.int("gestures:workspace_swipe_direction_lock_threshold") {
        cfg.gestures.workspace_swipe_direction_lock_threshold = v as i32;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_forever") {
        cfg.gestures.workspace_swipe_forever = v;
    }
    if let Some(v) = opts.boolean("gestures:workspace_swipe_use_r") {
        cfg.gestures.workspace_swipe_use_r = v;
    }

    // ── misc ─────────────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("misc:disable_hyprland_logo") {
        cfg.misc.disable_hyprland_logo = v;
    }
    if let Some(v) = opts.boolean("misc:disable_splash_rendering") {
        cfg.misc.disable_splash_rendering = v;
    }
    if let Some(v) = opts.string("misc:font_family") {
        cfg.misc.font_family = v;
    }
    if let Some(v) = opts.int("misc:force_default_wallpaper") {
        cfg.misc.force_default_wallpaper = v as i32;
    }
    if let Some(v) = opts.boolean("misc:vfr") {
        cfg.misc.vfr = v;
    }
    if let Some(v) = opts.int("misc:vrr") {
        cfg.misc.vrr = v as i32;
    }
    if let Some(v) = opts.boolean("misc:mouse_move_enables_dpms") {
        cfg.misc.mouse_move_enables_dpms = v;
    }
    if let Some(v) = opts.boolean("misc:key_press_enables_dpms") {
        cfg.misc.key_press_enables_dpms = v;
    }
    if let Some(v) = opts.boolean("misc:always_follow_on_dnd") {
        cfg.misc.always_follow_on_dnd = v;
    }
    if let Some(v) = opts.boolean("misc:layers_hog_keyboard_focus") {
        cfg.misc.layers_hog_keyboard_focus = v;
    }
    if let Some(v) = opts.boolean("misc:animate_manual_resizes") {
        cfg.misc.animate_manual_resizes = v;
    }
    if let Some(v) = opts.boolean("misc:animate_mouse_windowdragging") {
        cfg.misc.animate_mouse_windowdragging = v;
    }
    if let Some(v) = opts.boolean("misc:disable_autoreload") {
        cfg.misc.disable_autoreload = v;
    }
    if let Some(v) = opts.boolean("misc:enable_swallow") {
        cfg.misc.enable_swallow = v;
    }
    if let Some(v) = opts.string("misc:swallow_regex") {
        cfg.misc.swallow_regex = v;
    }
    if let Some(v) = opts.string("misc:swallow_exception_regex") {
        cfg.misc.swallow_exception_regex = v;
    }
    if let Some(v) = opts.boolean("misc:focus_on_activate") {
        cfg.misc.focus_on_activate = v;
    }
    if let Some(v) = opts.boolean("misc:mouse_move_focuses_monitor") {
        cfg.misc.mouse_move_focuses_monitor = v;
    }
    if let Some(v) = opts.boolean("misc:close_special_on_empty") {
        cfg.misc.close_special_on_empty = v;
    }
    if let Some(v) = opts.int("misc:on_focus_under_fullscreen") {
        cfg.misc.on_focus_under_fullscreen = v as i32;
    }
    if let Some(v) = opts.boolean("misc:exit_window_retains_fullscreen") {
        cfg.misc.exit_window_retains_fullscreen = v;
    }
    if let Some(v) = opts.int("misc:initial_workspace_tracking") {
        cfg.misc.initial_workspace_tracking = v as i32;
    }
    if let Some(v) = opts.boolean("misc:middle_click_paste") {
        cfg.misc.middle_click_paste = v;
    }

    // ── binds ────────────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("binds:pass_mouse_when_bound") {
        cfg.binds.pass_mouse_when_bound = v;
    }
    if let Some(v) = opts.int("binds:scroll_event_delay") {
        cfg.binds.scroll_event_delay = v as i32;
    }
    if let Some(v) = opts.boolean("binds:workspace_back_and_forth") {
        cfg.binds.workspace_back_and_forth = v;
    }
    if let Some(v) = opts.boolean("binds:hide_special_on_workspace_change") {
        cfg.binds.hide_special_on_workspace_change = v;
    }
    if let Some(v) = opts.boolean("binds:allow_workspace_cycles") {
        cfg.binds.allow_workspace_cycles = v;
    }
    if let Some(v) = opts.int("binds:workspace_center_on") {
        cfg.binds.workspace_center_on = v as i32;
    }
    if let Some(v) = opts.int("binds:focus_preferred_method") {
        cfg.binds.focus_preferred_method = v as i32;
    }
    if let Some(v) = opts.boolean("binds:ignore_group_lock") {
        cfg.binds.ignore_group_lock = v;
    }
    if let Some(v) = opts.boolean("binds:movefocus_cycles_fullscreen") {
        cfg.binds.movefocus_cycles_fullscreen = v;
    }
    if let Some(v) = opts.boolean("binds:movefocus_cycles_groupfirst") {
        cfg.binds.movefocus_cycles_groupfirst = v;
    }
    if let Some(v) = opts.boolean("binds:disable_keybind_grabbing") {
        cfg.binds.disable_keybind_grabbing = v;
    }
    if let Some(v) = opts.boolean("binds:window_direction_monitor_fallback") {
        cfg.binds.window_direction_monitor_fallback = v;
    }
    if let Some(v) = opts.boolean("binds:allow_pin_fullscreen") {
        cfg.binds.allow_pin_fullscreen = v;
    }
    if let Some(v) = opts.int("binds:drag_threshold") {
        cfg.binds.drag_threshold = v as i32;
    }

    // ── cursor ───────────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("cursor:invisible") {
        cfg.cursor.invisible = v;
    }
    if let Some(v) = opts.boolean("cursor:sync_gsettings_theme") {
        cfg.cursor.sync_gsettings_theme = v;
    }
    if let Some(v) = opts.float("cursor:inactive_timeout") {
        cfg.cursor.inactive_timeout = v;
    }
    if let Some(v) = opts.boolean("cursor:no_warps") {
        cfg.cursor.no_warps = v;
    }
    if let Some(v) = opts.boolean("cursor:persistent_warps") {
        cfg.cursor.persistent_warps = v;
    }
    if let Some(v) = opts.float("cursor:zoom_factor") {
        cfg.cursor.zoom_factor = v;
    }
    if let Some(v) = opts.boolean("cursor:zoom_rigid") {
        cfg.cursor.zoom_rigid = v;
    }
    if let Some(v) = opts.boolean("cursor:enable_hyprcursor") {
        cfg.cursor.enable_hyprcursor = v;
    }
    if let Some(v) = opts.boolean("cursor:hide_on_key_press") {
        cfg.cursor.hide_on_key_press = v;
    }
    if let Some(v) = opts.boolean("cursor:hide_on_touch") {
        cfg.cursor.hide_on_touch = v;
    }
    if let Some(v) = opts.boolean("cursor:hide_on_tablet") {
        cfg.cursor.hide_on_tablet = v;
    }
    if let Some(v) = opts.boolean("cursor:warp_back_after_non_mouse_input") {
        cfg.cursor.warp_back_after_non_mouse_input = v;
    }

    // ── xwayland ─────────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("xwayland:enabled") {
        cfg.xwayland.enabled = v;
    }
    if let Some(v) = opts.boolean("xwayland:use_nearest_neighbor") {
        cfg.xwayland.use_nearest_neighbor = v;
    }
    if let Some(v) = opts.boolean("xwayland:force_zero_scaling") {
        cfg.xwayland.force_zero_scaling = v;
    }

    // ── opengl ───────────────────────────────────────────────────────────────
    if let Some(v) = opts.boolean("opengl:nvidia_anti_flicker") {
        cfg.opengl.nvidia_anti_flicker = v;
    }

    // ── render ───────────────────────────────────────────────────────────────
    if let Some(v) = opts.int("render:direct_scanout") {
        cfg.render.direct_scanout = v as i32;
    }
    if let Some(v) = opts.boolean("render:expand_undersized_textures") {
        cfg.render.expand_undersized_textures = v;
    }
    if let Some(v) = opts.boolean("render:xp_mode") {
        cfg.render.xp_mode = v;
    }
    if let Some(v) = opts.boolean("render:cm_enabled") {
        cfg.render.cm_enabled = v;
    }
    if let Some(v) = opts.boolean("render:new_render_scheduling") {
        cfg.render.new_render_scheduling = v;
    }

    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{"option": "general:border_size", "int": 3, "set": true }


{"option": "general:gaps_in", "css": "7 7 7 7", "set": true }


{"option": "general:resize_on_border", "bool": true, "set": true }


{"option": "decoration:active_opacity", "float": 0.950000, "set": true }


{"option": "general:layout", "str": "master", "set": true }
no such option
"#;

    #[test]
    fn parse_reads_every_value_type_and_skips_noise() {
        let opts = Options::parse(SAMPLE);
        assert_eq!(opts.int("general:border_size"), Some(3));
        assert_eq!(opts.custom_first_int("general:gaps_in"), Some(7));
        assert_eq!(opts.boolean("general:resize_on_border"), Some(true));
        assert_eq!(opts.float("decoration:active_opacity"), Some(0.95));
        assert_eq!(opts.string("general:layout").as_deref(), Some("master"));
        assert_eq!(opts.int("missing:key"), None);
    }

    #[test]
    fn read_from_options_overrides_only_what_was_returned() {
        let cfg = read_from_options(&Options::parse(SAMPLE));
        let defaults = HyprlandConfig::default();
        assert_eq!(cfg.general.border_size, 3);
        assert_eq!(cfg.general.gaps_in, 7);
        assert!(cfg.general.resize_on_border);
        assert_eq!(cfg.general.layout, "master");
        assert_eq!(cfg.general.gaps_out, defaults.general.gaps_out);
    }

    #[test]
    fn option_keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for key in OPTION_KEYS {
            assert!(seen.insert(key), "duplicate option key {key}");
        }
    }
}
