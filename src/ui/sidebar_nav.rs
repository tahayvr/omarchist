// Actions for the sidebar composite: one tab stop whose arrow keys move
// between the page items. Handled by `MainWindowView`.
use gpui::actions;

actions!(sidebar_nav, [Next, Prev, First, Last, Activate]);
