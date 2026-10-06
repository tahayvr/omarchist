//! A searchable list of values found on the machine (the installed
//! themes), loaded in the background.
use gpui::*;
use gpui_component::{
    Sizable,
    select::{SearchableVec, Select, SelectEvent, SelectState},
};

use crate::ui::keybinds_page::action_builder::LabeledItem;

pub enum OptionPickerEvent {
    Changed,
}

pub struct OptionPicker {
    select: Entity<SelectState<SearchableVec<LabeledItem>>>,
    /// The value chosen, or carried over from a step.
    value: String,
    placeholder: &'static str,
    _subscription: Subscription,
}

impl EventEmitter<OptionPickerEvent> for OptionPicker {}

impl OptionPicker {
    /// The list's focus handle, for putting the keyboard on it.
    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.select.read(cx).focus_handle(cx)
    }
}

/// The names `omarchy-theme-set` takes, as `omarchy-theme-list` prints
/// them.
pub fn installed_themes() -> Vec<String> {
    std::process::Command::new("omarchy-theme-list")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

impl OptionPicker {
    pub fn new(
        value: &str,
        placeholder: &'static str,
        load: fn() -> Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // The carried value is offered until the real list arrives, so the
        // step reads the same before and after.
        let initial: Vec<LabeledItem> = (!value.trim().is_empty())
            .then(|| item(value.trim()))
            .into_iter()
            .collect();
        let select = cx.new(|cx| {
            let mut state =
                SelectState::new(SearchableVec::new(initial), None, window, cx).searchable(true);
            if !value.trim().is_empty() {
                state.set_selected_value(&value.trim().to_string(), window, cx);
            }
            state
        });
        let subscription = cx.subscribe_in(
            &select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<LabeledItem>>, _, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.value = value.clone();
                    cx.emit(OptionPickerEvent::Changed);
                    cx.notify();
                }
            },
        );
        cx.spawn_in(window, async move |this, cx| {
            let options = cx.background_spawn(async move { load() }).await;
            this.update_in(cx, |this, window, cx| {
                let mut items: Vec<LabeledItem> = options.iter().map(|o| item(o)).collect();
                // A value this machine does not have stays selectable.
                if !this.value.is_empty() && !options.contains(&this.value) {
                    items.insert(0, item(&this.value));
                }
                let current = this.value.clone();
                this.select.update(cx, |select, cx| {
                    select.set_items(SearchableVec::new(items), window, cx);
                    if !current.is_empty() {
                        select.set_selected_value(&current, window, cx);
                    }
                });
                cx.notify();
            })
            .ok();
        })
        .detach();
        Self {
            select,
            value: value.trim().to_string(),
            placeholder,
            _subscription: subscription,
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

fn item(value: &str) -> LabeledItem {
    LabeledItem {
        id: value.to_string(),
        label: value.to_string().into(),
        group: SharedString::default(),
    }
}

impl Render for OptionPicker {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Select::new(&self.select)
            .placeholder(self.placeholder)
            .menu_max_h(px(320.))
            .small()
    }
}
