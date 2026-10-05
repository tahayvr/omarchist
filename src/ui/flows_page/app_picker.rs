//! A searchable list of the installed apps that yields the class of the
//! app's windows, for steps and triggers that are about a window rather
//! than about launching something.
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};

use crate::system::apps::{DesktopApp, installed_apps};
use crate::ui::keybinds_page::action_builder::AppItem;
use crate::ui::text::selectable;

pub enum AppPickerEvent {
    Changed,
}

pub struct AppPicker {
    select: Entity<SelectState<SearchableVec<AppItem>>>,
    apps: Vec<DesktopApp>,
    /// The window class chosen, or carried over from a step whose app is
    /// not installed here.
    class: String,
    _subscription: Subscription,
}

impl EventEmitter<AppPickerEvent> for AppPicker {}

impl AppPicker {
    pub fn new(class: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select = cx.new(|cx| {
            SelectState::new(SearchableVec::new(Vec::<AppItem>::new()), None, window, cx)
                .searchable(true)
        });
        let subscription = cx.subscribe_in(
            &select,
            window,
            |this, _, event: &SelectEvent<SearchableVec<AppItem>>, _, cx| {
                if let SelectEvent::Confirm(Some(id)) = event
                    && let Some(app) = this.apps.iter().find(|a| &a.id == id)
                {
                    this.class = app.wm_class.clone();
                    cx.emit(AppPickerEvent::Changed);
                    cx.notify();
                }
            },
        );
        cx.spawn_in(window, async move |this, cx| {
            let apps = cx.background_spawn(async { installed_apps() }).await;
            this.update_in(cx, |this, window, cx| this.set_apps(apps, window, cx))
                .ok();
        })
        .detach();
        Self {
            select,
            apps: Vec::new(),
            class: class.trim().to_string(),
            _subscription: subscription,
        }
    }

    /// The class of the chosen app's windows; empty until one is chosen.
    pub fn class(&self) -> &str {
        &self.class
    }

    fn set_apps(&mut self, apps: Vec<DesktopApp>, window: &mut Window, cx: &mut Context<Self>) {
        let apps: Vec<DesktopApp> = apps
            .into_iter()
            .filter(|a| !a.is_webapp() && !a.wm_class.trim().is_empty())
            .collect();
        let items: Vec<AppItem> = apps
            .iter()
            .map(|a| AppItem {
                id: a.id.clone(),
                name: a.name.clone().into(),
                exec: a.wm_class.clone().into(),
                icon: a.icon.clone(),
            })
            .collect();
        let current = apps
            .iter()
            .find(|a| a.wm_class.eq_ignore_ascii_case(&self.class))
            .map(|a| a.id.clone());
        self.apps = apps;
        self.select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(items), window, cx);
            if let Some(id) = &current {
                select.set_selected_value(id, window, cx);
            }
        });
        cx.notify();
    }

    /// Whether the class came with the step and no installed app has it.
    fn carried(&self) -> bool {
        !self.class.is_empty()
            && !self
                .apps
                .iter()
                .any(|a| a.wm_class.eq_ignore_ascii_case(&self.class))
    }
}

impl Render for AppPicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let carried = self.carried().then(|| self.class.clone());
        v_flex()
            .gap_1()
            .child(
                Select::new(&self.select)
                    .placeholder("Choose an installed app")
                    .search_placeholder("Search apps")
                    .menu_max_h(px(320.))
                    .small(),
            )
            .children(carried.map(|class| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(selectable("app-picker-class", class))
            }))
    }
}
