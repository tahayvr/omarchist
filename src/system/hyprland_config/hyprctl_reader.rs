use std::process::Command;

use serde::Deserialize;

use crate::types::hyprland_config::*;

/// The JSON shape returned by `hyprctl getoption <key> -j`.
#[derive(Debug, Deserialize)]
struct HyprctlOption {
    #[serde(default)]
    int: Option<i64>,
    #[serde(default)]
    float: Option<f64>,
    #[serde(rename = "str", default)]
    string: Option<String>,
    #[serde(default)]
    custom: Option<String>,
}

/// Run `hyprctl getoption <key> -j` and parse the result.
fn get_option(key: &str) -> Option<HyprctlOption> {
    let output = Command::new("hyprctl")
        .args(["getoption", key, "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

fn opt_int(key: &str) -> Option<i64> {
    get_option(key)?.int
}

fn opt_bool(key: &str) -> Option<bool> {
    opt_int(key).map(|v| v != 0)
}

fn opt_float(key: &str) -> Option<f64> {
    get_option(key)?.float
}

fn opt_str(key: &str) -> Option<String> {
    get_option(key)?.string
}

/// For options like `gaps_in` / `gaps_out` that return `"custom": "5 5 5 5"`,
/// we just take the first token as a single i32 value.
fn opt_custom_first_int(key: &str) -> Option<i32> {
    let opt = get_option(key)?;
    // May come back as `custom` or sometimes as `int`
    if let Some(i) = opt.int {
        return Some(i as i32);
    }
    let raw = opt.custom?;
    raw.split_whitespace().next()?.parse::<i32>().ok()
}

/// Read as many Hyprland settings as possible from the running compositor via
/// `hyprctl getoption` and return them as a [`HyprlandConfig`].
///
/// Only the fields that can be retrieved are overridden; everything else
/// keeps its `Default` value so we always return a fully-populated struct.
pub fn read_from_hyprctl() -> HyprlandConfig {
    let mut cfg = HyprlandConfig::default();

    // ── general ──────────────────────────────────────────────────────────────
    if let Some(v) = opt_int("general:border_size") {
        cfg.general.border_size = v as i32;
    }
    if let Some(v) = opt_custom_first_int("general:gaps_in") {
        cfg.general.gaps_in = v;
    }
    if let Some(v) = opt_custom_first_int("general:gaps_out") {
        cfg.general.gaps_out = v;
    }
    if let Some(v) = opt_custom_first_int("general:gaps_workspaces") {
        cfg.general.gaps_workspaces = v;
    }
    if let Some(v) = opt_str("general:layout") {
        cfg.general.layout = v;
    }
    if let Some(v) = opt_bool("general:no_focus_fallback") {
        cfg.general.no_focus_fallback = v;
    }
    if let Some(v) = opt_bool("general:resize_on_border") {
        cfg.general.resize_on_border = v;
    }
    if let Some(v) = opt_int("general:extend_border_grab_area") {
        cfg.general.extend_border_grab_area = v as i32;
    }
    if let Some(v) = opt_bool("general:hover_icon_on_border") {
        cfg.general.hover_icon_on_border = v;
    }
    if let Some(v) = opt_bool("general:allow_tearing") {
        cfg.general.allow_tearing = v;
    }
    if let Some(v) = opt_int("general:resize_corner") {
        cfg.general.resize_corner = v as i32;
    }
    if let Some(v) = opt_bool("general:modal_parent_blocking") {
        cfg.general.modal_parent_blocking = v;
    }

    // general:snap
    if let Some(v) = opt_bool("general:snap:enabled") {
        cfg.general.snap.enabled = v;
    }
    if let Some(v) = opt_int("general:snap:window_gap") {
        cfg.general.snap.window_gap = v as i32;
    }
    if let Some(v) = opt_int("general:snap:monitor_gap") {
        cfg.general.snap.monitor_gap = v as i32;
    }
    if let Some(v) = opt_bool("general:snap:border_overlap") {
        cfg.general.snap.border_overlap = v;
    }
    if let Some(v) = opt_bool("general:snap:respect_gaps") {
        cfg.general.snap.respect_gaps = v;
    }

    // ── decoration ───────────────────────────────────────────────────────────
    if let Some(v) = opt_int("decoration:rounding") {
        cfg.decoration.rounding = v as i32;
    }
    if let Some(v) = opt_float("decoration:rounding_power") {
        cfg.decoration.rounding_power = v;
    }
    if let Some(v) = opt_float("decoration:active_opacity") {
        cfg.decoration.active_opacity = v;
    }
    if let Some(v) = opt_float("decoration:inactive_opacity") {
        cfg.decoration.inactive_opacity = v;
    }
    if let Some(v) = opt_float("decoration:fullscreen_opacity") {
        cfg.decoration.fullscreen_opacity = v;
    }
    if let Some(v) = opt_bool("decoration:dim_inactive") {
        cfg.decoration.dim_inactive = v;
    }
    if let Some(v) = opt_float("decoration:dim_strength") {
        cfg.decoration.dim_strength = v;
    }
    if let Some(v) = opt_float("decoration:dim_special") {
        cfg.decoration.dim_special = v;
    }
    if let Some(v) = opt_float("decoration:dim_around") {
        cfg.decoration.dim_around = v;
    }
    if let Some(v) = opt_bool("decoration:border_part_of_window") {
        cfg.decoration.border_part_of_window = v;
    }
    if let Some(v) = opt_bool("decoration:dim_modal") {
        cfg.decoration.dim_modal = v;
    }

    // decoration:blur
    if let Some(v) = opt_bool("decoration:blur:enabled") {
        cfg.decoration.blur.enabled = v;
    }
    if let Some(v) = opt_int("decoration:blur:size") {
        cfg.decoration.blur.size = v as i32;
    }
    if let Some(v) = opt_int("decoration:blur:passes") {
        cfg.decoration.blur.passes = v as i32;
    }
    if let Some(v) = opt_bool("decoration:blur:ignore_opacity") {
        cfg.decoration.blur.ignore_opacity = v;
    }
    if let Some(v) = opt_bool("decoration:blur:new_optimizations") {
        cfg.decoration.blur.new_optimizations = v;
    }
    if let Some(v) = opt_bool("decoration:blur:xray") {
        cfg.decoration.blur.xray = v;
    }
    if let Some(v) = opt_float("decoration:blur:noise") {
        cfg.decoration.blur.noise = v;
    }
    if let Some(v) = opt_float("decoration:blur:contrast") {
        cfg.decoration.blur.contrast = v;
    }
    if let Some(v) = opt_float("decoration:blur:brightness") {
        cfg.decoration.blur.brightness = v;
    }
    if let Some(v) = opt_float("decoration:blur:vibrancy") {
        cfg.decoration.blur.vibrancy = v;
    }
    if let Some(v) = opt_float("decoration:blur:vibrancy_darkness") {
        cfg.decoration.blur.vibrancy_darkness = v;
    }
    if let Some(v) = opt_bool("decoration:blur:special") {
        cfg.decoration.blur.special = v;
    }
    if let Some(v) = opt_bool("decoration:blur:popups") {
        cfg.decoration.blur.popups = v;
    }
    if let Some(v) = opt_float("decoration:blur:popups_ignorealpha") {
        cfg.decoration.blur.popups_ignorealpha = v;
    }

    // decoration:shadow
    if let Some(v) = opt_bool("decoration:shadow:enabled") {
        cfg.decoration.shadow.enabled = v;
    }
    if let Some(v) = opt_int("decoration:shadow:range") {
        cfg.decoration.shadow.range = v as i32;
    }
    if let Some(v) = opt_int("decoration:shadow:render_power") {
        cfg.decoration.shadow.render_power = v as i32;
    }
    if let Some(v) = opt_bool("decoration:shadow:sharp") {
        cfg.decoration.shadow.sharp = v;
    }
    if let Some(v) = opt_bool("decoration:shadow:ignore_window") {
        cfg.decoration.shadow.ignore_window = v;
    }
    if let Some(v) = opt_float("decoration:shadow:offset_x") {
        cfg.decoration.shadow.offset_x = v;
    }
    if let Some(v) = opt_float("decoration:shadow:offset_y") {
        cfg.decoration.shadow.offset_y = v;
    }
    if let Some(v) = opt_float("decoration:shadow:scale") {
        cfg.decoration.shadow.scale = v;
    }

    // ── animations ───────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("animations:enabled") {
        cfg.animations.enabled = v;
    }
    if let Some(v) = opt_bool("animations:workspace_wraparound") {
        cfg.animations.workspace_wraparound = v;
    }

    // ── input ────────────────────────────────────────────────────────────────
    if let Some(v) = opt_str("input:kb_model") {
        cfg.input.kb_model = v;
    }
    if let Some(v) = opt_str("input:kb_layout") {
        cfg.input.kb_layout = v;
    }
    if let Some(v) = opt_str("input:kb_variant") {
        cfg.input.kb_variant = v;
    }
    if let Some(v) = opt_str("input:kb_options") {
        cfg.input.kb_options = v;
    }
    if let Some(v) = opt_str("input:kb_rules") {
        cfg.input.kb_rules = v;
    }
    if let Some(v) = opt_bool("input:numlock_by_default") {
        cfg.input.numlock_by_default = v;
    }
    if let Some(v) = opt_bool("input:resolve_binds_by_sym") {
        cfg.input.resolve_binds_by_sym = v;
    }
    if let Some(v) = opt_int("input:repeat_rate") {
        cfg.input.repeat_rate = v as i32;
    }
    if let Some(v) = opt_int("input:repeat_delay") {
        cfg.input.repeat_delay = v as i32;
    }
    if let Some(v) = opt_float("input:sensitivity") {
        cfg.input.sensitivity = v;
    }
    if let Some(v) = opt_str("input:accel_profile") {
        cfg.input.accel_profile = v;
    }
    if let Some(v) = opt_bool("input:force_no_accel") {
        cfg.input.force_no_accel = v;
    }
    if let Some(v) = opt_int("input:rotation") {
        cfg.input.rotation = v as i32;
    }
    if let Some(v) = opt_bool("input:left_handed") {
        cfg.input.left_handed = v;
    }
    if let Some(v) = opt_str("input:scroll_method") {
        cfg.input.scroll_method = v;
    }
    if let Some(v) = opt_int("input:scroll_button") {
        cfg.input.scroll_button = v as i32;
    }
    if let Some(v) = opt_bool("input:scroll_button_lock") {
        cfg.input.scroll_button_lock = v;
    }
    if let Some(v) = opt_float("input:scroll_factor") {
        cfg.input.scroll_factor = v;
    }
    if let Some(v) = opt_bool("input:natural_scroll") {
        cfg.input.natural_scroll = v;
    }
    if let Some(v) = opt_int("input:follow_mouse") {
        cfg.input.follow_mouse = v as i32;
    }
    if let Some(v) = opt_float("input:follow_mouse_threshold") {
        cfg.input.follow_mouse_threshold = v;
    }
    if let Some(v) = opt_int("input:focus_on_close") {
        cfg.input.focus_on_close = v as i32;
    }
    if let Some(v) = opt_bool("input:mouse_refocus") {
        cfg.input.mouse_refocus = v;
    }
    if let Some(v) = opt_int("input:float_switch_override_focus") {
        cfg.input.float_switch_override_focus = v as i32;
    }
    if let Some(v) = opt_bool("input:special_fallthrough") {
        cfg.input.special_fallthrough = v;
    }
    if let Some(v) = opt_int("input:off_window_axis_events") {
        cfg.input.off_window_axis_events = v as i32;
    }
    if let Some(v) = opt_int("input:emulate_discrete_scroll") {
        cfg.input.emulate_discrete_scroll = v as i32;
    }

    // input:touchpad
    if let Some(v) = opt_bool("input:touchpad:disable_while_typing") {
        cfg.input.touchpad.disable_while_typing = v;
    }
    if let Some(v) = opt_bool("input:touchpad:natural_scroll") {
        cfg.input.touchpad.natural_scroll = v;
    }
    if let Some(v) = opt_float("input:touchpad:scroll_factor") {
        cfg.input.touchpad.scroll_factor = v;
    }
    if let Some(v) = opt_bool("input:touchpad:middle_button_emulation") {
        cfg.input.touchpad.middle_button_emulation = v;
    }
    if let Some(v) = opt_str("input:touchpad:tap_button_map") {
        cfg.input.touchpad.tap_button_map = v;
    }
    if let Some(v) = opt_bool("input:touchpad:clickfinger_behavior") {
        cfg.input.touchpad.clickfinger_behavior = v;
    }
    if let Some(v) = opt_bool("input:touchpad:tap_to_click") {
        cfg.input.touchpad.tap_to_click = v;
    }
    if let Some(v) = opt_bool("input:touchpad:tap_and_drag") {
        cfg.input.touchpad.tap_and_drag = v;
    }
    if let Some(v) = opt_bool("input:touchpad:flip_x") {
        cfg.input.touchpad.flip_x = v;
    }
    if let Some(v) = opt_bool("input:touchpad:flip_y") {
        cfg.input.touchpad.flip_y = v;
    }

    // ── gestures ─────────────────────────────────────────────────────────────
    if let Some(v) = opt_int("gestures:workspace_swipe_distance") {
        cfg.gestures.workspace_swipe_distance = v as i32;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_touch") {
        cfg.gestures.workspace_swipe_touch = v;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_invert") {
        cfg.gestures.workspace_swipe_invert = v;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_touch_invert") {
        cfg.gestures.workspace_swipe_touch_invert = v;
    }
    if let Some(v) = opt_int("gestures:workspace_swipe_min_speed_to_force") {
        cfg.gestures.workspace_swipe_min_speed_to_force = v as i32;
    }
    if let Some(v) = opt_float("gestures:workspace_swipe_cancel_ratio") {
        cfg.gestures.workspace_swipe_cancel_ratio = v;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_create_new") {
        cfg.gestures.workspace_swipe_create_new = v;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_direction_lock") {
        cfg.gestures.workspace_swipe_direction_lock = v;
    }
    if let Some(v) = opt_int("gestures:workspace_swipe_direction_lock_threshold") {
        cfg.gestures.workspace_swipe_direction_lock_threshold = v as i32;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_forever") {
        cfg.gestures.workspace_swipe_forever = v;
    }
    if let Some(v) = opt_bool("gestures:workspace_swipe_use_r") {
        cfg.gestures.workspace_swipe_use_r = v;
    }

    // ── misc ─────────────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("misc:disable_hyprland_logo") {
        cfg.misc.disable_hyprland_logo = v;
    }
    if let Some(v) = opt_bool("misc:disable_splash_rendering") {
        cfg.misc.disable_splash_rendering = v;
    }
    if let Some(v) = opt_str("misc:font_family") {
        cfg.misc.font_family = v;
    }
    if let Some(v) = opt_int("misc:force_default_wallpaper") {
        cfg.misc.force_default_wallpaper = v as i32;
    }
    if let Some(v) = opt_bool("misc:vfr") {
        cfg.misc.vfr = v;
    }
    if let Some(v) = opt_int("misc:vrr") {
        cfg.misc.vrr = v as i32;
    }
    if let Some(v) = opt_bool("misc:mouse_move_enables_dpms") {
        cfg.misc.mouse_move_enables_dpms = v;
    }
    if let Some(v) = opt_bool("misc:key_press_enables_dpms") {
        cfg.misc.key_press_enables_dpms = v;
    }
    if let Some(v) = opt_bool("misc:always_follow_on_dnd") {
        cfg.misc.always_follow_on_dnd = v;
    }
    if let Some(v) = opt_bool("misc:layers_hog_keyboard_focus") {
        cfg.misc.layers_hog_keyboard_focus = v;
    }
    if let Some(v) = opt_bool("misc:animate_manual_resizes") {
        cfg.misc.animate_manual_resizes = v;
    }
    if let Some(v) = opt_bool("misc:animate_mouse_windowdragging") {
        cfg.misc.animate_mouse_windowdragging = v;
    }
    if let Some(v) = opt_bool("misc:disable_autoreload") {
        cfg.misc.disable_autoreload = v;
    }
    if let Some(v) = opt_bool("misc:enable_swallow") {
        cfg.misc.enable_swallow = v;
    }
    if let Some(v) = opt_str("misc:swallow_regex") {
        cfg.misc.swallow_regex = v;
    }
    if let Some(v) = opt_str("misc:swallow_exception_regex") {
        cfg.misc.swallow_exception_regex = v;
    }
    if let Some(v) = opt_bool("misc:focus_on_activate") {
        cfg.misc.focus_on_activate = v;
    }
    if let Some(v) = opt_bool("misc:mouse_move_focuses_monitor") {
        cfg.misc.mouse_move_focuses_monitor = v;
    }
    if let Some(v) = opt_bool("misc:close_special_on_empty") {
        cfg.misc.close_special_on_empty = v;
    }
    if let Some(v) = opt_int("misc:on_focus_under_fullscreen") {
        cfg.misc.on_focus_under_fullscreen = v as i32;
    }
    if let Some(v) = opt_bool("misc:exit_window_retains_fullscreen") {
        cfg.misc.exit_window_retains_fullscreen = v;
    }
    if let Some(v) = opt_int("misc:initial_workspace_tracking") {
        cfg.misc.initial_workspace_tracking = v as i32;
    }
    if let Some(v) = opt_bool("misc:middle_click_paste") {
        cfg.misc.middle_click_paste = v;
    }

    // ── binds ────────────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("binds:pass_mouse_when_bound") {
        cfg.binds.pass_mouse_when_bound = v;
    }
    if let Some(v) = opt_int("binds:scroll_event_delay") {
        cfg.binds.scroll_event_delay = v as i32;
    }
    if let Some(v) = opt_bool("binds:workspace_back_and_forth") {
        cfg.binds.workspace_back_and_forth = v;
    }
    if let Some(v) = opt_bool("binds:hide_special_on_workspace_change") {
        cfg.binds.hide_special_on_workspace_change = v;
    }
    if let Some(v) = opt_bool("binds:allow_workspace_cycles") {
        cfg.binds.allow_workspace_cycles = v;
    }
    if let Some(v) = opt_int("binds:workspace_center_on") {
        cfg.binds.workspace_center_on = v as i32;
    }
    if let Some(v) = opt_int("binds:focus_preferred_method") {
        cfg.binds.focus_preferred_method = v as i32;
    }
    if let Some(v) = opt_bool("binds:ignore_group_lock") {
        cfg.binds.ignore_group_lock = v;
    }
    if let Some(v) = opt_bool("binds:movefocus_cycles_fullscreen") {
        cfg.binds.movefocus_cycles_fullscreen = v;
    }
    if let Some(v) = opt_bool("binds:movefocus_cycles_groupfirst") {
        cfg.binds.movefocus_cycles_groupfirst = v;
    }
    if let Some(v) = opt_bool("binds:disable_keybind_grabbing") {
        cfg.binds.disable_keybind_grabbing = v;
    }
    if let Some(v) = opt_bool("binds:window_direction_monitor_fallback") {
        cfg.binds.window_direction_monitor_fallback = v;
    }
    if let Some(v) = opt_bool("binds:allow_pin_fullscreen") {
        cfg.binds.allow_pin_fullscreen = v;
    }
    if let Some(v) = opt_int("binds:drag_threshold") {
        cfg.binds.drag_threshold = v as i32;
    }

    // ── cursor ───────────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("cursor:invisible") {
        cfg.cursor.invisible = v;
    }
    if let Some(v) = opt_bool("cursor:sync_gsettings_theme") {
        cfg.cursor.sync_gsettings_theme = v;
    }
    if let Some(v) = opt_float("cursor:inactive_timeout") {
        cfg.cursor.inactive_timeout = v;
    }
    if let Some(v) = opt_bool("cursor:no_warps") {
        cfg.cursor.no_warps = v;
    }
    if let Some(v) = opt_bool("cursor:persistent_warps") {
        cfg.cursor.persistent_warps = v;
    }
    if let Some(v) = opt_float("cursor:zoom_factor") {
        cfg.cursor.zoom_factor = v;
    }
    if let Some(v) = opt_bool("cursor:zoom_rigid") {
        cfg.cursor.zoom_rigid = v;
    }
    if let Some(v) = opt_bool("cursor:enable_hyprcursor") {
        cfg.cursor.enable_hyprcursor = v;
    }
    if let Some(v) = opt_bool("cursor:hide_on_key_press") {
        cfg.cursor.hide_on_key_press = v;
    }
    if let Some(v) = opt_bool("cursor:hide_on_touch") {
        cfg.cursor.hide_on_touch = v;
    }
    if let Some(v) = opt_bool("cursor:hide_on_tablet") {
        cfg.cursor.hide_on_tablet = v;
    }
    if let Some(v) = opt_bool("cursor:warp_back_after_non_mouse_input") {
        cfg.cursor.warp_back_after_non_mouse_input = v;
    }

    // ── xwayland ─────────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("xwayland:enabled") {
        cfg.xwayland.enabled = v;
    }
    if let Some(v) = opt_bool("xwayland:use_nearest_neighbor") {
        cfg.xwayland.use_nearest_neighbor = v;
    }
    if let Some(v) = opt_bool("xwayland:force_zero_scaling") {
        cfg.xwayland.force_zero_scaling = v;
    }

    // ── opengl ───────────────────────────────────────────────────────────────
    if let Some(v) = opt_bool("opengl:nvidia_anti_flicker") {
        cfg.opengl.nvidia_anti_flicker = v;
    }

    // ── render ───────────────────────────────────────────────────────────────
    if let Some(v) = opt_int("render:direct_scanout") {
        cfg.render.direct_scanout = v as i32;
    }
    if let Some(v) = opt_bool("render:expand_undersized_textures") {
        cfg.render.expand_undersized_textures = v;
    }
    if let Some(v) = opt_bool("render:xp_mode") {
        cfg.render.xp_mode = v;
    }
    if let Some(v) = opt_bool("render:cm_enabled") {
        cfg.render.cm_enabled = v;
    }
    if let Some(v) = opt_bool("render:new_render_scheduling") {
        cfg.render.new_render_scheduling = v;
    }

    cfg
}
