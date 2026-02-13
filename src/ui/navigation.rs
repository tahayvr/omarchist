use crate::ui::app_view::{ActivePage, MainWindowView};
use std::cell::RefCell;

thread_local! {
    static MAIN_VIEW: RefCell<Option<gpui::Entity<MainWindowView>>> = RefCell::new(None);
}

pub fn set_main_view(view: gpui::Entity<MainWindowView>) {
    MAIN_VIEW.with(|v| {
        *v.borrow_mut() = Some(view);
    });
}

pub fn with_main_view<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut MainWindowView, &mut gpui::Window, &mut gpui::Context<MainWindowView>) -> R,
{
    MAIN_VIEW.with(|v| {
        if let Some(view) = v.borrow().as_ref() {
            // We can't easily get window here, so we need a different approach
            None
        } else {
            None
        }
    })
}

pub fn navigate_to_theme_edit(theme_name: String) {
    eprintln!("Navigation requested to theme: {}", theme_name);
    // This will be handled differently
}
