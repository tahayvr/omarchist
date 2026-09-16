// The Configuration page: Hyprland settings as a declarative table of
// pages, groups, and items, rendered with a keyboard-driven page list on
// the left and native tab stops on the right. (gpui-component's `Settings`
// component keeps its page selection private, so it cannot be driven from
// the keyboard; this page renders the same content itself.)
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, IndexPath, Sizable as _,
    button::Button,
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement as _,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    sidebar::{SidebarItem, SidebarMenuItem},
    v_flex,
};

use crate::system::hyprland_config::HyprlandConfigManager;
use crate::types::hyprland_config::HyprlandConfig;
use crate::ui::focus::{self, FocusSection, FocusableSwitch};

const KEY_CONTEXT: &str = "ConfigPage";
pub const NAV_CONTEXT: &str = "ConfigNav";
pub const CONTENT_CONTEXT: &str = "ConfigContent";

pub mod config_nav {
    gpui::actions!(config_nav, [Prev, Next, First, Last, Activate, Back]);
}

// ---------------------------------------------------------------------
// Declarative definition of the page
// ---------------------------------------------------------------------

enum FieldDef {
    Number {
        min: f64,
        max: f64,
        step: f64,
        get: fn(&HyprlandConfig) -> f64,
        set: fn(&mut HyprlandConfig, f64),
    },
    Switch {
        get: fn(&HyprlandConfig) -> bool,
        set: fn(&mut HyprlandConfig, bool),
    },
    Dropdown {
        options: &'static [(&'static str, &'static str)],
        get: fn(&HyprlandConfig) -> String,
        set: fn(&mut HyprlandConfig, String),
    },
    KeyboardLayout,
}

struct ItemDef {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    field: FieldDef,
}

struct GroupDef {
    title: &'static str,
    items: &'static [ItemDef],
}

struct PageDef {
    title: &'static str,
    description: &'static str,
    groups: &'static [GroupDef],
}

macro_rules! number_item {
    ($id:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr, |$c:ident| $get:expr, |$m:ident, $v:ident| $set:expr) => {
        ItemDef {
            id: $id,
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                get: |$c| $get,
                set: |$m, $v| $set,
            },
        }
    };
}

macro_rules! switch_item {
    ($id:expr, $label:expr, $desc:expr, |$c:ident| $get:expr, |$m:ident, $v:ident| $set:expr) => {
        ItemDef {
            id: $id,
            label: $label,
            description: $desc,
            field: FieldDef::Switch {
                get: |$c| $get,
                set: |$m, $v| $set,
            },
        }
    };
}

const PAGES: &[PageDef] = &[
    PageDef {
        title: "General",
        description: "General window manager settings",
        groups: &[
            GroupDef {
                title: "Window Borders",
                items: &[
                    number_item!(
                        "border-size",
                        "Border Size",
                        "Size of the border around windows",
                        0.0,
                        10.0,
                        1.0,
                        |c| c.general.border_size as f64,
                        |m, v| m.general.border_size = v as i32
                    ),
                    switch_item!(
                        "resize-on-border",
                        "Resize on Border",
                        "Enable resizing windows by clicking and dragging on borders",
                        |c| c.general.resize_on_border,
                        |m, v| m.general.resize_on_border = v
                    ),
                ],
            },
            GroupDef {
                title: "Gaps",
                items: &[
                    number_item!(
                        "gaps-in",
                        "Gaps In",
                        "Gaps between windows",
                        0.0,
                        100.0,
                        1.0,
                        |c| c.general.gaps_in as f64,
                        |m, v| m.general.gaps_in = v as i32
                    ),
                    number_item!(
                        "gaps-out",
                        "Gaps Out",
                        "Gaps between windows and monitor edges",
                        0.0,
                        100.0,
                        1.0,
                        |c| c.general.gaps_out as f64,
                        |m, v| m.general.gaps_out = v as i32
                    ),
                    number_item!(
                        "gaps-workspaces",
                        "Gaps Workspaces",
                        "Gaps between workspaces. Stacks with gaps out",
                        0.0,
                        100.0,
                        1.0,
                        |c| c.general.gaps_workspaces as f64,
                        |m, v| m.general.gaps_workspaces = v as i32
                    ),
                ],
            },
            GroupDef {
                title: "Layout",
                items: &[ItemDef {
                    id: "layout",
                    label: "Layout",
                    description: "Window layout algorithm",
                    field: FieldDef::Dropdown {
                        options: &[("dwindle", "Dwindle"), ("master", "Master")],
                        get: |c| c.general.layout.clone(),
                        set: |m, v| m.general.layout = v,
                    },
                }],
            },
        ],
    },
    PageDef {
        title: "Appearance",
        description: "Visual appearance and effects",
        groups: &[
            GroupDef {
                title: "Rounding",
                items: &[number_item!(
                    "rounding",
                    "Rounding",
                    "Rounded corners radius in pixels",
                    0.0,
                    50.0,
                    1.0,
                    |c| c.decoration.rounding as f64,
                    |m, v| m.decoration.rounding = v as i32
                )],
            },
            GroupDef {
                title: "Opacity",
                items: &[
                    number_item!(
                        "active-opacity",
                        "Active Opacity",
                        "Opacity of active windows",
                        0.0,
                        1.0,
                        0.1,
                        |c| c.decoration.active_opacity,
                        |m, v| m.decoration.active_opacity = v
                    ),
                    number_item!(
                        "inactive-opacity",
                        "Inactive Opacity",
                        "Opacity of inactive windows",
                        0.0,
                        1.0,
                        0.1,
                        |c| c.decoration.inactive_opacity,
                        |m, v| m.decoration.inactive_opacity = v
                    ),
                ],
            },
            GroupDef {
                title: "Blur",
                items: &[
                    switch_item!(
                        "blur-enabled",
                        "Enable Blur",
                        "Enable window background blur",
                        |c| c.decoration.blur.enabled,
                        |m, v| m.decoration.blur.enabled = v
                    ),
                    number_item!(
                        "blur-size",
                        "Blur Size",
                        "Blur size/distance",
                        1.0,
                        20.0,
                        1.0,
                        |c| c.decoration.blur.size as f64,
                        |m, v| m.decoration.blur.size = v as i32
                    ),
                    number_item!(
                        "blur-passes",
                        "Blur Passes",
                        "Number of blur passes",
                        1.0,
                        5.0,
                        1.0,
                        |c| c.decoration.blur.passes as f64,
                        |m, v| m.decoration.blur.passes = v as i32
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Input",
        description: "Mouse, keyboard, and touchpad settings",
        groups: &[
            GroupDef {
                title: "Keyboard",
                items: &[
                    ItemDef {
                        id: "kb-layout",
                        label: "Keyboard Layout",
                        description: "Keyboard layout (e.g., us, de, fr)",
                        field: FieldDef::KeyboardLayout,
                    },
                    number_item!(
                        "repeat-rate",
                        "Repeat Rate",
                        "Repeat rate for held-down keys (repeats per second)",
                        1.0,
                        100.0,
                        1.0,
                        |c| c.input.repeat_rate as f64,
                        |m, v| m.input.repeat_rate = v as i32
                    ),
                    number_item!(
                        "repeat-delay",
                        "Repeat Delay",
                        "Delay before key repeat starts (milliseconds)",
                        100.0,
                        2000.0,
                        50.0,
                        |c| c.input.repeat_delay as f64,
                        |m, v| m.input.repeat_delay = v as i32
                    ),
                ],
            },
            GroupDef {
                title: "Mouse",
                items: &[
                    number_item!(
                        "sensitivity",
                        "Sensitivity",
                        "Mouse sensitivity (-1.0 to 1.0)",
                        -1.0,
                        1.0,
                        0.1,
                        |c| c.input.sensitivity,
                        |m, v| m.input.sensitivity = v
                    ),
                    switch_item!(
                        "natural-scroll",
                        "Natural Scroll",
                        "Invert scrolling direction",
                        |c| c.input.natural_scroll,
                        |m, v| m.input.natural_scroll = v
                    ),
                    switch_item!(
                        "left-handed",
                        "Left Handed",
                        "Switch left and right mouse buttons",
                        |c| c.input.left_handed,
                        |m, v| m.input.left_handed = v
                    ),
                ],
            },
            GroupDef {
                title: "Touchpad",
                items: &[
                    switch_item!(
                        "disable-while-typing",
                        "Disable While Typing",
                        "Disable touchpad while typing",
                        |c| c.input.touchpad.disable_while_typing,
                        |m, v| m.input.touchpad.disable_while_typing = v
                    ),
                    switch_item!(
                        "tap-to-click",
                        "Tap to Click",
                        "Tap on touchpad to click",
                        |c| c.input.touchpad.tap_to_click,
                        |m, v| m.input.touchpad.tap_to_click = v
                    ),
                    switch_item!(
                        "touchpad-natural-scroll",
                        "Natural Scroll",
                        "Invert touchpad scrolling direction",
                        |c| c.input.touchpad.natural_scroll,
                        |m, v| m.input.touchpad.natural_scroll = v
                    ),
                ],
            },
        ],
    },
    PageDef {
        title: "Miscellaneous",
        description: "Miscellaneous settings",
        groups: &[GroupDef {
            title: "General",
            items: &[switch_item!(
                "vfr",
                "VFR",
                "Variable refresh rate (saves battery)",
                |c| c.misc.vfr,
                |m, v| m.misc.vfr = v
            )],
        }],
    },
];

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        let text = format!("{value:.3}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

// ---------------------------------------------------------------------
// View
// ---------------------------------------------------------------------

#[derive(Clone, Debug)]
struct KeyboardLayoutItem {
    value: SharedString,
    // The human-readable label
    label: SharedString,
}

impl SelectItem for KeyboardLayoutItem {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.label.clone()
    }

    fn value(&self) -> &SharedString {
        &self.value
    }
}

pub struct ConfigView {
    config_manager: Rc<RefCell<HyprlandConfigManager>>,
    keyboard_layout_select: Entity<SelectState<SearchableVec<KeyboardLayoutItem>>>,
    /// One text state per number field, keyed by the item id.
    number_inputs: HashMap<&'static str, Entity<InputState>>,
    search: Entity<InputState>,
    active_page: usize,
    pub focus_handle: FocusHandle,
    /// The page list is one tab stop; up/down move between pages.
    nav_focus: FocusHandle,
    /// Non-tab-stop handle on the content column for `focus_first_in`.
    content_focus: FocusHandle,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl ConfigView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config_manager = match HyprlandConfigManager::load() {
            Ok(manager) => manager,
            Err(e) => {
                eprintln!("Failed to load Hyprland config: {}", e);
                HyprlandConfigManager::load().expect("Failed to create default config")
            }
        };

        let catalog = crate::system::hyprland_config::keyboard::load_keyboard_catalog();
        let mut layout_items: Vec<KeyboardLayoutItem> = match catalog {
            Ok(c) => c
                .layouts
                .into_iter()
                .map(|l| KeyboardLayoutItem {
                    value: l.name.clone().into(),
                    label: l.description.clone().into(),
                })
                .collect(),
            Err(e) => {
                eprintln!("Failed to load keyboard catalog: {}", e);
                vec![KeyboardLayoutItem {
                    value: "us".into(),
                    label: "English (US)".into(),
                }]
            }
        };
        layout_items.sort_by(|a, b| a.label.cmp(&b.label));

        let current_kb = config_manager.get().input.kb_layout.clone();
        let initial_index = layout_items
            .iter()
            .position(|item| item.value.as_ref() == current_kb.as_str())
            .map(|i| IndexPath::default().row(i));

        let delegate = SearchableVec::new(layout_items);
        let keyboard_layout_select =
            cx.new(|cx| SelectState::new(delegate, initial_index, window, cx).searchable(true));

        let mut subscriptions = vec![cx.subscribe_in(
            &keyboard_layout_select,
            window,
            |this, _select, event: &SelectEvent<SearchableVec<KeyboardLayoutItem>>, _window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    let layout = value.to_string();
                    this.update_config(cx, |c| c.input.kb_layout = layout);
                }
            },
        )];

        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search settings"));
        subscriptions.push(
            cx.subscribe_in(&search, window, |_, _, event: &InputEvent, _, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            }),
        );

        let config = config_manager.get().clone();
        let mut number_inputs = HashMap::new();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            let FieldDef::Number {
                min,
                max,
                step,
                get,
                set,
            } = item.field
            else {
                continue;
            };
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(get(&config))));
            subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = get(this.config_manager.borrow().get());
                    let next = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    let next = (next * 1000.0).round() / 1000.0;
                    let next = next.clamp(min, max);
                    this.update_config(cx, |c| set(c, next));
                    input.update(cx, |input, cx| {
                        input.set_value(format_number(next), window, cx);
                    });
                },
            ));
            subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => {
                        if let Ok(value) = input.read(cx).value().trim().parse::<f64>() {
                            let value = value.clamp(min, max);
                            if get(this.config_manager.borrow().get()) != value {
                                this.update_config(cx, |c| set(c, value));
                            }
                        }
                    }
                    InputEvent::Blur => {
                        // Normalise the text (clamped, trimmed) once editing ends.
                        let value = get(this.config_manager.borrow().get());
                        input.update(cx, |input, cx| {
                            let text = format_number(value);
                            if input.value() != text {
                                input.set_value(text, window, cx);
                            }
                        });
                    }
                    _ => {}
                },
            ));
            number_inputs.insert(item.id, input);
        }

        Self {
            config_manager: Rc::new(RefCell::new(config_manager)),
            keyboard_layout_select,
            number_inputs,
            search,
            active_page: 0,
            focus_handle: cx.focus_handle(),
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// Focuses the page list, the page's first control.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.nav_focus.focus(window, cx);
    }

    /// Re-reads the saved configuration from disk and refreshes the fields.
    pub fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match HyprlandConfigManager::load() {
            Ok(manager) => {
                *self.config_manager.borrow_mut() = manager;
                self.sync_inputs(window, cx);
                cx.notify();
            }
            Err(e) => eprintln!("Failed to reload Hyprland config: {}", e),
        }
    }

    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let config = self.config_manager.borrow().get().clone();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            if let FieldDef::Number { get, .. } = item.field
                && let Some(input) = self.number_inputs.get(item.id)
            {
                input.update(cx, |input, cx| {
                    input.set_value(format_number(get(&config)), window, cx);
                });
            }
        }
        let layout: SharedString = config.input.kb_layout.clone().into();
        self.keyboard_layout_select.update(cx, |select, cx| {
            select.set_selected_value(&layout, window, cx);
        });
    }

    fn update_config<F>(&mut self, cx: &mut Context<Self>, f: F)
    where
        F: FnOnce(&mut HyprlandConfig),
    {
        self.config_manager.borrow_mut().update(f);
        let _ = self.config_manager.borrow().save();
        cx.notify();
    }

    fn set_page(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(PAGES.len() - 1);
        if self.active_page != index {
            self.active_page = index;
            self.scroll.set_offset(Point::default());
            cx.notify();
        }
    }

    fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().trim().to_lowercase()
    }

    fn item_matches(item: &ItemDef, query: &str) -> bool {
        query.is_empty()
            || item.label.to_lowercase().contains(query)
            || item.description.to_lowercase().contains(query)
    }

    fn render_nav(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_focused = self.nav_focus.is_focused(window);
        let radius = cx.theme().radius;
        let transparent = cx.theme().transparent;

        v_flex()
            .id("config-nav")
            .key_context(NAV_CONTEXT)
            .track_focus(&self.nav_focus)
            .on_action(cx.listener(|this, _: &config_nav::Prev, _, cx| {
                this.set_page(this.active_page.saturating_sub(1), cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Next, _, cx| {
                this.set_page(this.active_page + 1, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::First, _, cx| {
                this.set_page(0, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Last, _, cx| {
                this.set_page(usize::MAX, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Activate, window, cx| {
                focus::focus_first_in(&this.content_focus, window, cx);
            }))
            .w(px(220.))
            .flex_none()
            .gap_2()
            .pr_4()
            .cursor_pointer()
            .children(PAGES.iter().enumerate().map(|(ix, page)| {
                let focused = nav_focused && self.active_page == ix;
                div()
                    .id(("config-page", ix))
                    .rounded(radius)
                    .border_1()
                    .border_color(focus::focus_border(focused, transparent, cx))
                    .child(
                        SidebarMenuItem::new(page.title)
                            .active(self.active_page == ix)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.nav_focus.focus(window, cx);
                                this.set_page(ix, cx);
                            }))
                            .render(("config-page-item", ix), window, cx),
                    )
            }))
    }

    fn render_item(&self, item: &'static ItemDef, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let control: AnyElement = match &item.field {
            FieldDef::Number { .. } => match self.number_inputs.get(item.id) {
                Some(input) => NumberInput::new(input)
                    .small()
                    .w(px(140.))
                    .into_any_element(),
                None => div().into_any_element(),
            },
            FieldDef::Switch { get, set } => {
                let checked = get(self.config_manager.borrow().get());
                let set = *set;
                FocusableSwitch::new(item.id)
                    .checked(checked)
                    .on_change(cx.listener(move |this, value, _, cx| {
                        let value = *value;
                        this.update_config(cx, |c| set(c, value));
                    }))
                    .into_any_element()
            }
            FieldDef::Dropdown { options, get, set } => {
                let current = get(self.config_manager.borrow().get());
                let label = options
                    .iter()
                    .find(|(value, _)| *value == current)
                    .map(|(_, label)| *label)
                    .unwrap_or(current.as_str())
                    .to_string();
                let set = *set;
                let view = cx.entity();
                Button::new(item.id)
                    .label(label)
                    .dropdown_caret(true)
                    .outline()
                    .small()
                    .cursor_pointer()
                    .dropdown_menu(move |menu, _, _| {
                        options.iter().fold(menu, |menu, (value, label)| {
                            let checked = *value == current;
                            let view = view.clone();
                            menu.item(PopupMenuItem::new(*label).checked(checked).on_click(
                                move |_, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.update_config(cx, |c| set(c, value.to_string()));
                                    });
                                },
                            ))
                        })
                    })
                    .into_any_element()
            }
            // `Select` renders a full-width root; the box keeps it off the label column.
            FieldDef::KeyboardLayout => div()
                .w(px(260.))
                .flex_none()
                .child(
                    Select::new(&self.keyboard_layout_select)
                        .search_placeholder("Search layouts...")
                        .small(),
                )
                .into_any_element(),
        };

        h_flex()
            .id(ElementId::Name(format!("config-item-{}", item.id).into()))
            .w_full()
            .gap_4()
            .items_center()
            .justify_between()
            .py_2()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(div().text_sm().child(item.label))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(item.description),
                    ),
            )
            .child(div().flex_none().child(control))
    }

    fn render_group(
        &self,
        page_ix: usize,
        group: &'static GroupDef,
        query: &str,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let items: Vec<AnyElement> = group
            .items
            .iter()
            .filter(|item| Self::item_matches(item, query))
            .map(|item| self.render_item(item, cx).into_any_element())
            .collect();
        if items.is_empty() {
            return None;
        }
        Some(
            FocusSection::new(
                ElementId::Name(format!("config-group-{page_ix}-{}", group.title).into()),
                &self.scroll,
            )
            .child(
                GroupBox::new()
                    .with_variant(GroupBoxVariant::Outline)
                    .title(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(group.title),
                    )
                    .children(items),
            )
            .into_any_element(),
        )
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.query(cx);
        let muted = cx.theme().muted_foreground;
        let searching = !query.is_empty();

        let pages: Vec<(usize, &'static PageDef)> = if searching {
            PAGES.iter().enumerate().collect()
        } else {
            vec![(self.active_page, &PAGES[self.active_page])]
        };

        let mut sections: Vec<AnyElement> = Vec::new();
        for (page_ix, page) in pages {
            let groups: Vec<AnyElement> = page
                .groups
                .iter()
                .filter_map(|group| self.render_group(page_ix, group, &query, cx))
                .collect();
            if groups.is_empty() {
                continue;
            }
            sections.push(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(page.title),
                    )
                    .child(div().text_sm().text_color(muted).child(page.description))
                    .into_any_element(),
            );
            sections.extend(groups);
        }
        if sections.is_empty() {
            sections.push(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child("No settings match your search.")
                    .into_any_element(),
            );
        }

        div()
            .id("config-content")
            .key_context(CONTENT_CONTEXT)
            .track_focus(&self.content_focus)
            .on_action(cx.listener(|this, _: &config_nav::Back, window, cx| {
                this.nav_focus.focus(window, cx);
            }))
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .child(v_flex().gap_4().pb_8().pr_4().children(sections))
    }
}

impl Render for ConfigView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("config-view-root")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .size_full()
            .gap_4()
            .child(
                div().w(px(320.)).child(
                    Input::new(&self.search)
                        .cleanable(true)
                        .small()
                        .prefix(Icon::new(IconName::Search).size_4()),
                ),
            )
            .child(
                // Not `h_flex`: its `items_center` would stop the content pane from
                // filling the row height, which it needs to scroll.
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_nav(window, cx))
                    .child(self.render_content(cx))
                    .vertical_scrollbar(&self.scroll),
            )
    }
}

#[cfg(test)]
mod tests {
    // `use super::*` would import gpui's `test` attribute macro and shadow `#[test]`.
    use super::{FieldDef, HyprlandConfig, PAGES, format_number};
    use std::collections::HashSet;

    #[test]
    fn item_ids_are_unique() {
        let mut seen = HashSet::new();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            assert!(seen.insert(item.id), "duplicate item id {}", item.id);
        }
    }

    #[test]
    fn every_field_round_trips_through_the_config() {
        let mut config = HyprlandConfig::default();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            match &item.field {
                FieldDef::Number {
                    min, max, get, set, ..
                } => {
                    set(&mut config, *max);
                    assert_eq!(get(&config), *max, "{}", item.id);
                    set(&mut config, *min);
                    assert_eq!(get(&config), *min, "{}", item.id);
                }
                FieldDef::Switch { get, set } => {
                    set(&mut config, true);
                    assert!(get(&config), "{}", item.id);
                    set(&mut config, false);
                    assert!(!get(&config), "{}", item.id);
                }
                FieldDef::Dropdown { options, get, set } => {
                    for (value, _) in options.iter() {
                        set(&mut config, value.to_string());
                        assert_eq!(get(&config), *value, "{}", item.id);
                    }
                }
                FieldDef::KeyboardLayout => {}
            }
        }
    }

    #[test]
    fn numbers_format_without_noise() {
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(0.1), "0.1");
        assert_eq!(format_number(0.30000000000000004), "0.3");
        assert_eq!(format_number(-1.0), "-1");
    }
}
