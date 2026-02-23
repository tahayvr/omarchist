use gpui::*;
use gpui_component::{select::SelectEvent, v_flex};

use crate::system::waybar::list_waybar_profiles;
use crate::ui::dialogs::create_waybar_profile_dialog::take_pending_profile_navigation;
use crate::ui::dialogs::manage_waybar_profile_dialogs::{
    ProfileManagementResult, take_pending_profile_management,
};
use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

pub struct StatusBarView {
    header: Entity<StatusBarHeader>,
    design_area: Entity<DesignArea>,
    _select_subscription: Subscription,
}

impl StatusBarView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut profiles = list_waybar_profiles();
        profiles.sort();
        let initial_profile = profiles
            .first()
            .cloned()
            .unwrap_or_else(|| "omarchy-default".to_string());

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
        }
    }
}

impl Render for StatusBarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // After a new profile is created, reload the header and switch to it
        if let Some(new_profile) = take_pending_profile_navigation() {
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
            self.header.update(cx, |header, cx| {
                header.reload_and_select(&active_profile, window, cx);
            });
            self.design_area.update(cx, |area, cx| {
                area.switch_profile(&active_profile, window, cx);
            });
        }

        v_flex()
            .id("status-bar-page")
            .size_full()
            .gap_4()
            .child(self.header.clone())
            .child(self.design_area.clone())
    }
}
