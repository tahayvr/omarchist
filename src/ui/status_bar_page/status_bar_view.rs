use gpui::*;
use gpui_component::{select::SelectEvent, v_flex};

use crate::shell::waybar_sh_commands::restart_waybar;
use crate::system::waybar::{apply_waybar_profile, list_waybar_profiles};
use crate::ui::dialogs::create_waybar_profile_dialog::take_pending_profile_navigation;
use crate::ui::dialogs::manage_waybar_profile_dialogs::{
    ProfileManagementResult, take_pending_profile_management,
};
use crate::ui::menu::app_menu;
use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

const KEY_CONTEXT: &str = "StatusBar";
const HEADER_ITEM_COUNT: usize = 4;

fn apply_and_restart(profile_name: &str) {
    if let Err(e) = apply_waybar_profile(profile_name) {
        eprintln!("Failed to apply waybar profile \"{}\": {e}", profile_name);
    }
    if let Err(e) = restart_waybar() {
        eprintln!("Failed to restart waybar: {e}");
    }
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
        let mut profiles = list_waybar_profiles();
        profiles.sort();
        let initial_profile = profiles
            .first()
            .cloned()
            .unwrap_or_else(|| "omarchy-default".to_string());

        // is always in sync when the page is first opened.
        apply_and_restart(&initial_profile);

        let header = cx.new(|cx| StatusBarHeader::new(window, cx));
        let design_area = cx.new(|cx| DesignArea::new(&initial_profile, window, cx));

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
                    apply_and_restart(&profile_name);
                    design_area_ref.update(cx, |area, cx| {
                        area.switch_profile(&profile_name, window, cx);
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

    pub fn enter_focus(&mut self, cx: &mut Context<Self>) {
        self.focused_header_item = Some(0);
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
        // Sync keyboard focus state to the header for visual rendering
        let focused_header_item = self.focused_header_item;
        self.header.update(cx, |header, _| {
            header.focused_item = focused_header_item;
        });

        // After a new profile is created, reload the header and switch to it
        if let Some(new_profile) = take_pending_profile_navigation() {
            apply_and_restart(&new_profile);
            self.header.update(cx, |header, cx| {
                header.reload_and_select(&new_profile, window, cx);
            });
            self.design_area.update(cx, |area, cx| {
                area.switch_profile(&new_profile, window, cx);
            });
        }

        // After rename / duplicate / delete, update header and design area
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
                area.switch_profile(&active_profile, window, cx);
            });
        }

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
                // Only consume if not at first item — otherwise bubble to MainWindow
                // to move focus back to the sidebar
                if !this.at_first_or_none() {
                    this.cycle_prev(cx);
                }
            }))
            .on_action(cx.listener(|this, _: &app_menu::ActivateItem, window, cx| {
                match this.focused_header_item {
                    Some(0) => {
                        // Focus the profile Select widget so it handles keyboard navigation
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
                    Some(3) => {
                        if let Err(e) = crate::shell::waybar_sh_commands::restart_waybar() {
                            eprintln!("Failed to restart waybar: {e}");
                        }
                    }
                    _ => {}
                }
            }))
            .on_action(cx.listener(|this, _: &app_menu::EscapeFocus, _window, cx| {
                this.reset_focus(cx);
            }))
            .child(self.header.clone())
            .child(self.design_area.clone())
    }
}
