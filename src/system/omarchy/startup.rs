/// How often the background Omarchy version-check loop re-runs (seconds).
///
/// This watcher is started unconditionally in `MainWindowView::new()` so the
/// title-bar update badge is kept fresh even when the user never opens the
/// Omarchy page.  The `OmarchyView` page itself is constructed lazily on
/// first navigation and runs its own one-shot check for the in-page status
/// display.
pub const PERIODIC_CHECK_INTERVAL_SECS: u64 = 30 * 60;
