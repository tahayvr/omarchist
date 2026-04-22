use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt, h_flex, select::SelectEvent, v_flex};

use crate::shell::waybar_sh_commands::restart_waybar;
use crate::system::waybar::{
    CUSTOM_WAYBAR_PROFILE, UNKNOWN_MANAGED_PROFILE, apply_waybar_profile,
    current_live_waybar_profile, ensure_custom_waybar_profile, has_unknown_managed_live_waybar,
    is_read_only_waybar_profile, list_waybar_profiles,
};
use crate::ui::dialogs::create_waybar_profile_dialog::take_pending_profile_navigation;
use crate::ui::dialogs::manage_waybar_profile_dialogs::{
    ProfileManagementResult, take_pending_profile_management,
};
use crate::ui::menu::app_menu;
use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

const KEY_CONTEXT: &str = "StatusBar";
const HEADER_ITEM_COUNT: usize = 6;

fn apply_and_restart(profile_name: &str) {
    if let Err(e) = apply_waybar_profile(profile_name) {
        eprintln!("Failed to apply waybar profile \"{}\": {e}", profile_name);
    }
    if let Err(e) = restart_waybar() {
        eprintln!("Failed to restart waybar: {e}");
    }
}

fn current_status_bar_selection() -> (String, bool) {
    if let Err(e) = ensure_custom_waybar_profile() {
        eprintln!("Failed to ensure custom Waybar profile: {e}");
    }

    if let Some(profile_name) = current_live_waybar_profile() {
        let is_read_only = is_read_only_waybar_profile(&profile_name);
        return (profile_name, is_read_only);
    }

    if has_unknown_managed_live_waybar() {
        return (UNKNOWN_MANAGED_PROFILE.to_string(), true);
    }

    let mut profiles = list_waybar_profiles();
    profiles.sort();
    let initial_profile = profiles
        .first()
        .cloned()
        .unwrap_or_else(|| "omarchy-default".to_string());
    let is_read_only = is_read_only_waybar_profile(&initial_profile);
    (initial_profile, is_read_only)
}

pub struct StatusBarView {
    header: Entity<StatusBarHeader>,
    design_area: Entity<DesignArea>,
    _select_subscription: Subscription,
    pub focused_header_item: Option<usize>,
    pub focus_handle: FocusHandle,
}

impl StatusBarView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (initial_profile, is_read_only) = current_status_bar_selection();

        let header = cx.new(|cx| StatusBarHeader::new(window, cx));
        let design_area = cx.new(|cx| DesignArea::new(&initial_profile, is_read_only, window, cx));

        let select_entity = header.read(cx).select_entity();
        let design_area_ref = design_area.clone();
        let header_ref = header.clone();
        let subscription = cx.subscribe_in(
            &select_entity,
            window,
            move |_this, _select, event: &SelectEvent<Vec<SharedString>>, window, cx| {
                if let SelectEvent::Confirm(Some(_)) = event {
                    let profile_name = header_ref
                        .read(cx)
                        .selected_profile(cx)
                        .unwrap_or("omarchy-default")
                        .to_string();
                    let is_read_only = is_read_only_waybar_profile(&profile_name);
                    design_area_ref.update(cx, |area, cx| {
                        area.switch_profile(&profile_name, is_read_only, window, cx);
                    });
                }
            },
        );

        Self {
            header,
            design_area,
            _select_subscription: subscription,
            focused_header_item: None,
            focus_handle: cx.focus_handle(),
        }
    }

    pub fn cycle_next(&mut self, cx: &mut Context<Self>) {
        self.focused_header_item = Some(match self.focused_header_item {
            None => 0,
            Some(i) => (i + 1) % HEADER_ITEM_COUNT,
        });
        cx.notify();
    }

    pub fn cycle_prev(&mut self, cx: &mut Context<Self>) {
        self.focused_header_item = Some(match self.focused_header_item {
            None => HEADER_ITEM_COUNT - 1,
            Some(0) => HEADER_ITEM_COUNT - 1,
            Some(i) => i - 1,
        });
        cx.notify();
    }

    pub fn reset_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_header_item = None;
        cx.notify();
    }

    pub fn at_first_or_none(&self) -> bool {
        matches!(self.focused_header_item, None | Some(0))
    }

    pub fn header_entity(&self) -> Entity<StatusBarHeader> {
        self.header.clone()
    }
}

impl Render for StatusBarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused_header_item = self.focused_header_item;
        self.header.update(cx, |header, _| {
            header.focused_item = focused_header_item;
        });

        if let Some(new_profile) = take_pending_profile_navigation() {
            let is_read_only = is_read_only_waybar_profile(&new_profile);
            self.header.update(cx, |header, cx| {
                header.reload_and_select(&new_profile, window, cx);
            });
            self.design_area.update(cx, |area, cx| {
                area.switch_profile(&new_profile, is_read_only, window, cx);
            });
        }

        if let Some(result) = take_pending_profile_management() {
            let active_profile = match result {
                ProfileManagementResult::Renamed { new_name } => new_name,
                ProfileManagementResult::Duplicated { new_name } => new_name,
                ProfileManagementResult::Deleted { switch_to } => switch_to,
            };
            apply_and_restart(&active_profile);
            self.header.update(cx, |header, cx| {
                header.reload_and_select(&active_profile, window, cx);
            });
            self.design_area.update(cx, |area, cx| {
                area.switch_profile(
                    &active_profile,
                    is_read_only_waybar_profile(&active_profile),
                    window,
                    cx,
                );
            });
        }

        let selected_profile = self.header.read(cx).current_profile_name(cx);
        let is_custom_selected = selected_profile == CUSTOM_WAYBAR_PROFILE;
        let is_unknown_managed_selected = selected_profile == UNKNOWN_MANAGED_PROFILE;

        v_flex()
            .id("status-bar-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .gap_4()
            .on_action(cx.listener(|this, _: &app_menu::NextFocus, _window, cx| {
                this.cycle_next(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::PrevFocus, _window, cx| {
                this.cycle_prev(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::SelectNext, _window, cx| {
                this.cycle_next(cx);
            }))
            .on_action(cx.listener(|this, _: &app_menu::SelectPrev, _window, cx| {
                if !this.at_first_or_none() {
                    this.cycle_prev(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &app_menu::ActivateItem, window, cx| {
                match this.focused_header_item {
                    Some(0) => {
                        let header = this.header_entity();
                        let select = header.read(cx).select_entity();
                        let fh = {
                            use gpui::Focusable;
                            select.read(cx).focus_handle(cx).clone()
                        };
                        fh.focus(window);
                    }
                    Some(1) => {
                        crate::ui::dialogs::create_waybar_profile_dialog::open_create_waybar_profile_dialog(window, cx);
                    }
                    Some(2) => {
                        let profile = this.header.read(cx).current_profile_name(cx);
                        if !is_read_only_waybar_profile(&profile) {
                            crate::ui::dialogs::manage_waybar_profile_dialogs::open_rename_waybar_profile_dialog(profile, window, cx);
                        }
                    }
                    Some(3) => {
                        let profile = this.header.read(cx).current_profile_name(cx);
                        if !is_read_only_waybar_profile(&profile) {
                            crate::ui::dialogs::manage_waybar_profile_dialogs::open_duplicate_waybar_profile_dialog(profile, window, cx);
                        }
                    }
                    Some(4) => {
                        let can_delete = {
                            let header = this.header.read(cx);
                            header.profile_names.len() > 1
                                && !is_read_only_waybar_profile(&header.current_profile_name(cx))
                        };
                        if can_delete {
                            let profile = this.header.read(cx).current_profile_name(cx);
                            crate::ui::dialogs::manage_waybar_profile_dialogs::open_delete_waybar_profile_dialog(profile, window, cx);
                        }
                    }
                    Some(5) => {
                        let profile = this.header.read(cx).current_profile_name(cx);
                        apply_and_restart(&profile);
                    }
                    _ => {}
                }
            }))
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.reset_focus(cx);
            }))
            .child(self.header.clone())
            .when(is_custom_selected, |this| {
                this.child(render_custom_waybar_notice(cx))
            })
            .when(is_unknown_managed_selected, |this| {
                this.child(render_unknown_managed_notice(cx))
            })
            .child(self.design_area.clone())
    }
}

fn render_custom_waybar_notice(cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .w_full()
        .gap_2()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(theme.accent.opacity(0.5))
        .bg(theme.accent.opacity(0.06))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(IconName::Info).text_color(theme.accent))
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child("This is your config and currently can't be edited by Omarchist."),
                ),
        )
}

fn render_unknown_managed_notice(cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();

    v_flex()
        .w_full()
        .gap_2()
        .p_4()
        .rounded_md()
        .border_1()
        .border_color(theme.warning.opacity(0.6))
        .bg(theme.warning.opacity(0.08))
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(IconName::TriangleAlert).text_color(theme.warning))
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child("Omarchist is managing your current Waybar config, but it no longer matches any known profile."),
                ),
        )
}
