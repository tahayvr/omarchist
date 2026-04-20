use crate::types::hyprland_config::*;

/// Parse a hyprland config file, starting from `base` instead of defaults.
/// This lets the caller seed values from hyprctl first, then overlay only the
/// fields that are explicitly set in the saved file.
pub fn parse_config_onto(content: &str, base: HyprlandConfig) -> HyprlandConfig {
    let mut config = base;
    parse_into(content, &mut config);
    config
}

pub fn parse_config(content: &str) -> HyprlandConfig {
    let mut config = HyprlandConfig::default();
    parse_into(content, &mut config);
    config
}

fn parse_into(content: &str, config: &mut HyprlandConfig) {
    let mut current_section: Option<String> = None;
    let mut current_subsection: Option<String> = None;
    let mut brace_depth = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle section start: "section {" or "section:subsection {"
        if let Some(section_def) = trimmed.strip_suffix('{') {
            let section_def = section_def.trim();
            brace_depth += 1;

            if section_def.contains(':') {
                let parts: Vec<&str> = section_def.splitn(2, ':').collect();
                current_section = Some(parts[0].trim().to_string());
                current_subsection = Some(parts[1].trim().to_string());
            } else {
                current_section = Some(section_def.to_string());
                current_subsection = None;
            }
            continue;
        }

        // Handle section end
        if trimmed == "}" {
            brace_depth -= 1;
            if brace_depth == 0 {
                current_subsection = None;
            } else if brace_depth < 0 {
                brace_depth = 0;
                current_section = None;
                current_subsection = None;
            }
            continue;
        }

        // Parse key-value pairs
        if let Some((key, value)) = parse_key_value(trimmed) {
            let section = current_section.as_deref().unwrap_or("");
            let subsection = current_subsection.as_deref();

            apply_setting(config, section, subsection, &key, &value);
        }
    }
}

fn parse_key_value(line: &str) -> Option<(String, String)> {
    // Handle special case for col. prefix (colors)
    if let Some(without_prefix) = line
        .strip_prefix("col.")
        .or_else(|| line.strip_prefix("col:"))
        && let Some(eq_pos) = without_prefix.find('=')
    {
        let key = format!("col.{}", without_prefix[..eq_pos].trim());
        let value = without_prefix[eq_pos + 1..].trim().to_string();
        return Some((key, value));
    }

    if let Some(eq_pos) = line.find('=') {
        let key = line[..eq_pos].trim().to_string();
        let value = line[eq_pos + 1..].trim().to_string();
        return Some((key, value));
    }

    None
}

fn apply_setting(
    config: &mut HyprlandConfig,
    section: &str,
    subsection: Option<&str>,
    key: &str,
    value: &str,
) {
    match section {
        "general" => apply_general_setting(&mut config.general, subsection, key, value),
        "decoration" => apply_decoration_setting(&mut config.decoration, subsection, key, value),
        "animations" => apply_animations_setting(&mut config.animations, key, value),
        "input" => apply_input_setting(&mut config.input, subsection, key, value),
        "gestures" => apply_gestures_setting(&mut config.gestures, key, value),
        "group" => apply_group_setting(&mut config.group, subsection, key, value),
        "misc" => apply_misc_setting(&mut config.misc, key, value),
        "binds" => apply_binds_setting(&mut config.binds, key, value),
        "xwayland" => apply_xwayland_setting(&mut config.xwayland, key, value),
        "opengl" => apply_opengl_setting(&mut config.opengl, key, value),
        "render" => apply_render_setting(&mut config.render, key, value),
        "cursor" => apply_cursor_setting(&mut config.cursor, key, value),
        "ecosystem" => apply_ecosystem_setting(&mut config.ecosystem, key, value),
        "quirks" => apply_quirks_setting(&mut config.quirks, key, value),
        "debug" => apply_debug_setting(&mut config.debug, key, value),
        _ => {}
    }
}

fn apply_general_setting(
    config: &mut GeneralConfig,
    subsection: Option<&str>,
    key: &str,
    value: &str,
) {
    match subsection {
        Some("snap") => match key {
            "enabled" => config.snap.enabled = parse_bool(value),
            "window_gap" => config.snap.window_gap = parse_int(value),
            "monitor_gap" => config.snap.monitor_gap = parse_int(value),
            "border_overlap" => config.snap.border_overlap = parse_bool(value),
            "respect_gaps" => config.snap.respect_gaps = parse_bool(value),
            _ => {}
        },
        _ => match key {
            "border_size" => config.border_size = parse_int(value),
            "gaps_in" => config.gaps_in = parse_int(value),
            "gaps_out" => config.gaps_out = parse_int(value),
            "float_gaps" => config.float_gaps = parse_int(value),
            "gaps_workspaces" => config.gaps_workspaces = parse_int(value),
            "layout" => config.layout = value.to_string(),
            "no_focus_fallback" => config.no_focus_fallback = parse_bool(value),
            "resize_on_border" => config.resize_on_border = parse_bool(value),
            "extend_border_grab_area" => config.extend_border_grab_area = parse_int(value),
            "hover_icon_on_border" => config.hover_icon_on_border = parse_bool(value),
            "allow_tearing" => config.allow_tearing = parse_bool(value),
            "resize_corner" => config.resize_corner = parse_int(value),
            "modal_parent_blocking" => config.modal_parent_blocking = parse_bool(value),
            "locale" => config.locale = value.to_string(),
            _ => {}
        },
    }
}

fn apply_decoration_setting(
    config: &mut DecorationConfig,
    subsection: Option<&str>,
    key: &str,
    value: &str,
) {
    match subsection {
        Some("blur") => match key {
            "enabled" => config.blur.enabled = parse_bool(value),
            "size" => config.blur.size = parse_int(value),
            "passes" => config.blur.passes = parse_int(value),
            "ignore_opacity" => config.blur.ignore_opacity = parse_bool(value),
            "new_optimizations" => config.blur.new_optimizations = parse_bool(value),
            "xray" => config.blur.xray = parse_bool(value),
            "noise" => config.blur.noise = parse_float(value),
            "contrast" => config.blur.contrast = parse_float(value),
            "brightness" => config.blur.brightness = parse_float(value),
            "vibrancy" => config.blur.vibrancy = parse_float(value),
            "vibrancy_darkness" => config.blur.vibrancy_darkness = parse_float(value),
            "special" => config.blur.special = parse_bool(value),
            "popups" => config.blur.popups = parse_bool(value),
            "popups_ignorealpha" => config.blur.popups_ignorealpha = parse_float(value),
            "input_methods" => config.blur.input_methods = parse_bool(value),
            "input_methods_ignorealpha" => {
                config.blur.input_methods_ignorealpha = parse_float(value)
            }
            _ => {}
        },
        Some("shadow") => match key {
            "enabled" => config.shadow.enabled = parse_bool(value),
            "range" => config.shadow.range = parse_int(value),
            "render_power" => config.shadow.render_power = parse_int(value),
            "sharp" => config.shadow.sharp = parse_bool(value),
            "ignore_window" => config.shadow.ignore_window = parse_bool(value),
            "color" => config.shadow.color = value.to_string(),
            "color_inactive" => config.shadow.color_inactive = value.to_string(),
            "offset" => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                if parts.len() >= 2 {
                    config.shadow.offset_x = parse_float(parts[0]);
                    config.shadow.offset_y = parse_float(parts[1]);
                }
            }
            "scale" => config.shadow.scale = parse_float(value),
            _ => {}
        },
        _ => match key {
            "rounding" => config.rounding = parse_int(value),
            "rounding_power" => config.rounding_power = parse_float(value),
            "active_opacity" => config.active_opacity = parse_float(value),
            "inactive_opacity" => config.inactive_opacity = parse_float(value),
            "fullscreen_opacity" => config.fullscreen_opacity = parse_float(value),
            "dim_modal" => config.dim_modal = parse_bool(value),
            "dim_inactive" => config.dim_inactive = parse_bool(value),
            "dim_strength" => config.dim_strength = parse_float(value),
            "dim_special" => config.dim_special = parse_float(value),
            "dim_around" => config.dim_around = parse_float(value),
            "screen_shader" => config.screen_shader = value.to_string(),
            "border_part_of_window" => config.border_part_of_window = parse_bool(value),
            _ => {}
        },
    }
}

fn apply_animations_setting(config: &mut AnimationsConfig, key: &str, value: &str) {
    match key {
        "enabled" => config.enabled = parse_bool(value),
        "workspace_wraparound" => config.workspace_wraparound = parse_bool(value),
        _ => {}
    }
}

fn apply_input_setting(config: &mut InputConfig, subsection: Option<&str>, key: &str, value: &str) {
    match subsection {
        Some("touchpad") => match key {
            "disable_while_typing" => config.touchpad.disable_while_typing = parse_bool(value),
            "natural_scroll" => config.touchpad.natural_scroll = parse_bool(value),
            "scroll_factor" => config.touchpad.scroll_factor = parse_float(value),
            "middle_button_emulation" => {
                config.touchpad.middle_button_emulation = parse_bool(value)
            }
            "tap_button_map" => config.touchpad.tap_button_map = value.to_string(),
            "clickfinger_behavior" => config.touchpad.clickfinger_behavior = parse_bool(value),
            "tap-to-click" => config.touchpad.tap_to_click = parse_bool(value),
            "drag_lock" => config.touchpad.drag_lock = parse_int(value),
            "tap-and-drag" => config.touchpad.tap_and_drag = parse_bool(value),
            "flip_x" => config.touchpad.flip_x = parse_bool(value),
            "flip_y" => config.touchpad.flip_y = parse_bool(value),
            "drag_3fg" => config.touchpad.drag_3fg = parse_int(value),
            _ => {}
        },
        _ => match key {
            "kb_model" => config.kb_model = value.to_string(),
            "kb_layout" => config.kb_layout = value.to_string(),
            "kb_variant" => config.kb_variant = value.to_string(),
            "kb_options" => config.kb_options = value.to_string(),
            "kb_rules" => config.kb_rules = value.to_string(),
            "kb_file" => config.kb_file = value.to_string(),
            "numlock_by_default" => config.numlock_by_default = parse_bool(value),
            "resolve_binds_by_sym" => config.resolve_binds_by_sym = parse_bool(value),
            "repeat_rate" => config.repeat_rate = parse_int(value),
            "repeat_delay" => config.repeat_delay = parse_int(value),
            "sensitivity" => config.sensitivity = parse_float(value),
            "accel_profile" => config.accel_profile = value.to_string(),
            "force_no_accel" => config.force_no_accel = parse_bool(value),
            "rotation" => config.rotation = parse_int(value),
            "left_handed" => config.left_handed = parse_bool(value),
            "scroll_points" => config.scroll_points = value.to_string(),
            "scroll_method" => config.scroll_method = value.to_string(),
            "scroll_button" => config.scroll_button = parse_int(value),
            "scroll_button_lock" => config.scroll_button_lock = parse_bool(value),
            "scroll_factor" => config.scroll_factor = parse_float(value),
            "natural_scroll" => config.natural_scroll = parse_bool(value),
            "follow_mouse" => config.follow_mouse = parse_int(value),
            "follow_mouse_threshold" => config.follow_mouse_threshold = parse_float(value),
            "focus_on_close" => config.focus_on_close = parse_int(value),
            "mouse_refocus" => config.mouse_refocus = parse_bool(value),
            "float_switch_override_focus" => config.float_switch_override_focus = parse_int(value),
            "special_fallthrough" => config.special_fallthrough = parse_bool(value),
            "off_window_axis_events" => config.off_window_axis_events = parse_int(value),
            "emulate_discrete_scroll" => config.emulate_discrete_scroll = parse_int(value),
            _ => {}
        },
    }
}

fn apply_gestures_setting(config: &mut GesturesConfig, key: &str, value: &str) {
    match key {
        "workspace_swipe_distance" => config.workspace_swipe_distance = parse_int(value),
        "workspace_swipe_touch" => config.workspace_swipe_touch = parse_bool(value),
        "workspace_swipe_invert" => config.workspace_swipe_invert = parse_bool(value),
        "workspace_swipe_touch_invert" => config.workspace_swipe_touch_invert = parse_bool(value),
        "workspace_swipe_min_speed_to_force" => {
            config.workspace_swipe_min_speed_to_force = parse_int(value)
        }
        "workspace_swipe_cancel_ratio" => config.workspace_swipe_cancel_ratio = parse_float(value),
        "workspace_swipe_create_new" => config.workspace_swipe_create_new = parse_bool(value),
        "workspace_swipe_direction_lock" => {
            config.workspace_swipe_direction_lock = parse_bool(value)
        }
        "workspace_swipe_direction_lock_threshold" => {
            config.workspace_swipe_direction_lock_threshold = parse_int(value)
        }
        "workspace_swipe_forever" => config.workspace_swipe_forever = parse_bool(value),
        "workspace_swipe_use_r" => config.workspace_swipe_use_r = parse_bool(value),
        "close_max_timeout" => config.close_max_timeout = parse_int(value),
        _ => {}
    }
}

fn apply_group_setting(config: &mut GroupConfig, subsection: Option<&str>, key: &str, value: &str) {
    match subsection {
        Some("groupbar") => match key {
            "enabled" => config.groupbar.enabled = parse_bool(value),
            "font_family" => config.groupbar.font_family = value.to_string(),
            "font_size" => config.groupbar.font_size = parse_int(value),
            "gradients" => config.groupbar.gradients = parse_bool(value),
            "height" => config.groupbar.height = parse_int(value),
            "indicator_gap" => config.groupbar.indicator_gap = parse_int(value),
            "indicator_height" => config.groupbar.indicator_height = parse_int(value),
            "stacked" => config.groupbar.stacked = parse_bool(value),
            "priority" => config.groupbar.priority = parse_int(value),
            "render_titles" => config.groupbar.render_titles = parse_bool(value),
            "text_offset" => config.groupbar.text_offset = parse_int(value),
            "text_padding" => config.groupbar.text_padding = parse_int(value),
            "scrolling" => config.groupbar.scrolling = parse_bool(value),
            "rounding" => config.groupbar.rounding = parse_int(value),
            "rounding_power" => config.groupbar.rounding_power = parse_float(value),
            _ => {}
        },
        _ => match key {
            "auto_group" => config.auto_group = parse_bool(value),
            "insert_after_current" => config.insert_after_current = parse_bool(value),
            "focus_removed_window" => config.focus_removed_window = parse_bool(value),
            "drag_into_group" => config.drag_into_group = parse_int(value),
            "merge_groups_on_drag" => config.merge_groups_on_drag = parse_bool(value),
            "merge_groups_on_groupbar" => config.merge_groups_on_groupbar = parse_bool(value),
            "merge_floated_into_tiled_on_groupbar" => {
                config.merge_floated_into_tiled_on_groupbar = parse_bool(value)
            }
            "group_on_movetoworkspace" => config.group_on_movetoworkspace = parse_bool(value),
            _ => {}
        },
    }
}

fn apply_misc_setting(config: &mut MiscConfig, key: &str, value: &str) {
    match key {
        "disable_hyprland_logo" => config.disable_hyprland_logo = parse_bool(value),
        "disable_splash_rendering" => config.disable_splash_rendering = parse_bool(value),
        "disable_scale_notification" => config.disable_scale_notification = parse_bool(value),
        "font_family" => config.font_family = value.to_string(),
        "force_default_wallpaper" => config.force_default_wallpaper = parse_int(value),
        "vfr" => config.vfr = parse_bool(value),
        "vrr" => config.vrr = parse_int(value),
        "mouse_move_enables_dpms" => config.mouse_move_enables_dpms = parse_bool(value),
        "key_press_enables_dpms" => config.key_press_enables_dpms = parse_bool(value),
        "always_follow_on_dnd" => config.always_follow_on_dnd = parse_bool(value),
        "layers_hog_keyboard_focus" => config.layers_hog_keyboard_focus = parse_bool(value),
        "animate_manual_resizes" => config.animate_manual_resizes = parse_bool(value),
        "animate_mouse_windowdragging" => config.animate_mouse_windowdragging = parse_bool(value),
        "disable_autoreload" => config.disable_autoreload = parse_bool(value),
        "enable_swallow" => config.enable_swallow = parse_bool(value),
        "swallow_regex" => config.swallow_regex = value.to_string(),
        "swallow_exception_regex" => config.swallow_exception_regex = value.to_string(),
        "focus_on_activate" => config.focus_on_activate = parse_bool(value),
        "mouse_move_focuses_monitor" => config.mouse_move_focuses_monitor = parse_bool(value),
        "close_special_on_empty" => config.close_special_on_empty = parse_bool(value),
        "on_focus_under_fullscreen" => config.on_focus_under_fullscreen = parse_int(value),
        "exit_window_retains_fullscreen" => {
            config.exit_window_retains_fullscreen = parse_bool(value)
        }
        "initial_workspace_tracking" => config.initial_workspace_tracking = parse_int(value),
        "middle_click_paste" => config.middle_click_paste = parse_bool(value),
        _ => {}
    }
}

fn apply_binds_setting(config: &mut BindsConfig, key: &str, value: &str) {
    match key {
        "pass_mouse_when_bound" => config.pass_mouse_when_bound = parse_bool(value),
        "scroll_event_delay" => config.scroll_event_delay = parse_int(value),
        "workspace_back_and_forth" => config.workspace_back_and_forth = parse_bool(value),
        "hide_special_on_workspace_change" => {
            config.hide_special_on_workspace_change = parse_bool(value)
        }
        "allow_workspace_cycles" => config.allow_workspace_cycles = parse_bool(value),
        "workspace_center_on" => config.workspace_center_on = parse_int(value),
        "focus_preferred_method" => config.focus_preferred_method = parse_int(value),
        "ignore_group_lock" => config.ignore_group_lock = parse_bool(value),
        "movefocus_cycles_fullscreen" => config.movefocus_cycles_fullscreen = parse_bool(value),
        "movefocus_cycles_groupfirst" => config.movefocus_cycles_groupfirst = parse_bool(value),
        "disable_keybind_grabbing" => config.disable_keybind_grabbing = parse_bool(value),
        "window_direction_monitor_fallback" => {
            config.window_direction_monitor_fallback = parse_bool(value)
        }
        "allow_pin_fullscreen" => config.allow_pin_fullscreen = parse_bool(value),
        "drag_threshold" => config.drag_threshold = parse_int(value),
        _ => {}
    }
}

fn apply_xwayland_setting(config: &mut XWaylandConfig, key: &str, value: &str) {
    match key {
        "enabled" => config.enabled = parse_bool(value),
        "use_nearest_neighbor" => config.use_nearest_neighbor = parse_bool(value),
        "force_zero_scaling" => config.force_zero_scaling = parse_bool(value),
        "create_abstract_socket" => config.create_abstract_socket = parse_bool(value),
        _ => {}
    }
}

fn apply_opengl_setting(config: &mut OpenGlConfig, key: &str, value: &str) {
    if key == "nvidia_anti_flicker" {
        config.nvidia_anti_flicker = parse_bool(value);
    }
}

fn apply_render_setting(config: &mut RenderConfig, key: &str, value: &str) {
    match key {
        "direct_scanout" => config.direct_scanout = parse_int(value),
        "expand_undersized_textures" => config.expand_undersized_textures = parse_bool(value),
        "xp_mode" => config.xp_mode = parse_bool(value),
        "ctm_animation" => config.ctm_animation = parse_int(value),
        "cm_fs_passthrough" => config.cm_fs_passthrough = parse_int(value),
        "cm_enabled" => config.cm_enabled = parse_bool(value),
        "send_content_type" => config.send_content_type = parse_bool(value),
        "cm_auto_hdr" => config.cm_auto_hdr = parse_int(value),
        "new_render_scheduling" => config.new_render_scheduling = parse_bool(value),
        "non_shader_cm" => config.non_shader_cm = parse_int(value),
        "cm_sdr_eotf" => config.cm_sdr_eotf = parse_int(value),
        _ => {}
    }
}

fn apply_cursor_setting(config: &mut CursorConfig, key: &str, value: &str) {
    match key {
        "invisible" => config.invisible = parse_bool(value),
        "sync_gsettings_theme" => config.sync_gsettings_theme = parse_bool(value),
        "no_hardware_cursors" => config.no_hardware_cursors = parse_int(value),
        "no_break_fs_vrr" => config.no_break_fs_vrr = parse_int(value),
        "min_refresh_rate" => config.min_refresh_rate = parse_int(value),
        "hotspot_padding" => config.hotspot_padding = parse_int(value),
        "inactive_timeout" => config.inactive_timeout = parse_float(value),
        "no_warps" => config.no_warps = parse_bool(value),
        "persistent_warps" => config.persistent_warps = parse_bool(value),
        "warp_on_change_workspace" => config.warp_on_change_workspace = parse_int(value),
        "warp_on_toggle_special" => config.warp_on_toggle_special = parse_int(value),
        "default_monitor" => config.default_monitor = value.to_string(),
        "zoom_factor" => config.zoom_factor = parse_float(value),
        "zoom_rigid" => config.zoom_rigid = parse_bool(value),
        "zoom_detached_camera" => config.zoom_detached_camera = parse_bool(value),
        "enable_hyprcursor" => config.enable_hyprcursor = parse_bool(value),
        "hide_on_key_press" => config.hide_on_key_press = parse_bool(value),
        "hide_on_touch" => config.hide_on_touch = parse_bool(value),
        "hide_on_tablet" => config.hide_on_tablet = parse_bool(value),
        "use_cpu_buffer" => config.use_cpu_buffer = parse_int(value),
        "warp_back_after_non_mouse_input" => {
            config.warp_back_after_non_mouse_input = parse_bool(value)
        }
        "zoom_disable_aa" => config.zoom_disable_aa = parse_bool(value),
        _ => {}
    }
}

fn apply_ecosystem_setting(config: &mut EcosystemConfig, key: &str, value: &str) {
    match key {
        "no_update_news" => config.no_update_news = parse_bool(value),
        "no_donation_nag" => config.no_donation_nag = parse_bool(value),
        "enforce_permissions" => config.enforce_permissions = parse_bool(value),
        _ => {}
    }
}

fn apply_quirks_setting(config: &mut QuirksConfig, key: &str, value: &str) {
    if key == "prefer_hdr" {
        config.prefer_hdr = parse_int(value);
    }
}

fn apply_debug_setting(config: &mut DebugConfig, key: &str, value: &str) {
    match key {
        "overlay" => config.overlay = parse_bool(value),
        "damage_blink" => config.damage_blink = parse_bool(value),
        "gl_debugging" => config.gl_debugging = parse_bool(value),
        "disable_logs" => config.disable_logs = parse_bool(value),
        "disable_time" => config.disable_time = parse_bool(value),
        "damage_tracking" => config.damage_tracking = parse_int(value),
        "enable_stdout_logs" => config.enable_stdout_logs = parse_bool(value),
        "manual_crash" => config.manual_crash = parse_int(value),
        "suppress_errors" => config.suppress_errors = parse_bool(value),
        "watchdog_timeout" => config.watchdog_timeout = parse_int(value),
        "disable_scale_checks" => config.disable_scale_checks = parse_bool(value),
        "error_limit" => config.error_limit = parse_int(value),
        "error_position" => config.error_position = parse_int(value),
        "colored_stdout_logs" => config.colored_stdout_logs = parse_bool(value),
        "pass" => config.pass = parse_bool(value),
        "full_cm_proto" => config.full_cm_proto = parse_bool(value),
        _ => {}
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "true" | "yes" | "on" | "1")
}

fn parse_int(value: &str) -> i32 {
    value.parse().unwrap_or(0)
}

fn parse_float(value: &str) -> f64 {
    value.parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bool_true_variants_return_true() {
        for val in &[
            "true", "True", "TRUE", "yes", "Yes", "YES", "on", "On", "ON", "1",
        ] {
            assert!(
                parse_bool(val),
                "parse_bool(\"{}\") should return true",
                val
            );
        }
    }

    #[test]
    fn parse_bool_false_variants_return_false() {
        for val in &[
            "false", "False", "FALSE", "no", "No", "off", "0", "2", "", "maybe",
        ] {
            assert!(
                !parse_bool(val),
                "parse_bool(\"{}\") should return false",
                val
            );
        }
    }

    #[test]
    fn parse_int_valid_positive() {
        assert_eq!(parse_int("42"), 42);
    }

    #[test]
    fn parse_int_valid_negative() {
        assert_eq!(parse_int("-5"), -5);
    }

    #[test]
    fn parse_int_zero() {
        assert_eq!(parse_int("0"), 0);
    }

    #[test]
    fn parse_int_invalid_returns_zero() {
        assert_eq!(
            parse_int("abc"),
            0,
            "non-numeric input should fall back to 0"
        );
        assert_eq!(parse_int(""), 0, "empty string should fall back to 0");
        assert_eq!(parse_int("3.14"), 0, "float string should fall back to 0");
    }

    #[test]
    fn parse_float_valid_decimal() {
        let result = parse_float("1.5");
        assert!(
            (result - 1.5).abs() < f64::EPSILON,
            "parse_float(\"1.5\") should return 1.5"
        );
    }

    #[test]
    fn parse_float_valid_integer_string() {
        let result = parse_float("2");
        assert!(
            (result - 2.0).abs() < f64::EPSILON,
            "parse_float(\"2\") should return 2.0"
        );
    }

    #[test]
    fn parse_float_invalid_returns_zero() {
        assert!(
            parse_float("abc").abs() < f64::EPSILON,
            "non-numeric input should fall back to 0.0"
        );
        assert!(
            parse_float("").abs() < f64::EPSILON,
            "empty string should fall back to 0.0"
        );
    }

    #[test]
    fn parse_key_value_simple_pair() {
        let result = parse_key_value("border_size = 2");
        assert_eq!(
            result,
            Some(("border_size".to_string(), "2".to_string())),
            "simple key = value should parse correctly"
        );
    }

    #[test]
    fn parse_key_value_trims_whitespace() {
        let result = parse_key_value("  gaps_in   =   5  ");
        assert_eq!(
            result,
            Some(("gaps_in".to_string(), "5".to_string())),
            "leading/trailing whitespace should be trimmed from key and value"
        );
    }

    #[test]
    fn parse_key_value_col_prefix_preserved() {
        // Keys prefixed with "col." or "col:" are a special case.
        let result = parse_key_value("col.active_border = 0xff89b4fa");
        let (key, val) = result.expect("col. prefix key should parse successfully");
        assert_eq!(key, "col.active_border");
        assert_eq!(val, "0xff89b4fa");
    }

    #[test]
    fn parse_key_value_no_equals_returns_none() {
        assert_eq!(
            parse_key_value("no equals sign here"),
            None,
            "a line without '=' should return None"
        );
    }

    #[test]
    fn parse_key_value_empty_line_returns_none() {
        assert_eq!(parse_key_value(""), None, "empty input should return None");
    }

    #[test]
    fn parse_key_value_value_can_be_empty() {
        let result = parse_key_value("kb_model =");
        let (key, val) = result.expect("key with empty value should still parse");
        assert_eq!(key, "kb_model");
        assert_eq!(val, "", "empty value should be an empty string");
    }

    #[test]
    fn parse_config_general_section_basic_fields() {
        let input = "general {\n    border_size = 3\n    gaps_in = 8\n}";
        let config = parse_config(input);
        assert_eq!(
            config.general.border_size, 3,
            "border_size should be parsed from the general section"
        );
        assert_eq!(
            config.general.gaps_in, 8,
            "gaps_in should be parsed from the general section"
        );
    }

    #[test]
    fn parse_config_skips_comments_and_empty_lines() {
        let input = "# this is a top-level comment\ngeneral {\n    # inline comment\n    border_size = 5\n}";
        let config = parse_config(input);
        assert_eq!(
            config.general.border_size, 5,
            "parser should skip comment lines and still read values"
        );
    }

    #[test]
    fn parse_config_subsection_colon_syntax() {
        // Hyprland uses "section:subsection {" on a single line.
        let input = "decoration:blur {\n    enabled = false\n    size = 4\n}";
        let config = parse_config(input);
        assert!(
            !config.decoration.blur.enabled,
            "blur.enabled should be parsed from the 'decoration:blur' subsection"
        );
        assert_eq!(
            config.decoration.blur.size, 4,
            "blur.size should be parsed from the 'decoration:blur' subsection"
        );
    }

    #[test]
    fn parse_config_input_kb_layout() {
        let input = "input {\n    kb_layout = gb\n}";
        let config = parse_config(input);
        assert_eq!(
            config.input.kb_layout, "gb",
            "kb_layout should be parsed from the input section"
        );
    }

    #[test]
    fn parse_config_multiple_sections_independent() {
        let input = "general {\n    gaps_out = 10\n}\nmisc {\n    vfr = false\n}";
        let config = parse_config(input);
        assert_eq!(config.general.gaps_out, 10);
        assert!(!config.misc.vfr);
    }

    #[test]
    fn parse_config_empty_input_returns_default() {
        let config = parse_config("");
        // Spot-check that struct defaults are returned for fields not present in the input.
        // border_size defaults to 2 (as defined in HyprlandConfig::default()).
        assert_eq!(
            config.general.border_size, 2,
            "empty input should yield the struct default border_size of 2"
        );
        assert_eq!(
            config.input.kb_layout, "us",
            "empty input should yield default kb_layout of 'us'"
        );
    }

    #[test]
    fn parse_config_unknown_section_ignored() {
        // A section name we don't recognise must not panic.
        let input = "nonexistent_section {\n    foo = bar\n}";
        let _config = parse_config(input); // must not panic
    }

    #[test]
    fn parse_config_decoration_shadow_subsection() {
        let input = "decoration:shadow {\n    enabled = false\n    range = 8\n}";
        let config = parse_config(input);
        assert!(!config.decoration.shadow.enabled);
        assert_eq!(config.decoration.shadow.range, 8);
    }

    #[test]
    fn parse_config_animations_enabled_false() {
        let input = "animations {\n    enabled = false\n}";
        let config = parse_config(input);
        assert!(
            !config.animations.enabled,
            "animations.enabled should be false when set to false"
        );
    }
}
