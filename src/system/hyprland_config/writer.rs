use crate::types::hyprland_config::*;
use std::fmt::Write;

/// Write a HyprlandConfig struct to a configuration file string
pub fn write_config(config: &HyprlandConfig) -> String {
    let mut output = String::new();

    // Header comment
    writeln!(
        output,
        "# Omarchist Hyprland Configuration\n# Generated automatically - Do not edit manually\n"
    )
    .unwrap();

    // Write each section
    write_general_section(&mut output, &config.general);
    write_decoration_section(&mut output, &config.decoration);
    write_animations_section(&mut output, &config.animations);
    write_input_section(&mut output, &config.input);
    write_gestures_section(&mut output, &config.gestures);
    write_group_section(&mut output, &config.group);
    write_misc_section(&mut output, &config.misc);
    write_binds_section(&mut output, &config.binds);
    write_xwayland_section(&mut output, &config.xwayland);
    write_opengl_section(&mut output, &config.opengl);
    write_render_section(&mut output, &config.render);
    write_cursor_section(&mut output, &config.cursor);
    write_ecosystem_section(&mut output, &config.ecosystem);
    write_quirks_section(&mut output, &config.quirks);
    write_debug_section(&mut output, &config.debug);

    output
}

fn write_general_section(output: &mut String, config: &GeneralConfig) {
    writeln!(output, "general {{").unwrap();

    if config.border_size != 1 {
        writeln!(output, "    border_size = {}", config.border_size).unwrap();
    }
    if config.gaps_in != 5 {
        writeln!(output, "    gaps_in = {}", config.gaps_in).unwrap();
    }
    if config.gaps_out != 20 {
        writeln!(output, "    gaps_out = {}", config.gaps_out).unwrap();
    }
    if config.float_gaps != 0 {
        writeln!(output, "    float_gaps = {}", config.float_gaps).unwrap();
    }
    if config.gaps_workspaces != 0 {
        writeln!(output, "    gaps_workspaces = {}", config.gaps_workspaces).unwrap();
    }
    if config.layout != "dwindle" {
        writeln!(output, "    layout = {}", config.layout).unwrap();
    }
    if config.no_focus_fallback {
        writeln!(output, "    no_focus_fallback = true").unwrap();
    }
    if config.resize_on_border {
        writeln!(output, "    resize_on_border = true").unwrap();
    }
    if config.extend_border_grab_area != 15 {
        writeln!(
            output,
            "    extend_border_grab_area = {}",
            config.extend_border_grab_area
        )
        .unwrap();
    }
    if !config.hover_icon_on_border {
        writeln!(output, "    hover_icon_on_border = false").unwrap();
    }
    if config.allow_tearing {
        writeln!(output, "    allow_tearing = true").unwrap();
    }
    if config.resize_corner != 0 {
        writeln!(output, "    resize_corner = {}", config.resize_corner).unwrap();
    }
    if !config.modal_parent_blocking {
        writeln!(output, "    modal_parent_blocking = false").unwrap();
    }
    if !config.locale.is_empty() {
        writeln!(output, "    locale = {}", config.locale).unwrap();
    }

    // Snap subcategory
    if config.snap.enabled
        || config.snap.window_gap != 10
        || config.snap.monitor_gap != 10
        || config.snap.border_overlap
        || config.snap.respect_gaps
    {
        writeln!(output).unwrap();
        writeln!(output, "    snap {{").unwrap();
        if config.snap.enabled {
            writeln!(output, "        enabled = true").unwrap();
        }
        if config.snap.window_gap != 10 {
            writeln!(output, "        window_gap = {}", config.snap.window_gap).unwrap();
        }
        if config.snap.monitor_gap != 10 {
            writeln!(output, "        monitor_gap = {}", config.snap.monitor_gap).unwrap();
        }
        if config.snap.border_overlap {
            writeln!(output, "        border_overlap = true").unwrap();
        }
        if config.snap.respect_gaps {
            writeln!(output, "        respect_gaps = true").unwrap();
        }
        writeln!(output, "    }}").unwrap();
    }

    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_decoration_section(output: &mut String, config: &DecorationConfig) {
    writeln!(output, "decoration {{").unwrap();

    if config.rounding != 0 {
        writeln!(output, "    rounding = {}", config.rounding).unwrap();
    }
    if config.rounding_power != 2.0 {
        writeln!(output, "    rounding_power = {}", config.rounding_power).unwrap();
    }
    if config.active_opacity != 1.0 {
        writeln!(output, "    active_opacity = {}", config.active_opacity).unwrap();
    }
    if config.inactive_opacity != 1.0 {
        writeln!(output, "    inactive_opacity = {}", config.inactive_opacity).unwrap();
    }
    if config.fullscreen_opacity != 1.0 {
        writeln!(
            output,
            "    fullscreen_opacity = {}",
            config.fullscreen_opacity
        )
        .unwrap();
    }
    if !config.dim_modal {
        writeln!(output, "    dim_modal = false").unwrap();
    }
    if config.dim_inactive {
        writeln!(output, "    dim_inactive = true").unwrap();
    }
    if config.dim_strength != 0.5 {
        writeln!(output, "    dim_strength = {}", config.dim_strength).unwrap();
    }
    if config.dim_special != 0.2 {
        writeln!(output, "    dim_special = {}", config.dim_special).unwrap();
    }
    if config.dim_around != 0.4 {
        writeln!(output, "    dim_around = {}", config.dim_around).unwrap();
    }
    if !config.screen_shader.is_empty() {
        writeln!(output, "    screen_shader = {}", config.screen_shader).unwrap();
    }
    if !config.border_part_of_window {
        writeln!(output, "    border_part_of_window = false").unwrap();
    }

    // Blur subcategory
    let blur = &config.blur;
    if !blur.enabled
        || blur.size != 8
        || blur.passes != 1
        || !blur.ignore_opacity
        || !blur.new_optimizations
        || blur.xray
        || blur.noise != 0.0117
        || blur.contrast != 0.8916
        || blur.brightness != 0.8172
        || blur.vibrancy != 0.1696
        || blur.vibrancy_darkness != 0.0
        || blur.special
        || blur.popups
        || blur.popups_ignorealpha != 0.2
        || blur.input_methods
        || blur.input_methods_ignorealpha != 0.2
    {
        writeln!(output).unwrap();
        writeln!(output, "    blur {{").unwrap();
        if !blur.enabled {
            writeln!(output, "        enabled = false").unwrap();
        }
        if blur.size != 8 {
            writeln!(output, "        size = {}", blur.size).unwrap();
        }
        if blur.passes != 1 {
            writeln!(output, "        passes = {}", blur.passes).unwrap();
        }
        if !blur.ignore_opacity {
            writeln!(output, "        ignore_opacity = false").unwrap();
        }
        if !blur.new_optimizations {
            writeln!(output, "        new_optimizations = false").unwrap();
        }
        if blur.xray {
            writeln!(output, "        xray = true").unwrap();
        }
        if blur.noise != 0.0117 {
            writeln!(output, "        noise = {}", blur.noise).unwrap();
        }
        if blur.contrast != 0.8916 {
            writeln!(output, "        contrast = {}", blur.contrast).unwrap();
        }
        if blur.brightness != 0.8172 {
            writeln!(output, "        brightness = {}", blur.brightness).unwrap();
        }
        if blur.vibrancy != 0.1696 {
            writeln!(output, "        vibrancy = {}", blur.vibrancy).unwrap();
        }
        if blur.vibrancy_darkness != 0.0 {
            writeln!(
                output,
                "        vibrancy_darkness = {}",
                blur.vibrancy_darkness
            )
            .unwrap();
        }
        if blur.special {
            writeln!(output, "        special = true").unwrap();
        }
        if blur.popups {
            writeln!(output, "        popups = true").unwrap();
        }
        if blur.popups_ignorealpha != 0.2 {
            writeln!(
                output,
                "        popups_ignorealpha = {}",
                blur.popups_ignorealpha
            )
            .unwrap();
        }
        if blur.input_methods {
            writeln!(output, "        input_methods = true").unwrap();
        }
        if blur.input_methods_ignorealpha != 0.2 {
            writeln!(
                output,
                "        input_methods_ignorealpha = {}",
                blur.input_methods_ignorealpha
            )
            .unwrap();
        }
        writeln!(output, "    }}").unwrap();
    }

    // Shadow subcategory
    let shadow = &config.shadow;
    if !shadow.enabled
        || shadow.range != 4
        || shadow.render_power != 3
        || shadow.sharp
        || !shadow.ignore_window
        || shadow.color != "0xee1a1a1a"
        || !shadow.color_inactive.is_empty()
        || shadow.offset_x != 0.0
        || shadow.offset_y != 0.0
        || shadow.scale != 1.0
    {
        writeln!(output).unwrap();
        writeln!(output, "    shadow {{").unwrap();
        if !shadow.enabled {
            writeln!(output, "        enabled = false").unwrap();
        }
        if shadow.range != 4 {
            writeln!(output, "        range = {}", shadow.range).unwrap();
        }
        if shadow.render_power != 3 {
            writeln!(output, "        render_power = {}", shadow.render_power).unwrap();
        }
        if shadow.sharp {
            writeln!(output, "        sharp = true").unwrap();
        }
        if !shadow.ignore_window {
            writeln!(output, "        ignore_window = false").unwrap();
        }
        if shadow.color != "0xee1a1a1a" {
            writeln!(output, "        color = {}", shadow.color).unwrap();
        }
        if !shadow.color_inactive.is_empty() {
            writeln!(output, "        color_inactive = {}", shadow.color_inactive).unwrap();
        }
        if shadow.offset_x != 0.0 || shadow.offset_y != 0.0 {
            writeln!(
                output,
                "        offset = {} {}",
                shadow.offset_x, shadow.offset_y
            )
            .unwrap();
        }
        if shadow.scale != 1.0 {
            writeln!(output, "        scale = {}", shadow.scale).unwrap();
        }
        writeln!(output, "    }}").unwrap();
    }

    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_animations_section(output: &mut String, config: &AnimationsConfig) {
    if !config.enabled || config.workspace_wraparound {
        writeln!(output, "animations {{").unwrap();
        if !config.enabled {
            writeln!(output, "    enabled = false").unwrap();
        }
        if config.workspace_wraparound {
            writeln!(output, "    workspace_wraparound = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_input_section(output: &mut String, config: &InputConfig) {
    writeln!(output, "input {{").unwrap();

    if !config.kb_model.is_empty() {
        writeln!(output, "    kb_model = {}", config.kb_model).unwrap();
    }
    if config.kb_layout != "us" {
        writeln!(output, "    kb_layout = {}", config.kb_layout).unwrap();
    }
    if !config.kb_variant.is_empty() {
        writeln!(output, "    kb_variant = {}", config.kb_variant).unwrap();
    }
    if !config.kb_options.is_empty() {
        writeln!(output, "    kb_options = {}", config.kb_options).unwrap();
    }
    if !config.kb_rules.is_empty() {
        writeln!(output, "    kb_rules = {}", config.kb_rules).unwrap();
    }
    if !config.kb_file.is_empty() {
        writeln!(output, "    kb_file = {}", config.kb_file).unwrap();
    }
    if config.numlock_by_default {
        writeln!(output, "    numlock_by_default = true").unwrap();
    }
    if config.resolve_binds_by_sym {
        writeln!(output, "    resolve_binds_by_sym = true").unwrap();
    }
    if config.repeat_rate != 25 {
        writeln!(output, "    repeat_rate = {}", config.repeat_rate).unwrap();
    }
    if config.repeat_delay != 600 {
        writeln!(output, "    repeat_delay = {}", config.repeat_delay).unwrap();
    }
    if config.sensitivity != 0.0 {
        writeln!(output, "    sensitivity = {}", config.sensitivity).unwrap();
    }
    if !config.accel_profile.is_empty() {
        writeln!(output, "    accel_profile = {}", config.accel_profile).unwrap();
    }
    if config.force_no_accel {
        writeln!(output, "    force_no_accel = true").unwrap();
    }
    if config.rotation != 0 {
        writeln!(output, "    rotation = {}", config.rotation).unwrap();
    }
    if config.left_handed {
        writeln!(output, "    left_handed = true").unwrap();
    }
    if !config.scroll_points.is_empty() {
        writeln!(output, "    scroll_points = {}", config.scroll_points).unwrap();
    }
    if !config.scroll_method.is_empty() {
        writeln!(output, "    scroll_method = {}", config.scroll_method).unwrap();
    }
    if config.scroll_button != 0 {
        writeln!(output, "    scroll_button = {}", config.scroll_button).unwrap();
    }
    if config.scroll_button_lock {
        writeln!(output, "    scroll_button_lock = true").unwrap();
    }
    if config.scroll_factor != 1.0 {
        writeln!(output, "    scroll_factor = {}", config.scroll_factor).unwrap();
    }
    if config.natural_scroll {
        writeln!(output, "    natural_scroll = true").unwrap();
    }
    if config.follow_mouse != 1 {
        writeln!(output, "    follow_mouse = {}", config.follow_mouse).unwrap();
    }
    if config.follow_mouse_threshold != 0.0 {
        writeln!(
            output,
            "    follow_mouse_threshold = {}",
            config.follow_mouse_threshold
        )
        .unwrap();
    }
    if config.focus_on_close != 0 {
        writeln!(output, "    focus_on_close = {}", config.focus_on_close).unwrap();
    }
    if !config.mouse_refocus {
        writeln!(output, "    mouse_refocus = false").unwrap();
    }
    if config.float_switch_override_focus != 1 {
        writeln!(
            output,
            "    float_switch_override_focus = {}",
            config.float_switch_override_focus
        )
        .unwrap();
    }
    if config.special_fallthrough {
        writeln!(output, "    special_fallthrough = true").unwrap();
    }
    if config.off_window_axis_events != 1 {
        writeln!(
            output,
            "    off_window_axis_events = {}",
            config.off_window_axis_events
        )
        .unwrap();
    }
    if config.emulate_discrete_scroll != 1 {
        writeln!(
            output,
            "    emulate_discrete_scroll = {}",
            config.emulate_discrete_scroll
        )
        .unwrap();
    }

    // Touchpad subcategory
    let touchpad = &config.touchpad;
    if !touchpad.disable_while_typing
        || touchpad.natural_scroll
        || touchpad.scroll_factor != 1.0
        || touchpad.middle_button_emulation
        || !touchpad.tap_button_map.is_empty()
        || touchpad.clickfinger_behavior
        || !touchpad.tap_to_click
        || touchpad.drag_lock != 0
        || !touchpad.tap_and_drag
        || touchpad.flip_x
        || touchpad.flip_y
        || touchpad.drag_3fg != 0
    {
        writeln!(output).unwrap();
        writeln!(output, "    touchpad {{").unwrap();
        if !touchpad.disable_while_typing {
            writeln!(output, "        disable_while_typing = false").unwrap();
        }
        if touchpad.natural_scroll {
            writeln!(output, "        natural_scroll = true").unwrap();
        }
        if touchpad.scroll_factor != 1.0 {
            writeln!(output, "        scroll_factor = {}", touchpad.scroll_factor).unwrap();
        }
        if touchpad.middle_button_emulation {
            writeln!(output, "        middle_button_emulation = true").unwrap();
        }
        if !touchpad.tap_button_map.is_empty() {
            writeln!(
                output,
                "        tap_button_map = {}",
                touchpad.tap_button_map
            )
            .unwrap();
        }
        if touchpad.clickfinger_behavior {
            writeln!(output, "        clickfinger_behavior = true").unwrap();
        }
        if !touchpad.tap_to_click {
            writeln!(output, "        tap-to-click = false").unwrap();
        }
        if touchpad.drag_lock != 0 {
            writeln!(output, "        drag_lock = {}", touchpad.drag_lock).unwrap();
        }
        if !touchpad.tap_and_drag {
            writeln!(output, "        tap-and-drag = false").unwrap();
        }
        if touchpad.flip_x {
            writeln!(output, "        flip_x = true").unwrap();
        }
        if touchpad.flip_y {
            writeln!(output, "        flip_y = true").unwrap();
        }
        if touchpad.drag_3fg != 0 {
            writeln!(output, "        drag_3fg = {}", touchpad.drag_3fg).unwrap();
        }
        writeln!(output, "    }}").unwrap();
    }

    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_gestures_section(output: &mut String, config: &GesturesConfig) {
    if config.workspace_swipe_distance != 300
        || config.workspace_swipe_touch
        || !config.workspace_swipe_invert
        || config.workspace_swipe_touch_invert
        || config.workspace_swipe_min_speed_to_force != 30
        || config.workspace_swipe_cancel_ratio != 0.5
        || !config.workspace_swipe_create_new
        || !config.workspace_swipe_direction_lock
        || config.workspace_swipe_direction_lock_threshold != 10
        || config.workspace_swipe_forever
        || config.workspace_swipe_use_r
        || config.close_max_timeout != 1000
    {
        writeln!(output, "gestures {{").unwrap();
        if config.workspace_swipe_distance != 300 {
            writeln!(
                output,
                "    workspace_swipe_distance = {}",
                config.workspace_swipe_distance
            )
            .unwrap();
        }
        if config.workspace_swipe_touch {
            writeln!(output, "    workspace_swipe_touch = true").unwrap();
        }
        if !config.workspace_swipe_invert {
            writeln!(output, "    workspace_swipe_invert = false").unwrap();
        }
        if config.workspace_swipe_touch_invert {
            writeln!(output, "    workspace_swipe_touch_invert = true").unwrap();
        }
        if config.workspace_swipe_min_speed_to_force != 30 {
            writeln!(
                output,
                "    workspace_swipe_min_speed_to_force = {}",
                config.workspace_swipe_min_speed_to_force
            )
            .unwrap();
        }
        if config.workspace_swipe_cancel_ratio != 0.5 {
            writeln!(
                output,
                "    workspace_swipe_cancel_ratio = {}",
                config.workspace_swipe_cancel_ratio
            )
            .unwrap();
        }
        if !config.workspace_swipe_create_new {
            writeln!(output, "    workspace_swipe_create_new = false").unwrap();
        }
        if !config.workspace_swipe_direction_lock {
            writeln!(output, "    workspace_swipe_direction_lock = false").unwrap();
        }
        if config.workspace_swipe_direction_lock_threshold != 10 {
            writeln!(
                output,
                "    workspace_swipe_direction_lock_threshold = {}",
                config.workspace_swipe_direction_lock_threshold
            )
            .unwrap();
        }
        if config.workspace_swipe_forever {
            writeln!(output, "    workspace_swipe_forever = true").unwrap();
        }
        if config.workspace_swipe_use_r {
            writeln!(output, "    workspace_swipe_use_r = true").unwrap();
        }
        if config.close_max_timeout != 1000 {
            writeln!(
                output,
                "    close_max_timeout = {}",
                config.close_max_timeout
            )
            .unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_group_section(output: &mut String, config: &GroupConfig) {
    if !config.auto_group
        || !config.insert_after_current
        || !config.focus_removed_window
        || config.drag_into_group != 1
        || !config.merge_groups_on_drag
        || !config.merge_groups_on_groupbar
        || config.merge_floated_into_tiled_on_groupbar
        || config.group_on_movetoworkspace
    {
        writeln!(output, "group {{").unwrap();
        if !config.auto_group {
            writeln!(output, "    auto_group = false").unwrap();
        }
        if !config.insert_after_current {
            writeln!(output, "    insert_after_current = false").unwrap();
        }
        if !config.focus_removed_window {
            writeln!(output, "    focus_removed_window = false").unwrap();
        }
        if config.drag_into_group != 1 {
            writeln!(output, "    drag_into_group = {}", config.drag_into_group).unwrap();
        }
        if !config.merge_groups_on_drag {
            writeln!(output, "    merge_groups_on_drag = false").unwrap();
        }
        if !config.merge_groups_on_groupbar {
            writeln!(output, "    merge_groups_on_groupbar = false").unwrap();
        }
        if config.merge_floated_into_tiled_on_groupbar {
            writeln!(output, "    merge_floated_into_tiled_on_groupbar = true").unwrap();
        }
        if config.group_on_movetoworkspace {
            writeln!(output, "    group_on_movetoworkspace = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_misc_section(output: &mut String, config: &MiscConfig) {
    writeln!(output, "misc {{").unwrap();

    if config.disable_hyprland_logo {
        writeln!(output, "    disable_hyprland_logo = true").unwrap();
    }
    if config.disable_splash_rendering {
        writeln!(output, "    disable_splash_rendering = true").unwrap();
    }
    if config.disable_scale_notification {
        writeln!(output, "    disable_scale_notification = true").unwrap();
    }
    if config.font_family != "Sans" {
        writeln!(output, "    font_family = {}", config.font_family).unwrap();
    }
    if config.force_default_wallpaper != -1 {
        writeln!(
            output,
            "    force_default_wallpaper = {}",
            config.force_default_wallpaper
        )
        .unwrap();
    }
    if !config.vfr {
        writeln!(output, "    vfr = false").unwrap();
    }
    if config.vrr != 0 {
        writeln!(output, "    vrr = {}", config.vrr).unwrap();
    }
    if config.mouse_move_enables_dpms {
        writeln!(output, "    mouse_move_enables_dpms = true").unwrap();
    }
    if config.key_press_enables_dpms {
        writeln!(output, "    key_press_enables_dpms = true").unwrap();
    }
    if !config.always_follow_on_dnd {
        writeln!(output, "    always_follow_on_dnd = false").unwrap();
    }
    if !config.layers_hog_keyboard_focus {
        writeln!(output, "    layers_hog_keyboard_focus = false").unwrap();
    }
    if config.animate_manual_resizes {
        writeln!(output, "    animate_manual_resizes = true").unwrap();
    }
    if config.animate_mouse_windowdragging {
        writeln!(output, "    animate_mouse_windowdragging = true").unwrap();
    }
    if config.disable_autoreload {
        writeln!(output, "    disable_autoreload = true").unwrap();
    }
    if config.enable_swallow {
        writeln!(output, "    enable_swallow = true").unwrap();
    }
    if !config.swallow_regex.is_empty() {
        writeln!(output, "    swallow_regex = {}", config.swallow_regex).unwrap();
    }
    if !config.swallow_exception_regex.is_empty() {
        writeln!(
            output,
            "    swallow_exception_regex = {}",
            config.swallow_exception_regex
        )
        .unwrap();
    }
    if config.focus_on_activate {
        writeln!(output, "    focus_on_activate = true").unwrap();
    }
    if !config.mouse_move_focuses_monitor {
        writeln!(output, "    mouse_move_focuses_monitor = false").unwrap();
    }
    if !config.close_special_on_empty {
        writeln!(output, "    close_special_on_empty = false").unwrap();
    }
    if config.on_focus_under_fullscreen != 2 {
        writeln!(
            output,
            "    on_focus_under_fullscreen = {}",
            config.on_focus_under_fullscreen
        )
        .unwrap();
    }
    if config.exit_window_retains_fullscreen {
        writeln!(output, "    exit_window_retains_fullscreen = true").unwrap();
    }
    if config.initial_workspace_tracking != 1 {
        writeln!(
            output,
            "    initial_workspace_tracking = {}",
            config.initial_workspace_tracking
        )
        .unwrap();
    }
    if !config.middle_click_paste {
        writeln!(output, "    middle_click_paste = false").unwrap();
    }

    writeln!(output, "}}").unwrap();
    writeln!(output).unwrap();
}

fn write_binds_section(output: &mut String, config: &BindsConfig) {
    if config.pass_mouse_when_bound
        || config.scroll_event_delay != 300
        || config.workspace_back_and_forth
        || config.hide_special_on_workspace_change
        || config.allow_workspace_cycles
        || config.workspace_center_on != 0
        || config.focus_preferred_method != 0
        || config.ignore_group_lock
        || config.movefocus_cycles_fullscreen
        || config.movefocus_cycles_groupfirst
        || config.disable_keybind_grabbing
        || !config.window_direction_monitor_fallback
        || config.allow_pin_fullscreen
        || config.drag_threshold != 0
    {
        writeln!(output, "binds {{").unwrap();
        if config.pass_mouse_when_bound {
            writeln!(output, "    pass_mouse_when_bound = true").unwrap();
        }
        if config.scroll_event_delay != 300 {
            writeln!(
                output,
                "    scroll_event_delay = {}",
                config.scroll_event_delay
            )
            .unwrap();
        }
        if config.workspace_back_and_forth {
            writeln!(output, "    workspace_back_and_forth = true").unwrap();
        }
        if config.hide_special_on_workspace_change {
            writeln!(output, "    hide_special_on_workspace_change = true").unwrap();
        }
        if config.allow_workspace_cycles {
            writeln!(output, "    allow_workspace_cycles = true").unwrap();
        }
        if config.workspace_center_on != 0 {
            writeln!(
                output,
                "    workspace_center_on = {}",
                config.workspace_center_on
            )
            .unwrap();
        }
        if config.focus_preferred_method != 0 {
            writeln!(
                output,
                "    focus_preferred_method = {}",
                config.focus_preferred_method
            )
            .unwrap();
        }
        if config.ignore_group_lock {
            writeln!(output, "    ignore_group_lock = true").unwrap();
        }
        if config.movefocus_cycles_fullscreen {
            writeln!(output, "    movefocus_cycles_fullscreen = true").unwrap();
        }
        if config.movefocus_cycles_groupfirst {
            writeln!(output, "    movefocus_cycles_groupfirst = true").unwrap();
        }
        if config.disable_keybind_grabbing {
            writeln!(output, "    disable_keybind_grabbing = true").unwrap();
        }
        if !config.window_direction_monitor_fallback {
            writeln!(output, "    window_direction_monitor_fallback = false").unwrap();
        }
        if config.allow_pin_fullscreen {
            writeln!(output, "    allow_pin_fullscreen = true").unwrap();
        }
        if config.drag_threshold != 0 {
            writeln!(output, "    drag_threshold = {}", config.drag_threshold).unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_xwayland_section(output: &mut String, config: &XWaylandConfig) {
    if !config.enabled
        || !config.use_nearest_neighbor
        || config.force_zero_scaling
        || config.create_abstract_socket
    {
        writeln!(output, "xwayland {{").unwrap();
        if !config.enabled {
            writeln!(output, "    enabled = false").unwrap();
        }
        if !config.use_nearest_neighbor {
            writeln!(output, "    use_nearest_neighbor = false").unwrap();
        }
        if config.force_zero_scaling {
            writeln!(output, "    force_zero_scaling = true").unwrap();
        }
        if config.create_abstract_socket {
            writeln!(output, "    create_abstract_socket = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_opengl_section(output: &mut String, config: &OpenGlConfig) {
    if !config.nvidia_anti_flicker {
        writeln!(output, "opengl {{").unwrap();
        writeln!(output, "    nvidia_anti_flicker = false").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_render_section(output: &mut String, config: &RenderConfig) {
    if config.direct_scanout != 0
        || !config.expand_undersized_textures
        || config.xp_mode
        || config.ctm_animation != 2
        || config.cm_fs_passthrough != 2
        || !config.cm_enabled
        || !config.send_content_type
        || config.cm_auto_hdr != 1
        || config.new_render_scheduling
        || config.non_shader_cm != 3
        || config.cm_sdr_eotf != 0
    {
        writeln!(output, "render {{").unwrap();
        if config.direct_scanout != 0 {
            writeln!(output, "    direct_scanout = {}", config.direct_scanout).unwrap();
        }
        if !config.expand_undersized_textures {
            writeln!(output, "    expand_undersized_textures = false").unwrap();
        }
        if config.xp_mode {
            writeln!(output, "    xp_mode = true").unwrap();
        }
        if config.ctm_animation != 2 {
            writeln!(output, "    ctm_animation = {}", config.ctm_animation).unwrap();
        }
        if config.cm_fs_passthrough != 2 {
            writeln!(
                output,
                "    cm_fs_passthrough = {}",
                config.cm_fs_passthrough
            )
            .unwrap();
        }
        if !config.cm_enabled {
            writeln!(output, "    cm_enabled = false").unwrap();
        }
        if !config.send_content_type {
            writeln!(output, "    send_content_type = false").unwrap();
        }
        if config.cm_auto_hdr != 1 {
            writeln!(output, "    cm_auto_hdr = {}", config.cm_auto_hdr).unwrap();
        }
        if config.new_render_scheduling {
            writeln!(output, "    new_render_scheduling = true").unwrap();
        }
        if config.non_shader_cm != 3 {
            writeln!(output, "    non_shader_cm = {}", config.non_shader_cm).unwrap();
        }
        if config.cm_sdr_eotf != 0 {
            writeln!(output, "    cm_sdr_eotf = {}", config.cm_sdr_eotf).unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_cursor_section(output: &mut String, config: &CursorConfig) {
    if config.invisible
        || !config.sync_gsettings_theme
        || config.no_hardware_cursors != 2
        || config.no_break_fs_vrr != 2
        || config.min_refresh_rate != 24
        || config.hotspot_padding != 1
        || config.inactive_timeout != 0.0
        || config.no_warps
        || config.persistent_warps
        || config.warp_on_change_workspace != 0
        || config.warp_on_toggle_special != 0
        || !config.default_monitor.is_empty()
        || config.zoom_factor != 1.0
        || config.zoom_rigid
        || !config.zoom_detached_camera
        || !config.enable_hyprcursor
        || config.hide_on_key_press
        || !config.hide_on_touch
        || !config.hide_on_tablet
        || config.use_cpu_buffer != 2
        || config.warp_back_after_non_mouse_input
        || config.zoom_disable_aa
    {
        writeln!(output, "cursor {{").unwrap();
        if config.invisible {
            writeln!(output, "    invisible = true").unwrap();
        }
        if !config.sync_gsettings_theme {
            writeln!(output, "    sync_gsettings_theme = false").unwrap();
        }
        if config.no_hardware_cursors != 2 {
            writeln!(
                output,
                "    no_hardware_cursors = {}",
                config.no_hardware_cursors
            )
            .unwrap();
        }
        if config.no_break_fs_vrr != 2 {
            writeln!(output, "    no_break_fs_vrr = {}", config.no_break_fs_vrr).unwrap();
        }
        if config.min_refresh_rate != 24 {
            writeln!(output, "    min_refresh_rate = {}", config.min_refresh_rate).unwrap();
        }
        if config.hotspot_padding != 1 {
            writeln!(output, "    hotspot_padding = {}", config.hotspot_padding).unwrap();
        }
        if config.inactive_timeout != 0.0 {
            writeln!(output, "    inactive_timeout = {}", config.inactive_timeout).unwrap();
        }
        if config.no_warps {
            writeln!(output, "    no_warps = true").unwrap();
        }
        if config.persistent_warps {
            writeln!(output, "    persistent_warps = true").unwrap();
        }
        if config.warp_on_change_workspace != 0 {
            writeln!(
                output,
                "    warp_on_change_workspace = {}",
                config.warp_on_change_workspace
            )
            .unwrap();
        }
        if config.warp_on_toggle_special != 0 {
            writeln!(
                output,
                "    warp_on_toggle_special = {}",
                config.warp_on_toggle_special
            )
            .unwrap();
        }
        if !config.default_monitor.is_empty() {
            writeln!(output, "    default_monitor = {}", config.default_monitor).unwrap();
        }
        if config.zoom_factor != 1.0 {
            writeln!(output, "    zoom_factor = {}", config.zoom_factor).unwrap();
        }
        if config.zoom_rigid {
            writeln!(output, "    zoom_rigid = true").unwrap();
        }
        if !config.zoom_detached_camera {
            writeln!(output, "    zoom_detached_camera = false").unwrap();
        }
        if !config.enable_hyprcursor {
            writeln!(output, "    enable_hyprcursor = false").unwrap();
        }
        if config.hide_on_key_press {
            writeln!(output, "    hide_on_key_press = true").unwrap();
        }
        if !config.hide_on_touch {
            writeln!(output, "    hide_on_touch = false").unwrap();
        }
        if !config.hide_on_tablet {
            writeln!(output, "    hide_on_tablet = false").unwrap();
        }
        if config.use_cpu_buffer != 2 {
            writeln!(output, "    use_cpu_buffer = {}", config.use_cpu_buffer).unwrap();
        }
        if config.warp_back_after_non_mouse_input {
            writeln!(output, "    warp_back_after_non_mouse_input = true").unwrap();
        }
        if config.zoom_disable_aa {
            writeln!(output, "    zoom_disable_aa = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_ecosystem_section(output: &mut String, config: &EcosystemConfig) {
    if config.no_update_news || config.no_donation_nag || config.enforce_permissions {
        writeln!(output, "ecosystem {{").unwrap();
        if config.no_update_news {
            writeln!(output, "    no_update_news = true").unwrap();
        }
        if config.no_donation_nag {
            writeln!(output, "    no_donation_nag = true").unwrap();
        }
        if config.enforce_permissions {
            writeln!(output, "    enforce_permissions = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_quirks_section(output: &mut String, config: &QuirksConfig) {
    if config.prefer_hdr != 0 {
        writeln!(output, "quirks {{").unwrap();
        writeln!(output, "    prefer_hdr = {}", config.prefer_hdr).unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}

fn write_debug_section(output: &mut String, config: &DebugConfig) {
    if config.overlay
        || config.damage_blink
        || config.gl_debugging
        || !config.disable_logs
        || !config.disable_time
        || config.damage_tracking != 2
        || config.enable_stdout_logs
        || config.manual_crash != 0
        || config.suppress_errors
        || config.watchdog_timeout != 5
        || config.disable_scale_checks
        || config.error_limit != 5
        || config.error_position != 0
        || !config.colored_stdout_logs
        || config.pass
        || config.full_cm_proto
    {
        writeln!(output, "debug {{").unwrap();
        if config.overlay {
            writeln!(output, "    overlay = true").unwrap();
        }
        if config.damage_blink {
            writeln!(output, "    damage_blink = true").unwrap();
        }
        if config.gl_debugging {
            writeln!(output, "    gl_debugging = true").unwrap();
        }
        if !config.disable_logs {
            writeln!(output, "    disable_logs = false").unwrap();
        }
        if !config.disable_time {
            writeln!(output, "    disable_time = false").unwrap();
        }
        if config.damage_tracking != 2 {
            writeln!(output, "    damage_tracking = {}", config.damage_tracking).unwrap();
        }
        if config.enable_stdout_logs {
            writeln!(output, "    enable_stdout_logs = true").unwrap();
        }
        if config.manual_crash != 0 {
            writeln!(output, "    manual_crash = {}", config.manual_crash).unwrap();
        }
        if config.suppress_errors {
            writeln!(output, "    suppress_errors = true").unwrap();
        }
        if config.watchdog_timeout != 5 {
            writeln!(output, "    watchdog_timeout = {}", config.watchdog_timeout).unwrap();
        }
        if config.disable_scale_checks {
            writeln!(output, "    disable_scale_checks = true").unwrap();
        }
        if config.error_limit != 5 {
            writeln!(output, "    error_limit = {}", config.error_limit).unwrap();
        }
        if config.error_position != 0 {
            writeln!(output, "    error_position = {}", config.error_position).unwrap();
        }
        if !config.colored_stdout_logs {
            writeln!(output, "    colored_stdout_logs = false").unwrap();
        }
        if config.pass {
            writeln!(output, "    pass = true").unwrap();
        }
        if config.full_cm_proto {
            writeln!(output, "    full_cm_proto = true").unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }
}
