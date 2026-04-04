use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, IndexPath, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex,
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
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
    // Raw profile names in the same order as the select items
    pub profile_names: Vec<String>,
    /// Which header item currently has keyboard focus (set by parent StatusBarView)
    /// 0 = Profile Select, 1 = Add Profile, 2 = More Options, 3 = Restart Waybar
    pub focused_item: Option<usize>,
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
            focused_item: None,
        }
    }

    // updates the existing entity in-place — never replaces it
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

    // Returns the underlying select entity so callers can subscribe to changes.
    pub fn select_entity(&self) -> Entity<SelectState<Vec<SharedString>>> {
        self.profile_select.clone()
    }
}

impl Render for StatusBarHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let ring = theme.ring;
        let focused = self.focused_item;

        let current_profile = self
            .profile_select
            .read(cx)
            .selected_value()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let profile_for_rename = current_profile.clone();
        let profile_for_duplicate = current_profile.clone();
        let profile_for_delete = current_profile.clone();
        let profile_for_restart = current_profile.clone();

        // Determine if deleting is allowed (need > 1 profile)
        let can_delete = self.profile_names.len() > 1;
        let can_restore = has_original_waybar_backup();

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
                            .rounded_md()
                            .when(focused == Some(0), move |this: gpui::Div| {
                                this.border_2().border_color(ring)
                            })
                            .child(Select::new(&self.profile_select).small()),
                    )
                    .child(
                        div()
                            .rounded_md()
                            .when(focused == Some(1), move |this: gpui::Div| {
                                this.border_2().border_color(ring)
                            })
                            .child(
                                Button::new("add-profile")
                                    .icon(Icon::new(IconName::Plus))
                                    .ghost()
                                    .small()
                                    .tooltip("Add profile")
                                    .on_click(|_, window, cx| {
                                        open_create_waybar_profile_dialog(window, cx);
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .rounded_md()
                            .when(focused == Some(2), move |this: gpui::Div| {
                                this.border_2().border_color(ring)
                            })
                            .child(
                        Button::new("more-profile-options-btn")
                            .icon(Icon::new(IconName::Ellipsis))
                            .ghost()
                            .small()
                            .tooltip("More options")
                            .dropdown_menu(move |menu: PopupMenu, _, _| {
                                let p_rename = profile_for_rename.clone();
                                let p_dup = profile_for_duplicate.clone();
                                let p_del = profile_for_delete.clone();

                                let menu = menu
                                    .item(PopupMenuItem::new("Rename profile").on_click(
                                        move |_, window, cx| {
                                            open_rename_waybar_profile_dialog(
                                                p_rename.clone(),
                                                window,
                                                cx,
                                            );
                                        },
                                    ))
                                    .item(PopupMenuItem::new("Duplicate profile").on_click(
                                        move |_, window, cx| {
                                            open_duplicate_waybar_profile_dialog(
                                                p_dup.clone(),
                                                window,
                                                cx,
                                            );
                                        },
                                    ))
                                    .separator();

                                let menu = if can_delete {
                                    menu.item(PopupMenuItem::new("Delete profile").on_click(
                                        move |_, window, cx| {
                                            open_delete_waybar_profile_dialog(
                                                p_del.clone(),
                                                window,
                                                cx,
                                            );
                                        },
                                    ))
                                } else {
                                    menu.item(PopupMenuItem::new("Delete profile").disabled(true))
                                };

                                let menu = menu.separator();

                                if can_restore {
                                    menu.item(
                                        PopupMenuItem::new("Restore original config").on_click(
                                            move |_, _, _| {
                                                if let Err(e) = restore_original_waybar_config() {
                                                    eprintln!(
                                                        "Failed to restore original waybar config: {e}"
                                                    );
                                                }
                                                if let Err(e) = restart_waybar() {
                                                    eprintln!("Failed to restart waybar: {e}");
                                                }
                                            },
                                        ),
                                    )
                                } else {
                                    menu.item(
                                        PopupMenuItem::new("Restore original config")
                                            .disabled(true),
                                    )
                                }
                            }),
                    ),
            ),
        )
        .child(
            div()
                .rounded_md()
                .when(focused == Some(3), move |this: gpui::Div| {
                    this.border_2().border_color(ring)
                })
                .child(
                    Button::new("refresh-status-bar")
                        .icon(Icon::new(IconName::LoaderCircle))
                        .ghost()
                        .small()
                        .tooltip("Restart Waybar")
                        .on_click(move |_, _, _| {
                            if let Err(e) = crate::system::waybar::apply_waybar_profile(&profile_for_restart) {
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
