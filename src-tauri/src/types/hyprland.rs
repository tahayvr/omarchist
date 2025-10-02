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
    pub no_border_on_floating: Option<bool>,
    pub layout: Option<LayoutMode>,
    pub no_focus_fallback: Option<bool>,
    pub resize_on_border: Option<bool>,
    pub extend_border_grab_area: Option<i32>,
    pub hover_icon_on_border: Option<bool>,
    pub allow_tearing: Option<bool>,
    pub resize_corner: Option<i32>,
}

impl HyprlandGeneralSettings {
    /// Construct settings populated with Hyprland defaults.
    pub fn with_defaults() -> Self {
        Self {
            no_border_on_floating: Some(false),
            layout: Some(LayoutMode::Dwindle),
            no_focus_fallback: Some(false),
            resize_on_border: Some(false),
            extend_border_grab_area: Some(15),
            hover_icon_on_border: Some(true),
            allow_tearing: Some(false),
            resize_corner: Some(0),
        }
    }

    /// Merge a map of overrides (Hyprland key -> value) into the settings instance.
    pub fn apply_overrides(&mut self, overrides: &BTreeMap<String, HyprlandValue>) -> Result<(), HyprlandConfigError> {
        for field in general_field_registry() {
            if let Some(value) = overrides.get(field.key()) {
                field.apply(value.clone(), self)?;
            }
        }
        Ok(())
    }

    /// Convert the explicit overrides contained in this struct into Hyprland key/value pairs.
    pub fn to_override_map(&self) -> BTreeMap<String, HyprlandValue> {
        let mut map = BTreeMap::new();
        for field in general_field_registry() {
            if let Some(value) = field.extract(self) {
                map.insert(field.key().to_string(), value);
            }
        }
        map
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
        Self { effective, overrides }
    }
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
    NoBorderOnFloating,
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
            "no_border_on_floating" => Some(GeneralField::NoBorderOnFloating),
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
            GeneralField::NoBorderOnFloating => "no_border_on_floating",
            GeneralField::Layout => "layout",
            GeneralField::NoFocusFallback => "no_focus_fallback",
            GeneralField::ResizeOnBorder => "resize_on_border",
            GeneralField::ExtendBorderGrabArea => "extend_border_grab_area",
            GeneralField::HoverIconOnBorder => "hover_icon_on_border",
            GeneralField::AllowTearing => "allow_tearing",
            GeneralField::ResizeCorner => "resize_corner",
        }
    }

    pub fn apply(&self, value: HyprlandValue, settings: &mut HyprlandGeneralSettings) -> Result<(), HyprlandConfigError> {
        match self {
            GeneralField::NoBorderOnFloating => set_bool(settings, |s, v| s.no_border_on_floating = v, "no_border_on_floating", value),
            GeneralField::Layout => {
                let layout = match value {
                    HyprlandValue::String(v) => LayoutMode::parse(&v)?,
                    other => {
                        return Err(HyprlandConfigError::Validation {
                            field: "layout".to_string(),
                            message: format!("Expected string, received {other:?}"),
                        })
                    }
                };
                settings.layout = Some(layout);
                Ok(())
            }
            GeneralField::NoFocusFallback => set_bool(settings, |s, v| s.no_focus_fallback = v, "no_focus_fallback", value),
            GeneralField::ResizeOnBorder => set_bool(settings, |s, v| s.resize_on_border = v, "resize_on_border", value),
            GeneralField::ExtendBorderGrabArea => set_i32(settings, |s, v| s.extend_border_grab_area = v, "extend_border_grab_area", 0..=i32::MAX, value),
            GeneralField::HoverIconOnBorder => set_bool(settings, |s, v| s.hover_icon_on_border = v, "hover_icon_on_border", value),
            GeneralField::AllowTearing => set_bool(settings, |s, v| s.allow_tearing = v, "allow_tearing", value),
            GeneralField::ResizeCorner => set_i32(settings, |s, v| s.resize_corner = v, "resize_corner", 0..=4, value),
        }
    }

    pub fn parse_raw(&self, raw: &str) -> Result<HyprlandValue, HyprlandConfigError> {
        match self {
            GeneralField::Layout => {
                let value = raw.trim();
                LayoutMode::parse(value)?;
                Ok(HyprlandValue::String(value.to_string()))
            }
            GeneralField::ExtendBorderGrabArea => {
                let int_value = parse_i32(self.key(), raw)?;
                if int_value < 0 {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be >= 0".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(int_value))
                }
            }
            GeneralField::ResizeCorner => {
                let int_value = parse_i32(self.key(), raw)?;
                if !(0..=4).contains(&int_value) {
                    Err(HyprlandConfigError::Validation {
                        field: self.key().to_string(),
                        message: "Value must be between 0 and 4".into(),
                    })
                } else {
                    Ok(HyprlandValue::Int(int_value))
                }
            }
            _ => {
                let bool_value = parse_bool(self.key(), raw)?;
                Ok(HyprlandValue::Bool(bool_value))
            }
        }
    }

    pub fn extract(&self, settings: &HyprlandGeneralSettings) -> Option<HyprlandValue> {
        match self {
            GeneralField::NoBorderOnFloating => settings.no_border_on_floating.map(HyprlandValue::from),
            GeneralField::Layout => settings.layout.map(HyprlandValue::from),
            GeneralField::NoFocusFallback => settings.no_focus_fallback.map(HyprlandValue::from),
            GeneralField::ResizeOnBorder => settings.resize_on_border.map(HyprlandValue::from),
            GeneralField::ExtendBorderGrabArea => settings.extend_border_grab_area.map(HyprlandValue::from),
            GeneralField::HoverIconOnBorder => settings.hover_icon_on_border.map(HyprlandValue::from),
            GeneralField::AllowTearing => settings.allow_tearing.map(HyprlandValue::from),
            GeneralField::ResizeCorner => settings.resize_corner.map(HyprlandValue::from),
        }
    }

    pub fn default_value(&self) -> HyprlandValue {
        match self {
            GeneralField::NoBorderOnFloating => HyprlandValue::Bool(false),
            GeneralField::Layout => HyprlandValue::from(LayoutMode::Dwindle),
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
            GeneralField::Layout => match value {
                HyprlandValue::String(v) => {
                    LayoutMode::parse(v)?;
                    Ok(())
                }
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected string, received {other:?}"),
                }),
            },
            GeneralField::ExtendBorderGrabArea => match value {
                HyprlandValue::Int(v) => {
                    if *v < 0 {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be >= 0".into(),
                        })
                    } else {
                        Ok(())
                    }
                }
                other => Err(HyprlandConfigError::Validation {
                    field: self.key().to_string(),
                    message: format!("Expected integer, received {other:?}"),
                }),
            },
            GeneralField::ResizeCorner => match value {
                HyprlandValue::Int(v) => {
                    if !matches!(*v, 0..=4) {
                        Err(HyprlandConfigError::Validation {
                            field: self.key().to_string(),
                            message: "Value must be between 0 and 4".into(),
                        })
                    } else {
                        Ok(())
                    }
                }
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
    const GENERAL_FIELDS: [GeneralField; 8] = [
        GeneralField::NoBorderOnFloating,
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
        }
        HyprlandValue::Int(v) => {
            match v {
                0 => setter(settings, Some(false)),
                1 => setter(settings, Some(true)),
                other => {
                    return Err(HyprlandConfigError::Validation {
                        field: field.to_string(),
                        message: format!("Integer value '{other}' is not valid for boolean field"),
                    });
                }
            }
            Ok(())
        }
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
        }
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
    raw
        .trim()
        .parse::<i32>()
        .map_err(|err| HyprlandConfigError::Parse {
            field: field.to_string(),
            message: err.to_string(),
        })
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
        overrides.insert(
            "no_border_on_floating".into(),
            HyprlandValue::Bool(true),
        );
        overrides.insert("layout".into(), HyprlandValue::String("master".into()));
        overrides.insert("extend_border_grab_area".into(), HyprlandValue::Int(20));

        settings.apply_overrides(&overrides).unwrap();

        assert_eq!(settings.no_border_on_floating, Some(true));
        assert_eq!(settings.layout, Some(LayoutMode::Master));
        assert_eq!(settings.extend_border_grab_area, Some(20));

        let serialized = settings.to_override_map();
        assert_eq!(serialized.get("layout"), Some(&HyprlandValue::String("master".into())));
        assert_eq!(serialized.get("extend_border_grab_area"), Some(&HyprlandValue::Int(20)));
    }

    #[test]
    fn general_field_parsing() {
        let field = GeneralField::NoBorderOnFloating;
        assert_eq!(field.parse_raw("true").unwrap(), HyprlandValue::Bool(true));
        assert_eq!(field.parse_raw("0").unwrap(), HyprlandValue::Bool(false));

        let layout = GeneralField::Layout;
        assert_eq!(layout.parse_raw("dwindle").unwrap(), HyprlandValue::String("dwindle".into()));
        assert!(layout.parse_raw("spiral").is_err());

        let corner = GeneralField::ResizeCorner;
        assert_eq!(corner.parse_raw("4").unwrap(), HyprlandValue::Int(4));
        assert!(corner.parse_raw("7").is_err());
    }
}
