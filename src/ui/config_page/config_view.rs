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
use crate::ui::config_page::pages::{
    FieldDef, GroupDef, ItemDef, KEYBOARD_LAYOUT_PATH, PAGES, PageDef,
};
use crate::ui::focus::{self, FocusSection, FocusableSwitch};
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "ConfigPage";
pub const NAV_CONTEXT: &str = "ConfigNav";
pub const CONTENT_CONTEXT: &str = "ConfigContent";

pub mod config_nav {
    gpui::actions!(config_nav, [Prev, Next, First, Last, Activate, Back]);
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        let text = format!("{value:.3}");
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// The number a `Number` or `Pair` item shows.
fn number_at(manager: &HyprlandConfigManager, item: &ItemDef) -> f64 {
    let value = manager.value(item.path);
    match item.field {
        FieldDef::Pair { index, .. } => value
            .and_then(|v| v.get(index).and_then(Value::as_f64))
            .unwrap_or_default(),
        _ => value.and_then(|v| v.as_f64()).unwrap_or_default(),
    }
}

/// The JSON value a `Number` or `Pair` item writes for `next`.
fn number_value_for(manager: &HyprlandConfigManager, item: &ItemDef, next: f64) -> Value {
    match item.field {
        FieldDef::Number { integer, .. } => number_value(next, integer),
        FieldDef::Pair { index, .. } => {
            let mut pair = manager
                .value(item.path)
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_else(|| vec![Value::from(0.0), Value::from(0.0)]);
            pair.resize(2, Value::from(0.0));
            pair[index] = Value::from(next);
            Value::Array(pair)
        }
        _ => Value::from(next),
    }
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
        (Value::Number(n), FieldDef::Choice { options }) => n
            .as_i64()
            .and_then(|n| options.iter().find(|(v, _)| *v == n))
            .map(|(_, label)| label.to_string())
            .unwrap_or_else(|| n.to_string()),
        (Value::Number(n), _) => format_number(n.as_f64().unwrap_or_default()),
        (Value::Array(pair), FieldDef::Pair { index, .. }) => pair
            .get(*index)
            .and_then(Value::as_f64)
            .map(format_number)
            .unwrap_or_default(),
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
            let (min, max, step) = match item.field {
                FieldDef::Number { min, max, step, .. } | FieldDef::Pair { min, max, step, .. } => {
                    (min, max, step)
                }
                _ => continue,
            };
            let initial = number_at(&config_manager, item);
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(initial)));
            subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = number_at(&this.config_manager.borrow(), item);
                    let next = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    let next = (next * 1000.0).round() / 1000.0;
                    let next = next.clamp(min, max);
                    let value = number_value_for(&this.config_manager.borrow(), item, next);
                    this.set_value(item.path, value, cx);
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
                            if number_at(&this.config_manager.borrow(), item) != value {
                                let value =
                                    number_value_for(&this.config_manager.borrow(), item, value);
                                this.set_value(item.path, value, cx);
                            }
                        }
                    }
                    InputEvent::Blur => {
                        // Normalise the text (clamped, trimmed) once editing ends.
                        let value = number_at(&this.config_manager.borrow(), item);
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
            if let FieldDef::Number { .. } | FieldDef::Pair { .. } = item.field
                && let Some(input) = self.number_inputs.get(item.id)
            {
                let value = number_at(&self.config_manager.borrow(), item);
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
            FieldDef::Number { .. } | FieldDef::Pair { .. } => {
                match self.number_inputs.get(item.id) {
                    Some(input) => NumberInput::new(input)
                        .small()
                        .w(px(140.))
                        .into_any_element(),
                    None => div().into_any_element(),
                }
            }
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
            FieldDef::Choice { options } => {
                let current = self
                    .config_manager
                    .borrow()
                    .value(path)
                    .and_then(|v| v.as_i64())
                    .unwrap_or_default();
                let label = options
                    .iter()
                    .find(|(value, _)| *value == current)
                    .map(|(_, label)| label.to_string())
                    .unwrap_or_else(|| current.to_string());
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
                            let value = *value;
                            menu.item(PopupMenuItem::new(*label).checked(checked).on_click(
                                move |_, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.set_value(path, Value::from(value), cx);
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
                FieldDef::Pair {
                    index, min, max, ..
                } => {
                    assert!(
                        value.as_array().is_some_and(|a| a.len() == 2),
                        "{}: not a pair",
                        item.id
                    );
                    assert!(*index < 2, "{}: index", item.id);
                    assert!(min < max, "{}: empty range", item.id);
                }
                FieldDef::Switch => assert!(value.is_boolean(), "{}: not a bool", item.id),
                FieldDef::Dropdown { options } => {
                    assert!(value.is_string(), "{}: not a string", item.id);
                    let default = value.as_str().unwrap();
                    assert!(
                        options.iter().any(|(v, _)| *v == default),
                        "{}: default {default:?} is not an option",
                        item.id
                    );
                }
                FieldDef::Choice { options } => {
                    assert!(value.is_i64(), "{}: not an integer", item.id);
                    let default = value.as_i64().unwrap();
                    assert!(
                        options.iter().any(|(v, _)| *v == default),
                        "{}: default {default} is not an option",
                        item.id
                    );
                }
                FieldDef::KeyboardLayout => assert_eq!(item.path, KEYBOARD_LAYOUT_PATH),
            }
        }
    }

    /// Ranges and choices match what the running compositor declares.
    #[test]
    fn ranges_and_choices_match_hyprland_descriptions() {
        use crate::system::hyprland_config::hyprctl_reader::defaults_audit::described_options;
        let Some(described) = described_options() else {
            eprintln!("skipping: no hyprctl");
            return;
        };
        let mut problems = Vec::new();
        for item in PAGES.iter().flat_map(|p| p.groups).flat_map(|g| g.items) {
            let Some(option) = described.get(item.path) else {
                problems.push(format!("{}: {} is not described", item.id, item.path));
                continue;
            };
            let declared = (option["min"].as_f64(), option["max"].as_f64());
            match &item.field {
                FieldDef::Number { min, max, .. } => {
                    if let (Some(lo), Some(hi)) = declared
                        && (*min < lo || *max > hi)
                    {
                        problems.push(format!(
                            "{}: range {min}..{max} outside Hyprland's {lo}..{hi}",
                            item.id
                        ));
                    }
                }
                FieldDef::Choice { options } => {
                    if let Some(map) = option["map"].as_array() {
                        let declared: Vec<i64> = map
                            .iter()
                            .filter_map(|e| e.as_object()?.values().next()?.as_i64())
                            .collect();
                        for (value, _) in options.iter() {
                            if !declared.contains(value) {
                                problems.push(format!(
                                    "{}: value {value} is not in Hyprland's map",
                                    item.id
                                ));
                            }
                        }
                        for value in &declared {
                            if !options.iter().any(|(v, _)| v == value) {
                                problems.push(format!(
                                    "{}: Hyprland's value {value} is missing",
                                    item.id
                                ));
                            }
                        }
                    } else if let (Some(lo), Some(hi)) = declared {
                        for (value, _) in options.iter() {
                            if (*value as f64) < lo || (*value as f64) > hi {
                                problems
                                    .push(format!("{}: value {value} outside {lo}..{hi}", item.id));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn numbers_format_without_noise() {
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(0.1), "0.1");
        assert_eq!(format_number(0.30000000000000004), "0.3");
        assert_eq!(format_number(-1.0), "-1");
    }
}
