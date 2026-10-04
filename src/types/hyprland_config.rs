use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HyprlandConfig {
    pub general: GeneralConfig,
    pub decoration: DecorationConfig,
    pub animations: AnimationsConfig,
    pub layout: LayoutConfig,
    pub dwindle: DwindleConfig,
    pub master: MasterConfig,
    pub scrolling: ScrollingConfig,
    pub input: InputConfig,
    pub gestures: GesturesConfig,
    pub group: GroupConfig,
    pub misc: MiscConfig,
    pub binds: BindsConfig,
    pub xwayland: XWaylandConfig,
    pub opengl: OpenGlConfig,
    pub render: RenderConfig,
    pub cursor: CursorConfig,
    pub ecosystem: EcosystemConfig,
    pub quirks: QuirksConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub border_size: i32,
    pub gaps_in: i32,
    pub gaps_out: i32,
    pub float_gaps: i32,
    pub gaps_workspaces: i32,
    pub layout: String,
    pub no_focus_fallback: bool,
    pub resize_on_border: bool,
    pub extend_border_grab_area: i32,
    pub hover_icon_on_border: bool,
    pub allow_tearing: bool,
    pub resize_corner: i32,
    pub modal_parent_blocking: bool,
    pub locale: String,
    pub snap: SnapConfig,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            border_size: 1,
            gaps_in: 5,
            gaps_out: 20,
            float_gaps: 0,
            gaps_workspaces: 0,
            layout: "dwindle".to_string(),
            no_focus_fallback: false,
            resize_on_border: false,
            extend_border_grab_area: 15,
            hover_icon_on_border: true,
            allow_tearing: false,
            resize_corner: 0,
            modal_parent_blocking: true,
            locale: String::new(),
            snap: SnapConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapConfig {
    pub enabled: bool,
    pub window_gap: i32,
    pub monitor_gap: i32,
    pub border_overlap: bool,
    pub respect_gaps: bool,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            window_gap: 10,
            monitor_gap: 10,
            border_overlap: false,
            respect_gaps: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DecorationConfig {
    pub rounding: i32,
    pub rounding_power: f64,
    pub active_opacity: f64,
    pub inactive_opacity: f64,
    pub fullscreen_opacity: f64,
    pub dim_modal: bool,
    pub dim_inactive: bool,
    pub dim_strength: f64,
    pub dim_special: f64,
    pub dim_around: f64,
    pub screen_shader: String,
    pub border_part_of_window: bool,
    pub blur: BlurConfig,
    pub shadow: ShadowConfig,
    pub glow: GlowConfig,
    pub motion_blur: MotionBlurConfig,
}

impl Default for DecorationConfig {
    fn default() -> Self {
        Self {
            rounding: 0,
            rounding_power: 2.0,
            active_opacity: 1.0,
            inactive_opacity: 1.0,
            fullscreen_opacity: 1.0,
            dim_modal: true,
            dim_inactive: false,
            dim_strength: 0.5,
            dim_special: 0.2,
            dim_around: 0.4,
            screen_shader: String::new(),
            border_part_of_window: true,
            blur: BlurConfig::default(),
            shadow: ShadowConfig::default(),
            glow: GlowConfig::default(),
            motion_blur: MotionBlurConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GlowConfig {
    pub enabled: bool,
    pub range: i32,
    pub render_power: i32,
}

impl Default for GlowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            range: 10,
            render_power: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionBlurConfig {
    pub enabled: bool,
    pub samples: i32,
}

impl Default for MotionBlurConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            samples: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BlurConfig {
    pub enabled: bool,
    pub size: i32,
    pub passes: i32,
    pub ignore_opacity: bool,
    pub new_optimizations: bool,
    pub xray: bool,
    pub noise: f64,
    pub contrast: f64,
    pub brightness: f64,
    pub vibrancy: f64,
    pub vibrancy_darkness: f64,
    pub special: bool,
    pub popups: bool,
    pub popups_ignorealpha: f64,
    pub input_methods: bool,
    pub input_methods_ignorealpha: f64,
}

impl Default for BlurConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 8,
            passes: 1,
            ignore_opacity: true,
            new_optimizations: true,
            xray: false,
            noise: 0.0117,
            contrast: 0.8916,
            brightness: 1.0,
            vibrancy: 0.1696,
            vibrancy_darkness: 0.0,
            special: false,
            popups: false,
            popups_ignorealpha: 0.2,
            input_methods: false,
            input_methods_ignorealpha: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShadowConfig {
    pub enabled: bool,
    pub range: i32,
    pub render_power: i32,
    pub sharp: bool,
    /// `[x, y]`, Hyprland's `Vec2`.
    pub offset: [f64; 2],
    pub scale: f64,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            range: 4,
            render_power: 3,
            sharp: false,
            offset: [0.0, 0.0],
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnimationsConfig {
    pub enabled: bool,
    pub workspace_wraparound: bool,
}

impl Default for AnimationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            workspace_wraparound: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// `[width, height]`; `[0, 0]` is off.
    pub single_window_aspect_ratio: [f64; 2],
    pub single_window_aspect_ratio_tolerance: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            single_window_aspect_ratio: [0.0, 0.0],
            single_window_aspect_ratio_tolerance: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DwindleConfig {
    pub default_split_ratio: f64,
    pub force_split: i32,
    pub permanent_direction_override: bool,
    pub precise_mouse_move: bool,
    pub preserve_split: bool,
    pub smart_resizing: bool,
    pub smart_split: bool,
    pub special_scale_factor: f64,
    pub split_bias: i32,
    pub split_width_multiplier: f64,
    pub use_active_for_splits: bool,
}

impl Default for DwindleConfig {
    fn default() -> Self {
        Self {
            default_split_ratio: 1.0,
            force_split: 0,
            permanent_direction_override: false,
            precise_mouse_move: false,
            preserve_split: false,
            smart_resizing: true,
            smart_split: false,
            special_scale_factor: 1.0,
            split_bias: 0,
            split_width_multiplier: 1.0,
            use_active_for_splits: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MasterConfig {
    pub allow_small_split: bool,
    pub always_keep_position: bool,
    pub center_ignores_reserved: bool,
    pub center_master_fallback: String,
    pub drop_at_cursor: bool,
    pub focus_master_on_close: bool,
    pub mfact: f64,
    pub new_on_active: String,
    pub new_on_top: bool,
    pub new_status: String,
    pub orientation: String,
    pub slave_count_for_center_master: i32,
    pub smart_resizing: bool,
    pub special_scale_factor: f64,
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            allow_small_split: false,
            always_keep_position: false,
            center_ignores_reserved: false,
            center_master_fallback: "left".to_string(),
            drop_at_cursor: true,
            focus_master_on_close: false,
            mfact: 0.55,
            new_on_active: "none".to_string(),
            new_on_top: false,
            new_status: "slave".to_string(),
            orientation: "left".to_string(),
            slave_count_for_center_master: 2,
            smart_resizing: true,
            special_scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScrollingConfig {
    pub column_width: f64,
    pub direction: String,
    pub explicit_column_widths: String,
    pub focus_fit_method: i32,
    pub follow_focus: bool,
    pub follow_min_visible: f64,
    pub fullscreen_on_one_column: bool,
    pub wrap_focus: bool,
    pub wrap_swapcol: bool,
}

impl Default for ScrollingConfig {
    fn default() -> Self {
        Self {
            column_width: 0.5,
            direction: "right".to_string(),
            explicit_column_widths: "0.333, 0.5, 0.667, 1.0".to_string(),
            focus_fit_method: 1,
            follow_focus: true,
            follow_min_visible: 0.4,
            fullscreen_on_one_column: true,
            wrap_focus: true,
            wrap_swapcol: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub kb_model: String,
    pub kb_layout: String,
    pub kb_variant: String,
    pub kb_options: String,
    pub kb_rules: String,
    pub kb_file: String,
    pub numlock_by_default: bool,
    pub resolve_binds_by_sym: bool,
    pub repeat_rate: i32,
    pub repeat_delay: i32,
    pub sensitivity: f64,
    pub accel_profile: String,
    pub force_no_accel: bool,
    pub rotation: i32,
    pub left_handed: bool,
    pub scroll_points: String,
    pub scroll_method: String,
    pub scroll_button: i32,
    pub scroll_button_lock: bool,
    pub scroll_factor: f64,
    pub natural_scroll: bool,
    pub follow_mouse: i32,
    pub follow_mouse_threshold: f64,
    pub follow_mouse_shrink: i32,
    pub focus_on_close: i32,
    pub mouse_refocus: bool,
    pub float_switch_override_focus: i32,
    pub special_fallthrough: bool,
    pub off_window_axis_events: i32,
    pub emulate_discrete_scroll: i32,
    pub touchpad: TouchpadConfig,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            kb_model: String::new(),
            kb_layout: "us".to_string(),
            kb_variant: String::new(),
            kb_options: String::new(),
            kb_rules: String::new(),
            kb_file: String::new(),
            numlock_by_default: false,
            resolve_binds_by_sym: false,
            repeat_rate: 25,
            repeat_delay: 600,
            sensitivity: 0.0,
            accel_profile: String::new(),
            force_no_accel: false,
            rotation: 0,
            left_handed: false,
            scroll_points: String::new(),
            scroll_method: String::new(),
            scroll_button: 0,
            scroll_button_lock: false,
            scroll_factor: 1.0,
            natural_scroll: false,
            follow_mouse: 1,
            follow_mouse_threshold: 0.0,
            follow_mouse_shrink: 0,
            focus_on_close: 0,
            mouse_refocus: true,
            float_switch_override_focus: 1,
            special_fallthrough: false,
            off_window_axis_events: 1,
            emulate_discrete_scroll: 1,
            touchpad: TouchpadConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TouchpadConfig {
    pub disable_while_typing: bool,
    pub natural_scroll: bool,
    pub scroll_factor: f64,
    pub middle_button_emulation: bool,
    pub tap_button_map: String,
    pub clickfinger_behavior: bool,
    pub tap_to_click: bool,
    pub drag_lock: i32,
    pub tap_and_drag: bool,
    pub flip_x: bool,
    pub flip_y: bool,
    pub drag_3fg: i32,
}

impl Default for TouchpadConfig {
    fn default() -> Self {
        Self {
            disable_while_typing: true,
            natural_scroll: false,
            scroll_factor: 1.0,
            middle_button_emulation: false,
            tap_button_map: String::new(),
            clickfinger_behavior: false,
            tap_to_click: true,
            drag_lock: 0,
            tap_and_drag: true,
            flip_x: false,
            flip_y: false,
            drag_3fg: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GesturesConfig {
    pub workspace_swipe_distance: i32,
    pub workspace_swipe_touch: bool,
    pub workspace_swipe_invert: bool,
    pub workspace_swipe_touch_invert: bool,
    pub workspace_swipe_min_speed_to_force: i32,
    pub workspace_swipe_cancel_ratio: f64,
    pub workspace_swipe_create_new: bool,
    pub workspace_swipe_direction_lock: bool,
    pub workspace_swipe_direction_lock_threshold: i32,
    pub workspace_swipe_forever: bool,
    pub workspace_swipe_use_r: bool,
    pub close_max_timeout: i32,
    pub scrolling: GestureScrollingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GestureScrollingConfig {
    pub move_snap_cursor: bool,
    pub move_snap_to_grid: bool,
}

impl Default for GestureScrollingConfig {
    fn default() -> Self {
        Self {
            move_snap_cursor: true,
            move_snap_to_grid: true,
        }
    }
}

impl Default for GesturesConfig {
    fn default() -> Self {
        Self {
            workspace_swipe_distance: 300,
            workspace_swipe_touch: false,
            workspace_swipe_invert: true,
            workspace_swipe_touch_invert: false,
            workspace_swipe_min_speed_to_force: 30,
            workspace_swipe_cancel_ratio: 0.5,
            workspace_swipe_create_new: true,
            workspace_swipe_direction_lock: true,
            workspace_swipe_direction_lock_threshold: 10,
            workspace_swipe_forever: false,
            workspace_swipe_use_r: false,
            close_max_timeout: 1000,
            scrolling: GestureScrollingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupConfig {
    pub auto_group: bool,
    pub insert_after_current: bool,
    pub focus_removed_window: bool,
    pub drag_into_group: i32,
    pub merge_groups_on_drag: bool,
    pub merge_groups_on_groupbar: bool,
    pub merge_floated_into_tiled_on_groupbar: bool,
    pub group_on_movetoworkspace: bool,
    pub groupbar: GroupbarConfig,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            auto_group: true,
            insert_after_current: true,
            focus_removed_window: true,
            drag_into_group: 1,
            merge_groups_on_drag: true,
            merge_groups_on_groupbar: true,
            merge_floated_into_tiled_on_groupbar: false,
            group_on_movetoworkspace: false,
            groupbar: GroupbarConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupbarConfig {
    pub enabled: bool,
    pub font_family: String,
    pub font_size: i32,
    pub font_weight_active: String,
    pub font_weight_inactive: String,
    pub gradients: bool,
    pub gradient_rounding: i32,
    pub gradient_rounding_power: f64,
    pub gradient_round_only_edges: bool,
    pub round_only_edges: bool,
    pub gaps_in: i32,
    pub gaps_out: i32,
    pub height: i32,
    pub indicator_gap: i32,
    pub indicator_height: i32,
    pub keep_upper_gap: bool,
    pub stacked: bool,
    pub priority: i32,
    pub render_titles: bool,
    pub text_offset: i32,
    pub text_padding: i32,
    pub scrolling: bool,
    pub rounding: i32,
    pub rounding_power: f64,
    pub blur: bool,
    pub disable_when_only: bool,
    pub middle_click_close: bool,
}

impl Default for GroupbarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            font_family: String::new(),
            font_size: 8,
            font_weight_active: "400".to_string(),
            font_weight_inactive: "400".to_string(),
            gradients: false,
            gradient_rounding: 2,
            gradient_rounding_power: 2.0,
            gradient_round_only_edges: true,
            round_only_edges: true,
            gaps_in: 2,
            gaps_out: 2,
            height: 14,
            indicator_gap: 0,
            indicator_height: 3,
            keep_upper_gap: true,
            stacked: false,
            priority: 3,
            render_titles: true,
            text_offset: 0,
            text_padding: 0,
            scrolling: true,
            rounding: 1,
            rounding_power: 2.0,
            blur: false,
            disable_when_only: false,
            middle_click_close: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MiscConfig {
    pub disable_hyprland_logo: bool,
    pub disable_splash_rendering: bool,
    pub disable_scale_notification: bool,
    pub font_family: String,
    pub force_default_wallpaper: i32,
    pub vrr: i32,
    pub mouse_move_enables_dpms: bool,
    pub key_press_enables_dpms: bool,
    pub always_follow_on_dnd: bool,
    pub layers_hog_keyboard_focus: bool,
    pub animate_manual_resizes: bool,
    pub animate_mouse_windowdragging: bool,
    pub disable_autoreload: bool,
    pub enable_swallow: bool,
    pub swallow_regex: String,
    pub swallow_exception_regex: String,
    pub focus_on_activate: bool,
    pub mouse_move_focuses_monitor: bool,
    pub close_special_on_empty: bool,
    pub on_focus_under_fullscreen: i32,
    pub exit_window_retains_fullscreen: bool,
    pub initial_workspace_tracking: i32,
    pub initial_workspace_token_timeout: i32,
    pub middle_click_paste: bool,
    pub allow_session_lock_restore: bool,
    pub session_lock_xray: bool,
    pub session_lock_blur: bool,
    pub enable_anr_dialog: bool,
    pub anr_missed_pings: i32,
    pub render_unfocused_fps: i32,
    pub lockdead_screen_delay: i32,
    pub size_limits_tiled: bool,
    pub splash_font_family: String,
    pub screencopy_force_8b: bool,
    pub name_vk_after_proc: bool,
    pub disable_xdg_env_checks: bool,
    pub disable_hyprland_guiutils_check: bool,
    pub disable_watchdog_warning: bool,
}

impl Default for MiscConfig {
    fn default() -> Self {
        Self {
            disable_hyprland_logo: false,
            disable_splash_rendering: false,
            disable_scale_notification: false,
            font_family: "Sans".to_string(),
            force_default_wallpaper: -1,
            vrr: 0,
            mouse_move_enables_dpms: false,
            key_press_enables_dpms: false,
            always_follow_on_dnd: true,
            layers_hog_keyboard_focus: true,
            animate_manual_resizes: false,
            animate_mouse_windowdragging: false,
            disable_autoreload: false,
            enable_swallow: false,
            swallow_regex: String::new(),
            swallow_exception_regex: String::new(),
            focus_on_activate: false,
            mouse_move_focuses_monitor: true,
            close_special_on_empty: true,
            on_focus_under_fullscreen: 2,
            exit_window_retains_fullscreen: false,
            initial_workspace_tracking: 1,
            initial_workspace_token_timeout: 10,
            middle_click_paste: true,
            allow_session_lock_restore: false,
            session_lock_xray: false,
            session_lock_blur: false,
            enable_anr_dialog: true,
            anr_missed_pings: 5,
            render_unfocused_fps: 15,
            lockdead_screen_delay: 1000,
            size_limits_tiled: false,
            splash_font_family: String::new(),
            screencopy_force_8b: true,
            name_vk_after_proc: true,
            disable_xdg_env_checks: false,
            disable_hyprland_guiutils_check: false,
            disable_watchdog_warning: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BindsConfig {
    pub pass_mouse_when_bound: bool,
    pub scroll_event_delay: i32,
    pub workspace_back_and_forth: bool,
    pub hide_special_on_workspace_change: bool,
    pub allow_workspace_cycles: bool,
    pub workspace_center_on: i32,
    pub focus_preferred_method: i32,
    pub ignore_group_lock: bool,
    pub movefocus_cycles_fullscreen: bool,
    pub movefocus_cycles_groupfirst: bool,
    pub disable_keybind_grabbing: bool,
    pub window_direction_monitor_fallback: bool,
    pub allow_pin_fullscreen: bool,
    pub drag_threshold: i32,
}

impl Default for BindsConfig {
    fn default() -> Self {
        Self {
            pass_mouse_when_bound: false,
            scroll_event_delay: 300,
            workspace_back_and_forth: false,
            hide_special_on_workspace_change: false,
            allow_workspace_cycles: false,
            workspace_center_on: 1,
            focus_preferred_method: 0,
            ignore_group_lock: false,
            movefocus_cycles_fullscreen: false,
            movefocus_cycles_groupfirst: false,
            disable_keybind_grabbing: false,
            window_direction_monitor_fallback: true,
            allow_pin_fullscreen: false,
            drag_threshold: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct XWaylandConfig {
    pub enabled: bool,
    pub use_nearest_neighbor: bool,
    pub force_zero_scaling: bool,
    pub create_abstract_socket: bool,
}

impl Default for XWaylandConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_nearest_neighbor: true,
            force_zero_scaling: false,
            create_abstract_socket: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenGlConfig {
    pub nvidia_anti_flicker: bool,
}

impl Default for OpenGlConfig {
    fn default() -> Self {
        Self {
            nvidia_anti_flicker: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    pub direct_scanout: i32,
    pub expand_undersized_textures: bool,
    pub xp_mode: bool,
    pub ctm_animation: i32,
    pub cm_enabled: bool,
    pub send_content_type: bool,
    pub cm_auto_hdr: i32,
    pub new_render_scheduling: bool,
    pub non_shader_cm: i32,
    pub non_shader_cm_interop: i32,
    pub cm_sdr_eotf: String,
    pub commit_timing_enabled: bool,
    pub icc_vcgt_enabled: bool,
    pub use_shader_blur_blend: bool,
    pub use_fp16: i32,
    pub fp16_sdr_tf: i32,
    pub keep_unmodified_copy: i32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            direct_scanout: 0,
            expand_undersized_textures: true,
            xp_mode: false,
            ctm_animation: 2,
            cm_enabled: true,
            send_content_type: true,
            cm_auto_hdr: 1,
            new_render_scheduling: false,
            non_shader_cm: 3,
            non_shader_cm_interop: 2,
            cm_sdr_eotf: "default".to_string(),
            commit_timing_enabled: true,
            icc_vcgt_enabled: true,
            use_shader_blur_blend: false,
            use_fp16: 2,
            fp16_sdr_tf: 0,
            keep_unmodified_copy: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CursorConfig {
    pub invisible: bool,
    pub sync_gsettings_theme: bool,
    pub no_hardware_cursors: i32,
    pub no_break_fs_vrr: i32,
    pub min_refresh_rate: i32,
    pub hotspot_padding: i32,
    pub inactive_timeout: f64,
    pub no_warps: bool,
    pub persistent_warps: bool,
    pub warp_on_change_workspace: i32,
    pub warp_on_toggle_special: i32,
    pub default_monitor: String,
    pub zoom_factor: f64,
    pub zoom_rigid: bool,
    pub zoom_detached_camera: bool,
    pub enable_hyprcursor: bool,
    pub hide_on_key_press: bool,
    pub hide_on_touch: bool,
    pub hide_on_tablet: bool,
    pub use_cpu_buffer: i32,
    pub warp_back_after_non_mouse_input: bool,
    pub zoom_disable_aa: bool,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            invisible: false,
            sync_gsettings_theme: true,
            no_hardware_cursors: 2,
            no_break_fs_vrr: 2,
            min_refresh_rate: 24,
            hotspot_padding: 0,
            inactive_timeout: 0.0,
            no_warps: false,
            persistent_warps: false,
            warp_on_change_workspace: 0,
            warp_on_toggle_special: 0,
            default_monitor: String::new(),
            zoom_factor: 1.0,
            zoom_rigid: false,
            zoom_detached_camera: true,
            enable_hyprcursor: true,
            hide_on_key_press: false,
            hide_on_touch: true,
            hide_on_tablet: false,
            use_cpu_buffer: 2,
            warp_back_after_non_mouse_input: false,
            zoom_disable_aa: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EcosystemConfig {
    pub no_update_news: bool,
    pub no_donation_nag: bool,
    pub enforce_permissions: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct QuirksConfig {
    pub prefer_hdr: i32,
    pub skip_non_kms_dmabuf_formats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    /// Variable frame rate: render only when something changed. Lives
    /// under `debug` in Hyprland 0.56 (`misc:vfr` no longer exists).
    pub vfr: bool,
    pub overlay: bool,
    pub damage_blink: bool,
    pub gl_debugging: bool,
    pub disable_logs: bool,
    pub disable_time: bool,
    pub damage_tracking: i32,
    pub enable_stdout_logs: bool,
    pub manual_crash: i32,
    pub suppress_errors: bool,
    pub disable_scale_checks: bool,
    pub error_limit: i32,
    pub error_position: i32,
    pub colored_stdout_logs: bool,
    pub pass: bool,
    pub full_cm_proto: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            vfr: true,
            overlay: false,
            damage_blink: false,
            gl_debugging: false,
            disable_logs: true,
            disable_time: true,
            damage_tracking: 2,
            enable_stdout_logs: false,
            manual_crash: 0,
            suppress_errors: false,
            disable_scale_checks: false,
            error_limit: 5,
            error_position: 0,
            colored_stdout_logs: true,
            pass: false,
            full_cm_proto: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyboardCatalog {
    pub models: Vec<KeyboardModel>,
    pub layouts: Vec<KeyboardLayout>,
    pub option_groups: Vec<KeyboardOptionGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardModel {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardLayout {
    pub name: String,
    pub description: String,
    pub variants: Vec<KeyboardVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardVariant {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardOptionGroup {
    pub name: String,
    pub description: String,
    pub options: Vec<KeyboardOption>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardOption {
    pub name: String,
    pub description: String,
}
