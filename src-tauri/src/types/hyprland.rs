use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::errors::HyprlandConfigError;

/// Represents the set of Hyprland "general" settings managed by Omarchist.
///
/// Each field is optional to distinguish between unset values (inherit Hyprland default)
/// and explicit overrides written to `~/.config/omarchist/hyprland.conf`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandGeneralSettings {
    pub border_size: Option<i32>,
    pub no_border_on_floating: Option<bool>,
    pub gaps_in: Option<String>,
    pub gaps_out: Option<String>,
    pub float_gaps: Option<String>,
    pub gaps_workspaces: Option<i32>,
    pub layout: Option<LayoutMode>,
    pub no_focus_fallback: Option<bool>,
    pub resize_on_border: Option<bool>,
    pub extend_border_grab_area: Option<i32>,
    pub hover_icon_on_border: Option<bool>,
    pub allow_tearing: Option<bool>,
    pub resize_corner: Option<i32>,
    #[serde(default)]
    pub snap: HyprlandGeneralSnapSettings,
}

impl HyprlandGeneralSettings {
    /// Construct settings populated with Hyprland defaults.
    pub fn with_defaults() -> Self {
        Self {
            border_size: Some(2),
            no_border_on_floating: Some(false),
            gaps_in: Some("5".into()),
            gaps_out: Some("10".into()),
            float_gaps: Some("0".into()),
            gaps_workspaces: Some(0),
            layout: Some(LayoutMode::Dwindle),
            no_focus_fallback: Some(false),
            resize_on_border: Some(false),
            extend_border_grab_area: Some(15),
            hover_icon_on_border: Some(true),
            allow_tearing: Some(false),
            resize_corner: Some(0),
            snap: HyprlandGeneralSnapSettings::with_defaults(),
        }
    }

    /// Merge a map of overrides (Hyprland key -> value) into the settings instance.
    pub fn apply_overrides(
        &mut self,
        general_overrides: &BTreeMap<String, HyprlandValue>,
        snap_overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in general_field_registry() {
            if let Some(value) = general_overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        self.snap.apply_overrides(snap_overrides)?;
        Ok(())
    }

    /// Convert the explicit overrides contained in this struct into Hyprland key/value pairs.
    pub fn to_override_maps(&self) -> HyprlandGeneralOverrideMaps {
        let mut general = BTreeMap::new();
        for field in general_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    general.insert(field.key().to_string(), value);
                }
            }
        }

        let snap = self.snap.to_override_map();

        HyprlandGeneralOverrideMaps { general, snap }
    }
}

/// Hyprland general snap sub-settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandGeneralSnapSettings {
    pub enabled: Option<bool>,
    pub window_gap: Option<i32>,
    pub monitor_gap: Option<i32>,
    pub border_overlap: Option<bool>,
    pub respect_gaps: Option<bool>,
}

impl HyprlandGeneralSnapSettings {
    pub fn with_defaults() -> Self {
        Self {
            enabled: Some(false),
            window_gap: Some(10),
            monitor_gap: Some(10),
            border_overlap: Some(false),
            respect_gaps: Some(false),
        }
    }

    pub fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in snap_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in snap_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    map.insert(field.key().to_string(), value);
                }
            }
        }
        map
    }
}

/// Container representing flattened override maps for persistence.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandGeneralOverrideMaps {
    pub general: BTreeMap<String, HyprlandValue>,
    pub snap: BTreeMap<String, HyprlandValue>,
}

impl HyprlandGeneralOverrideMaps {
    pub fn is_empty(&self) -> bool {
        self.general.is_empty() && self.snap.is_empty()
    }
}

/// Data transfer object returned to the frontend representing Hyprland general settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandGeneralSnapshot {
    pub effective: HyprlandGeneralSettings,
    pub overrides: HyprlandGeneralSettings,
}

impl Default for HyprlandGeneralSnapshot {
    fn default() -> Self {
        Self {
            effective: HyprlandGeneralSettings::with_defaults(),
            overrides: HyprlandGeneralSettings::default(),
        }
    }
}

impl HyprlandGeneralSnapshot {
    pub fn new(effective: HyprlandGeneralSettings, overrides: HyprlandGeneralSettings) -> Self {
        Self {
            effective,
            overrides,
        }
    }
}

/// Represents Hyprland "decoration" settings including blur and shadow subsections.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandDecorationSettings {
    pub rounding: Option<i32>,
    pub rounding_power: Option<f32>,
    pub active_opacity: Option<f32>,
    pub inactive_opacity: Option<f32>,
    pub fullscreen_opacity: Option<f32>,
    pub dim_modal: Option<bool>,
    pub dim_inactive: Option<bool>,
    pub dim_strength: Option<f32>,
    pub dim_special: Option<f32>,
    pub dim_around: Option<f32>,
    pub screen_shader: Option<String>,
    pub border_part_of_window: Option<bool>,
    #[serde(default)]
    pub blur: HyprlandDecorationBlurSettings,
    #[serde(default)]
    pub shadow: HyprlandDecorationShadowSettings,
}

impl HyprlandDecorationSettings {
    pub fn with_defaults() -> Self {
        Self {
            rounding: Some(0),
            rounding_power: Some(2.0_f32),
            active_opacity: Some(1.0_f32),
            inactive_opacity: Some(1.0_f32),
            fullscreen_opacity: Some(1.0_f32),
            dim_modal: Some(true),
            dim_inactive: Some(false),
            dim_strength: Some(0.5_f32),
            dim_special: Some(0.2_f32),
            dim_around: Some(0.4_f32),
            screen_shader: Some(String::new()),
            border_part_of_window: Some(true),
            blur: HyprlandDecorationBlurSettings::with_defaults(),
            shadow: HyprlandDecorationShadowSettings::with_defaults(),
        }
    }

    pub fn apply_overrides(
        &mut self,
        decoration_overrides: &BTreeMap<String, HyprlandValue>,
        blur_overrides: &BTreeMap<String, HyprlandValue>,
        shadow_overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in decoration_field_registry() {
            if let Some(value) = decoration_overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }

        self.blur.apply_overrides(blur_overrides)?;
        self.shadow.apply_overrides(shadow_overrides)?;
        Ok(())
    }

    pub fn to_override_maps(&self) -> HyprlandDecorationOverrideMaps {
        let mut decoration = BTreeMap::new();
        for field in decoration_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    decoration.insert(field.key().to_string(), value);
                }
            }
        }

        HyprlandDecorationOverrideMaps {
            decoration,
            blur: self.blur.to_override_map(),
            shadow: self.shadow.to_override_map(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandDecorationBlurSettings {
    pub enabled: Option<bool>,
    pub size: Option<i32>,
    pub passes: Option<i32>,
    pub ignore_opacity: Option<bool>,
    pub new_optimizations: Option<bool>,
    pub xray: Option<bool>,
    pub noise: Option<f32>,
    pub contrast: Option<f32>,
    pub brightness: Option<f32>,
    pub vibrancy: Option<f32>,
    pub vibrancy_darkness: Option<f32>,
    pub special: Option<bool>,
    pub popups: Option<bool>,
    pub popups_ignorealpha: Option<f32>,
    pub input_methods: Option<bool>,
    pub input_methods_ignorealpha: Option<f32>,
}

impl HyprlandDecorationBlurSettings {
    pub fn with_defaults() -> Self {
        Self {
            enabled: Some(true),
            size: Some(8),
            passes: Some(1),
            ignore_opacity: Some(true),
            new_optimizations: Some(true),
            xray: Some(false),
            noise: Some(0.0117_f32),
            contrast: Some(0.8916_f32),
            brightness: Some(0.8172_f32),
            vibrancy: Some(0.1696_f32),
            vibrancy_darkness: Some(0.0_f32),
            special: Some(false),
            popups: Some(false),
            popups_ignorealpha: Some(0.2_f32),
            input_methods: Some(false),
            input_methods_ignorealpha: Some(0.2_f32),
        }
    }

    pub fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in blur_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in blur_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    map.insert(field.key().to_string(), value);
                }
            }
        }
        map
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandDecorationShadowSettings {
    pub enabled: Option<bool>,
    pub range: Option<i32>,
    pub render_power: Option<i32>,
    pub sharp: Option<bool>,
    pub ignore_window: Option<bool>,
    pub color: Option<String>,
    pub color_inactive: Option<String>,
    pub offset: Option<String>,
    pub scale: Option<f32>,
}

impl HyprlandDecorationShadowSettings {
    pub fn with_defaults() -> Self {
        Self {
            enabled: Some(true),
            range: Some(4),
            render_power: Some(3),
            sharp: Some(false),
            ignore_window: Some(true),
            color: Some("0xee1a1a1a".into()),
            color_inactive: Some("".into()),
            offset: Some("0 0".into()),
            scale: Some(1.0_f32),
        }
    }

    pub fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in shadow_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in shadow_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    map.insert(field.key().to_string(), value);
                }
            }
        }
        map
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandDecorationOverrideMaps {
    pub decoration: BTreeMap<String, HyprlandValue>,
    pub blur: BTreeMap<String, HyprlandValue>,
    pub shadow: BTreeMap<String, HyprlandValue>,
}

impl HyprlandDecorationOverrideMaps {
    pub fn is_empty(&self) -> bool {
        self.decoration.is_empty() && self.blur.is_empty() && self.shadow.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandDecorationSnapshot {
    pub effective: HyprlandDecorationSettings,
    pub overrides: HyprlandDecorationSettings,
}

impl Default for HyprlandDecorationSnapshot {
    fn default() -> Self {
        Self {
            effective: HyprlandDecorationSettings::with_defaults(),
            overrides: HyprlandDecorationSettings::default(),
        }
    }
}

impl HyprlandDecorationSnapshot {
    pub fn new(
        effective: HyprlandDecorationSettings,
        overrides: HyprlandDecorationSettings,
    ) -> Self {
        Self {
            effective,
            overrides,
        }
    }
}

/// Represents Hyprland "animations" settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandAnimationSettings {
    pub enabled: Option<bool>,
    pub workspace_wraparound: Option<bool>,
}

impl HyprlandAnimationSettings {
    pub fn with_defaults() -> Self {
        Self {
            enabled: Some(true),
            workspace_wraparound: Some(false),
        }
    }

    pub fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in animation_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in animation_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    map.insert(field.key().to_string(), value);
                }
            }
        }
        map
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandAnimationSnapshot {
    pub effective: HyprlandAnimationSettings,
    pub overrides: HyprlandAnimationSettings,
}

impl Default for HyprlandAnimationSnapshot {
    fn default() -> Self {
        Self {
            effective: HyprlandAnimationSettings::with_defaults(),
            overrides: HyprlandAnimationSettings::default(),
        }
    }
}

impl HyprlandAnimationSnapshot {
    pub fn new(effective: HyprlandAnimationSettings, overrides: HyprlandAnimationSettings) -> Self {
        Self {
            effective,
            overrides,
        }
    }
}

/// Represents Hyprland "input" settings.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HyprlandInputSettings {
    // Keyboard settings
    pub kb_model: Option<String>,
    pub kb_layout: Option<String>,
    pub kb_variant: Option<String>,
    pub kb_options: Option<String>,
    pub kb_rules: Option<String>,
    pub kb_file: Option<String>,
    pub numlock_by_default: Option<bool>,
    pub resolve_binds_by_sym: Option<bool>,
    pub repeat_rate: Option<i32>,
    pub repeat_delay: Option<i32>,
    
    // Mouse settings
    pub sensitivity: Option<f32>,
    pub accel_profile: Option<String>,
    pub force_no_accel: Option<bool>,
    pub left_handed: Option<bool>,
    
    // Scroll settings
    pub scroll_points: Option<String>,
    pub scroll_method: Option<String>,
    pub scroll_button: Option<i32>,
    pub scroll_button_lock: Option<bool>,
    pub scroll_factor: Option<f32>,
    pub natural_scroll: Option<bool>,
    
    // Focus settings
    pub follow_mouse: Option<i32>,
    pub follow_mouse_threshold: Option<f32>,
    pub focus_on_close: Option<i32>,
    pub mouse_refocus: Option<bool>,
    pub float_switch_override_focus: Option<i32>,
    pub special_fallthrough: Option<bool>,
    
    // Misc settings
    pub off_window_axis_events: Option<i32>,
    pub emulate_discrete_scroll: Option<i32>,
}

impl HyprlandInputSettings {
    pub fn with_defaults() -> Self {
        Self {
            // Keyboard settings
            kb_model: Some(String::new()),
            kb_layout: Some("us".into()),
            kb_variant: Some(String::new()),
            kb_options: Some(String::new()),
            kb_rules: Some(String::new()),
            kb_file: Some(String::new()),
            numlock_by_default: Some(false),
            resolve_binds_by_sym: Some(false),
            repeat_rate: Some(25),
            repeat_delay: Some(600),
            
            // Mouse settings
            sensitivity: Some(0.0),
            accel_profile: Some(String::new()),
            force_no_accel: Some(false),
            left_handed: Some(false),
            
            // Scroll settings
            scroll_points: Some(String::new()),
            scroll_method: Some(String::new()),
            scroll_button: Some(0),
            scroll_button_lock: Some(false),
            scroll_factor: Some(1.0),
            natural_scroll: Some(false),
            
            // Focus settings
            follow_mouse: Some(1),
            follow_mouse_threshold: Some(0.0),
            focus_on_close: Some(0),
            mouse_refocus: Some(true),
            float_switch_override_focus: Some(1),
            special_fallthrough: Some(false),
            
            // Misc settings
            off_window_axis_events: Some(1),
            emulate_discrete_scroll: Some(1),
        }
    }

    pub fn apply_overrides(
        &mut self,
        overrides: &BTreeMap<String, HyprlandValue>,
    ) -> Result<(), HyprlandConfigError> {
        for field in input_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in input_field_registry() {
            if let Some(value) = field.extract(self) {
                if value != field.default_value() {
                    map.insert(field.key().to_string(), value);
                }
            }
        }
        map
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandInputOverrideMap {
    pub settings: BTreeMap<String, HyprlandValue>,
}

impl HyprlandInputOverrideMap {
    pub fn is_empty(&self) -> bool {
        self.settings.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyprlandInputSnapshot {
    pub effective: HyprlandInputSettings,
    pub overrides: HyprlandInputSettings,
}

impl Default for HyprlandInputSnapshot {
    fn default() -> Self {
        Self {
            effective: HyprlandInputSettings::with_defaults(),
            overrides: HyprlandInputSettings::default(),
        }
    }
}

impl HyprlandInputSnapshot {
    pub fn new(effective: HyprlandInputSettings, overrides: HyprlandInputSettings) -> Self {
        Self {
            effective,
            overrides,
        }
    }
}

/// Catalog of keyboard models, layouts, variants, and options sourced from `base.lst`.
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

/// Hyprland layout modes supported by Omarchist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutMode {
    Master,
    Dwindle,
}

impl LayoutMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            LayoutMode::Master => "master",
            LayoutMode::Dwindle => "dwindle",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, HyprlandConfigError> {
        match raw.trim().to_lowercase().as_str() {
            "master" => Ok(LayoutMode::Master),
            "dwindle" => Ok(LayoutMode::Dwindle),
            other => Err(HyprlandConfigError::Validation {
                field: "layout".to_string(),
                message: format!("Unsupported layout value '{other}'"),
            }),
        }
    }
}

impl From<LayoutMode> for HyprlandValue {
    fn from(value: LayoutMode) -> Self {
        HyprlandValue::String(value.as_str().to_string())
    }
}

/// Registry of supported Hyprland general fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeneralField {
    BorderSize,
    NoBorderOnFloating,
    GapsIn,
    GapsOut,
    FloatGaps,
    GapsWorkspaces,
    Layout,
    NoFocusFallback,
    ResizeOnBorder,
    ExtendBorderGrabArea,
    HoverIconOnBorder,
    AllowTearing,
    ResizeCorner,
}

impl GeneralField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "border_size" => Some(GeneralField::BorderSize),
            "no_border_on_floating" => Some(GeneralField::NoBorderOnFloating),
            "gaps_in" => Some(GeneralField::GapsIn),
            "gaps_out" => Some(GeneralField::GapsOut),
            "float_gaps" => Some(GeneralField::FloatGaps),
            "gaps_workspaces" => Some(GeneralField::GapsWorkspaces),
            "layout" => Some(GeneralField::Layout),
            "no_focus_fallback" => Some(GeneralField::NoFocusFallback),
            "resize_on_border" => Some(GeneralField::ResizeOnBorder),
            "extend_border_grab_area" => Some(GeneralField::ExtendBorderGrabArea),
            "hover_icon_on_border" => Some(GeneralField::HoverIconOnBorder),
            "allow_tearing" => Some(GeneralField::AllowTearing),
            "resize_corner" => Some(GeneralField::ResizeCorner),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            GeneralField::BorderSize => "border_size",
            GeneralField::NoBorderOnFloating => "no_border_on_floating",
            GeneralField::GapsIn => "gaps_in",
            GeneralField::GapsOut => "gaps_out",
            GeneralField::FloatGaps => "float_gaps",
            GeneralField::GapsWorkspaces => "gaps_workspaces",
            GeneralField::Layout => "layout",
            GeneralField::NoFocusFallback => "no_focus_fallback",
            GeneralField::ResizeOnBorder => "resize_on_border",
            GeneralField::ExtendBorderGrabArea => "extend_border_grab_area",
            GeneralField::HoverIconOnBorder => "hover_icon_on_border",
            GeneralField::AllowTearing => "allow_tearing",
            GeneralField::ResizeCorner => "resize_corner",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandGeneralSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            GeneralField::BorderSize => set_i32(
                settings,
                |s, v| s.border_size = v,
                self.key(),
                0..=1000,
                value,
            ),
            GeneralField::NoBorderOnFloating => set_bool(
                settings,
                |s, v| s.no_border_on_floating = v,
                self.key(),
                value,
            ),
            GeneralField::GapsIn => set_string(settings, |s, v| s.gaps_in = v, self.key(), value),
            GeneralField::GapsOut => set_string(settings, |s, v| s.gaps_out = v, self.key(), value),
            GeneralField::FloatGaps => {
                set_string(settings, |s, v| s.float_gaps = v, self.key(), value)
            },
            GeneralField::GapsWorkspaces => set_i32(
                settings,
                |s, v| s.gaps_workspaces = v,
                self.key(),
                0..=1000,
                value,
            ),
            GeneralField::Layout => match value {
                HyprlandValue::String(ref raw) => {
                    let layout_mode = LayoutMode::parse(raw)?;
                    settings.layout = Some(layout_mode);
                    Ok(())
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected layout string, received {other:?}"),
                }),
            },
            GeneralField::NoFocusFallback => {
                set_bool(settings, |s, v| s.no_focus_fallback = v, self.key(), value)
            },
            GeneralField::ResizeOnBorder => {
                set_bool(settings, |s, v| s.resize_on_border = v, self.key(), value)
            },
            GeneralField::ExtendBorderGrabArea => set_i32(
                settings,
                |s, v| s.extend_border_grab_area = v,
                self.key(),
                0..=1000,
                value,
            ),
            GeneralField::HoverIconOnBorder => set_bool(
                settings,
                |s, v| s.hover_icon_on_border = v,
                self.key(),
                value,
            ),
            GeneralField::AllowTearing => {
                set_bool(settings, |s, v| s.allow_tearing = v, self.key(), value)
            },
            GeneralField::ResizeCorner => set_i32(
                settings,
                |s, v| s.resize_corner = v,
                self.key(),
                0..=4,
                value,
            ),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            GeneralField::BorderSize => Ok(HyprlandValue::Int(parse_i32(self.key(), raw)?)),
            GeneralField::NoBorderOnFloating => {
                Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?))
            },
            GeneralField::GapsIn => Ok(HyprlandValue::String(raw.trim().to_string())),
            GeneralField::GapsOut => Ok(HyprlandValue::String(raw.trim().to_string())),
            GeneralField::FloatGaps => Ok(HyprlandValue::String(raw.trim().to_string())),
            GeneralField::GapsWorkspaces => Ok(HyprlandValue::Int(parse_i32(self.key(), raw)?)),
            GeneralField::Layout => {
                let layout_mode = LayoutMode::parse(raw)?;
                Ok(layout_mode.into())
            },
            GeneralField::NoFocusFallback => Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?)),
            GeneralField::ResizeOnBorder => Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?)),
            GeneralField::ExtendBorderGrabArea => {
                Ok(HyprlandValue::Int(parse_i32(self.key(), raw)?))
            },
            GeneralField::HoverIconOnBorder => {
                Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?))
            },
            GeneralField::AllowTearing => Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?)),
            GeneralField::ResizeCorner => Ok(HyprlandValue::Int(parse_i32(self.key(), raw)?)),
        }
    }

    pub fn extract(&self, settings: &HyprlandGeneralSettings) -> Option<HyprlandValue> {
        match self {
            GeneralField::BorderSize => settings.border_size.map(HyprlandValue::Int),
            GeneralField::NoBorderOnFloating => {
                settings.no_border_on_floating.map(HyprlandValue::Bool)
            },
            GeneralField::GapsIn => settings
                .gaps_in
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            GeneralField::GapsOut => settings
                .gaps_out
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            GeneralField::FloatGaps => settings
                .float_gaps
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            GeneralField::GapsWorkspaces => settings.gaps_workspaces.map(HyprlandValue::Int),
            GeneralField::Layout => settings.layout.map(|v| v.into()),
            GeneralField::NoFocusFallback => settings.no_focus_fallback.map(HyprlandValue::Bool),
            GeneralField::ResizeOnBorder => settings.resize_on_border.map(HyprlandValue::Bool),
            GeneralField::ExtendBorderGrabArea => {
                settings.extend_border_grab_area.map(HyprlandValue::Int)
            },
            GeneralField::HoverIconOnBorder => {
                settings.hover_icon_on_border.map(HyprlandValue::Bool)
            },
            GeneralField::AllowTearing => settings.allow_tearing.map(HyprlandValue::Bool),
            GeneralField::ResizeCorner => settings.resize_corner.map(HyprlandValue::Int),
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            GeneralField::BorderSize => HyprlandValue::Int(2),
            GeneralField::NoBorderOnFloating => HyprlandValue::Bool(false),
            GeneralField::GapsIn => HyprlandValue::String("5".to_string()),
            GeneralField::GapsOut => HyprlandValue::String("10".to_string()),
            GeneralField::FloatGaps => HyprlandValue::String("0".to_string()),
            GeneralField::GapsWorkspaces => HyprlandValue::Int(0),
            GeneralField::Layout => LayoutMode::Dwindle.into(),
            GeneralField::NoFocusFallback => HyprlandValue::Bool(false),
            GeneralField::ResizeOnBorder => HyprlandValue::Bool(false),
            GeneralField::ExtendBorderGrabArea => HyprlandValue::Int(15),
            GeneralField::HoverIconOnBorder => HyprlandValue::Bool(true),
            GeneralField::AllowTearing => HyprlandValue::Bool(false),
            GeneralField::ResizeCorner => HyprlandValue::Int(0),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            GeneralField::BorderSize => match value {
                HyprlandValue::Int(v) if (0..=1000).contains(v) => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Value '{v}' is outside of allowed range 0-1000"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            GeneralField::GapsIn | GeneralField::GapsOut | GeneralField::FloatGaps => match value {
                HyprlandValue::String(v) if !v.trim().is_empty() => Ok(()),
                HyprlandValue::String(_) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: "Value cannot be empty".to_string(),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected string, received {other:?}"),
                }),
            },
            GeneralField::GapsWorkspaces => match value {
                HyprlandValue::Int(v) if (0..=1000).contains(v) => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Value '{v}' is outside of allowed range 0-1000"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            GeneralField::Layout => match value {
                HyprlandValue::String(raw) => {
                    LayoutMode::parse(raw)?;
                    Ok(())
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected layout string, received {other:?}"),
                }),
            },
            GeneralField::ExtendBorderGrabArea => match value {
                HyprlandValue::Int(v) if (0..=1000).contains(v) => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Value '{v}' is outside of allowed range 0-1000"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            GeneralField::ResizeCorner => match value {
                HyprlandValue::Int(v) if (0..=4).contains(v) => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Value '{v}' is outside of allowed range 0-4"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            _ => match value {
                HyprlandValue::Bool(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected boolean, received {other:?}"),
                }),
            },
        }
    }
}

/// Return the registry of all supported Hyprland general fields.
pub fn general_field_registry() -> &'static [GeneralField] {
    const GENERAL_FIELDS: [GeneralField; 13] = [
        GeneralField::BorderSize,
        GeneralField::NoBorderOnFloating,
        GeneralField::GapsIn,
        GeneralField::GapsOut,
        GeneralField::FloatGaps,
        GeneralField::GapsWorkspaces,
        GeneralField::Layout,
        GeneralField::NoFocusFallback,
        GeneralField::ResizeOnBorder,
        GeneralField::ExtendBorderGrabArea,
        GeneralField::HoverIconOnBorder,
        GeneralField::AllowTearing,
        GeneralField::ResizeCorner,
    ];
    &GENERAL_FIELDS
}

/// Registry of supported Hyprland general:snap fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnapField {
    Enabled,
    WindowGap,
    MonitorGap,
    BorderOverlap,
    RespectGaps,
}

impl SnapField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "enabled" => Some(SnapField::Enabled),
            "window_gap" => Some(SnapField::WindowGap),
            "monitor_gap" => Some(SnapField::MonitorGap),
            "border_overlap" => Some(SnapField::BorderOverlap),
            "respect_gaps" => Some(SnapField::RespectGaps),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            SnapField::Enabled => "enabled",
            SnapField::WindowGap => "window_gap",
            SnapField::MonitorGap => "monitor_gap",
            SnapField::BorderOverlap => "border_overlap",
            SnapField::RespectGaps => "respect_gaps",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandGeneralSnapSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            SnapField::Enabled => set_snap_bool(|s, v| s.enabled = v, self.key(), value, settings),
            SnapField::WindowGap => set_snap_i32(
                |s, v| s.window_gap = v,
                self.key(),
                0..=1000,
                value,
                settings,
            ),
            SnapField::MonitorGap => set_snap_i32(
                |s, v| s.monitor_gap = v,
                self.key(),
                0..=1000,
                value,
                settings,
            ),
            SnapField::BorderOverlap => {
                set_snap_bool(|s, v| s.border_overlap = v, self.key(), value, settings)
            },
            SnapField::RespectGaps => {
                set_snap_bool(|s, v| s.respect_gaps = v, self.key(), value, settings)
            },
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            SnapField::Enabled | SnapField::BorderOverlap | SnapField::RespectGaps => {
                Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?))
            },
            SnapField::WindowGap | SnapField::MonitorGap => {
                Ok(HyprlandValue::Int(parse_i32(self.key(), raw)?))
            },
        }
    }

    pub fn extract(&self, settings: &HyprlandGeneralSnapSettings) -> Option<HyprlandValue> {
        match self {
            SnapField::Enabled => settings.enabled.map(HyprlandValue::Bool),
            SnapField::WindowGap => settings.window_gap.map(HyprlandValue::Int),
            SnapField::MonitorGap => settings.monitor_gap.map(HyprlandValue::Int),
            SnapField::BorderOverlap => settings.border_overlap.map(HyprlandValue::Bool),
            SnapField::RespectGaps => settings.respect_gaps.map(HyprlandValue::Bool),
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            SnapField::Enabled => HyprlandValue::Bool(false),
            SnapField::WindowGap => HyprlandValue::Int(10),
            SnapField::MonitorGap => HyprlandValue::Int(10),
            SnapField::BorderOverlap => HyprlandValue::Bool(false),
            SnapField::RespectGaps => HyprlandValue::Bool(false),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            SnapField::WindowGap | SnapField::MonitorGap => match value {
                HyprlandValue::Int(v) if (0..=1000).contains(v) => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Value '{v}' is outside of allowed range 0-1000"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            _ => match value {
                HyprlandValue::Bool(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected boolean, received {other:?}"),
                }),
            },
        }
    }
}

pub fn snap_field_registry() -> &'static [SnapField] {
    const SNAP_FIELDS: [SnapField; 5] = [
        SnapField::Enabled,
        SnapField::WindowGap,
        SnapField::MonitorGap,
        SnapField::BorderOverlap,
        SnapField::RespectGaps,
    ];
    &SNAP_FIELDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DecorationField {
    Rounding,
    RoundingPower,
    ActiveOpacity,
    InactiveOpacity,
    FullscreenOpacity,
    DimModal,
    DimInactive,
    DimStrength,
    DimSpecial,
    DimAround,
    ScreenShader,
    BorderPartOfWindow,
}

impl DecorationField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "rounding" => Some(DecorationField::Rounding),
            "rounding_power" => Some(DecorationField::RoundingPower),
            "active_opacity" => Some(DecorationField::ActiveOpacity),
            "inactive_opacity" => Some(DecorationField::InactiveOpacity),
            "fullscreen_opacity" => Some(DecorationField::FullscreenOpacity),
            "dim_modal" => Some(DecorationField::DimModal),
            "dim_inactive" => Some(DecorationField::DimInactive),
            "dim_strength" => Some(DecorationField::DimStrength),
            "dim_special" => Some(DecorationField::DimSpecial),
            "dim_around" => Some(DecorationField::DimAround),
            "screen_shader" => Some(DecorationField::ScreenShader),
            "border_part_of_window" => Some(DecorationField::BorderPartOfWindow),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            DecorationField::Rounding => "rounding",
            DecorationField::RoundingPower => "rounding_power",
            DecorationField::ActiveOpacity => "active_opacity",
            DecorationField::InactiveOpacity => "inactive_opacity",
            DecorationField::FullscreenOpacity => "fullscreen_opacity",
            DecorationField::DimModal => "dim_modal",
            DecorationField::DimInactive => "dim_inactive",
            DecorationField::DimStrength => "dim_strength",
            DecorationField::DimSpecial => "dim_special",
            DecorationField::DimAround => "dim_around",
            DecorationField::ScreenShader => "screen_shader",
            DecorationField::BorderPartOfWindow => "border_part_of_window",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandDecorationSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            DecorationField::Rounding => set_decoration_i32(
                settings,
                |s, v| s.rounding = v,
                "rounding",
                0..=i32::MAX,
                value,
            ),
            DecorationField::RoundingPower => set_decoration_f32(
                settings,
                |s, v| s.rounding_power = v,
                "rounding_power",
                1.0_f32..=10.0_f32,
                value,
            ),
            DecorationField::ActiveOpacity => set_decoration_f32(
                settings,
                |s, v| s.active_opacity = v,
                "active_opacity",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::InactiveOpacity => set_decoration_f32(
                settings,
                |s, v| s.inactive_opacity = v,
                "inactive_opacity",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::FullscreenOpacity => set_decoration_f32(
                settings,
                |s, v| s.fullscreen_opacity = v,
                "fullscreen_opacity",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::DimModal => {
                set_decoration_bool(settings, |s, v| s.dim_modal = v, "dim_modal", value)
            },
            DecorationField::DimInactive => {
                set_decoration_bool(settings, |s, v| s.dim_inactive = v, "dim_inactive", value)
            },
            DecorationField::DimStrength => set_decoration_f32(
                settings,
                |s, v| s.dim_strength = v,
                "dim_strength",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::DimSpecial => set_decoration_f32(
                settings,
                |s, v| s.dim_special = v,
                "dim_special",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::DimAround => set_decoration_f32(
                settings,
                |s, v| s.dim_around = v,
                "dim_around",
                0.0_f32..=1.0_f32,
                value,
            ),
            DecorationField::ScreenShader => {
                set_decoration_string(settings, |s, v| s.screen_shader = v, "screen_shader", value)
            },
            DecorationField::BorderPartOfWindow => set_decoration_bool(
                settings,
                |s, v| s.border_part_of_window = v,
                "border_part_of_window",
                value,
            ),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            DecorationField::Rounding => {
                let value = parse_i32(self.key(), raw)?;
                if value < 0 {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be >= 0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(value))
                }
            },
            DecorationField::RoundingPower => {
                let value = parse_f32(self.key(), raw)?;
                if !(1.0_f32..=10.0_f32).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 1.0 and 10.0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Float(value))
                }
            },
            DecorationField::ActiveOpacity
            | DecorationField::InactiveOpacity
            | DecorationField::FullscreenOpacity
            | DecorationField::DimStrength
            | DecorationField::DimSpecial
            | DecorationField::DimAround => {
                let value = parse_f32(self.key(), raw)?;
                if !(0.0_f32..=1.0_f32).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 0.0 and 1.0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Float(value))
                }
            },
            DecorationField::ScreenShader => Ok(HyprlandValue::String(raw.trim().to_string())),
            DecorationField::DimModal
            | DecorationField::DimInactive
            | DecorationField::BorderPartOfWindow => {
                let value = parse_bool(self.key(), raw)?;
                Ok(HyprlandValue::Bool(value))
            },
        }
    }

    pub fn extract(&self, settings: &HyprlandDecorationSettings) -> Option<HyprlandValue> {
        match self {
            DecorationField::Rounding => settings.rounding.map(HyprlandValue::from),
            DecorationField::RoundingPower => settings.rounding_power.map(HyprlandValue::from),
            DecorationField::ActiveOpacity => settings.active_opacity.map(HyprlandValue::from),
            DecorationField::InactiveOpacity => settings.inactive_opacity.map(HyprlandValue::from),
            DecorationField::FullscreenOpacity => {
                settings.fullscreen_opacity.map(HyprlandValue::from)
            },
            DecorationField::DimModal => settings.dim_modal.map(HyprlandValue::from),
            DecorationField::DimInactive => settings.dim_inactive.map(HyprlandValue::from),
            DecorationField::DimStrength => settings.dim_strength.map(HyprlandValue::from),
            DecorationField::DimSpecial => settings.dim_special.map(HyprlandValue::from),
            DecorationField::DimAround => settings.dim_around.map(HyprlandValue::from),
            DecorationField::ScreenShader => settings
                .screen_shader
                .as_ref()
                .map(|value| HyprlandValue::String(value.clone())),
            DecorationField::BorderPartOfWindow => {
                settings.border_part_of_window.map(HyprlandValue::from)
            },
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            DecorationField::Rounding => HyprlandValue::Int(0),
            DecorationField::RoundingPower => HyprlandValue::Float(2.0_f32),
            DecorationField::ActiveOpacity => HyprlandValue::Float(1.0_f32),
            DecorationField::InactiveOpacity => HyprlandValue::Float(1.0_f32),
            DecorationField::FullscreenOpacity => HyprlandValue::Float(1.0_f32),
            DecorationField::DimModal => HyprlandValue::Bool(true),
            DecorationField::DimInactive => HyprlandValue::Bool(false),
            DecorationField::DimStrength => HyprlandValue::Float(0.5_f32),
            DecorationField::DimSpecial => HyprlandValue::Float(0.2_f32),
            DecorationField::DimAround => HyprlandValue::Float(0.4_f32),
            DecorationField::ScreenShader => HyprlandValue::String(String::new()),
            DecorationField::BorderPartOfWindow => HyprlandValue::Bool(true),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            DecorationField::Rounding => match value {
                HyprlandValue::Int(v) => {
                    if *v < 0 {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be >= 0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            DecorationField::RoundingPower => match value {
                HyprlandValue::Float(v) => {
                    if !(*v >= 1.0 && *v <= 10.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 1.0 and 10.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                HyprlandValue::Int(v) => {
                    let as_float = *v as f32;
                    if !(as_float >= 1.0 && as_float <= 10.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 1.0 and 10.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
            DecorationField::ActiveOpacity
            | DecorationField::InactiveOpacity
            | DecorationField::FullscreenOpacity
            | DecorationField::DimStrength
            | DecorationField::DimSpecial
            | DecorationField::DimAround => match value {
                HyprlandValue::Float(v) => {
                    if !(*v >= 0.0 && *v <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                HyprlandValue::Int(v) => {
                    let as_float = *v as f32;
                    if !(as_float >= 0.0 && as_float <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
            DecorationField::ScreenShader => match value {
                HyprlandValue::String(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected string, received {other:?}"),
                }),
            },
            DecorationField::DimModal
            | DecorationField::DimInactive
            | DecorationField::BorderPartOfWindow => match value {
                HyprlandValue::Bool(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected boolean, received {other:?}"),
                }),
            },
        }
    }
}

pub fn decoration_field_registry() -> &'static [DecorationField] {
    const DECORATION_FIELDS: [DecorationField; 12] = [
        DecorationField::Rounding,
        DecorationField::RoundingPower,
        DecorationField::ActiveOpacity,
        DecorationField::InactiveOpacity,
        DecorationField::FullscreenOpacity,
        DecorationField::DimModal,
        DecorationField::DimInactive,
        DecorationField::DimStrength,
        DecorationField::DimSpecial,
        DecorationField::DimAround,
        DecorationField::ScreenShader,
        DecorationField::BorderPartOfWindow,
    ];
    &DECORATION_FIELDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlurField {
    Enabled,
    Size,
    Passes,
    IgnoreOpacity,
    NewOptimizations,
    Xray,
    Noise,
    Contrast,
    Brightness,
    Vibrancy,
    VibrancyDarkness,
    Special,
    Popups,
    PopupsIgnorealpha,
    InputMethods,
    InputMethodsIgnorealpha,
}

impl BlurField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "enabled" => Some(BlurField::Enabled),
            "size" => Some(BlurField::Size),
            "passes" => Some(BlurField::Passes),
            "ignore_opacity" => Some(BlurField::IgnoreOpacity),
            "new_optimizations" => Some(BlurField::NewOptimizations),
            "xray" => Some(BlurField::Xray),
            "noise" => Some(BlurField::Noise),
            "contrast" => Some(BlurField::Contrast),
            "brightness" => Some(BlurField::Brightness),
            "vibrancy" => Some(BlurField::Vibrancy),
            "vibrancy_darkness" => Some(BlurField::VibrancyDarkness),
            "special" => Some(BlurField::Special),
            "popups" => Some(BlurField::Popups),
            "popups_ignorealpha" => Some(BlurField::PopupsIgnorealpha),
            "input_methods" => Some(BlurField::InputMethods),
            "input_methods_ignorealpha" => Some(BlurField::InputMethodsIgnorealpha),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            BlurField::Enabled => "enabled",
            BlurField::Size => "size",
            BlurField::Passes => "passes",
            BlurField::IgnoreOpacity => "ignore_opacity",
            BlurField::NewOptimizations => "new_optimizations",
            BlurField::Xray => "xray",
            BlurField::Noise => "noise",
            BlurField::Contrast => "contrast",
            BlurField::Brightness => "brightness",
            BlurField::Vibrancy => "vibrancy",
            BlurField::VibrancyDarkness => "vibrancy_darkness",
            BlurField::Special => "special",
            BlurField::Popups => "popups",
            BlurField::PopupsIgnorealpha => "popups_ignorealpha",
            BlurField::InputMethods => "input_methods",
            BlurField::InputMethodsIgnorealpha => "input_methods_ignorealpha",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandDecorationBlurSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            BlurField::Enabled => set_blur_bool(settings, |s, v| s.enabled = v, self.key(), value),
            BlurField::IgnoreOpacity => {
                set_blur_bool(settings, |s, v| s.ignore_opacity = v, self.key(), value)
            },
            BlurField::NewOptimizations => {
                set_blur_bool(settings, |s, v| s.new_optimizations = v, self.key(), value)
            },
            BlurField::Xray => set_blur_bool(settings, |s, v| s.xray = v, self.key(), value),
            BlurField::Special => set_blur_bool(settings, |s, v| s.special = v, self.key(), value),
            BlurField::Popups => set_blur_bool(settings, |s, v| s.popups = v, self.key(), value),
            BlurField::InputMethods => {
                set_blur_bool(settings, |s, v| s.input_methods = v, self.key(), value)
            },
            BlurField::Size => {
                set_blur_i32(settings, |s, v| s.size = v, self.key(), 1..=i32::MAX, value)
            },
            BlurField::Passes => set_blur_i32(
                settings,
                |s, v| s.passes = v,
                self.key(),
                1..=i32::MAX,
                value,
            ),
            BlurField::Noise => set_blur_f32(
                settings,
                |s, v| s.noise = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
            BlurField::Contrast => set_blur_f32(
                settings,
                |s, v| s.contrast = v,
                self.key(),
                0.0_f32..=2.0_f32,
                value,
            ),
            BlurField::Brightness => set_blur_f32(
                settings,
                |s, v| s.brightness = v,
                self.key(),
                0.0_f32..=2.0_f32,
                value,
            ),
            BlurField::Vibrancy => set_blur_f32(
                settings,
                |s, v| s.vibrancy = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
            BlurField::VibrancyDarkness => set_blur_f32(
                settings,
                |s, v| s.vibrancy_darkness = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
            BlurField::PopupsIgnorealpha => set_blur_f32(
                settings,
                |s, v| s.popups_ignorealpha = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
            BlurField::InputMethodsIgnorealpha => set_blur_f32(
                settings,
                |s, v| s.input_methods_ignorealpha = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            BlurField::Enabled
            | BlurField::IgnoreOpacity
            | BlurField::NewOptimizations
            | BlurField::Xray
            | BlurField::Special
            | BlurField::Popups
            | BlurField::InputMethods => {
                let value = parse_bool(self.key(), raw)?;
                Ok(HyprlandValue::Bool(value))
            },
            BlurField::Size | BlurField::Passes => {
                let value = parse_i32(self.key(), raw)?;
                if value < 1 {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be >= 1".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(value))
                }
            },
            BlurField::Noise
            | BlurField::Vibrancy
            | BlurField::VibrancyDarkness
            | BlurField::PopupsIgnorealpha
            | BlurField::InputMethodsIgnorealpha => {
                let value = parse_f32(self.key(), raw)?;
                if !(0.0_f32..=1.0_f32).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 0.0 and 1.0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Float(value))
                }
            },
            BlurField::Contrast | BlurField::Brightness => {
                let value = parse_f32(self.key(), raw)?;
                if !(0.0_f32..=2.0_f32).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 0.0 and 2.0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Float(value))
                }
            },
        }
    }

    pub fn extract(&self, settings: &HyprlandDecorationBlurSettings) -> Option<HyprlandValue> {
        match self {
            BlurField::Enabled => settings.enabled.map(HyprlandValue::from),
            BlurField::Size => settings.size.map(HyprlandValue::from),
            BlurField::Passes => settings.passes.map(HyprlandValue::from),
            BlurField::IgnoreOpacity => settings.ignore_opacity.map(HyprlandValue::from),
            BlurField::NewOptimizations => settings.new_optimizations.map(HyprlandValue::from),
            BlurField::Xray => settings.xray.map(HyprlandValue::from),
            BlurField::Noise => settings.noise.map(HyprlandValue::from),
            BlurField::Contrast => settings.contrast.map(HyprlandValue::from),
            BlurField::Brightness => settings.brightness.map(HyprlandValue::from),
            BlurField::Vibrancy => settings.vibrancy.map(HyprlandValue::from),
            BlurField::VibrancyDarkness => settings.vibrancy_darkness.map(HyprlandValue::from),
            BlurField::Special => settings.special.map(HyprlandValue::from),
            BlurField::Popups => settings.popups.map(HyprlandValue::from),
            BlurField::PopupsIgnorealpha => settings.popups_ignorealpha.map(HyprlandValue::from),
            BlurField::InputMethods => settings.input_methods.map(HyprlandValue::from),
            BlurField::InputMethodsIgnorealpha => {
                settings.input_methods_ignorealpha.map(HyprlandValue::from)
            },
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            BlurField::Enabled => HyprlandValue::Bool(true),
            BlurField::Size => HyprlandValue::Int(8),
            BlurField::Passes => HyprlandValue::Int(1),
            BlurField::IgnoreOpacity => HyprlandValue::Bool(true),
            BlurField::NewOptimizations => HyprlandValue::Bool(true),
            BlurField::Xray => HyprlandValue::Bool(false),
            BlurField::Noise => HyprlandValue::Float(0.0117_f32),
            BlurField::Contrast => HyprlandValue::Float(0.8916_f32),
            BlurField::Brightness => HyprlandValue::Float(0.8172_f32),
            BlurField::Vibrancy => HyprlandValue::Float(0.1696_f32),
            BlurField::VibrancyDarkness => HyprlandValue::Float(0.0_f32),
            BlurField::Special => HyprlandValue::Bool(false),
            BlurField::Popups => HyprlandValue::Bool(false),
            BlurField::PopupsIgnorealpha => HyprlandValue::Float(0.2_f32),
            BlurField::InputMethods => HyprlandValue::Bool(false),
            BlurField::InputMethodsIgnorealpha => HyprlandValue::Float(0.2_f32),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            BlurField::Enabled
            | BlurField::IgnoreOpacity
            | BlurField::NewOptimizations
            | BlurField::Xray
            | BlurField::Special
            | BlurField::Popups
            | BlurField::InputMethods => match value {
                HyprlandValue::Bool(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected boolean, received {other:?}"),
                }),
            },
            BlurField::Size | BlurField::Passes => match value {
                HyprlandValue::Int(v) => {
                    if *v < 1 {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be >= 1".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            BlurField::Contrast | BlurField::Brightness => match value {
                HyprlandValue::Float(v) => {
                    if !(*v >= 0.0 && *v <= 2.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 2.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                HyprlandValue::Int(v) => {
                    let as_float = *v as f32;
                    if !(as_float >= 0.0 && as_float <= 2.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 2.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
            BlurField::Noise
            | BlurField::Vibrancy
            | BlurField::VibrancyDarkness
            | BlurField::PopupsIgnorealpha
            | BlurField::InputMethodsIgnorealpha => match value {
                HyprlandValue::Float(v) => {
                    if !(*v >= 0.0 && *v <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                HyprlandValue::Int(v) => {
                    let as_float = *v as f32;
                    if !(as_float >= 0.0 && as_float <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
        }
    }
}

pub fn blur_field_registry() -> &'static [BlurField] {
    const BLUR_FIELDS: [BlurField; 16] = [
        BlurField::Enabled,
        BlurField::Size,
        BlurField::Passes,
        BlurField::IgnoreOpacity,
        BlurField::NewOptimizations,
        BlurField::Xray,
        BlurField::Noise,
        BlurField::Contrast,
        BlurField::Brightness,
        BlurField::Vibrancy,
        BlurField::VibrancyDarkness,
        BlurField::Special,
        BlurField::Popups,
        BlurField::PopupsIgnorealpha,
        BlurField::InputMethods,
        BlurField::InputMethodsIgnorealpha,
    ];
    &BLUR_FIELDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadowField {
    Enabled,
    Range,
    RenderPower,
    Sharp,
    IgnoreWindow,
    Color,
    ColorInactive,
    Offset,
    Scale,
}

impl ShadowField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "enabled" => Some(ShadowField::Enabled),
            "range" => Some(ShadowField::Range),
            "render_power" => Some(ShadowField::RenderPower),
            "sharp" => Some(ShadowField::Sharp),
            "ignore_window" => Some(ShadowField::IgnoreWindow),
            "color" => Some(ShadowField::Color),
            "color_inactive" => Some(ShadowField::ColorInactive),
            "offset" => Some(ShadowField::Offset),
            "scale" => Some(ShadowField::Scale),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            ShadowField::Enabled => "enabled",
            ShadowField::Range => "range",
            ShadowField::RenderPower => "render_power",
            ShadowField::Sharp => "sharp",
            ShadowField::IgnoreWindow => "ignore_window",
            ShadowField::Color => "color",
            ShadowField::ColorInactive => "color_inactive",
            ShadowField::Offset => "offset",
            ShadowField::Scale => "scale",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandDecorationShadowSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            ShadowField::Enabled => {
                set_shadow_bool(settings, |s, v| s.enabled = v, self.key(), value)
            },
            ShadowField::Sharp => set_shadow_bool(settings, |s, v| s.sharp = v, self.key(), value),
            ShadowField::IgnoreWindow => {
                set_shadow_bool(settings, |s, v| s.ignore_window = v, self.key(), value)
            },
            ShadowField::Range => set_shadow_i32(
                settings,
                |s, v| s.range = v,
                self.key(),
                0..=i32::MAX,
                value,
            ),
            ShadowField::RenderPower => set_shadow_i32(
                settings,
                |s, v| s.render_power = v,
                self.key(),
                1..=4,
                value,
            ),
            ShadowField::Color => {
                set_shadow_string(settings, |s, v| s.color = v, self.key(), value)
            },
            ShadowField::ColorInactive => {
                set_shadow_string(settings, |s, v| s.color_inactive = v, self.key(), value)
            },
            ShadowField::Offset => {
                set_shadow_string(settings, |s, v| s.offset = v, self.key(), value)
            },
            ShadowField::Scale => set_shadow_f32(
                settings,
                |s, v| s.scale = v,
                self.key(),
                0.0_f32..=1.0_f32,
                value,
            ),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            ShadowField::Enabled | ShadowField::Sharp | ShadowField::IgnoreWindow => {
                let value = parse_bool(self.key(), raw)?;
                Ok(HyprlandValue::Bool(value))
            },
            ShadowField::Range => {
                let value = parse_i32(self.key(), raw)?;
                if value < 0 {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be >= 0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(value))
                }
            },
            ShadowField::RenderPower => {
                let value = parse_i32(self.key(), raw)?;
                if !(1..=4).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 1 and 4".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(value))
                }
            },
            ShadowField::Color | ShadowField::ColorInactive | ShadowField::Offset => {
                Ok(HyprlandValue::String(raw.trim().to_string()))
            },
            ShadowField::Scale => {
                let value = parse_f32(self.key(), raw)?;
                if !(0.0_f32..=1.0_f32).contains(&value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 0.0 and 1.0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Float(value))
                }
            },
        }
    }

    pub fn extract(&self, settings: &HyprlandDecorationShadowSettings) -> Option<HyprlandValue> {
        match self {
            ShadowField::Enabled => settings.enabled.map(HyprlandValue::from),
            ShadowField::Range => settings.range.map(HyprlandValue::from),
            ShadowField::RenderPower => settings.render_power.map(HyprlandValue::from),
            ShadowField::Sharp => settings.sharp.map(HyprlandValue::from),
            ShadowField::IgnoreWindow => settings.ignore_window.map(HyprlandValue::from),
            ShadowField::Color => settings
                .color
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            ShadowField::ColorInactive => settings
                .color_inactive
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            ShadowField::Offset => settings
                .offset
                .as_ref()
                .map(|v| HyprlandValue::String(v.clone())),
            ShadowField::Scale => settings.scale.map(HyprlandValue::from),
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            ShadowField::Enabled => HyprlandValue::Bool(true),
            ShadowField::Range => HyprlandValue::Int(4),
            ShadowField::RenderPower => HyprlandValue::Int(3),
            ShadowField::Sharp => HyprlandValue::Bool(false),
            ShadowField::IgnoreWindow => HyprlandValue::Bool(true),
            ShadowField::Color => HyprlandValue::String("0xee1a1a1a".into()),
            ShadowField::ColorInactive => HyprlandValue::String("".into()),
            ShadowField::Offset => HyprlandValue::String("0 0".into()),
            ShadowField::Scale => HyprlandValue::Float(1.0_f32),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            ShadowField::Enabled | ShadowField::Sharp | ShadowField::IgnoreWindow => match value {
                HyprlandValue::Bool(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected boolean, received {other:?}"),
                }),
            },
            ShadowField::Range => match value {
                HyprlandValue::Int(v) => {
                    if *v < 0 {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be >= 0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            ShadowField::RenderPower => match value {
                HyprlandValue::Int(v) => {
                    if !matches!(*v, 1..=4) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 1 and 4".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            ShadowField::Color | ShadowField::ColorInactive | ShadowField::Offset => match value {
                HyprlandValue::String(_) => Ok(()),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected string, received {other:?}"),
                }),
            },
            ShadowField::Scale => match value {
                HyprlandValue::Float(v) => {
                    if !(*v >= 0.0 && *v <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                HyprlandValue::Int(v) => {
                    let as_float = *v as f32;
                    if !(as_float >= 0.0 && as_float <= 1.0) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0.0 and 1.0".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
        }
    }
}

pub fn shadow_field_registry() -> &'static [ShadowField] {
    const SHADOW_FIELDS: [ShadowField; 9] = [
        ShadowField::Enabled,
        ShadowField::Range,
        ShadowField::RenderPower,
        ShadowField::Sharp,
        ShadowField::IgnoreWindow,
        ShadowField::Color,
        ShadowField::ColorInactive,
        ShadowField::Offset,
        ShadowField::Scale,
    ];
    &SHADOW_FIELDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationField {
    Enabled,
    WorkspaceWraparound,
}

impl AnimationField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "enabled" => Some(AnimationField::Enabled),
            "workspace_wraparound" => Some(AnimationField::WorkspaceWraparound),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            AnimationField::Enabled => "enabled",
            AnimationField::WorkspaceWraparound => "workspace_wraparound",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandAnimationSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            AnimationField::Enabled => {
                set_animation_bool(settings, |s, v| s.enabled = v, "enabled", value)
            },
            AnimationField::WorkspaceWraparound => set_animation_bool(
                settings,
                |s, v| s.workspace_wraparound = v,
                "workspace_wraparound",
                value,
            ),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            AnimationField::Enabled | AnimationField::WorkspaceWraparound => {
                Ok(HyprlandValue::Bool(parse_bool(self.key(), raw)?))
            },
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            AnimationField::Enabled | AnimationField::WorkspaceWraparound => {
                if !matches!(value, HyprlandValue::Bool(_)) {
                    return Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: format!("Expected boolean, received {value:?}"),
                    });
                }
                Ok(())
            },
        }
    }

    pub fn extract(&self, settings: &HyprlandAnimationSettings) -> Option<HyprlandValue> {
        match self {
            AnimationField::Enabled => settings.enabled.map(HyprlandValue::Bool),
            AnimationField::WorkspaceWraparound => {
                settings.workspace_wraparound.map(HyprlandValue::Bool)
            },
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            AnimationField::Enabled => HyprlandValue::Bool(true),
            AnimationField::WorkspaceWraparound => HyprlandValue::Bool(false),
        }
    }
}

pub fn animation_field_registry() -> &'static [AnimationField] {
    const ANIMATION_FIELDS: [AnimationField; 2] =
        [AnimationField::Enabled, AnimationField::WorkspaceWraparound];
    &ANIMATION_FIELDS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputField {
    // Keyboard settings
    KbModel,
    KbLayout,
    KbVariant,
    KbOptions,
    KbRules,
    KbFile,
    NumlockByDefault,
    ResolveBindsBySym,
    RepeatRate,
    RepeatDelay,
    
    // Mouse settings
    Sensitivity,
    AccelProfile,
    ForceNoAccel,
    LeftHanded,
    
    // Scroll settings
    ScrollPoints,
    ScrollMethod,
    ScrollButton,
    ScrollButtonLock,
    ScrollFactor,
    NaturalScroll,
    
    // Focus settings
    FollowMouse,
    FollowMouseThreshold,
    FocusOnClose,
    MouseRefocus,
    FloatSwitchOverrideFocus,
    SpecialFallthrough,
    
    // Misc settings
    OffWindowAxisEvents,
    EmulateDiscreteScroll,
}

impl InputField {
    pub fn from_key(key: &str) -> Option<Self> {
        match key.trim() {
            "kb_model" => Some(InputField::KbModel),
            "kb_layout" => Some(InputField::KbLayout),
            "kb_variant" => Some(InputField::KbVariant),
            "kb_options" => Some(InputField::KbOptions),
            "kb_rules" => Some(InputField::KbRules),
            "kb_file" => Some(InputField::KbFile),
            "numlock_by_default" => Some(InputField::NumlockByDefault),
            "resolve_binds_by_sym" => Some(InputField::ResolveBindsBySym),
            "repeat_rate" => Some(InputField::RepeatRate),
            "repeat_delay" => Some(InputField::RepeatDelay),
            "sensitivity" => Some(InputField::Sensitivity),
            "accel_profile" => Some(InputField::AccelProfile),
            "force_no_accel" => Some(InputField::ForceNoAccel),
            "left_handed" => Some(InputField::LeftHanded),
            "scroll_points" => Some(InputField::ScrollPoints),
            "scroll_method" => Some(InputField::ScrollMethod),
            "scroll_button" => Some(InputField::ScrollButton),
            "scroll_button_lock" => Some(InputField::ScrollButtonLock),
            "scroll_factor" => Some(InputField::ScrollFactor),
            "natural_scroll" => Some(InputField::NaturalScroll),
            "follow_mouse" => Some(InputField::FollowMouse),
            "follow_mouse_threshold" => Some(InputField::FollowMouseThreshold),
            "focus_on_close" => Some(InputField::FocusOnClose),
            "mouse_refocus" => Some(InputField::MouseRefocus),
            "float_switch_override_focus" => Some(InputField::FloatSwitchOverrideFocus),
            "special_fallthrough" => Some(InputField::SpecialFallthrough),
            "off_window_axis_events" => Some(InputField::OffWindowAxisEvents),
            "emulate_discrete_scroll" => Some(InputField::EmulateDiscreteScroll),
            _ => None,
        }
    }

    pub fn key(&self) -> &'static str {
        match self {
            InputField::KbModel => "kb_model",
            InputField::KbLayout => "kb_layout",
            InputField::KbVariant => "kb_variant",
            InputField::KbOptions => "kb_options",
            InputField::KbRules => "kb_rules",
            InputField::KbFile => "kb_file",
            InputField::NumlockByDefault => "numlock_by_default",
            InputField::ResolveBindsBySym => "resolve_binds_by_sym",
            InputField::RepeatRate => "repeat_rate",
            InputField::RepeatDelay => "repeat_delay",
            InputField::Sensitivity => "sensitivity",
            InputField::AccelProfile => "accel_profile",
            InputField::ForceNoAccel => "force_no_accel",
            InputField::LeftHanded => "left_handed",
            InputField::ScrollPoints => "scroll_points",
            InputField::ScrollMethod => "scroll_method",
            InputField::ScrollButton => "scroll_button",
            InputField::ScrollButtonLock => "scroll_button_lock",
            InputField::ScrollFactor => "scroll_factor",
            InputField::NaturalScroll => "natural_scroll",
            InputField::FollowMouse => "follow_mouse",
            InputField::FollowMouseThreshold => "follow_mouse_threshold",
            InputField::FocusOnClose => "focus_on_close",
            InputField::MouseRefocus => "mouse_refocus",
            InputField::FloatSwitchOverrideFocus => "float_switch_override_focus",
            InputField::SpecialFallthrough => "special_fallthrough",
            InputField::OffWindowAxisEvents => "off_window_axis_events",
            InputField::EmulateDiscreteScroll => "emulate_discrete_scroll",
        }
    }

    pub fn apply(
        &self,
        value: HyprlandValue,
        settings: &mut HyprlandInputSettings,
    ) -> Result<(), HyprlandConfigError> {
        match self {
            // Keyboard - string fields
            InputField::KbModel => {
                set_input_string(settings, |s, v| s.kb_model = v, self.key(), value)
            },
            InputField::KbLayout => {
                set_input_string(settings, |s, v| s.kb_layout = v, self.key(), value)
            },
            InputField::KbVariant => {
                set_input_string(settings, |s, v| s.kb_variant = v, self.key(), value)
            },
            InputField::KbOptions => {
                set_input_string(settings, |s, v| s.kb_options = v, self.key(), value)
            },
            InputField::KbRules => {
                set_input_string(settings, |s, v| s.kb_rules = v, self.key(), value)
            },
            InputField::KbFile => {
                set_input_string(settings, |s, v| s.kb_file = v, self.key(), value)
            },
            
            // Keyboard - bool fields
            InputField::NumlockByDefault => {
                set_input_bool(settings, |s, v| s.numlock_by_default = v, self.key(), value)
            },
            InputField::ResolveBindsBySym => {
                set_input_bool(settings, |s, v| s.resolve_binds_by_sym = v, self.key(), value)
            },
            
            // Keyboard - int fields
            InputField::RepeatRate => {
                set_input_int(settings, |s, v| s.repeat_rate = v, self.key(), value)
            },
            InputField::RepeatDelay => {
                set_input_int(settings, |s, v| s.repeat_delay = v, self.key(), value)
            },
            
            // Mouse - float fields
            InputField::Sensitivity => {
                set_input_float(settings, |s, v| s.sensitivity = v, self.key(), value)
            },
            
            // Mouse - string fields
            InputField::AccelProfile => {
                set_input_string(settings, |s, v| s.accel_profile = v, self.key(), value)
            },
            
            // Mouse - bool fields
            InputField::ForceNoAccel => {
                set_input_bool(settings, |s, v| s.force_no_accel = v, self.key(), value)
            },
            InputField::LeftHanded => {
                set_input_bool(settings, |s, v| s.left_handed = v, self.key(), value)
            },
            
            // Scroll - string fields
            InputField::ScrollPoints => {
                set_input_string(settings, |s, v| s.scroll_points = v, self.key(), value)
            },
            InputField::ScrollMethod => {
                set_input_string(settings, |s, v| s.scroll_method = v, self.key(), value)
            },
            
            // Scroll - int fields
            InputField::ScrollButton => {
                set_input_int(settings, |s, v| s.scroll_button = v, self.key(), value)
            },
            
            // Scroll - bool fields
            InputField::ScrollButtonLock => {
                set_input_bool(settings, |s, v| s.scroll_button_lock = v, self.key(), value)
            },
            InputField::NaturalScroll => {
                set_input_bool(settings, |s, v| s.natural_scroll = v, self.key(), value)
            },
            
            // Scroll - float fields
            InputField::ScrollFactor => {
                set_input_float(settings, |s, v| s.scroll_factor = v, self.key(), value)
            },
            
            // Focus - int fields
            InputField::FollowMouse => {
                set_input_int(settings, |s, v| s.follow_mouse = v, self.key(), value)
            },
            InputField::FocusOnClose => {
                set_input_int(settings, |s, v| s.focus_on_close = v, self.key(), value)
            },
            InputField::FloatSwitchOverrideFocus => {
                set_input_int(settings, |s, v| s.float_switch_override_focus = v, self.key(), value)
            },
            
            // Focus - float fields
            InputField::FollowMouseThreshold => {
                set_input_float(settings, |s, v| s.follow_mouse_threshold = v, self.key(), value)
            },
            
            // Focus - bool fields
            InputField::MouseRefocus => {
                set_input_bool(settings, |s, v| s.mouse_refocus = v, self.key(), value)
            },
            InputField::SpecialFallthrough => {
                set_input_bool(settings, |s, v| s.special_fallthrough = v, self.key(), value)
            },
            
            // Misc - int fields
            InputField::OffWindowAxisEvents => {
                set_input_int(settings, |s, v| s.off_window_axis_events = v, self.key(), value)
            },
            InputField::EmulateDiscreteScroll => {
                set_input_int(settings, |s, v| s.emulate_discrete_scroll = v, self.key(), value)
            },
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        Ok(HyprlandValue::String(raw.trim().to_string()))
    }

    pub fn extract(&self, settings: &HyprlandInputSettings) -> Option<HyprlandValue> {
        match self {
            // Keyboard - string fields
            InputField::KbModel => settings.kb_model.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::KbLayout => settings.kb_layout.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::KbVariant => settings.kb_variant.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::KbOptions => settings.kb_options.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::KbRules => settings.kb_rules.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::KbFile => settings.kb_file.as_ref().map(|v| HyprlandValue::String(v.clone())),
            
            // Keyboard - bool fields
            InputField::NumlockByDefault => settings.numlock_by_default.map(HyprlandValue::Bool),
            InputField::ResolveBindsBySym => settings.resolve_binds_by_sym.map(HyprlandValue::Bool),
            
            // Keyboard - int fields
            InputField::RepeatRate => settings.repeat_rate.map(HyprlandValue::Int),
            InputField::RepeatDelay => settings.repeat_delay.map(HyprlandValue::Int),
            
            // Mouse - float fields
            InputField::Sensitivity => settings.sensitivity.map(HyprlandValue::Float),
            
            // Mouse - string fields
            InputField::AccelProfile => settings.accel_profile.as_ref().map(|v| HyprlandValue::String(v.clone())),
            
            // Mouse - bool fields
            InputField::ForceNoAccel => settings.force_no_accel.map(HyprlandValue::Bool),
            InputField::LeftHanded => settings.left_handed.map(HyprlandValue::Bool),
            
            // Scroll - string fields
            InputField::ScrollPoints => settings.scroll_points.as_ref().map(|v| HyprlandValue::String(v.clone())),
            InputField::ScrollMethod => settings.scroll_method.as_ref().map(|v| HyprlandValue::String(v.clone())),
            
            // Scroll - int fields
            InputField::ScrollButton => settings.scroll_button.map(HyprlandValue::Int),
            
            // Scroll - bool fields
            InputField::ScrollButtonLock => settings.scroll_button_lock.map(HyprlandValue::Bool),
            InputField::NaturalScroll => settings.natural_scroll.map(HyprlandValue::Bool),
            
            // Scroll - float fields
            InputField::ScrollFactor => settings.scroll_factor.map(HyprlandValue::Float),
            
            // Focus - int fields
            InputField::FollowMouse => settings.follow_mouse.map(HyprlandValue::Int),
            InputField::FocusOnClose => settings.focus_on_close.map(HyprlandValue::Int),
            InputField::FloatSwitchOverrideFocus => settings.float_switch_override_focus.map(HyprlandValue::Int),
            
            // Focus - float fields
            InputField::FollowMouseThreshold => settings.follow_mouse_threshold.map(HyprlandValue::Float),
            
            // Focus - bool fields
            InputField::MouseRefocus => settings.mouse_refocus.map(HyprlandValue::Bool),
            InputField::SpecialFallthrough => settings.special_fallthrough.map(HyprlandValue::Bool),
            
            // Misc - int fields
            InputField::OffWindowAxisEvents => settings.off_window_axis_events.map(HyprlandValue::Int),
            InputField::EmulateDiscreteScroll => settings.emulate_discrete_scroll.map(HyprlandValue::Int),
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            // Keyboard - string fields
            InputField::KbModel => HyprlandValue::String(String::new()),
            InputField::KbLayout => HyprlandValue::String("us".into()),
            InputField::KbVariant => HyprlandValue::String(String::new()),
            InputField::KbOptions => HyprlandValue::String(String::new()),
            InputField::KbRules => HyprlandValue::String(String::new()),
            InputField::KbFile => HyprlandValue::String(String::new()),
            
            // Keyboard - bool fields
            InputField::NumlockByDefault => HyprlandValue::Bool(false),
            InputField::ResolveBindsBySym => HyprlandValue::Bool(false),
            
            // Keyboard - int fields
            InputField::RepeatRate => HyprlandValue::Int(25),
            InputField::RepeatDelay => HyprlandValue::Int(600),
            
            // Mouse - float fields
            InputField::Sensitivity => HyprlandValue::Float(0.0),
            
            // Mouse - string fields
            InputField::AccelProfile => HyprlandValue::String(String::new()),
            
            // Mouse - bool fields
            InputField::ForceNoAccel => HyprlandValue::Bool(false),
            InputField::LeftHanded => HyprlandValue::Bool(false),
            
            // Scroll - string fields
            InputField::ScrollPoints => HyprlandValue::String(String::new()),
            InputField::ScrollMethod => HyprlandValue::String(String::new()),
            
            // Scroll - int fields
            InputField::ScrollButton => HyprlandValue::Int(0),
            
            // Scroll - bool fields
            InputField::ScrollButtonLock => HyprlandValue::Bool(false),
            InputField::NaturalScroll => HyprlandValue::Bool(false),
            
            // Scroll - float fields
            InputField::ScrollFactor => HyprlandValue::Float(1.0),
            
            // Focus - int fields
            InputField::FollowMouse => HyprlandValue::Int(1),
            InputField::FocusOnClose => HyprlandValue::Int(0),
            InputField::FloatSwitchOverrideFocus => HyprlandValue::Int(1),
            
            // Focus - float fields
            InputField::FollowMouseThreshold => HyprlandValue::Float(0.0),
            
            // Focus - bool fields
            InputField::MouseRefocus => HyprlandValue::Bool(true),
            InputField::SpecialFallthrough => HyprlandValue::Bool(false),
            
            // Misc - int fields
            InputField::OffWindowAxisEvents => HyprlandValue::Int(1),
            InputField::EmulateDiscreteScroll => HyprlandValue::Int(1),
        }
    }

    pub fn validate(&self, value: &HyprlandValue) -> Result<(), HyprlandConfigError> {
        match self {
            // String fields that can be empty
            InputField::KbModel | InputField::KbVariant | InputField::KbOptions | 
            InputField::KbRules | InputField::KbFile | InputField::AccelProfile |
            InputField::ScrollPoints | InputField::ScrollMethod => {
                if matches!(value, HyprlandValue::String(_)) {
                    Ok(())
                } else {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: format!("Expected string, received {value:?}"),
                    })
                }
            },
            
            // Layout cannot be empty
            InputField::KbLayout => match value {
                HyprlandValue::String(raw) => {
                    if raw.trim().is_empty() {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Layout value cannot be empty".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected string, received {other:?}"),
                }),
            },
            
            // Bool fields
            InputField::NumlockByDefault | InputField::ResolveBindsBySym |
            InputField::ForceNoAccel | InputField::LeftHanded |
            InputField::ScrollButtonLock | InputField::NaturalScroll |
            InputField::MouseRefocus | InputField::SpecialFallthrough => {
                if matches!(value, HyprlandValue::Bool(_)) {
                    Ok(())
                } else {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: format!("Expected bool, received {value:?}"),
                    })
                }
            },
            
            // Int fields with range validation
            InputField::RepeatRate => match value {
                HyprlandValue::Int(v) if *v >= 1 && *v <= 100 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Repeat rate must be between 1 and 100, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::RepeatDelay => match value {
                HyprlandValue::Int(v) if *v >= 100 && *v <= 10000 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Repeat delay must be between 100 and 10000 ms, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::FollowMouse => match value {
                HyprlandValue::Int(v) if *v >= 0 && *v <= 3 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Follow mouse must be between 0 and 3, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::FocusOnClose => match value {
                HyprlandValue::Int(v) if *v >= 0 && *v <= 1 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Focus on close must be 0 or 1, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::FloatSwitchOverrideFocus => match value {
                HyprlandValue::Int(v) if *v >= 0 && *v <= 2 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Float switch override focus must be between 0 and 2, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::OffWindowAxisEvents => match value {
                HyprlandValue::Int(v) if *v >= 0 && *v <= 3 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Off window axis events must be between 0 and 3, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            InputField::EmulateDiscreteScroll => match value {
                HyprlandValue::Int(v) if *v >= 0 && *v <= 2 => Ok(()),
                HyprlandValue::Int(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Emulate discrete scroll must be between 0 and 2, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected int, received {other:?}"),
                }),
            },
            
            // Int fields without range validation
            InputField::ScrollButton => {
                if matches!(value, HyprlandValue::Int(_)) {
                    Ok(())
                } else {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: format!("Expected int, received {value:?}"),
                    })
                }
            },
            
            // Float fields with range validation
            InputField::Sensitivity => match value {
                HyprlandValue::Float(v) if *v >= -1.0 && *v <= 1.0 => Ok(()),
                HyprlandValue::Int(v) if *v >= -1 && *v <= 1 => Ok(()),
                HyprlandValue::Float(v) => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Sensitivity must be between -1.0 and 1.0, got {v}"),
                }),
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected float, received {other:?}"),
                }),
            },
            
            // Float fields without range validation
            InputField::ScrollFactor | InputField::FollowMouseThreshold => {
                if matches!(value, HyprlandValue::Float(_) | HyprlandValue::Int(_)) {
                    Ok(())
                } else {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: format!("Expected float, received {value:?}"),
                    })
                }
            },
        }
    }
}

pub fn input_field_registry() -> &'static [InputField] {
    const INPUT_FIELDS: [InputField; 28] = [
        // Keyboard settings
        InputField::KbModel,
        InputField::KbLayout,
        InputField::KbVariant,
        InputField::KbOptions,
        InputField::KbRules,
        InputField::KbFile,
        InputField::NumlockByDefault,
        InputField::ResolveBindsBySym,
        InputField::RepeatRate,
        InputField::RepeatDelay,
        
        // Mouse settings
        InputField::Sensitivity,
        InputField::AccelProfile,
        InputField::ForceNoAccel,
        InputField::LeftHanded,
        
        // Scroll settings
        InputField::ScrollPoints,
        InputField::ScrollMethod,
        InputField::ScrollButton,
        InputField::ScrollButtonLock,
        InputField::ScrollFactor,
        InputField::NaturalScroll,
        
        // Focus settings
        InputField::FollowMouse,
        InputField::FollowMouseThreshold,
        InputField::FocusOnClose,
        InputField::MouseRefocus,
        InputField::FloatSwitchOverrideFocus,
        InputField::SpecialFallthrough,
        
        // Misc settings
        InputField::OffWindowAxisEvents,
        InputField::EmulateDiscreteScroll,
    ];
    &INPUT_FIELDS
}

/// Canonical Hyprland configuration value representation used by the backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HyprlandValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
}

impl std::fmt::Display for HyprlandValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyprlandValue::Bool(value) => write!(f, "{}", if *value { "true" } else { "false" }),
            HyprlandValue::Int(value) => write!(f, "{}", value),
            HyprlandValue::Float(value) => write!(f, "{}", value),
            HyprlandValue::String(value) => write!(f, "{}", value),
        }
    }
}

impl From<bool> for HyprlandValue {
    fn from(value: bool) -> Self {
        HyprlandValue::Bool(value)
    }
}

impl From<i32> for HyprlandValue {
    fn from(value: i32) -> Self {
        HyprlandValue::Int(value)
    }
}

impl From<f32> for HyprlandValue {
    fn from(value: f32) -> Self {
        HyprlandValue::Float(value)
    }
}

impl From<String> for HyprlandValue {
    fn from(value: String) -> Self {
        HyprlandValue::String(value)
    }
}

impl From<&str> for HyprlandValue {
    fn from(value: &str) -> Self {
        HyprlandValue::String(value.to_string())
    }
}

fn set_bool(
    settings: &mut HyprlandGeneralSettings,
    setter: impl Fn(&mut HyprlandGeneralSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => {
            match v {
                0 => setter(settings, Some(false)),
                1 => setter(settings, Some(true)),
                other => {
                    return Err(HyprlandConfigError::Validation {
                        field: field.to_string(),
                        message: format!("Integer value '{other}' is not valid for boolean field"),
                    });
                },
            }
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_i32(
    settings: &mut HyprlandGeneralSettings,
    setter: impl Fn(&mut HyprlandGeneralSettings, Option<i32>),
    field: &str,
    range: RangeInclusive<i32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            if !range.contains(&v) {
                return Err(HyprlandConfigError::Validation {
                    field: field.to_string(),
                    message: format!("Value '{v}' is outside of allowed range"),
                });
            }
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

fn parse_bool(field: &str, raw: &str) -> Result<bool, HyprlandConfigError> {
    let normalized = raw.trim().to_lowercase();
    match normalized.as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(HyprlandConfigError::Parse {
            field: field.to_string(),
            message: format!("Unable to parse boolean value from '{other}'"),
        }),
    }
}

fn parse_i32(field: &str, raw: &str) -> Result<i32, HyprlandConfigError> {
    raw.trim()
        .parse::<i32>()
        .map_err(|err| HyprlandConfigError::Parse {
            field: field.to_string(),
            message: err.to_string(),
        })
}

fn parse_f32(field: &str, raw: &str) -> Result<f32, HyprlandConfigError> {
    raw.trim()
        .parse::<f32>()
        .map_err(|err| HyprlandConfigError::Parse {
            field: field.to_string(),
            message: err.to_string(),
        })
}

fn set_string(
    settings: &mut HyprlandGeneralSettings,
    setter: impl Fn(&mut HyprlandGeneralSettings, Option<String>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let assigned = match value {
        HyprlandValue::String(v) => v,
        HyprlandValue::Int(v) => v.to_string(),
        HyprlandValue::Float(v) => v.to_string(),
        HyprlandValue::Bool(v) => v.to_string(),
    };

    let trimmed = assigned.trim().to_string();
    if trimmed.is_empty() {
        return Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: "Value cannot be empty".into(),
        });
    }

    setter(settings, Some(trimmed));
    Ok(())
}

fn set_input_string(
    settings: &mut HyprlandInputSettings,
    setter: impl Fn(&mut HyprlandInputSettings, Option<String>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::String(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected string, received {other:?}"),
        }),
    }
}

fn set_input_bool(
    settings: &mut HyprlandInputSettings,
    setter: impl Fn(&mut HyprlandInputSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Expected bool or 0/1, received integer {other}"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected bool, received {other:?}"),
        }),
    }
}

fn set_input_int(
    settings: &mut HyprlandInputSettings,
    setter: impl Fn(&mut HyprlandInputSettings, Option<i32>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

fn set_input_float(
    settings: &mut HyprlandInputSettings,
    setter: impl Fn(&mut HyprlandInputSettings, Option<f32>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Float(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => {
            setter(settings, Some(v as f32));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected float, received {other:?}"),
        }),
    }
}

fn set_decoration_bool(
    settings: &mut HyprlandDecorationSettings,
    setter: impl Fn(&mut HyprlandDecorationSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Integer value '{other}' is not valid for boolean field"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_decoration_i32(
    settings: &mut HyprlandDecorationSettings,
    setter: impl Fn(&mut HyprlandDecorationSettings, Option<i32>),
    field: &str,
    range: RangeInclusive<i32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            if !range.contains(&v) {
                return Err(HyprlandConfigError::Validation {
                    field: field.to_string(),
                    message: format!("Value '{v}' is outside of allowed range"),
                });
            }
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

fn set_decoration_f32(
    settings: &mut HyprlandDecorationSettings,
    setter: impl Fn(&mut HyprlandDecorationSettings, Option<f32>),
    field: &str,
    range: RangeInclusive<f32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let numeric = match value {
        HyprlandValue::Float(v) => v,
        HyprlandValue::Int(v) => v as f32,
        other => {
            return Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Expected float, received {other:?}"),
            });
        },
    };

    if !range.contains(&numeric) {
        return Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Value '{numeric}' is outside of allowed range"),
        });
    }

    setter(settings, Some(numeric));
    Ok(())
}

fn set_decoration_string(
    settings: &mut HyprlandDecorationSettings,
    setter: impl Fn(&mut HyprlandDecorationSettings, Option<String>),
    _field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let assigned = match value {
        HyprlandValue::String(v) => v,
        HyprlandValue::Int(v) => v.to_string(),
        HyprlandValue::Float(v) => v.to_string(),
        HyprlandValue::Bool(v) => v.to_string(),
    };

    setter(settings, Some(assigned.trim().to_string()));
    Ok(())
}

fn set_blur_bool(
    settings: &mut HyprlandDecorationBlurSettings,
    setter: impl Fn(&mut HyprlandDecorationBlurSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Integer value '{other}' is not valid for boolean field"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_blur_i32(
    settings: &mut HyprlandDecorationBlurSettings,
    setter: impl Fn(&mut HyprlandDecorationBlurSettings, Option<i32>),
    field: &str,
    range: RangeInclusive<i32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            if !range.contains(&v) {
                return Err(HyprlandConfigError::Validation {
                    field: field.to_string(),
                    message: format!("Value '{v}' is outside of allowed range"),
                });
            }
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

fn set_blur_f32(
    settings: &mut HyprlandDecorationBlurSettings,
    setter: impl Fn(&mut HyprlandDecorationBlurSettings, Option<f32>),
    field: &str,
    range: RangeInclusive<f32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let numeric = match value {
        HyprlandValue::Float(v) => v,
        HyprlandValue::Int(v) => v as f32,
        other => {
            return Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Expected float, received {other:?}"),
            });
        },
    };

    if !range.contains(&numeric) {
        return Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Value '{numeric}' is outside of allowed range"),
        });
    }

    setter(settings, Some(numeric));
    Ok(())
}

fn set_shadow_bool(
    settings: &mut HyprlandDecorationShadowSettings,
    setter: impl Fn(&mut HyprlandDecorationShadowSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Integer value '{other}' is not valid for boolean field"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_shadow_i32(
    settings: &mut HyprlandDecorationShadowSettings,
    setter: impl Fn(&mut HyprlandDecorationShadowSettings, Option<i32>),
    field: &str,
    range: RangeInclusive<i32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            if !range.contains(&v) {
                return Err(HyprlandConfigError::Validation {
                    field: field.to_string(),
                    message: format!("Value '{v}' is outside of allowed range"),
                });
            }
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

fn set_shadow_f32(
    settings: &mut HyprlandDecorationShadowSettings,
    setter: impl Fn(&mut HyprlandDecorationShadowSettings, Option<f32>),
    field: &str,
    range: RangeInclusive<f32>,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let numeric = match value {
        HyprlandValue::Float(v) => v,
        HyprlandValue::Int(v) => v as f32,
        other => {
            return Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Expected float, received {other:?}"),
            });
        },
    };

    if !range.contains(&numeric) {
        return Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Value '{numeric}' is outside of allowed range"),
        });
    }

    setter(settings, Some(numeric));
    Ok(())
}

fn set_shadow_string(
    settings: &mut HyprlandDecorationShadowSettings,
    setter: impl Fn(&mut HyprlandDecorationShadowSettings, Option<String>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    let assigned = match value {
        HyprlandValue::String(v) => v,
        other => {
            return Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Expected string, received {other:?}"),
            });
        },
    };

    let trimmed = assigned.trim().to_string();
    if trimmed.is_empty() {
        return Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: "Value cannot be empty".into(),
        });
    }

    setter(settings, Some(trimmed));
    Ok(())
}

fn set_animation_bool(
    settings: &mut HyprlandAnimationSettings,
    setter: impl Fn(&mut HyprlandAnimationSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Integer value '{other}' is not valid for boolean field"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_snap_bool(
    setter: impl Fn(&mut HyprlandGeneralSnapSettings, Option<bool>),
    field: &str,
    value: HyprlandValue,
    settings: &mut HyprlandGeneralSnapSettings,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Bool(v) => {
            setter(settings, Some(v));
            Ok(())
        },
        HyprlandValue::Int(v) => match v {
            0 => {
                setter(settings, Some(false));
                Ok(())
            },
            1 => {
                setter(settings, Some(true));
                Ok(())
            },
            other => Err(HyprlandConfigError::Validation {
                field: field.to_string(),
                message: format!("Integer value '{other}' is not valid for boolean field"),
            }),
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected boolean, received {other:?}"),
        }),
    }
}

fn set_snap_i32(
    setter: impl Fn(&mut HyprlandGeneralSnapSettings, Option<i32>),
    field: &str,
    range: RangeInclusive<i32>,
    value: HyprlandValue,
    settings: &mut HyprlandGeneralSnapSettings,
) -> Result<(), HyprlandConfigError> {
    match value {
        HyprlandValue::Int(v) => {
            if !range.contains(&v) {
                return Err(HyprlandConfigError::Validation {
                    field: field.to_string(),
                    message: format!("Value '{v}' is outside of allowed range"),
                });
            }
            setter(settings, Some(v));
            Ok(())
        },
        other => Err(HyprlandConfigError::Validation {
            field: field.to_string(),
            message: format!("Expected integer, received {other:?}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_mode_parse() {
        assert_eq!(LayoutMode::parse("master").unwrap(), LayoutMode::Master);
        assert_eq!(LayoutMode::parse("DWINdle").unwrap(), LayoutMode::Dwindle);
        assert!(LayoutMode::parse("spiral").is_err());
    }

    #[test]
    fn overrides_round_trip() {
        let mut settings = HyprlandGeneralSettings::default();
        let mut overrides = BTreeMap::new();
        overrides.insert("no_border_on_floating".into(), HyprlandValue::Bool(true));
        overrides.insert("layout".into(), HyprlandValue::String("master".into()));
        overrides.insert("extend_border_grab_area".into(), HyprlandValue::Int(20));

        settings
            .apply_overrides(&overrides, &BTreeMap::new())
            .unwrap();

        assert_eq!(settings.no_border_on_floating, Some(true));
        assert_eq!(settings.layout, Some(LayoutMode::Master));
        assert_eq!(settings.extend_border_grab_area, Some(20));

        let serialized = settings.to_override_maps();
        assert_eq!(
            serialized.general.get("layout"),
            Some(&HyprlandValue::String("master".into()))
        );
        assert_eq!(
            serialized.general.get("extend_border_grab_area"),
            Some(&HyprlandValue::Int(20))
        );
    }

    #[test]
    fn general_field_parsing() {
        let field = GeneralField::NoBorderOnFloating;
        assert_eq!(field.parse_raw("true").unwrap(), HyprlandValue::Bool(true));
        assert_eq!(field.parse_raw("0").unwrap(), HyprlandValue::Bool(false));

        let layout = GeneralField::Layout;
        assert_eq!(
            layout.parse_raw("dwindle").unwrap(),
            HyprlandValue::String("dwindle".into())
        );
        assert!(layout.parse_raw("spiral").is_err());

        let corner = GeneralField::ResizeCorner;
        assert_eq!(corner.parse_raw("4").unwrap(), HyprlandValue::Int(4));
        let invalid_corner = corner.parse_raw("7").unwrap();
        let mut general_settings = HyprlandGeneralSettings::default();
        assert!(corner.apply(invalid_corner, &mut general_settings).is_err());
    }

    #[test]
    fn snap_overrides_round_trip() {
        let mut settings = HyprlandGeneralSettings::default();
        let general = BTreeMap::new();
        let mut snap = BTreeMap::new();
        snap.insert("enabled".into(), HyprlandValue::Bool(true));
        snap.insert("window_gap".into(), HyprlandValue::Int(14));
        snap.insert("monitor_gap".into(), HyprlandValue::Int(8));
        snap.insert("border_overlap".into(), HyprlandValue::Bool(true));

        settings.apply_overrides(&general, &snap).unwrap();

        assert_eq!(settings.snap.enabled, Some(true));
        assert_eq!(settings.snap.window_gap, Some(14));
        assert_eq!(settings.snap.monitor_gap, Some(8));
        assert_eq!(settings.snap.border_overlap, Some(true));
        assert_eq!(settings.snap.respect_gaps, None);

        let serialized = settings.to_override_maps();
        assert_eq!(
            serialized.snap.get("enabled"),
            Some(&HyprlandValue::Bool(true))
        );
        assert_eq!(
            serialized.snap.get("window_gap"),
            Some(&HyprlandValue::Int(14))
        );
        assert_eq!(
            serialized.snap.get("monitor_gap"),
            Some(&HyprlandValue::Int(8))
        );
        assert_eq!(
            serialized.snap.get("border_overlap"),
            Some(&HyprlandValue::Bool(true))
        );
        assert!(serialized.snap.get("respect_gaps").is_none());
    }
}
