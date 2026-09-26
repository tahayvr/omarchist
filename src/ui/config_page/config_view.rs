// The Configuration page: Hyprland and Omarchy settings as a declarative
// table of pages, groups, and items (`pages.rs`), rendered with a
// keyboard-driven page list on the left and native tab stops on the
// right. (gpui-component's `Settings` component keeps its page selection
// private, so it cannot be driven from the keyboard; this page renders the
// same content itself.)
//
// Hyprland items read and write the `HyprlandConfigManager` at once.
// Omarchy items go through `omarchy_settings` backings, which run
// Omarchy's scripts, so their values are read in the background when a
// page opens and written in the background when a control changes.
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
use crate::system::omarchy_settings::{self, Choice};
use crate::ui::config_page::pages::{
    FieldDef, GroupDef, ItemDef, KEYBOARD_LAYOUT_PATH, PageDef, PageGroup, Source, items, page,
    page_count, pages,
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

/// What a background load of an Omarchy page found.
struct PageLoad {
    page: usize,
    values: Vec<(&'static str, Value)>,
    choices: Vec<(&'static str, Vec<Choice>)>,
    hidden: Vec<&'static str>,
}

/// Runs every read of one Omarchy page. Blocking; runs off the UI thread.
fn load_omarchy_page(page_ix: usize) -> PageLoad {
    let mut load = PageLoad {
        page: page_ix,
        values: Vec::new(),
        choices: Vec::new(),
        hidden: Vec::new(),
    };
    let Some(def) = page(page_ix) else {
        return load;
    };
    for item in def.groups.iter().flat_map(|g| g.items) {
        if let Some(when) = item.when
            && !omarchy_settings::condition_holds(when)
        {
            load.hidden.push(item.id);
            continue;
        }
        match (&item.source, &item.field) {
            (Source::Omarchy(backing), _) => {
                if let Some(value) = omarchy_settings::read(backing) {
                    load.values.push((item.id, value));
                }
            }
            (Source::None, FieldDef::Feature { status, .. }) => {
                if let Some(value) = omarchy_settings::read_from(status) {
                    load.values.push((item.id, value));
                }
            }
            _ => {}
        }
        if let FieldDef::DynamicDropdown { options } = &item.field {
            load.choices
                .push((item.id, omarchy_settings::choices(options)));
        }
    }
    load
}

/// Values of the Omarchy-backed items, filled in page by page.
#[derive(Default)]
struct OmarchyState {
    values: HashMap<&'static str, Value>,
    choices: HashMap<&'static str, Vec<Choice>>,
    hidden: HashSet<&'static str>,
    loaded: HashSet<usize>,
    loading: HashSet<usize>,
    errors: HashMap<&'static str, String>,
}

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
    omarchy: OmarchyState,
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
    nav_scroll: ScrollHandle,
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

        let current_kb = config_manager
            .value(KEYBOARD_LAYOUT_PATH)
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
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
                    this.set_hyprland(KEYBOARD_LAYOUT_PATH, Value::String(value.to_string()), cx);
                }
            },
        )];

        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search settings"));
        subscriptions.push(cx.subscribe_in(
            &search,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.load_matching_pages(window, cx);
                    cx.notify();
                }
            },
        ));

        let mut this = Self {
            config_manager: Rc::new(RefCell::new(config_manager)),
            omarchy: OmarchyState::default(),
            keyboard_layout_select,
            number_inputs: HashMap::new(),
            search,
            active_page: 0,
            focus_handle: cx.focus_handle(),
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            nav_scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };

        for item in items() {
            let (min, max, step) = match item.field {
                FieldDef::Number { min, max, step, .. } | FieldDef::Pair { min, max, step, .. } => {
                    (min, max, step)
                }
                _ => continue,
            };
            let initial = this.number_at(item);
            let input =
                cx.new(|cx| InputState::new(window, cx).default_value(format_number(initial)));
            this._subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &NumberInputEvent, window, cx| {
                    let NumberInputEvent::Step(action) = event;
                    let current = this.number_at(item);
                    let next = match action {
                        StepAction::Increment => current + step,
                        StepAction::Decrement => current - step,
                    };
                    let next = (next * 1000.0).round() / 1000.0;
                    let next = next.clamp(min, max);
                    let value = this.number_value_for(item, next);
                    this.write_item(item, value, cx);
                    input.update(cx, |input, cx| {
                        input.set_value(format_number(next), window, cx);
                    });
                },
            ));
            this._subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => {
                        if let Ok(value) = input.read(cx).value().trim().parse::<f64>() {
                            let value = value.clamp(min, max);
                            if this.number_at(item) != value {
                                let value = this.number_value_for(item, value);
                                this.write_item(item, value, cx);
                            }
                        }
                    }
                    InputEvent::Blur => {
                        // Normalise the text (clamped, trimmed) once editing ends.
                        let value = this.number_at(item);
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
            this.number_inputs.insert(item.id, input);
        }

        this.load_page(this.active_page, window, cx);
        this
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
        self.omarchy.loaded.clear();
        self.load_page(self.active_page, window, cx);
        self.load_matching_pages(window, cx);
    }

    // ── values ───────────────────────────────────────────────────────

    /// The item's current value, wherever it lives.
    fn item_value(&self, item: &ItemDef) -> Option<Value> {
        match &item.source {
            Source::Hyprland(path) => self.config_manager.borrow().value(path),
            Source::Omarchy(_) | Source::None => self.omarchy.values.get(item.id).cloned(),
        }
    }

    /// The number a `Number` or `Pair` item shows.
    fn number_at(&self, item: &ItemDef) -> f64 {
        let value = self.item_value(item);
        match item.field {
            FieldDef::Pair { index, .. } => value
                .and_then(|v| v.get(index).and_then(Value::as_f64))
                .unwrap_or_default(),
            _ => value.and_then(|v| v.as_f64()).unwrap_or_default(),
        }
    }

    /// The JSON value a `Number` or `Pair` item writes for `next`.
    fn number_value_for(&self, item: &ItemDef, next: f64) -> Value {
        match item.field {
            FieldDef::Number { integer, .. } => number_value(next, integer),
            FieldDef::Pair { index, .. } => {
                let mut pair = self
                    .item_value(item)
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_else(|| vec![Value::from(0.0), Value::from(0.0)]);
                pair.resize(2, Value::from(0.0));
                pair[index] = Value::from(next);
                Value::Array(pair)
            }
            _ => Value::from(next),
        }
    }

    fn string_at(&self, item: &ItemDef) -> String {
        self.item_value(item)
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
    }

    fn write_item(&mut self, item: &'static ItemDef, value: Value, cx: &mut Context<Self>) {
        match &item.source {
            Source::Hyprland(path) => self.set_hyprland(path, value, cx),
            Source::Omarchy(backing) => {
                // Optimistic: the control shows the choice at once and the
                // read-back after the write corrects it if Omarchy disagreed.
                self.omarchy.values.insert(item.id, value.clone());
                self.omarchy.errors.remove(item.id);
                cx.notify();
                let backing = *backing;
                cx.spawn(async move |this, cx| {
                    let result = cx
                        .background_spawn(async move {
                            let result = omarchy_settings::write(&backing, &value);
                            (result, omarchy_settings::read(&backing))
                        })
                        .await;
                    this.update(cx, |this, cx| {
                        let (result, current) = result;
                        if let Err(e) = result {
                            this.omarchy.errors.insert(item.id, e.to_string());
                        }
                        if let Some(current) = current {
                            this.omarchy.values.insert(item.id, current);
                        }
                        cx.notify();
                    })
                    .ok();
                })
                .detach();
            }
            Source::None => {}
        }
    }

    fn set_hyprland(&mut self, path: &str, value: Value, cx: &mut Context<Self>) {
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

    fn sync_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for item in items() {
            if let FieldDef::Number { .. } | FieldDef::Pair { .. } = item.field
                && let Some(input) = self.number_inputs.get(item.id)
            {
                let value = self.number_at(item);
                input.update(cx, |input, cx| {
                    input.set_value(format_number(value), window, cx);
                });
            }
        }
        let layout: SharedString = self
            .config_manager
            .borrow()
            .value(KEYBOARD_LAYOUT_PATH)
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
            .into();
        self.keyboard_layout_select.update(cx, |select, cx| {
            select.set_selected_value(&layout, window, cx);
        });
    }

    /// Runs an action item's command: in the floating terminal, or in the
    /// background with any failure shown under the item.
    fn run_action(
        &mut self,
        item: &'static ItemDef,
        argv: &'static [&'static str],
        terminal: bool,
        cx: &mut Context<Self>,
    ) {
        self.omarchy.errors.remove(item.id);
        if terminal {
            let argv: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
            if let Err(e) = omarchy_settings::launch_in_terminal(&argv) {
                self.omarchy.errors.insert(item.id, e.to_string());
            }
            cx.notify();
            return;
        }
        cx.spawn(async move |this, cx| {
            let ok = cx
                .background_spawn(async move { omarchy_settings::condition_holds(argv) })
                .await;
            this.update(cx, |this, cx| {
                if !ok {
                    this.omarchy
                        .errors
                        .insert(item.id, format!("{} failed", argv.join(" ")));
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    // ── loading ──────────────────────────────────────────────────────

    /// Reads an Omarchy page's values in the background, once.
    fn load_page(&mut self, page_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(def) = page(page_ix) else {
            return;
        };
        if def.group != PageGroup::Omarchy
            || self.omarchy.loaded.contains(&page_ix)
            || self.omarchy.loading.contains(&page_ix)
        {
            return;
        }
        self.omarchy.loading.insert(page_ix);
        cx.spawn_in(window, async move |this, cx| {
            let load = cx
                .background_spawn(async move { load_omarchy_page(page_ix) })
                .await;
            this.update_in(cx, |this, window, cx| this.apply_load(load, window, cx))
                .ok();
        })
        .detach();
    }

    fn apply_load(&mut self, load: PageLoad, window: &mut Window, cx: &mut Context<Self>) {
        self.omarchy.loading.remove(&load.page);
        self.omarchy.loaded.insert(load.page);
        for (id, value) in load.values {
            self.omarchy.values.insert(id, value);
        }
        for (id, choices) in load.choices {
            self.omarchy.choices.insert(id, choices);
        }
        for id in load.hidden {
            self.omarchy.hidden.insert(id);
        }
        // Number inputs were created before the values were known.
        let updates: Vec<(Entity<InputState>, String)> = page(load.page)
            .into_iter()
            .flat_map(|p| p.groups)
            .flat_map(|g| g.items)
            .filter(|item| matches!(item.field, FieldDef::Number { .. }))
            .filter_map(|item| {
                let input = self.number_inputs.get(item.id)?.clone();
                Some((input, format_number(self.number_at(item))))
            })
            .collect();
        for (input, text) in updates {
            input.update(cx, |input, cx| input.set_value(text, window, cx));
        }
        cx.notify();
    }

    /// While searching, every page with a match is shown, so each needs
    /// its values.
    fn load_matching_pages(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let query = self.query(cx);
        if query.is_empty() {
            return;
        }
        let matching: Vec<usize> = pages()
            .enumerate()
            .filter(|(_, page)| {
                page.groups
                    .iter()
                    .flat_map(|g| g.items)
                    .any(|item| Self::item_matches(item, &query))
            })
            .map(|(ix, _)| ix)
            .collect();
        for ix in matching {
            self.load_page(ix, window, cx);
        }
    }

    // ── navigation ───────────────────────────────────────────────────

    fn set_page(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let index = index.min(page_count() - 1);
        if self.active_page != index {
            self.active_page = index;
            self.scroll.set_offset(Point::default());
            self.nav_scroll.scroll_to_item(Self::nav_child_index(index));
            self.load_page(index, window, cx);
            cx.notify();
        }
    }

    /// The nav shows a heading before each group's pages, so a page's
    /// child index is offset by the headings above it.
    fn nav_child_index(page_ix: usize) -> usize {
        let mut child = 0;
        let mut last_group = None;
        for (ix, page) in pages().enumerate() {
            if last_group != Some(page.group) {
                last_group = Some(page.group);
                child += 1;
            }
            if ix == page_ix {
                return child;
            }
            child += 1;
        }
        child
    }

    fn query(&self, cx: &App) -> String {
        self.search.read(cx).value().trim().to_lowercase()
    }

    fn item_matches(item: &ItemDef, query: &str) -> bool {
        query.is_empty()
            || item.label.to_lowercase().contains(query)
            || item.description.to_lowercase().contains(query)
    }

    // ── rendering ────────────────────────────────────────────────────

    fn render_nav(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_focused = self.nav_focus.is_focused(window);
        let radius = cx.theme().radius;
        let transparent = cx.theme().transparent;
        let muted = cx.theme().muted_foreground;

        let mut children: Vec<AnyElement> = Vec::new();
        let mut last_group = None;
        for (ix, page) in pages().enumerate() {
            if last_group != Some(page.group) {
                last_group = Some(page.group);
                let heading = match page.group {
                    PageGroup::Hyprland => "HYPRLAND",
                    PageGroup::Omarchy => "OMARCHY",
                };
                children.push(
                    div()
                        .px_3()
                        .pt_2()
                        .pb_1()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(muted)
                        .child(heading)
                        .into_any_element(),
                );
            }
            let focused = nav_focused && self.active_page == ix;
            children.push(
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
                                this.set_page(ix, window, cx);
                            }))
                            .render(("config-page-item", ix), window, cx),
                    )
                    .into_any_element(),
            );
        }

        v_flex()
            .id("config-nav")
            .key_context(NAV_CONTEXT)
            .track_focus(&self.nav_focus)
            .on_action(cx.listener(|this, _: &config_nav::Prev, window, cx| {
                this.set_page(this.active_page.saturating_sub(1), window, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Next, window, cx| {
                this.set_page(this.active_page + 1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::First, window, cx| {
                this.set_page(0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Last, window, cx| {
                this.set_page(usize::MAX, window, cx);
            }))
            .on_action(cx.listener(|this, _: &config_nav::Activate, window, cx| {
                focus::focus_first_in(&this.content_focus, window, cx);
            }))
            .w(px(220.))
            .flex_none()
            .h_full()
            .overflow_y_scroll()
            .track_scroll(&self.nav_scroll)
            .gap_1()
            .pr_4()
            .cursor_pointer()
            .children(children)
    }

    fn render_dropdown_button(
        &self,
        item: &'static ItemDef,
        label: String,
        choices: Vec<(String, String, bool)>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let view = cx.entity();
        Button::new(item.id)
            .label(label)
            .dropdown_caret(true)
            .outline()
            .small()
            .cursor_pointer()
            .dropdown_menu(move |menu, _, _| {
                choices.iter().fold(menu, |menu, (value, label, checked)| {
                    let view = view.clone();
                    let value = value.clone();
                    menu.item(
                        PopupMenuItem::new(SharedString::from(label.clone()))
                            .checked(*checked)
                            .on_click(move |_, _, cx| {
                                let value = value.clone();
                                view.update(cx, |this, cx| {
                                    this.write_item(item, Value::String(value), cx);
                                });
                            }),
                    )
                })
            })
            .into_any_element()
    }

    fn render_item(&self, item: &'static ItemDef, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
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
                    .item_value(item)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                FocusableSwitch::new(item.id)
                    .checked(checked)
                    .on_change(cx.listener(move |this, value, _, cx| {
                        this.write_item(item, Value::Bool(*value), cx);
                    }))
                    .into_any_element()
            }
            FieldDef::Dropdown { options } => {
                let current = self.string_at(item);
                let label = options
                    .iter()
                    .find(|(value, _)| *value == current)
                    .map(|(_, label)| label.to_string())
                    .unwrap_or_else(|| current.clone());
                let choices = options
                    .iter()
                    .map(|(value, label)| (value.to_string(), label.to_string(), *value == current))
                    .collect();
                self.render_dropdown_button(item, label, choices, cx)
            }
            FieldDef::DynamicDropdown { .. } => {
                let current = self.string_at(item);
                let choices = self
                    .omarchy
                    .choices
                    .get(item.id)
                    .cloned()
                    .unwrap_or_default();
                let label = choices
                    .iter()
                    .find(|c| c.reads_as == current)
                    .map(|c| c.label.clone())
                    .unwrap_or_else(|| current.clone());
                let choices = choices
                    .into_iter()
                    .map(|c| {
                        let checked = c.reads_as == current;
                        (c.value, c.label, checked)
                    })
                    .collect();
                self.render_dropdown_button(item, label, choices, cx)
            }
            FieldDef::Choice { options } => {
                let current = self
                    .item_value(item)
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
                                        this.write_item(item, Value::from(value), cx);
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
            FieldDef::Action {
                button,
                argv,
                terminal,
            } => {
                let (argv, terminal) = (*argv, *terminal);
                Button::new(item.id)
                    .label(*button)
                    .outline()
                    .small()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.run_action(item, argv, terminal, cx);
                    }))
                    .into_any_element()
            }
            FieldDef::Feature { setup, remove, .. } => {
                let on = self
                    .item_value(item)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let (button, argv): (&str, &'static [&'static str]) = match (on, remove) {
                    (true, Some(remove)) => ("Remove…", remove),
                    _ => ("Set up…", setup),
                };
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(if on {
                                theme.success
                            } else {
                                theme.muted_foreground
                            })
                            .child(if on { "Set up" } else { "Not set up" }),
                    )
                    .child(
                        Button::new(item.id)
                            .label(button)
                            .outline()
                            .small()
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.run_action(item, argv, true, cx);
                            })),
                    )
                    .into_any_element()
            }
        };

        // A changed Hyprland setting gets a reset button that shows what
        // it goes back to.
        let reset = item.hyprland_path().and_then(|path| {
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
        });
        let error = self.omarchy.errors.get(item.id).cloned();

        v_flex()
            .id(ElementId::Name(format!("config-item-{}", item.id).into()))
            .w_full()
            .py_2()
            .gap_1()
            .child(
                h_flex()
                    .w_full()
                    .gap_4()
                    .items_center()
                    .justify_between()
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
                    ),
            )
            .children(error.map(|message| {
                div()
                    .text_xs()
                    .text_color(theme.danger)
                    .child(selectable("error", message))
            }))
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
            .filter(|item| !self.omarchy.hidden.contains(item.id))
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

        let shown: Vec<(usize, &'static PageDef)> = if searching {
            pages().enumerate().collect()
        } else {
            page(self.active_page)
                .map(|p| vec![(self.active_page, p)])
                .unwrap_or_default()
        };

        let mut sections: Vec<AnyElement> = Vec::new();
        for (page_ix, page) in shown {
            let loading =
                page.group == PageGroup::Omarchy && !self.omarchy.loaded.contains(&page_ix);
            let groups: Vec<AnyElement> = if loading {
                Vec::new()
            } else {
                page.groups
                    .iter()
                    .filter_map(|group| self.render_group(page_ix, group, &query, cx))
                    .collect()
            };
            if groups.is_empty() && !loading {
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
            if loading {
                sections.push(
                    div()
                        .text_sm()
                        .text_color(muted)
                        .child(selectable(("page-loading", page_ix), "Reading…"))
                        .into_any_element(),
                );
            }
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
    use super::{FieldDef, KEYBOARD_LAYOUT_PATH, Source, format_number, items};
    use crate::system::hyprland_config::baseline::get_path;
    use crate::types::hyprland_config::HyprlandConfig;
    use std::collections::HashSet;

    #[test]
    fn item_ids_are_unique() {
        let mut seen = HashSet::new();
        for item in items() {
            assert!(seen.insert(item.id), "duplicate item id {}", item.id);
        }
    }

    /// Every Hyprland item edits a real key of the model, with a control of
    /// the key's type; every Omarchy item has a control that fits its source.
    #[test]
    fn every_item_fits_its_source() {
        let defaults = serde_json::to_value(HyprlandConfig::default()).unwrap();
        for item in items() {
            match (&item.source, &item.field) {
                (Source::Hyprland(path), field) => {
                    let value = get_path(&defaults, path)
                        .unwrap_or_else(|| panic!("{}: no key {} in the model", item.id, path));
                    match field {
                        FieldDef::Number {
                            integer, min, max, ..
                        } => {
                            assert!(value.is_number(), "{}: {} is not a number", item.id, path);
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
                        FieldDef::KeyboardLayout => assert_eq!(*path, KEYBOARD_LAYOUT_PATH),
                        FieldDef::DynamicDropdown { .. }
                        | FieldDef::Action { .. }
                        | FieldDef::Feature { .. } => {
                            panic!("{}: a Hyprland item needs a value control", item.id)
                        }
                    }
                }
                (Source::Omarchy(_), field) => assert!(
                    matches!(
                        field,
                        FieldDef::Number { .. }
                            | FieldDef::Switch
                            | FieldDef::Dropdown { .. }
                            | FieldDef::DynamicDropdown { .. }
                    ),
                    "{}: an Omarchy item needs a value control",
                    item.id
                ),
                (Source::None, field) => assert!(
                    matches!(field, FieldDef::Action { .. } | FieldDef::Feature { .. }),
                    "{}: an item without a value must be an action or a feature",
                    item.id
                ),
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
        for item in items() {
            let Some(path) = item.hyprland_path() else {
                continue;
            };
            let Some(option) = described.get(path) else {
                problems.push(format!("{}: {} is not described", item.id, path));
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

    /// Every command an Omarchy item runs or reads exists on this machine
    /// (skipped without Omarchy).
    #[test]
    fn omarchy_items_reference_installed_commands() {
        use crate::system::omarchy_settings::{Read, Write};
        if !std::path::Path::new("/usr/share/omarchy/bin").is_dir() {
            eprintln!("skipping: no Omarchy");
            return;
        }
        let exists = |program: &str| {
            std::env::var_os("PATH").is_some_and(|path| {
                std::env::split_paths(&path).any(|dir| dir.join(program).is_file())
            }) || std::path::Path::new("/usr/share/omarchy/bin")
                .join(program)
                .is_file()
        };
        let mut missing = Vec::new();
        let mut check = |program: &str, id: &str| {
            if !exists(program) {
                missing.push(format!("{id}: {program}"));
            }
        };
        for item in items() {
            if let Some(when) = item.when {
                check(when[0], item.id);
            }
            match &item.source {
                Source::Omarchy(backing) => {
                    if let Read::Command { argv, .. } = backing.read {
                        check(argv[0], item.id);
                    }
                    match backing.write {
                        Write::Bool { on, off } => {
                            for argv in on.iter().chain(off.iter()) {
                                check(argv[0], item.id);
                            }
                        }
                        Write::ToggleIfDifferent(argv)
                        | Write::Command(argv)
                        | Write::CommandJson(argv)
                        | Write::Terminal(argv) => check(argv[0], item.id),
                        Write::ShellJson(_) => {}
                    }
                }
                Source::None => match &item.field {
                    FieldDef::Action { argv, .. } => check(argv[0], item.id),
                    FieldDef::Feature {
                        status,
                        setup,
                        remove,
                    } => {
                        if let Read::Command { argv, .. } = status {
                            check(argv[0], item.id);
                        }
                        check(setup[0], item.id);
                        if let Some(remove) = remove {
                            check(remove[0], item.id);
                        }
                    }
                    _ => {}
                },
                Source::Hyprland(_) => {}
            }
        }
        assert!(
            missing.is_empty(),
            "missing commands:\n{}",
            missing.join("\n")
        );
    }

    #[test]
    fn numbers_format_without_noise() {
        assert_eq!(format_number(2.0), "2");
        assert_eq!(format_number(0.1), "0.1");
        assert_eq!(format_number(0.30000000000000004), "0.3");
        assert_eq!(format_number(-1.0), "-1");
    }
}
