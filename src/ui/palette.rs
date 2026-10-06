//! The colours the app's areas wear, all taken from the theme's base
//! palette so they follow the look: the app's own palette under Light and
//! Dark (`ui_themes/theme.json`), the running theme's `colors.toml` under
//! Omarchy (`system::ui_theme_watcher`). A thing that belongs to an
//! area is drawn in that area's colour wherever it appears: its sidebar
//! entry, a section's icon, a badge. Flow steps have colours of their own
//! by group (`flows_page::step_types::StepGroup::accent`), drawn the same
//! way through [`tile`].
use gpui::*;
use gpui_component::{ActiveTheme, Icon};

use crate::ui::config_page::pages::PageGroup;

/// A part of the app with a colour of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Themes,
    Configuration,
    Keybinds,
    Flows,
    Settings,
    /// Omarchy itself: its settings, its updates, its bar.
    Omarchy,
}

impl Area {
    pub fn accent(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            Area::Themes => theme.magenta,
            Area::Configuration => theme.blue,
            Area::Keybinds => theme.yellow,
            Area::Flows => theme.green,
            Area::Settings => theme.cyan,
            Area::Omarchy => theme.red,
        }
    }

    /// The area a Configuration page belongs to: Hyprland's pages are the
    /// Configuration area's, Omarchy's are Omarchy's.
    pub fn of_group(group: PageGroup) -> Self {
        match group {
            PageGroup::Hyprland => Area::Configuration,
            PageGroup::Omarchy => Area::Omarchy,
        }
    }
}

/// A Lucide icon under `assets/icons/`.
pub fn icon(path: &'static str) -> Icon {
    Icon::new(Icon::empty()).path(path)
}

/// An icon on a square tinted with an accent: the one shape every coloured
/// icon in the app takes, from a flow step to a settings section.
pub fn tile(icon: Icon, accent: Hsla, size: Pixels, cx: &App) -> Div {
    div()
        .size(size)
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(cx.theme().radius)
        .bg(accent.opacity(0.16))
        .text_color(accent)
        .child(icon.size(size * 0.58))
}

/// A small dot in an accent, for a heading that groups things of that
/// colour.
pub fn dot(accent: Hsla) -> Div {
    div().size_2().flex_shrink_0().rounded_full().bg(accent)
}
