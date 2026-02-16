use gpui::*;
use gpui_component::{
    Sizable as _, Size,
    setting::{NumberFieldOptions, SettingGroup, SettingItem, SettingPage, Settings},
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::system::hyprland_config::HyprlandConfigManager;
use crate::types::hyprland_config::HyprlandConfig;

/// Main configuration view using gpui-component Settings API
pub struct ConfigView {
    config_manager: Rc<RefCell<HyprlandConfigManager>>,
    // This counter is incremented whenever config changes to force re-render
    _update_counter: u32,
}

impl ConfigView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let config_manager = match HyprlandConfigManager::load() {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("Failed to load Hyprland config: {}", e);
                // This will panic if it fails, but we have no other option
                HyprlandConfigManager::load().expect("Failed to create default config")
            }
        };

        Self {
            config_manager: Rc::new(RefCell::new(config_manager)),
            _update_counter: 0,
        }
    }

    /// Update config and trigger UI refresh
    fn update_config<F>(&mut self, cx: &mut Context<Self>, f: F)
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        self.config_manager.borrow_mut().update(f);
        let _ = self.config_manager.borrow().save();
        self._update_counter += 1;
        cx.notify();
    }
}

impl Render for ConfigView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Clone self reference for use in closures
        let view = cx.entity().clone();

        Settings::new(("hyprland-config", self._update_counter))
            .sidebar_width(px(220.0))
            .with_group_variant(gpui_component::group_box::GroupBoxVariant::Normal)
            .with_size(Size::default())
            .page(self.create_general_page(&view))
            .page(self.create_appearance_page(&view))
            .page(self.create_input_page(&view))
            .page(self.create_gestures_page(&view))
            .page(self.create_misc_page(&view))
    }
}

impl ConfigView {
    fn create_general_page(&self, view: &Entity<Self>) -> SettingPage {
        let cm = self.config_manager.clone();

        SettingPage::new("General")
            .description("General window manager settings")
            .group(
                SettingGroup::new()
                    .title("Window Borders")
                    .item(
                        SettingItem::new("Border Size", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 10.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().general.border_size as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.general.border_size = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(2.0)
                        })
                        .description("Size of the border around windows"),
                    )
                    .item(
                        SettingItem::new("Resize on Border", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().general.resize_on_border,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.general.resize_on_border = value
                                        });
                                    });
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
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().general.gaps_in as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.general.gaps_in = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(5.0)
                        })
                        .description("Gaps between windows"),
                    )
                    .item(
                        SettingItem::new("Gaps Out", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().general.gaps_out as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.general.gaps_out = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(10.0)
                        })
                        .description("Gaps between windows and monitor edges"),
                    ),
            )
            .group(
                SettingGroup::new().title("Layout").item(
                    SettingItem::new("Layout", {
                        let cm = cm.clone();
                        let view = view.clone();
                        gpui_component::setting::SettingField::dropdown(
                            vec![
                                ("dwindle".into(), "Dwindle".into()),
                                ("master".into(), "Master".into()),
                            ],
                            move |_cx| cm.borrow().get().general.layout.clone().into(),
                            move |value, cx| {
                                view.update(cx, |this, cx| {
                                    this.update_config(cx, |c| {
                                        c.general.layout = value.to_string()
                                    });
                                });
                            },
                        )
                        .default_value("dwindle")
                    })
                    .description("Window layout algorithm"),
                ),
            )
    }

    fn create_appearance_page(&self, view: &Entity<Self>) -> SettingPage {
        let cm = self.config_manager.clone();

        SettingPage::new("Appearance")
            .description("Visual appearance and effects")
            .group(
                SettingGroup::new().title("Rounding").item(
                    SettingItem::new("Rounding", {
                        let cm = cm.clone();
                        let view = view.clone();
                        gpui_component::setting::SettingField::number_input(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 50.0,
                                step: 1.0,
                            },
                            move |_cx| cm.borrow().get().decoration.rounding as f64,
                            move |value, cx| {
                                view.update(cx, |this, cx| {
                                    this.update_config(cx, |c| {
                                        c.decoration.rounding = value as i32
                                    });
                                });
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
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm.borrow().get().decoration.active_opacity,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.decoration.active_opacity = value
                                        });
                                    });
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Opacity of active windows"),
                    )
                    .item(
                        SettingItem::new("Inactive Opacity", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 0.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm.borrow().get().decoration.inactive_opacity,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.decoration.inactive_opacity = value
                                        });
                                    });
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
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().decoration.blur.enabled,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.decoration.blur.enabled = value
                                        });
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Enable window background blur"),
                    )
                    .item(
                        SettingItem::new("Blur Size", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 20.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().decoration.blur.size as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.decoration.blur.size = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(8.0)
                        })
                        .description("Blur size/distance"),
                    )
                    .item(
                        SettingItem::new("Blur Passes", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 5.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().decoration.blur.passes as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.decoration.blur.passes = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(1.0)
                        })
                        .description("Number of blur passes"),
                    ),
            )
    }

    fn create_input_page(&self, view: &Entity<Self>) -> SettingPage {
        let cm = self.config_manager.clone();

        SettingPage::new("Input")
            .description("Mouse, keyboard, and touchpad settings")
            .group(
                SettingGroup::new()
                    .title("Keyboard")
                    .item(
                        SettingItem::new("Keyboard Layout", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::input(
                                move |_cx| cm.borrow().get().input.kb_layout.clone().into(),
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.kb_layout = value.to_string()
                                        });
                                    });
                                },
                            )
                            .default_value("us")
                        })
                        .description("Keyboard layout (e.g., us, de, fr)"),
                    )
                    .item(
                        SettingItem::new("Repeat Rate", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 100.0,
                                    step: 1.0,
                                },
                                move |_cx| cm.borrow().get().input.repeat_rate as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.repeat_rate = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(25.0)
                        })
                        .description("Repeat rate for held-down keys (repeats per second)"),
                    )
                    .item(
                        SettingItem::new("Repeat Delay", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 100.0,
                                    max: 2000.0,
                                    step: 50.0,
                                },
                                move |_cx| cm.borrow().get().input.repeat_delay as f64,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.repeat_delay = value as i32
                                        });
                                    });
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
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: -1.0,
                                    max: 1.0,
                                    step: 0.1,
                                },
                                move |_cx| cm.borrow().get().input.sensitivity,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| c.input.sensitivity = value);
                                    });
                                },
                            )
                            .default_value(0.0)
                        })
                        .description("Mouse sensitivity (-1.0 to 1.0)"),
                    )
                    .item(
                        SettingItem::new("Natural Scroll", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().input.natural_scroll,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| c.input.natural_scroll = value);
                                    });
                                },
                            )
                            .default_value(false)
                        })
                        .description("Invert scrolling direction"),
                    )
                    .item(
                        SettingItem::new("Left Handed", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().input.left_handed,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| c.input.left_handed = value);
                                    });
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
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().input.touchpad.disable_while_typing,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.touchpad.disable_while_typing = value
                                        });
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Disable touchpad while typing"),
                    )
                    .item(
                        SettingItem::new("Tap to Click", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().input.touchpad.tap_to_click,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.touchpad.tap_to_click = value
                                        });
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Tap on touchpad to click"),
                    )
                    .item(
                        SettingItem::new("Natural Scroll", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().input.touchpad.natural_scroll,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.input.touchpad.natural_scroll = value
                                        });
                                    });
                                },
                            )
                            .default_value(false)
                        })
                        .description("Invert touchpad scrolling direction"),
                    ),
            )
    }

    fn create_gestures_page(&self, view: &Entity<Self>) -> SettingPage {
        let cm = self.config_manager.clone();

        SettingPage::new("Gestures")
            .description("Touchpad gesture settings")
            .group(
                SettingGroup::new()
                    .title("Workspace Swipe")
                    .item(
                        SettingItem::new("Swipe Distance", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::number_input(
                                NumberFieldOptions {
                                    min: 100.0,
                                    max: 1000.0,
                                    step: 50.0,
                                },
                                move |_cx| {
                                    cm.borrow().get().gestures.workspace_swipe_distance as f64
                                },
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.gestures.workspace_swipe_distance = value as i32
                                        });
                                    });
                                },
                            )
                            .default_value(300.0)
                        })
                        .description("Distance in pixels for workspace swipe gesture"),
                    )
                    .item(
                        SettingItem::new("Invert Direction", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().gestures.workspace_swipe_invert,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.gestures.workspace_swipe_invert = value
                                        });
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Invert swipe direction"),
                    )
                    .item(
                        SettingItem::new("Create New Workspace", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().gestures.workspace_swipe_create_new,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.gestures.workspace_swipe_create_new = value
                                        });
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Create new workspace when swiping past last workspace"),
                    ),
            )
    }

    fn create_misc_page(&self, view: &Entity<Self>) -> SettingPage {
        let cm = self.config_manager.clone();

        SettingPage::new("Miscellaneous")
            .description("Miscellaneous settings")
            .group(
                SettingGroup::new()
                    .title("General")
                    .item(
                        SettingItem::new("Disable Logo", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().misc.disable_hyprland_logo,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| {
                                            c.misc.disable_hyprland_logo = value
                                        });
                                    });
                                },
                            )
                            .default_value(false)
                        })
                        .description("Disable the Hyprland logo/anime background"),
                    )
                    .item(
                        SettingItem::new("VFR", {
                            let cm = cm.clone();
                            let view = view.clone();
                            gpui_component::setting::SettingField::switch(
                                move |_cx| cm.borrow().get().misc.vfr,
                                move |value, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| c.misc.vfr = value);
                                    });
                                },
                            )
                            .default_value(true)
                        })
                        .description("Variable refresh rate (saves battery)"),
                    ),
            )
    }
}
