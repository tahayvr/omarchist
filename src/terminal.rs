pub mod colors;
pub mod session;
pub mod view;

use std::cell::RefCell;

thread_local! {
    /// Thread-local storage for pending terminal navigation
    /// Used to pass command from buttons/navigation to the terminal page
    pub static PENDING_TERMINAL_NAVIGATION: RefCell<Option<String>> = const { RefCell::new(None) };
}
