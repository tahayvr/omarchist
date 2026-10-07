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
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable as _, Icon, IconName, IndexPath, Sizable as _,
    button::{Button, ButtonVariants as _},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput},
    menu::{DropdownMenu, PopupMenuItem},
    scroll::ScrollableElement as _,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    sidebar::{SidebarItem, SidebarMenuItem},
    v_flex,
};

use serde_json::Value;

use crate::system::config::hypr_setup::{self, HOOK_RESTORED_MESSAGE};
use crate::system::hyprland_config::HyprlandConfigManager;
use crate::system::hyprland_config::manager;
use crate::system::omarchy_settings::{self, Choice};
use crate::ui::config_page::pages::{
    FieldDef, GroupDef, ItemDef, KEYBOARD_LAYOUT_PATH, PageDef, PageGroup, Source, items, page,
    page_count, pages,
};
use crate::ui::explain::{explain, explained_label};
use crate::ui::focus::{self, FocusSection, FocusableSwitch};
use crate::ui::heading;
use crate::ui::palette::{self, Area};
use crate::ui::text::selectable;
/// A dynamic dropdown with more choices than this is a searchable select.
const LONG_LIST: usize = 40;

const KEY_CONTEXT: &str = "ConfigPage";
pub const NAV_CONTEXT: &str = "ConfigNav";
pub const CONTENT_CONTEXT: &str = "ConfigContent";

pub mod config_nav {
    gpui::actions!(
        config_nav,
        [Prev, Next, First, Last, Activate, Back, FocusSearch]
    );
}

/// The shortest text that reads back as the same number, so a value such
/// as 0.0117 is shown as it is (not 0.012) and focusing the field and
/// leaving it again changes nothing.
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("{}", value as i64);
    }
    // Round away float noise (0.30000000000000004) but keep every digit
    // the compositor reported.
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod format_tests {
    use super::format_number;

    #[test]
    fn numbers_keep_their_digits_and_lose_float_noise() {
        assert_eq!(format_number(5.0), "5");
        assert_eq!(format_number(0.0117), "0.0117");
        assert_eq!(format_number(0.8916), "0.8916");
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(-0.5), "-0.5");
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

/// Whether a page has anything that is read in the background: Omarchy
/// items, feature statuses, or conditions.
fn page_needs_load(page: &PageDef) -> bool {
    page.groups.iter().flat_map(|g| g.items).any(|item| {
        item.when.is_some()
            || matches!(item.source, Source::Omarchy(_))
            || matches!(item.field, FieldDef::Feature { .. })
    })
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
    /// Items whose change runs in Omarchy's terminal; read back once it
    /// is done, not the moment the terminal opens.
    pending: HashSet<&'static str>,
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
    /// A searchable select for each dynamic dropdown with more choices than
    /// a menu can show well (the timezone list), built once its choices load.
    long_selects: HashMap<&'static str, Entity<SelectState<SearchableVec<KeyboardLayoutItem>>>>,
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
    /// Loading or saving the Hyprland settings failed; shown above the page.
    error: Option<String>,
    /// A save found `hyprland.lua` without Omarchist's require line and put
    /// it back; shown above the page until the view is rebuilt.
    hook_restored: bool,
    /// The baseline changed off the UI thread; the number fields are
    /// brought up to date on the next render (which has the window).
    inputs_stale: bool,
    /// A terminal action finished; the Omarchy values are re-read on the
    /// next render (which has the window the loads need).
    stale_after_terminal: bool,
    /// The Hyprland settings could not be loaded (state file unreadable, no
    /// compositor): every Hyprland control is disabled.
    hyprland_unavailable: bool,
    /// The first load (a `hyprctl` batch and a scan of `hyprland.lua`) is
    /// running off the UI thread; Hyprland pages show "Reading…" until it
    /// lands.
    hyprland_loading: bool,
    /// What `hyprctl configerrors` reported after the last save.
    config_errors: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl ConfigView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // The settings themselves are read in the background (below), so
        // the page appears at once; until they land the controls are
        // disabled like an unavailable compositor's.
        let compositor = HyprlandConfigManager::compositor_reachable();
        let config_manager = HyprlandConfigManager::unavailable();
        let error = (!compositor).then(|| {
            "Hyprland is not running (or hyprctl is missing), so its settings cannot \
             be read or changed here"
                .to_string()
        });
        let hyprland_unavailable = true;

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

        // vconsole's layout is read with the page; until then the select
        // shows Hyprland's live value.
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
                if let SelectEvent::Confirm(Some(value)) = event
                    && let Some(item) = items().find(|item| item.id == "kb-layout")
                    && this.string_at(item) != value.as_ref()
                {
                    this.write_item(item, Value::String(value.to_string()), cx);
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
            long_selects: HashMap::new(),
            number_inputs: HashMap::new(),
            search,
            active_page: 0,
            focus_handle: cx.focus_handle(),
            nav_focus: focus::tab_stop(cx),
            content_focus: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            nav_scroll: ScrollHandle::new(),
            error,
            hook_restored: false,
            inputs_stale: false,
            stale_after_terminal: false,
            hyprland_unavailable,
            hyprland_loading: compositor,
            config_errors: None,
            _subscriptions: subscriptions,
        };
        if compositor {
            this.load_hyprland(cx);
        }

        for item in items() {
            let (min, max, step) = match item.field {
                FieldDef::Number { min, max, step, .. } | FieldDef::Pair { min, max, step, .. } => {
                    (min, max, step)
                }
                _ => continue,
            };
            let initial = this.number_at(item);
            // Set by the step hook right before the state changes its text,
            // so the change handler can tell a step from typing: a step is
            // committed at once, typed text on Enter or blur, so a half-typed
            // "0." never reaches the compositor.
            let stepped = Rc::new(Cell::new(false));
            let input = cx.new(|cx| {
                let stepped = stepped.clone();
                InputState::new(window, cx)
                    .default_value(format_number(initial))
                    .step_by(move |_, _, _| {
                        stepped.set(true);
                        step
                    })
                    .min(min)
                    .max(max)
            });
            this._subscriptions.push(cx.subscribe_in(
                &input,
                window,
                move |this, input, event: &InputEvent, window, cx| match event {
                    InputEvent::Change => {
                        // The hook fires even when a step at the range's
                        // edge changes nothing; then the next Change is
                        // typing, which waits for Enter or blur.
                        if stepped.take() {
                            let typed = input.read(cx).value().trim().parse::<f64>().ok();
                            let current = this.number_at(item);
                            let is_step = typed.is_some_and(|t| {
                                (t - current).abs() <= step + 1e-9 && t != current
                            });
                            if is_step {
                                this.commit_number(item, input, cx);
                            }
                        }
                    }
                    InputEvent::PressEnter { .. } => {
                        this.commit_number(item, input, cx);
                        // Show what was written: a clamped or rounded value,
                        // not the text as typed.
                        let text = format_number(this.number_at(item));
                        input.update(cx, |input, cx| {
                            if input.value() != text {
                                input.set_value(text, window, cx);
                            }
                        });
                    }
                    // The window loses keyboard focus for a moment on every
                    // `hyprctl reload` a save triggers, which blurs the field
                    // while it stays the focused element; only a blur that
                    // moved focus elsewhere commits.
                    InputEvent::Blur => {
                        if input.read(cx).focus_handle(cx).is_focused(window) {
                            return;
                        }
                        this.commit_number(item, input, cx);
                        let text = format_number(this.number_at(item));
                        input.update(cx, |input, cx| {
                            if input.value() != text {
                                input.set_value(text, window, cx);
                            }
                        });
                    }
                    InputEvent::Focus => {}
                },
            ));
            this.number_inputs.insert(item.id, input);
        }

        this.load_page(this.active_page, window, cx);
        // A page whose rows are disabled by a switch on another page (No
        // Gaps on General disables Rounding on Appearance) needs that switch
        // read whichever page opens first.
        let gating: Vec<usize> = pages()
            .enumerate()
            .filter(|(_, page)| {
                page.groups
                    .iter()
                    .flat_map(|g| g.items)
                    .any(|item| items().any(|other| other.disabled_by == Some(item.id)))
            })
            .map(|(ix, _)| ix)
            .collect();
        for page_ix in gating {
            this.load_page(page_ix, window, cx);
        }
        this
    }

    /// Reads every Omarchy value and feature status again: they change from
    /// Omarchy's own menu and the terminal actions run from this page, which
    /// this page cannot observe.
    pub fn refresh_omarchy_values(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.omarchy.loaded.clear();
        self.load_page(self.active_page, window, cx);
        self.load_matching_pages(window, cx);
    }

    /// Focuses the page list, the page's first control.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.nav_focus.focus(window, cx);
    }

    /// Re-reads the saved configuration from disk and refreshes the fields.
    pub fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if HyprlandConfigManager::compositor_reachable() {
            self.error = None;
            self.hyprland_loading = true;
            self.load_hyprland(cx);
        } else {
            *self.config_manager.borrow_mut() = HyprlandConfigManager::unavailable();
            self.hyprland_unavailable = true;
            self.error = Some(
                "Hyprland is not running (or hyprctl is missing), so its settings cannot \
                 be read or changed here"
                    .to_string(),
            );
        }
        cx.notify();
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

    /// Writes the number typed or stepped into `input`, clamped to the
    /// item's range, when it differs from the value in effect.
    fn commit_number(
        &mut self,
        item: &'static ItemDef,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) {
        let (min, max) = match item.field {
            FieldDef::Number { min, max, .. } | FieldDef::Pair { min, max, .. } => (min, max),
            _ => return,
        };
        let Ok(typed) = input.read(cx).value().trim().parse::<f64>() else {
            return;
        };
        let current = self.number_at(item);
        // Unchanged, or the same text the field showed for the current
        // value: nothing to write (and no reload to trigger).
        if typed == current || input.read(cx).value().trim() == format_number(current) {
            return;
        }
        let value = self.number_value_for(item, typed.clamp(min, max));
        self.write_item(item, value, cx);
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
                let in_terminal = matches!(backing.write, omarchy_settings::Write::Terminal(_));
                // A toggle Omarchy applies through its own Hyprland reload
                // changes the live values this page shows as the baseline.
                let reloads_hyprland = matches!(
                    backing.write,
                    omarchy_settings::Write::Bool { .. }
                        | omarchy_settings::Write::ToggleIfDifferent(_)
                        | omarchy_settings::Write::X11Keymap
                );
                if in_terminal {
                    self.omarchy.pending.insert(item.id);
                }
                cx.spawn(async move |this, cx| {
                    let wanted = value.clone();
                    let result = cx
                        .background_spawn(async move {
                            let result = omarchy_settings::write(&backing, &wanted);
                            let current = if in_terminal {
                                // The terminal has only just opened; wait
                                // for the value to change (or give up).
                                omarchy_settings::read_until(
                                    &backing,
                                    &wanted,
                                    Duration::from_secs(300),
                                )
                            } else {
                                omarchy_settings::read(&backing)
                            };
                            (result, current)
                        })
                        .await;
                    this.update(cx, |this, cx| {
                        let (result, current) = result;
                        this.omarchy.pending.remove(item.id);
                        // A script that did its work but ended with a slow
                        // `hyprctl reload` exits non-zero; the read-back is
                        // what counts.
                        let applied = current
                            .as_ref()
                            .is_some_and(|c| omarchy_settings::same_value(c, &value));
                        if let Err(e) = result
                            && !applied
                        {
                            this.omarchy.errors.insert(item.id, e.to_string());
                        }
                        if let Some(current) = current {
                            this.omarchy.values.insert(item.id, current);
                        }
                        cx.notify();
                    })
                    .ok();
                    if reloads_hyprland {
                        this.update(cx, |this, cx| this.reload_baseline(cx)).ok();
                    }
                })
                .detach();
            }
            Source::None => {}
        }
    }

    /// The first read of the Hyprland settings, off the UI thread; a failure
    /// leaves the Hyprland controls disabled with the reason.
    fn load_hyprland(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async { HyprlandConfigManager::load() })
                .await;
            this.update(cx, |this, cx| {
                this.hyprland_loading = false;
                match loaded {
                    Ok(manager) => {
                        *this.config_manager.borrow_mut() = manager;
                        this.hyprland_unavailable = false;
                        this.inputs_stale = true;
                    }
                    Err(e) => {
                        this.error = Some(format!("Hyprland settings could not be loaded: {e}"));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Re-reads the compositor's values (the baseline) off the UI thread,
    /// keeping this page's overrides, after something outside this page
    /// reloaded Hyprland.
    fn reload_baseline(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async { HyprlandConfigManager::load() })
                .await;
            this.update(cx, |this, cx| {
                if let Ok(manager) = loaded {
                    *this.config_manager.borrow_mut() = manager;
                    this.inputs_stale = true;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    fn set_hyprland(&mut self, path: &str, value: Value, cx: &mut Context<Self>) {
        self.config_manager.borrow_mut().set_value(path, value);
        self.persist(cx);
    }

    /// Drops the user's override so the setting follows Omarchy again, and
    /// shows the value it falls back to.
    fn reset(
        &mut self,
        item: &'static ItemDef,
        path: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // One component of a pair goes back to its baseline value; the
        // other keeps the user's.
        if let FieldDef::Pair { index, .. } = item.field
            && let Some(Value::Array(base)) = self.config_manager.borrow().baseline_value(path)
            && let Some(current) = self.item_value(item).and_then(|v| v.as_array().cloned())
            && let Some(component) = base.get(index).cloned()
        {
            let mut pair = current;
            pair.resize(2, Value::from(0.0));
            pair[index] = component;
            self.config_manager
                .borrow_mut()
                .set_value(path, Value::Array(pair));
        } else {
            self.config_manager.borrow_mut().reset(path);
        }
        self.persist(cx);
        self.sync_inputs(window, cx);
    }

    /// Keeps the keyboard on the page when the control it was on goes away
    /// (a reset button unmounts): the content column's handle, from which
    /// Tab continues down the page and Escape returns to the page list.
    fn focus_row(&self, _item: &'static ItemDef, window: &mut Window, cx: &mut Context<Self>) {
        self.content_focus.focus(window, cx);
    }

    fn persist(&mut self, cx: &mut Context<Self>) {
        let saved = self.config_manager.borrow().save();
        if saved.is_ok() {
            // Reload, then ask what Hyprland rejected: a value another
            // Hyprland version does not know must not look applied.
            cx.spawn(async move |this, cx| {
                let verdict = cx
                    .background_spawn(async { manager::reload_hyprland_checked() })
                    .await;
                this.update(cx, |this, cx| {
                    this.config_errors = match verdict {
                        Ok(errors) => errors,
                        Err(e) => Some(e.to_string()),
                    };
                    cx.notify();
                })
                .ok();
            })
            .detach();
        }
        match saved {
            Ok(hook_restored) => {
                if self
                    .error
                    .as_deref()
                    .is_some_and(|e| e.starts_with("Saving"))
                {
                    self.error = None;
                }
                if hook_restored {
                    self.hook_restored = true;
                }
            }
            Err(e) => {
                if self.error.is_none() {
                    self.error = Some(format!("Saving the Hyprland settings failed: {e}"));
                }
            }
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
            .omarchy
            .values
            .get("kb-layout")
            .and_then(|v| v.as_str().map(str::to_string))
            .or_else(|| {
                self.config_manager
                    .borrow()
                    .value(KEYBOARD_LAYOUT_PATH)
                    .and_then(|v| v.as_str().map(str::to_string))
            })
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
            match omarchy_settings::launch_in_terminal(&argv) {
                Ok(()) => {
                    self.guard_require_line(cx);
                    self.refresh_after_terminal(cx);
                }
                Err(e) => {
                    self.omarchy.errors.insert(item.id, e.to_string());
                }
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

    /// A terminal action may replace `hyprland.lua` (Omarchy's reset and
    /// migration scripts do), which drops the require line and leaves every
    /// setting on this page inert. Watch for the change and put it back.
    fn guard_require_line(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            let restored = cx
                .background_spawn(async move {
                    hypr_setup::restore_require_line_after_change(Duration::from_secs(600))
                })
                .await;
            if restored {
                this.update(cx, |this, cx| {
                    this.hook_restored = true;
                    manager::reload_hyprland();
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    // ── loading ──────────────────────────────────────────────────────

    /// Reads an Omarchy page's values in the background, once.
    fn load_page(&mut self, page_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(def) = page(page_ix) else {
            return;
        };
        if !page_needs_load(def)
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

    /// Re-reads the Omarchy values once the terminal an action opened has
    /// closed: that is when the install, removal or setup is done.
    fn refresh_after_terminal(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            cx.background_spawn(async {
                omarchy_settings::wait_for_terminal_to_close(Duration::from_secs(900))
            })
            .await;
            this.update(cx, |this, cx| {
                this.stale_after_terminal = true;
                cx.notify();
            })
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
            if choices.len() > LONG_LIST {
                self.ensure_long_select(id, &choices, window, cx);
            }
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
        if let Some(layout) = self
            .omarchy
            .values
            .get("kb-layout")
            .and_then(|v| v.as_str())
        {
            let layout: SharedString = layout.to_string().into();
            self.keyboard_layout_select.update(cx, |select, cx| {
                select.set_selected_value(&layout, window, cx);
            });
        }
        let selected: Vec<(
            Entity<SelectState<SearchableVec<KeyboardLayoutItem>>>,
            SharedString,
        )> = self
            .long_selects
            .iter()
            .filter_map(|(id, select)| {
                let value = self.omarchy.values.get(id)?.as_str()?.to_string();
                Some((select.clone(), value.into()))
            })
            .collect();
        for (select, value) in selected {
            select.update(cx, |select, cx| {
                select.set_selected_value(&value, window, cx)
            });
        }
        cx.notify();
    }

    /// Builds the searchable select for a long dynamic dropdown the first
    /// time its choices arrive; picking a choice writes it like the menu.
    fn ensure_long_select(
        &mut self,
        id: &'static str,
        choices: &[omarchy_settings::Choice],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.long_selects.contains_key(id) {
            return;
        }
        let Some(item) = items().find(|item| item.id == id) else {
            return;
        };
        let entries: Vec<KeyboardLayoutItem> = choices
            .iter()
            .filter(|c| c.available)
            .map(|c| KeyboardLayoutItem {
                value: c.value.clone().into(),
                label: c.label.clone().into(),
            })
            .collect();
        let current = self.string_at(item);
        let initial = entries
            .iter()
            .position(|e| e.value.as_ref() == current.as_str())
            .map(|i| IndexPath::default().row(i));
        let select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(entries), initial, window, cx).searchable(true)
        });
        self._subscriptions.push(cx.subscribe_in(
            &select,
            window,
            move |this, _, event: &SelectEvent<SearchableVec<KeyboardLayoutItem>>, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = event
                    && this.string_at(item) != value.as_ref()
                {
                    this.write_item(item, Value::String(value.to_string()), cx);
                }
            },
        ));
        self.long_selects.insert(id, select);
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
            || Self::folds(item.label).contains(&Self::folds(query))
            || Self::folds(item.description).contains(&Self::folds(query))
    }

    /// Lower case without spaces or hyphens, so "wifi" finds "Wi-Fi".
    fn folds(text: &str) -> String {
        text.chars()
            .filter(|c| !matches!(c, ' ' | '-' | '_'))
            .flat_map(char::to_lowercase)
            .collect()
    }

    // ── rendering ────────────────────────────────────────────────────

    fn render_nav(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let nav_focused = self.nav_focus.is_focused(window);
        let radius = cx.theme().radius;
        let transparent = cx.theme().transparent;

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
                    h_flex()
                        .gap_2()
                        .px_3()
                        .pt_2()
                        .pb_1()
                        .child(palette::dot(Area::of_group(page.group).accent(cx)))
                        .child(heading::section(heading, cx))
                        .into_any_element(),
                );
            }
            let focused = nav_focused && self.active_page == ix;
            let icon = palette::icon(page.icon).text_color(Area::of_group(page.group).accent(cx));
            children.push(
                div()
                    .id(("config-page", ix))
                    .rounded(radius)
                    .border_1()
                    .border_color(focus::focus_border(focused, transparent, cx))
                    .child(
                        SidebarMenuItem::new(page.title)
                            .icon(icon)
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
        let disabled = matches!(item.source, Source::Hyprland(_)) && self.hyprland_unavailable;
        Button::new(item.id)
            .label(label)
            .dropdown_caret(true)
            .outline()
            .small()
            .disabled(disabled)
            .cursor_pointer()
            .dropdown_menu(move |menu, _, _| {
                // Long lists (every monospace font) scroll instead of
                // running off the window.
                let menu = menu.scrollable(true).max_h(px(400.));
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
        // An Omarchy toggle that overrides this setting is on.
        let disabled = item
            .disabled_by
            .and_then(|id| self.omarchy.values.get(id))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
            // Nothing Hyprland-backed can be saved while the settings could
            // not be loaded; the controls must not pretend otherwise.
            || (matches!(item.source, Source::Hyprland(_)) && self.hyprland_unavailable);
        let control: AnyElement = match &item.field {
            FieldDef::Number { .. } | FieldDef::Pair { .. } => {
                match self.number_inputs.get(item.id) {
                    Some(input) => NumberInput::new(input)
                        .small()
                        .disabled(disabled)
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
                    .disabled(disabled)
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
            FieldDef::DynamicDropdown { .. } if self.long_selects.contains_key(item.id) => {
                let select = &self.long_selects[item.id];
                div()
                    .w(px(260.))
                    .flex_none()
                    .child(Select::new(select).search_placeholder("Search…").small())
                    .into_any_element()
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
                    .unwrap_or_else(|| {
                        if current.is_empty() {
                            "Not set".to_string()
                        } else {
                            current.clone()
                        }
                    });
                let choices = choices
                    .into_iter()
                    .filter(|c| c.available)
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
        // it goes back to. The layout row is Omarchy's now, but an override
        // from an older version still applies until it is reset here.
        let legacy_layout =
            matches!(item.field, FieldDef::KeyboardLayout).then_some(KEYBOARD_LAYOUT_PATH);
        let reset = item.hyprland_path().or(legacy_layout).and_then(|path| {
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
                        this.reset(item, path, window, cx);
                        // The button disappears with the override; keep the
                        // keyboard on the row's control (the number field,
                        // or the row's own focus target for the rest).
                        match this.number_inputs.get(item.id) {
                            Some(input) => input.read(cx).focus_handle(cx).focus(window, cx),
                            None => this.focus_row(item, window, cx),
                        }
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
                    .child(div().flex_1().min_w_0().child(explain(
                        SharedString::from(format!("desc-{}", item.id)),
                        explained_label(item.label, cx),
                        item.description,
                    )))
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
            .children(self.omarchy.pending.contains(item.id).then(|| {
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Running in the terminal…")
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
            .filter(|item| {
                !matches!(item.source, Source::Omarchy(_))
                    || self.omarchy.loaded.contains(&page_ix)
                    || matches!(item.field, FieldDef::KeyboardLayout)
            })
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
                    .title(heading::section(group.title, cx))
                    .children(items),
            )
            .into_any_element(),
        )
    }

    /// Load and save errors, the restored require line, and what Hyprland
    /// rejected after the last save.
    fn render_banners(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let theme = cx.theme();
        let mut lines: Vec<AnyElement> = Vec::new();
        if let Some(error) = &self.error {
            lines.push(
                div()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(selectable("config-error", error.clone()))
                    .into_any_element(),
            );
        }
        if self.hook_restored {
            lines.push(
                div()
                    .text_sm()
                    .text_color(theme.warning)
                    .child(selectable("config-hook-restored", HOOK_RESTORED_MESSAGE))
                    .into_any_element(),
            );
        }
        if let Some(errors) = &self.config_errors {
            lines.push(
                div()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(selectable(
                        "config-errors",
                        format!("Hyprland rejected part of its config: {errors}"),
                    ))
                    .into_any_element(),
            );
        }
        (!lines.is_empty()).then(|| v_flex().gap_1().children(lines))
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
            // While searching, only a page with a matching item is worth a
            // "Reading…" placeholder; the others are simply not shown.
            let has_match = query.is_empty()
                || page
                    .groups
                    .iter()
                    .flat_map(|g| g.items)
                    .any(|item| Self::item_matches(item, &query));
            let loading = has_match
                && match page.group {
                    PageGroup::Omarchy => !self.omarchy.loaded.contains(&page_ix),
                    PageGroup::Hyprland => self.hyprland_loading,
                };
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
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(palette::tile(
                        palette::icon(page.icon),
                        Area::of_group(page.group).accent(cx),
                        px(28.),
                        cx,
                    ))
                    .child(explain(
                        ("page-desc", page_ix),
                        div().flex().child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .border_b_1()
                                .border_dashed()
                                .border_color(muted.opacity(0.5))
                                .child(page.title),
                        ),
                        page.description,
                    ))
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
            // Full width, or the column sizes to its widest row and the
            // controls fall off the right edge of a narrow window.
            .child(
                focus::scroll_area(&self.scroll)
                    .child(v_flex().w_full().gap_4().pb_8().pr_4().children(sections)),
            )
    }
}

/// The page is centred and never wider than this: a setting's label and
/// its control stay within a glance of each other on a wide screen.
const PAGE_WIDTH: f32 = 1160.;

impl Render for ConfigView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if std::mem::take(&mut self.inputs_stale) {
            self.sync_inputs(window, cx);
        }
        if std::mem::take(&mut self.stale_after_terminal) {
            self.refresh_omarchy_values(window, cx);
        }
        v_flex()
            .id("config-view-root")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(
                cx.listener(|this, _: &config_nav::FocusSearch, window, cx| {
                    this.search.update(cx, |input, cx| input.focus(window, cx));
                }),
            )
            .size_full()
            .items_center()
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(PAGE_WIDTH))
                    .flex_1()
                    .min_h_0()
                    .gap_4()
                    .child(
                        div().w(px(320.)).child(
                            Input::new(&self.search)
                                .cleanable(true)
                                .small()
                                .prefix(Icon::new(IconName::Search).size_4()),
                        ),
                    )
                    // Problems stay in view above the scrolling column, whatever
                    // page or scroll position the user is on.
                    .children(self.render_banners(cx))
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
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    // `use super::*` would import gpui's `test` attribute macro and shadow `#[test]`.
    use super::{FieldDef, Source, format_number, items};
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

    /// A step moves the value by a useful amount: whole numbers for integer
    /// options, and between three and four hundred clicks across the range.
    #[test]
    fn steps_fit_their_ranges() {
        let mut problems = Vec::new();
        for item in items() {
            let (min, max, step, integer) = match item.field {
                FieldDef::Number {
                    min,
                    max,
                    step,
                    integer,
                } => (min, max, step, integer),
                FieldDef::Pair { min, max, step, .. } => (min, max, step, true),
                _ => continue,
            };
            let clicks = (max - min) / step;
            if step <= 0.0 || !(3.0..=400.0).contains(&clicks) {
                problems.push(format!("{}: step {step} for {min}..{max}", item.id));
            }
            if integer && step.fract() != 0.0 {
                problems.push(format!("{}: fractional step {step} on an integer", item.id));
            }
        }
        assert!(problems.is_empty(), "{}", problems.join("\n"));
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
                        FieldDef::KeyboardLayout
                        | FieldDef::DynamicDropdown { .. }
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
                            | FieldDef::KeyboardLayout
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
                        Write::CommandAnd { argv, then } => {
                            check(argv[0], item.id);
                            for argv in then {
                                check(argv[0], item.id);
                            }
                        }
                        Write::X11Keymap => check("localectl", item.id),
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
    fn disabled_by_points_at_a_switch() {
        for item in items() {
            if let Some(id) = item.disabled_by {
                let toggle = items()
                    .find(|i| i.id == id)
                    .unwrap_or_else(|| panic!("{}: no item {id}", item.id));
                assert!(
                    matches!(toggle.field, FieldDef::Switch),
                    "{}: {id} is not a switch",
                    item.id
                );
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
