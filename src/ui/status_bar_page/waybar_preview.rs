use gpui::*;
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::system::waybar::{
    load_waybar_config, save_waybar_config, WaybarConfig, WaybarModule, WaybarZone,
};
use crate::ui::status_bar_page::waybar_item::{render_module_chip, DragWaybarModule};

pub struct WaybarPreview {
    pub profile_name: String,
    pub config: Option<WaybarConfig>,
}

impl WaybarPreview {
    pub fn new(profile_name: &str) -> Self {
        let config = load_waybar_config(profile_name);
        Self {
            profile_name: profile_name.to_string(),
            config,
        }
    }

    pub fn reload(&mut self, profile_name: &str) {
        self.profile_name = profile_name.to_string();
        self.config = load_waybar_config(profile_name);
    }

    /// Remove the module at (zone, index) from the config and persist immediately.
    pub fn remove_module(&mut self, zone: &WaybarZone, index: usize) {
        let Some(cfg) = self.config.as_mut() else {
            return;
        };
        let modules = match zone {
            WaybarZone::Left => &mut cfg.modules_left,
            WaybarZone::Center => &mut cfg.modules_center,
            WaybarZone::Right => &mut cfg.modules_right,
        };
        if index < modules.len() {
            modules.remove(index);
        }
        if let Some(cfg) = self.config.as_ref()
            && let Err(e) = save_waybar_config(cfg)
        {
            eprintln!("Failed to save waybar config after remove: {}", e);
        }
    }

    /// Move a module from (src_zone, src_index) to (dst_zone, dst_index) within the config,
    /// then persist the new order to disk immediately.
    pub fn move_module(
        &mut self,
        src_zone: &WaybarZone,
        src_index: usize,
        dst_zone: &WaybarZone,
        dst_index: usize,
    ) {
        let Some(cfg) = self.config.as_mut() else {
            return;
        };

        if src_zone == dst_zone {
            let modules = match src_zone {
                WaybarZone::Left => &mut cfg.modules_left,
                WaybarZone::Center => &mut cfg.modules_center,
                WaybarZone::Right => &mut cfg.modules_right,
            };
            if src_index < modules.len() {
                let item = modules.remove(src_index);
                let insert_at = dst_index.min(modules.len());
                modules.insert(insert_at, item);
            }
        } else {
            // Remove from source
            let module = {
                let src = match src_zone {
                    WaybarZone::Left => &mut cfg.modules_left,
                    WaybarZone::Center => &mut cfg.modules_center,
                    WaybarZone::Right => &mut cfg.modules_right,
                };
                if src_index < src.len() {
                    Some(src.remove(src_index))
                } else {
                    None
                }
            };
            // Insert into destination
            if let Some(mut module) = module {
                module.zone = dst_zone.clone();
                let dst = match dst_zone {
                    WaybarZone::Left => &mut cfg.modules_left,
                    WaybarZone::Center => &mut cfg.modules_center,
                    WaybarZone::Right => &mut cfg.modules_right,
                };
                let insert_at = dst_index.min(dst.len());
                dst.insert(insert_at, module);
            }
        }

        // Persist the new order to disk
        if let Some(cfg) = self.config.as_ref()
            && let Err(e) = save_waybar_config(cfg)
        {
            eprintln!("Failed to save waybar config: {}", e);
        }
    }
}

fn render_zone(
    entity: Entity<WaybarPreview>,
    zone: WaybarZone,
    modules: Vec<WaybarModule>,
    profile_name: &str,
    cx: &mut App,
) -> AnyElement {
    let chips: Vec<AnyElement> = modules
        .iter()
        .enumerate()
        .map(|(i, module)| {
            let chip =
                render_module_chip(module, zone.clone(), i, profile_name, entity.clone(), cx);
            let zone_drop = zone.clone();
            let entity_drop = entity.clone();

            div()
                .child(chip)
                .drag_over::<DragWaybarModule>(|style, _, _, cx| {
                    style.border_l_2().border_color(cx.theme().drag_border)
                })
                .on_drop(move |payload: &DragWaybarModule, _window, cx| {
                    entity_drop.update(cx, |this, cx| {
                        if payload.zone != zone_drop || payload.index != i {
                            this.move_module(&payload.zone, payload.index, &zone_drop, i);
                        }
                        cx.notify();
                    });
                })
                .into_any()
        })
        .collect();

    // Trailing drop target (append to end of zone)
    let zone_end = zone.clone();
    let entity_end = entity.clone();

    let trail_drop = div()
        .w(px(12.))
        .h_full()
        .min_h(px(24.))
        .drag_over::<DragWaybarModule>(|style, _, _, cx| {
            style.border_l_2().border_color(cx.theme().drag_border)
        })
        .on_drop(move |payload: &DragWaybarModule, _window, cx| {
            let dst_index = {
                let preview = entity_end.read(cx);
                match &zone_end {
                    WaybarZone::Left => preview
                        .config
                        .as_ref()
                        .map(|c| c.modules_left.len())
                        .unwrap_or(0),
                    WaybarZone::Center => preview
                        .config
                        .as_ref()
                        .map(|c| c.modules_center.len())
                        .unwrap_or(0),
                    WaybarZone::Right => preview
                        .config
                        .as_ref()
                        .map(|c| c.modules_right.len())
                        .unwrap_or(0),
                }
            };
            entity_end.update(cx, |this, cx| {
                this.move_module(&payload.zone, payload.index, &zone_end, dst_index);
                cx.notify();
            });
        });

    h_flex()
        .gap_1()
        .items_center()
        .flex_wrap()
        .children(chips)
        .child(trail_drop)
        .into_any()
}

impl Render for WaybarPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let bar_bg = theme.title_bar;
        let border = theme.border;

        let Some(config) = self.config.as_ref() else {
            return v_flex()
                .w_full()
                .flex_1()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_color(muted)
                        .text_sm()
                        .child("No waybar config found for this profile"),
                )
                .into_any();
        };

        let left = config.modules_left.clone();
        let center = config.modules_center.clone();
        let right = config.modules_right.clone();
        let profile_name = self.profile_name.clone();

        let entity = cx.entity().clone();

        let left_zone = render_zone(entity.clone(), WaybarZone::Left, left, &profile_name, cx);
        let center_zone =
            render_zone(entity.clone(), WaybarZone::Center, center, &profile_name, cx);
        let right_zone = render_zone(entity.clone(), WaybarZone::Right, right, &profile_name, cx);

        v_flex()
            .w_full()
            .flex_1()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child("Drag modules to reorder between zones."),
            )
            .child(
                div()
                    .w_full()
                    .rounded_md()
                    .border_1()
                    .border_color(border)
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .w_full()
                            .h(px(36.))
                            .px_2()
                            .bg(bar_bg)
                            .items_center()
                            .justify_between()
                            .child(div().flex_shrink_0().child(left_zone))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(center_zone),
                            )
                            .child(div().flex_shrink_0().child(right_zone)),
                    ),
            )
            .into_any()
    }
}
