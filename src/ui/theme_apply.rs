//! Applying and re-applying a theme from the UI: off the UI thread, with a
//! toast for the outcome. Every Apply button and shortcut goes through here
//! so a missing `omarchy-theme-set`, a folder name the script rejects, or a
//! theme removed behind the app's back is never a silent no-op.
use gpui::{App, AppContext, Window};

use crate::shell::theme_sh_commands::{apply_theme_blocking, refresh_theme_blocking};
use crate::system::themes::utils::dir_to_title;
use crate::ui::notify;
/// Runs `omarchy-theme-set <dir>` and reports the result.
pub fn apply_theme(dir: String, window: &mut Window, cx: &mut App) {
    let handle = window.window_handle();
    let title = dir_to_title(&dir);
    cx.spawn(async move |cx| {
        let result = cx
            .background_spawn(async move { apply_theme_blocking(&dir) })
            .await;
        handle
            .update(cx, |_, window, cx| match result {
                Ok(()) => notify::success(window, format!("Applied '{title}'"), cx),
                Err(e) => notify::error(window, format!("Could not apply '{title}': {e}"), cx),
            })
            .ok();
    })
    .detach();
}

/// Runs `omarchy-theme-refresh` (the current theme again) and reports it.
pub fn refresh_theme(window: &mut Window, cx: &mut App) {
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let result = cx
            .background_spawn(async { refresh_theme_blocking() })
            .await;
        handle
            .update(cx, |_, window, cx| match result {
                Ok(()) => notify::success(window, "Re-applied the current theme", cx),
                Err(e) => notify::error(window, format!("Could not re-apply the theme: {e}"), cx),
            })
            .ok();
    })
    .detach();
}
