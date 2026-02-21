use gpui::*;
use gpui_component::{select::SelectEvent, v_flex};

use crate::system::waybar::list_waybar_profiles;
use crate::ui::status_bar_page::design_area::DesignArea;
use crate::ui::status_bar_page::header::StatusBarHeader;

pub struct StatusBarView {
    header: Entity<StatusBarHeader>,
    design_area: Entity<DesignArea>,
    _select_subscription: Subscription,
}

impl StatusBarView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Determine initial profile (first one on disk, else fallback)
        let mut profiles = list_waybar_profiles();
        profiles.sort();
        let initial_profile = profiles
            .first()
            .cloned()
            .unwrap_or_else(|| "omarchy-default".to_string());

        let header = cx.new(|cx| StatusBarHeader::new(window, cx));
        let design_area = cx.new(|cx| DesignArea::new(&initial_profile, cx));

        // Subscribe to profile selection changes
        let select_entity = header.read(cx).select_entity();
        let design_area_ref = design_area.clone();
        let header_ref = header.clone();
        let subscription = cx.subscribe(
            &select_entity,
            move |_this, _select, event: &SelectEvent<Vec<SharedString>>, cx| {
                if let SelectEvent::Confirm(Some(_)) = event {
                    // Read the newly selected profile name from the header
                    let profile_name = header_ref
                        .read(cx)
                        .selected_profile(cx)
                        .unwrap_or("omarchy-default")
                        .to_string();
                    design_area_ref.update(cx, |area, cx| {
                        area.switch_profile(&profile_name, cx);
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("status-bar-page")
            .size_full()
            .gap_4()
            .child(self.header.clone())
            .child(self.design_area.clone())
    }
}
