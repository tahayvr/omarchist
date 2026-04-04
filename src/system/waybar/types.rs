use serde::{Deserialize, Serialize};

// All fields are `Option` so that absent keys remain absent on save.
#[derive(Debug, Clone, Default)]
pub struct BarSettings {
    pub position: Option<String>,
    pub height: Option<u32>,
    pub layer: Option<String>,
    pub spacing: Option<u32>,
    pub exclusive: Option<bool>,
    pub passthrough: Option<bool>,
    pub output: Option<String>,
    pub margin_top: Option<i32>,
    pub margin_right: Option<i32>,
    pub margin_bottom: Option<i32>,
    pub margin_left: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaybarZone {
    Left,
    Center,
    Right,
}

/// A single module entry within a Waybar zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaybarModule {
    pub key: String,
    pub label: String,
    /// Icon char extracted from the module's format/format-icons fields.
    pub icon: String,
    pub zone: WaybarZone,
}

#[derive(Debug, Clone)]
pub struct WaybarConfig {
    pub profile_name: String,
    pub modules_left: Vec<WaybarModule>,
    pub modules_center: Vec<WaybarModule>,
    pub modules_right: Vec<WaybarModule>,
}

impl WaybarConfig {
    pub fn all_modules(&self) -> Vec<&WaybarModule> {
        self.modules_left
            .iter()
            .chain(self.modules_center.iter())
            .chain(self.modules_right.iter())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct LibraryModule {
    pub key: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub icon: &'static str,
    pub default_config: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_modules_concatenates_all_zones() {
        let make_module = |key: &str, zone: WaybarZone| WaybarModule {
            key: key.to_string(),
            label: key.to_string(),
            icon: String::new(),
            zone,
        };
        let config = WaybarConfig {
            profile_name: "test".to_string(),
            modules_left: vec![make_module("clock", WaybarZone::Left)],
            modules_center: vec![make_module("cpu", WaybarZone::Center)],
            modules_right: vec![
                make_module("tray", WaybarZone::Right),
                make_module("memory", WaybarZone::Right),
            ],
        };
        let all = config.all_modules();
        assert_eq!(all.len(), 4, "all_modules should return all four modules");
        assert_eq!(all[0].key, "clock");
        assert_eq!(all[1].key, "cpu");
        assert_eq!(all[2].key, "tray");
        assert_eq!(all[3].key, "memory");
    }

    #[test]
    fn all_modules_empty_config_returns_empty_vec() {
        let config = WaybarConfig {
            profile_name: "empty".to_string(),
            modules_left: vec![],
            modules_center: vec![],
            modules_right: vec![],
        };
        assert!(
            config.all_modules().is_empty(),
            "all_modules on an empty config should return an empty vec"
        );
    }
}
