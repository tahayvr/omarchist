use std::cell::RefCell;

use gpui::{App, AsyncApp, BorrowAppContext, Global};

use crate::ui::app_view::ActivePage;

// Requests that components without a handle to the main window (dialogs,
// theme cards, the title bar, background tasks) send to it.
//
// Producers call `emit` / `emit_async`, which updates the `AppEvents` global
// and thereby notifies its observers. `MainWindowView` registers one such
// observer in its constructor and drains the queue there, so nothing has
// to poll flags from `render`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Navigate(ActivePage),
    RefreshThemes,
    ToggleSidebar,
    OmarchyUpdateStatus(bool),
    ReloadUiTheme,
}

#[derive(Default)]
pub struct AppEvents {
    // Interior mutability so an observer can drain the queue through an
    // immutable `cx.global()` read. Draining via `update_global` would
    // notify the observer again and loop forever.
    queue: RefCell<Vec<AppEvent>>,
}

impl Global for AppEvents {}

impl AppEvents {
    pub fn drain(cx: &App) -> Vec<AppEvent> {
        cx.try_global::<AppEvents>()
            .map(|events| events.queue.take())
            .unwrap_or_default()
    }
}

pub fn emit(cx: &mut App, event: AppEvent) {
    if !cx.has_global::<AppEvents>() {
        cx.set_global(AppEvents::default());
    }
    cx.update_global::<AppEvents, _>(|events, _| events.queue.borrow_mut().push(event));
}

// For background tasks. Fails only when the app has already shut down.
pub fn emit_async(cx: &AsyncApp, event: AppEvent) -> anyhow::Result<()> {
    cx.update_global::<AppEvents, _>(|events, _| events.queue.borrow_mut().push(event))
}
