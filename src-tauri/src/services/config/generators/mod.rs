use serde_json::Value;

pub mod alacritty;
pub mod btop;
pub mod chromium;
pub mod ghostty;
pub mod hyprland;
pub mod hyprlock;
pub mod icons;
pub mod kitty;
pub mod mako;
pub mod neovim;
pub mod swayosd;
pub mod vscode;
pub mod walker;
pub mod waybar;

pub trait ConfigGenerator: Send + Sync {
    fn get_app_name(&self) -> &'static str;
    fn get_file_name(&self) -> &'static str;
    fn generate_config(&self, theme_data: &Value) -> Result<String, String>;
    fn get_config_schema(&self) -> Value;
    fn parse_existing_config(&self, content: &str) -> Result<Value, String>;
}

pub struct ConfigGeneratorRegistry {
    generators: std::collections::HashMap<String, Box<dyn ConfigGenerator>>,
}

impl Default for ConfigGeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigGeneratorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            generators: std::collections::HashMap::new(),
        };

        // Register all generators
        registry.register(Box::new(alacritty::AlacrittyGenerator));
        registry.register(Box::new(waybar::WaybarGenerator));
        registry.register(Box::new(btop::BtopGenerator));
        registry.register(Box::new(chromium::ChromiumGenerator));
        registry.register(Box::new(ghostty::GhosttyGenerator));
        registry.register(Box::new(hyprland::HyprlandGenerator));
        registry.register(Box::new(hyprlock::HyprlockGenerator));
        registry.register(Box::new(kitty::KittyGenerator));
        registry.register(Box::new(mako::MakoGenerator));
        registry.register(Box::new(swayosd::SwayosdGenerator));
        registry.register(Box::new(vscode::VscodeGenerator));
        registry.register(Box::new(walker::WalkerGenerator));
        registry.register(Box::new(neovim::NeovimGenerator));
        registry.register(Box::new(icons::IconsGenerator));

        registry
    }

    pub fn register(&mut self, generator: Box<dyn ConfigGenerator>) {
        self.generators
            .insert(generator.get_app_name().to_string(), generator);
    }

    pub fn get_generator(&self, app_name: &str) -> Option<&dyn ConfigGenerator> {
        self.generators.get(app_name).map(|boxed| boxed.as_ref())
    }

    pub fn get_all_apps(&self) -> Vec<&str> {
        self.generators.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_schema_for_app(&self, app_name: &str) -> Option<Value> {
        self.get_generator(app_name).map(|g| g.get_config_schema())
    }
}
