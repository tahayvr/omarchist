use gpui::*;
use gpui_component::{
    Sizable as _, Size,
    setting::{NumberFieldOptions, SettingGroup, SettingItem, SettingPage, Settings},
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::system::hyprland_config::HyprlandConfigManager;

/// Main configuration view
pub struct ConfigView {
    config_manager: Rc<RefCell<HyprlandConfigManager>>,
}

impl ConfigView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let config_manager = match HyprlandConfigManager::load() {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("Failed to load Hyprland config: {}", e);
                // This will panic if it fails, but we have no other option. I think? we should handle this error more gracefully.
                HyprlandConfigManager::load().expect("Failed to create default config")
            }
        };

        Self {
            config_manager: Rc::new(RefCell::new(config_manager)),
        }
    }
}

impl Render for ConfigView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Settings::new("hyprland-config")
            .sidebar_width(px(220.0))
            .with_group_variant(gpui_component::group_box::GroupBoxVariant::Normal)
            .with_size(Size::default())
            .page(self.create_general_page())
            .page(self.create_appearance_page())
            .page(self.create_input_page())
            .page(self.create_gestures_page())
            .page(self.create_misc_page())
    }
}

impl ConfigView {
    fn create_general_page(&self) -> SettingPage {
        SettingPage::new("General")
            .description("General window manager settings")
            .group(
                SettingGroup::new()
                    .title("Window Borders")
                    .item(
                        SettingItem::new("Border Size", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 10.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().general.border_size as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.general.border_size = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Size of the border around windows"),
                    )
                    .item(
                        SettingItem::new("Resize on Border", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().general.resize_on_border,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.general.resize_on_border = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(false)
                        })
                        .description("Enable resizing windows by clicking and dragging on borders"),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title("Gaps")
                    .item(
                        SettingItem::new("Gaps In", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().general.gaps_in as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.general.gaps_in = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(5.0)
                        })
                        .description("Gaps between windows"),
                    )
                    .item(
                        SettingItem::new("Gaps Out", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().general.gaps_out as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.general.gaps_out = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(20.0)
                        })
                        .description("Gaps between windows and monitor edges"),
                    ),
            )
            .group(
                SettingGroup::new().title("Layout").item(
                    SettingItem::new("Layout", {
                        let cm_get = self.config_manager.clone();
                        let cm_set = self.config_manager.clone();
                        gpui_component::setting::SettingField::dropdown(
                            vec![
                                ("dwindle".into(), "Dwindle".into()),
                                ("master".into(), "Master".into()),
                            ],
                            move |_cx| cm_get.borrow().get().general.layout.clone().into(),
                            move |value, _cx| {
                                cm_set
                                    .borrow_mut()
                                    .update(|c| c.general.layout = value.to_string());
                                let _ = cm_set.borrow().save();
                            },
                        )
                        .default_value("dwindle")
                    })
                    .description("Window layout algorithm"),
                ),
            )
    }

    fn create_appearance_page(&self) -> SettingPage {
        SettingPage::new("Appearance")
            .description("Visual appearance and effects")
            .group(
                SettingGroup::new().title("Rounding").item(
                    SettingItem::new("Rounding", {
                        let cm_get = self.config_manager.clone();
                        let cm_set = self.config_manager.clone();
                        gpui_component::setting::SettingField::number_input(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 50.0,
                                step: 1.0,
                            },
                            move |_cx| cm_get.borrow().get().decoration.rounding as f64,
                            move |value, _cx| {
                                cm_set
                                    .borrow_mut()
                                    .update(|c| c.decoration.rounding = value as i32);
                                let _ = cm_set.borrow().save();
                            },
                        )
                        .default_value(0.0)
                    })
                    .description("Rounded corners radius in pixels"),
                ),
            )
            .group(
                SettingGroup::new()
                    .title("Opacity")
                    .item(
                        SettingItem::new("Active Opacity", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm_get.borrow().get().decoration.active_opacity,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.decoration.active_opacity = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Opacity of active windows"),
                    )
                    .item(
                        SettingItem::new("Inactive Opacity", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm_get.borrow().get().decoration.inactive_opacity,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.decoration.inactive_opacity = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Opacity of inactive windows"),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title("Blur")
                    .item(
                        SettingItem::new("Enable Blur", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().decoration.blur.enabled,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.decoration.blur.enabled = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Enable window background blur"),
                    )
                    .item(
                        SettingItem::new("Blur Size", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 20.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().decoration.blur.size as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.decoration.blur.size = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(8.0)
                        })
                        .description("Blur size/distance"),
                    )
                    .item(
                        SettingItem::new("Blur Passes", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 5.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().decoration.blur.passes as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.decoration.blur.passes = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Number of blur passes"),
                    ),
            )
    }

    fn create_input_page(&self) -> SettingPage {
        SettingPage::new("Input")
            .description("Mouse, keyboard, and touchpad settings")
            .group(
                SettingGroup::new()
                    .title("Keyboard")
                    .item(
                        SettingItem::new("Keyboard Layout", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::input(
                                move |_cx| cm_get.borrow().get().input.kb_layout.clone().into(),
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.kb_layout = value.to_string());
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value("us")
                        })
                        .description("Keyboard layout (e.g., us, de, fr)"),
                    )
                    .item(
                        SettingItem::new("Repeat Rate", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm_get.borrow().get().input.repeat_rate as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.repeat_rate = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(25.0)
                        })
                        .description("Repeat rate for held-down keys (repeats per second)"),
                    )
                    .item(
                        SettingItem::new("Repeat Delay", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 100.0,
                                    max: 2000.0,
                                    step: 50.0,
                                },
                                move |_cx| cm_get.borrow().get().input.repeat_delay as f64,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.repeat_delay = value as i32);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(600.0)
                        })
                        .description("Delay before key repeat starts (milliseconds)"),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title("Mouse")
                    .item(
                        SettingItem::new("Sensitivity", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: -1.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm_get.borrow().get().input.sensitivity,
                                move |value, _cx| {
                                    cm_set.borrow_mut().update(|c| c.input.sensitivity = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(0.0)
                        })
                        .description("Mouse sensitivity (-1.0 to 1.0)"),
                    )
                    .item(
                        SettingItem::new("Natural Scroll", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().input.natural_scroll,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.natural_scroll = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(false)
                        })
                        .description("Invert scrolling direction"),
                    )
                    .item(
                        SettingItem::new("Left Handed", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().input.left_handed,
                                move |value, _cx| {
                                    cm_set.borrow_mut().update(|c| c.input.left_handed = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(false)
                        })
                        .description("Switch left and right mouse buttons"),
                    ),
            )
            .group(
                SettingGroup::new()
                    .title("Touchpad")
                    .item(
                        SettingItem::new("Disable While Typing", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| {
                                    cm_get.borrow().get().input.touchpad.disable_while_typing
                                },
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.touchpad.disable_while_typing = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Disable touchpad while typing"),
                    )
                    .item(
                        SettingItem::new("Tap to Click", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().input.touchpad.tap_to_click,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.touchpad.tap_to_click = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Tap on touchpad to click"),
                    )
                    .item(
                        SettingItem::new("Natural Scroll", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().input.touchpad.natural_scroll,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.input.touchpad.natural_scroll = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(false)
                        })
                        .description("Invert touchpad scrolling direction"),
                    ),
            )
    }

    fn create_gestures_page(&self) -> SettingPage {
        SettingPage::new("Gestures")
            .description("Touchpad gesture settings")
            .group(
                SettingGroup::new()
                    .title("Workspace Swipe")
                    .item(
                        SettingItem::new("Swipe Distance", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 100.0,
                                    max: 1000.0,
                                    step: 50.0,
                                },
                                move |_cx| {
                                    cm_get.borrow().get().gestures.workspace_swipe_distance as f64
                                },
                                move |value, _cx| {
                                    cm_set.borrow_mut().update(|c| {
                                        c.gestures.workspace_swipe_distance = value as i32
                                    });
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(300.0)
                        })
                        .description("Distance in pixels for workspace swipe gesture"),
                    )
                    .item(
                        SettingItem::new("Invert Direction", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().gestures.workspace_swipe_invert,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.gestures.workspace_swipe_invert = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Invert swipe direction"),
                    )
                    .item(
                        SettingItem::new("Create New Workspace", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| {
                                    cm_get.borrow().get().gestures.workspace_swipe_create_new
                                },
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.gestures.workspace_swipe_create_new = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Create new workspace when swiping past last workspace"),
                    ),
            )
    }

    fn create_misc_page(&self) -> SettingPage {
        SettingPage::new("Miscellaneous")
            .description("Miscellaneous settings")
            .group(
                SettingGroup::new()
                    .title("General")
                    .item(
                        SettingItem::new("Disable Logo", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().misc.disable_hyprland_logo,
                                move |value, _cx| {
                                    cm_set
                                        .borrow_mut()
                                        .update(|c| c.misc.disable_hyprland_logo = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(false)
                        })
                        .description("Disable the Hyprland logo/anime background"),
                    )
                    .item(
                        SettingItem::new("VFR", {
                            let cm_get = self.config_manager.clone();
                            let cm_set = self.config_manager.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm_get.borrow().get().misc.vfr,
                                move |value, _cx| {
                                    cm_set.borrow_mut().update(|c| c.misc.vfr = value);
                                    let _ = cm_set.borrow().save();
                                },
                            )
                            .default_value(true)
                        })
                        .description("Variable refresh rate (saves battery)"),
                    ),
            )
    }
}
