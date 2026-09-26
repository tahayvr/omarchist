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
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement as _,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    sidebar::{SidebarItem, SidebarMenuItem},
    v_flex,
};

use serde_json::Value;

use crate::system::hyprland_config::HyprlandConfigManager;
use crate::ui::focus::{self, FocusSection, FocusableSwitch};
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "ConfigPage";
pub const NAV_CONTEXT: &str = "ConfigNav";
pub const CONTENT_CONTEXT: &str = "ConfigContent";

pub mod config_nav {
    gpui::actions!(config_nav, [Prev, Next, First, Last, Activate, Back]);
}

// ---------------------------------------------------------------------
// Declarative definition of the page
// ---------------------------------------------------------------------

// How an item is edited. Values are read and written by the dotted
// Hyprland option path (`general.gaps_in`), the same path the manager
// stores overrides under.
enum FieldDef {
    Number {
        min: f64,
        max: f64,
        step: f64,
        /// Written as an integer, as Hyprland declares the option.
        integer: bool,
    },
    Switch,
    Dropdown {
        options: &'static [(&'static str, &'static str)],
    },
    KeyboardLayout,
}

struct ItemDef {
    id: &'static str,
    /// The Hyprland option, dotted (`input.touchpad.tap_to_click`).
    path: &'static str,
    label: &'static str,
    description: &'static str,
    field: FieldDef,
}

const KEYBOARD_LAYOUT_PATH: &str = "input.kb_layout";

struct GroupDef {
    title: &'static str,
    items: &'static [ItemDef],
}

struct PageDef {
    title: &'static str,
    description: &'static str,
    groups: &'static [GroupDef],
}

macro_rules! int_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            path: $path,
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                integer: true,
            },
        }
    };
}

macro_rules! float_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr, $min:expr, $max:expr, $step:expr) => {
        ItemDef {
            id: $id,
            path: $path,
            label: $label,
            description: $desc,
            field: FieldDef::Number {
                min: $min,
                max: $max,
                step: $step,
                integer: false,
            },
        }
    };
}

macro_rules! switch_item {
    ($id:expr, $path:expr, $label:expr, $desc:expr) => {
        ItemDef {
            id: $id,
            path: $path,
            label: $label,
            description: $desc,
            field: FieldDef::Switch,
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
                    int_item!(
                        "border-size",
                        "general.border_size",
                        "Border Size",
                        "Size of the border around windows",
                        0.0,
                        20.0,
                        1.0
                    ),
                    switch_item!(
                        "resize-on-border",
                        "general.resize_on_border",
                        "Resize on Border",
                        "Enable resizing windows by clicking and dragging on borders"
                    ),
                ],
            },
            GroupDef {
                title: "Gaps",
                items: &[
                    int_item!(
                        "gaps-in",
                        "general.gaps_in",
                        "Gaps In",
                        "Gaps between windows",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "gaps-out",
                        "general.gaps_out",
                        "Gaps Out",
                        "Gaps between windows and monitor edges",
                        0.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "gaps-workspaces",
                        "general.gaps_workspaces",
                        "Gaps Workspaces",
                        "Gaps between workspaces. Stacks with gaps out",
                        0.0,
                        100.0,
                        1.0
                    ),
                ],
            },
            GroupDef {
                title: "Layout",
                items: &[ItemDef {
                    id: "layout",
                    path: "general.layout",
                    label: "Layout",
                    description: "Window layout algorithm",
                    field: FieldDef::Dropdown {
                        options: &[
                            ("dwindle", "Dwindle"),
                            ("master", "Master"),
                            ("scrolling", "Scrolling"),
                            ("monocle", "Monocle"),
                        ],
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
                items: &[int_item!(
                    "rounding",
                    "decoration.rounding",
                    "Rounding",
                    "Rounded corners radius in pixels",
                    0.0,
                    20.0,
                    1.0
                )],
            },
            GroupDef {
                title: "Opacity",
                items: &[
                    float_item!(
                        "active-opacity",
                        "decoration.active_opacity",
                        "Active Opacity",
                        "Opacity of active windows",
                        0.0,
                        1.0,
                        0.05
                    ),
                    float_item!(
                        "inactive-opacity",
                        "decoration.inactive_opacity",
                        "Inactive Opacity",
                        "Opacity of inactive windows",
                        0.0,
                        1.0,
                        0.05
                    ),
                ],
            },
            GroupDef {
                title: "Blur",
                items: &[
                    switch_item!(
                        "blur-enabled",
                        "decoration.blur.enabled",
                        "Enable Blur",
                        "Enable window background blur"
                    ),
                    int_item!(
                        "blur-size",
                        "decoration.blur.size",
                        "Blur Size",
                        "Blur size/distance",
                        1.0,
                        100.0,
                        1.0
                    ),
                    int_item!(
                        "blur-passes",
                        "decoration.blur.passes",
                        "Blur Passes",
                        "Number of blur passes",
                        1.0,
                        10.0,
                        1.0
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
                        path: KEYBOARD_LAYOUT_PATH,
                        label: "Keyboard Layout",
                        description: "Keyboard layout (e.g., us, de, fr)",
                        field: FieldDef::KeyboardLayout,
                    },
                    int_item!(
                        "repeat-rate",
                        "input.repeat_rate",
                        "Repeat Rate",
                        "Repeat rate for held-down keys (repeats per second)",
                        1.0,
                        200.0,
                        1.0
                    ),
                    int_item!(
                        "repeat-delay",
                        "input.repeat_delay",
                        "Repeat Delay",
                        "Delay before key repeat starts (milliseconds)",
                        100.0,
                        2000.0,
                        25.0
                    ),
                ],
            },
            GroupDef {
                title: "Mouse",
                items: &[
                    float_item!(
                        "sensitivity",
                        "input.sensitivity",
                        "Sensitivity",
                        "Mouse sensitivity (-1.0 to 1.0)",
                        -1.0,
                        1.0,
                        0.05
                    ),
                    switch_item!(
                        "natural-scroll",
                        "input.natural_scroll",
                        "Natural Scroll",
                        "Invert scrolling direction"
                    ),
                    switch_item!(
                        "left-handed",
                        "input.left_handed",
                        "Left Handed",
                        "Switch left and right mouse buttons"
                    ),
                ],
            },
            GroupDef {
                title: "Touchpad",
                items: &[
                    switch_item!(
                        "disable-while-typing",
                        "input.touchpad.disable_while_typing",
                        "Disable While Typing",
                        "Disable touchpad while typing"
                    ),
                    switch_item!(
                        "tap-to-click",
                        "input.touchpad.tap_to_click",
                        "Tap to Click",
                        "Tap on touchpad to click"
                    ),
                    switch_item!(
                        "touchpad-natural-scroll",
                        "input.touchpad.natural_scroll",
                        "Natural Scroll",
                        "Invert touchpad scrolling direction"
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
                "debug.vfr",
                "VFR",
                "Variable frame rate: render only when something changes (saves battery)"
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

fn number_at(manager: &HyprlandConfigManager, path: &str) -> f64 {
    manager
        .value(path)
        .and_then(|v| v.as_f64())
        .unwrap_or_default()
}

fn string_at(manager: &HyprlandConfigManager, path: &str) -> String {
    manager
        .value(path)
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// The JSON value a number field writes: an integer when Hyprland declares
/// the option as one, so the Lua file reads `4`, not `4.0`.
fn number_value(value: f64, integer: bool) -> Value {
    if integer {
        Value::from(value.round() as i64)
    } else {
        Value::from(value)
    }
}

/// A setting's value as the page shows it, for the reset tooltip.
fn display_value(value: &Value, field: &FieldDef) -> String {
    match (value, field) {
        (Value::Bool(true), _) => "on".to_string(),
        (Value::Bool(false), _) => "off".to_string(),
        (Value::Number(n), _) => format_number(n.as_f64().unwrap_or_default()),
        (Value::String(s), FieldDef::Dropdown { options }) => options
            .iter()
            .find(|(v, _)| v == s)
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| s.clone()),
        (Value::String(s), _) => s.clone(),
        (other, _) => other.to_string(),
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

        let current_kb = string_at(&config_manager, KEYBOARD_LAYOUT_PATH);
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
                    this.set_value(KEYBOARD_LAYOUT_PATH, Value::String(value.to_string()), cx);
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

        let mut number_inputs = HashMap::new();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            let FieldDef::Number {
                min,
                max,
                step,
                integer,
            } = item.field
            else {
                continue;
            };
            let path = item.path;
            let initial = number_at(&config_manager, path);
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(initial)));
            subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = number_at(&this.config_manager.borrow(), path);
                    let next = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    let next = (next * 1000.0).round() / 1000.0;
                    let next = next.clamp(min, max);
                    this.set_value(path, number_value(next, integer), cx);
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
                            if number_at(&this.config_manager.borrow(), path) != value {
                                this.set_value(path, number_value(value, integer), cx);
                            }
                        }
                    }
                    InputEvent::Blur => {
                        // Normalise the text (clamped, trimmed) once editing ends.
                        let value = number_at(&this.config_manager.borrow(), path);
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
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            if let FieldDef::Number { .. } = item.field
                && let Some(input) = self.number_inputs.get(item.id)
            {
                let value = number_at(&self.config_manager.borrow(), item.path);
                input.update(cx, |input, cx| {
                    input.set_value(format_number(value), window, cx);
                });
            }
        }
        let layout: SharedString =
            string_at(&self.config_manager.borrow(), KEYBOARD_LAYOUT_PATH).into();
        self.keyboard_layout_select.update(cx, |select, cx| {
            select.set_selected_value(&layout, window, cx);
        });
    }

    fn set_value(&mut self, path: &str, value: Value, cx: &mut Context<Self>) {
        self.config_manager.borrow_mut().set_value(path, value);
        self.persist(cx);
    }

    /// Drops the user's override so the setting follows Omarchy again, and
    /// shows the value it falls back to.
    fn reset(&mut self, path: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.config_manager.borrow_mut().reset(path);
        self.persist(cx);
        self.sync_inputs(window, cx);
    }

    fn persist(&mut self, cx: &mut Context<Self>) {
        if let Err(e) = self.config_manager.borrow().save() {
            eprintln!("Failed to save Hyprland settings: {e}");
        }
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
        let path = item.path;
        let control: AnyElement = match &item.field {
            FieldDef::Number { .. } => match self.number_inputs.get(item.id) {
                Some(input) => NumberInput::new(input)
                    .small()
                    .w(px(140.))
                    .into_any_element(),
                None => div().into_any_element(),
            },
            FieldDef::Switch => {
                let checked = self
                    .config_manager
                    .borrow()
                    .value(path)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                FocusableSwitch::new(item.id)
                    .checked(checked)
                    .on_change(cx.listener(move |this, value, _, cx| {
                        this.set_value(path, Value::Bool(*value), cx);
                    }))
                    .into_any_element()
            }
            FieldDef::Dropdown { options } => {
                let current = string_at(&self.config_manager.borrow(), path);
                let label = options
                    .iter()
                    .find(|(value, _)| *value == current)
                    .map(|(_, label)| *label)
                    .unwrap_or(current.as_str())
                    .to_string();
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
                                        this.set_value(path, Value::String(value.to_string()), cx);
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

        // A changed setting gets a reset button that shows what it goes back to.
        let reset = {
            let manager = self.config_manager.borrow();
            manager.is_overridden(path).then(|| {
                let fallback = manager
                    .baseline_value(path)
                    .map(|v| display_value(&v, &item.field))
                    .unwrap_or_default();
                Button::new(ElementId::Name(format!("reset-{}", item.id).into()))
                    .icon(Icon::new(Icon::empty()).path("icons/rotate-ccw.svg"))
                    .ghost()
                    .small()
                    .cursor_pointer()
                    .tooltip(format!("Reset to Omarchy's value: {fallback}"))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.reset(path, window, cx);
                    }))
            })
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
                            .child(selectable("desc", item.description)),
                    ),
            )
            .child(
                h_flex()
                    .flex_none()
                    .gap_2()
                    .items_center()
                    .children(reset)
                    .child(control),
            )
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
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(selectable(("page-desc", page_ix), page.description)),
                    )
                    .into_any_element(),
            );
            sections.extend(groups);
        }
        if sections.is_empty() {
            sections.push(
                div()
                    .text_sm()
                    .text_color(muted)
                    .child(selectable("config-empty", "No settings match your search."))
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
    use super::{FieldDef, KEYBOARD_LAYOUT_PATH, PAGES, format_number};
    use crate::system::hyprland_config::baseline::get_path;
    use crate::types::hyprland_config::HyprlandConfig;
    use std::collections::HashSet;

    #[test]
    fn item_ids_are_unique() {
        let mut seen = HashSet::new();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            assert!(seen.insert(item.id), "duplicate item id {}", item.id);
        }
    }

    /// Every item edits a real key of the model, with a control of the
    /// key's type.
    #[test]
    fn every_item_path_exists_in_the_config_with_the_right_type() {
        let defaults = serde_json::to_value(HyprlandConfig::default()).unwrap();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            let value = get_path(&defaults, item.path)
                .unwrap_or_else(|| panic!("{}: no key {} in the model", item.id, item.path));
            match &item.field {
                FieldDef::Number {
                    integer, min, max, ..
                } => {
                    assert!(
                        value.is_number(),
                        "{}: {} is not a number",
                        item.id,
                        item.path
                    );
                    assert_eq!(value.is_i64(), *integer, "{}: integer flag", item.id);
                    assert!(min < max, "{}: empty range", item.id);
                }
                FieldDef::Switch => assert!(value.is_boolean(), "{}: not a bool", item.id),
                FieldDef::Dropdown { options } => {
                    assert!(value.is_string(), "{}: not a string", item.id);
                    assert!(!options.is_empty(), "{}: no options", item.id);
                }
                FieldDef::KeyboardLayout => assert_eq!(item.path, KEYBOARD_LAYOUT_PATH),
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
