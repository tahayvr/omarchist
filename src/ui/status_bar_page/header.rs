use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, IndexPath, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{Select, SelectState},
};

use crate::shell::waybar_sh_commands::restart_waybar;
use crate::system::waybar::{
    has_original_waybar_backup, list_waybar_profiles, restore_original_waybar_config,
};
use crate::ui::dialogs::create_waybar_profile_dialog::open_create_waybar_profile_dialog;
use crate::ui::dialogs::manage_waybar_profile_dialogs::{
    open_delete_waybar_profile_dialog, open_duplicate_waybar_profile_dialog,
    open_rename_waybar_profile_dialog,
};

pub struct StatusBarHeader {
    profile_select: Entity<SelectState<Vec<SharedString>>>,
    /// Raw profile names in the same order as the select items.
    pub profile_names: Vec<String>,
    /// Which header item has keyboard focus (None = no focus).
    /// Items: 0=select, 1=add, 2=rename, 3=duplicate, 4=delete, 5=restore, 6=restart
    pub focused_item: Option<usize>,
}

impl StatusBarHeader {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut profile_names = list_waybar_profiles();
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
            focused_item: None,
        }
    }

    /// Updates the select in-place after a profile is created/renamed/deleted.
    pub fn reload_and_select(
        &mut self,
        profile_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut profile_names = list_waybar_profiles();
        profile_names.sort();
        if profile_names.is_empty() {
            profile_names.push("omarchy-default".to_string());
        }

        let target = profile_name.to_string();
        let select_index = profile_names.iter().position(|n| n == &target).unwrap_or(0);

        let items: Vec<SharedString> = profile_names
            .iter()
            .map(|n| SharedString::from(n.clone()))
            .collect();

        self.profile_names = profile_names;

        let selected = items[select_index].clone();
        self.profile_select.update(cx, |state, cx| {
            state.set_items(items, window, cx);
            state.set_selected_value(&selected, window, cx);
        });
    }

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

    /// Returns the currently selected profile name.
    pub fn current_profile_name(&self, cx: &App) -> String {
        self.profile_select
            .read(cx)
            .selected_value()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}

impl Render for StatusBarHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let ring = theme.ring;
        let focused = self.focused_item;

        let current_profile = self.current_profile_name(cx);

        let profile_for_rename = current_profile.clone();
        let profile_for_duplicate = current_profile.clone();
        let profile_for_delete = current_profile.clone();
        let profile_for_restart = current_profile.clone();

        let can_delete = self.profile_names.len() > 1;
        let can_restore = has_original_waybar_backup();

        let focus_ring = move |idx: usize| {
            move |this: gpui::Div| {
                if focused == Some(idx) {
                    this.border_2().border_color(ring)
                } else {
                    this
                }
            }
        };

        h_flex()
            .w_full()
            .p_4()
            .gap_2()
            .items_center()
            .justify_between()
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .flex_wrap()
                    // Profile select
                    .child(
                        div()
                            .w(px(200.))
                            .rounded_md()
                            .map(focus_ring(0))
                            .child(Select::new(&self.profile_select).small()),
                    )
                    // Add Profile
                    .child(
                        div().rounded_md().map(focus_ring(1)).child(
                            Button::new("add-profile")
                                .icon(Icon::new(IconName::Plus))
                                .ghost()
                                .small()
                                .tooltip("Add Profile")
                                .on_click(|_, window, cx| {
                                    open_create_waybar_profile_dialog(window, cx);
                                }),
                        ),
                    )
                    // Rename profile
                    .child(
                        div().rounded_md().map(focus_ring(2)).child(
                            Button::new("rename-profile")
                                .icon(Icon::new(IconName::Replace))
                                .ghost()
                                .small()
                                .tooltip("Rename Profile")
                                .on_click(move |_, window, cx| {
                                    open_rename_waybar_profile_dialog(
                                        profile_for_rename.clone(),
                                        window,
                                        cx,
                                    );
                                }),
                        ),
                    )
                    // Duplicate profile
                    .child(
                        div().rounded_md().map(focus_ring(3)).child(
                            Button::new("duplicate-profile")
                                .icon(Icon::new(IconName::Copy))
                                .ghost()
                                .small()
                                .tooltip("Duplicate Profile")
                                .on_click(move |_, window, cx| {
                                    open_duplicate_waybar_profile_dialog(
                                        profile_for_duplicate.clone(),
                                        window,
                                        cx,
                                    );
                                }),
                        ),
                    )
                    // Delete profile (disabled when only 1 remains)
                    .child(
                        div().rounded_md().map(focus_ring(4)).child(
                            Button::new("delete-profile")
                                .icon(Icon::new(IconName::Delete))
                                .ghost()
                                .small()
                                .tooltip("Delete Profile")
                                .disabled(!can_delete)
                                .on_click(move |_, window, cx| {
                                    if can_delete {
                                        open_delete_waybar_profile_dialog(
                                            profile_for_delete.clone(),
                                            window,
                                            cx,
                                        );
                                    }
                                }),
                        ),
                    )
                    // Restore original config (disabled when no backup)
                    .child(
                        div().rounded_md().map(focus_ring(5)).child(
                            Button::new("restore-config")
                                .icon(Icon::new(IconName::Undo2))
                                .ghost()
                                .small()
                                .tooltip("Restore Original Config")
                                .disabled(!can_restore)
                                .on_click(move |_, _, _| {
                                    if can_restore {
                                        if let Err(e) = restore_original_waybar_config() {
                                            eprintln!(
                                                "Failed to restore original waybar config: {e}"
                                            );
                                        }
                                        if let Err(e) = restart_waybar() {
                                            eprintln!("Failed to restart waybar: {e}");
                                        }
                                    }
                                }),
                        ),
                    ),
            )
            // Restart waybar
            .child(
                div().rounded_md().map(focus_ring(6)).child(
                    Button::new("refresh-status-bar")
                        .icon(Icon::new(IconName::LoaderCircle))
                        .ghost()
                        .small()
                        .tooltip("Restart Waybar")
                        .on_click(move |_, _, _| {
                            if let Err(e) =
                                crate::system::waybar::apply_waybar_profile(&profile_for_restart)
                            {
                                eprintln!("Failed to apply waybar profile: {e}");
                            }
                            if let Err(e) = restart_waybar() {
                                eprintln!("Failed to restart waybar: {e}");
                            }
                        }),
                ),
            )
    }
}
