use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{Select, SelectState},
    ActiveTheme, Icon, IconName, IndexPath, Sizable,
};

use crate::system::waybar::list_waybar_profiles;

#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct ReloadStatusBar;

pub struct StatusBarHeader {
    profile_select: Entity<SelectState<Vec<SharedString>>>,
    /// Raw profile names in the same order as the select items
    pub profile_names: Vec<String>,
}

impl StatusBarHeader {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut profile_names = list_waybar_profiles();
        // Sort for stable ordering; ensure at least one fallback entry
        profile_names.sort();
        if profile_names.is_empty() {
            profile_names.push("omarchy-default".to_string());
        }

        let items: Vec<SharedString> = profile_names
            .iter()
            .map(|n| SharedString::from(n.clone()))
            .collect();

        let profile_select =
            cx.new(|cx| SelectState::new(items, Some(IndexPath::default()), window, cx));

        Self {
            profile_select,
            profile_names,
        }
    }

    /// Returns the currently selected profile name, if any.
    pub fn selected_profile<'a>(&'a self, cx: &'a App) -> Option<&'a str> {
        self.profile_select
            .read(cx)
            .selected_value()
            .map(|s| s.as_ref())
    }

    /// Returns the underlying select entity so callers can subscribe to changes.
    pub fn select_entity(&self) -> Entity<SelectState<Vec<SharedString>>> {
        self.profile_select.clone()
    }
}

impl Render for StatusBarHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .w_full()
            .p_4()
            .gap_4()
            .items_center()
            .justify_between()
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(200.))
                            .child(Select::new(&self.profile_select).small()),
                    )
                    .child(
                        Button::new("add-profile")
                            .icon(Icon::new(IconName::Plus))
                            .ghost()
                            .small()
                            .tooltip("Add profile")
                            .on_click(|_, _, _| {}),
                    ),
            )
            .child(
                Button::new("refresh-status-bar")
                    .icon(Icon::new(IconName::LoaderCircle))
                    .ghost()
                    .small()
                    .tooltip("Refresh status bar")
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(ReloadStatusBar), cx);
                    }),
            )
    }
}
