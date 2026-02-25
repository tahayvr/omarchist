use gpui::*;
use std::cell::RefCell;

thread_local! {
    /// Thread-local storage for the current focus state across the app
    pub static FOCUS_STATE: RefCell<FocusState> = RefCell::new(FocusState::new());
}

/// Global focus state manager for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusState {
    /// Which major section has focus (sidebar, content, etc.)
    pub focused_section: FocusedSection,
    /// Index within the sidebar items
    pub sidebar_index: usize,
    /// Maximum number of sidebar items
    pub sidebar_count: usize,
}

impl FocusState {
    pub fn new() -> Self {
        Self {
            focused_section: FocusedSection::Content,
            sidebar_index: 0,
            sidebar_count: 3, // Themes, Configuration, Status Bar
        }
    }

    pub fn with_section(section: FocusedSection) -> Self {
        Self {
            focused_section: section,
            sidebar_index: 0,
            sidebar_count: 3,
        }
    }

    /// Move focus to next section
    pub fn next_section(&mut self) {
        self.focused_section = match self.focused_section {
            FocusedSection::Sidebar => FocusedSection::Content,
            FocusedSection::Content => FocusedSection::Sidebar,
        };
    }

    /// Move focus to previous section
    pub fn prev_section(&mut self) {
        self.next_section(); // Only 2 sections, so toggle
    }

    /// Move to next sidebar item
    pub fn next_sidebar_item(&mut self) {
        if self.sidebar_index < self.sidebar_count.saturating_sub(1) {
            self.sidebar_index += 1;
        }
    }

    /// Move to previous sidebar item
    pub fn prev_sidebar_item(&mut self) {
        if self.sidebar_index > 0 {
            self.sidebar_index -= 1;
        }
    }
}

impl Default for FocusState {
    fn default() -> Self {
        Self::new()
    }
}

/// Major sections of the app that can receive focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedSection {
    Sidebar,
    Content,
}

/// Trait for components that support keyboard navigation
pub trait KeyboardNavigable {
    /// Handle a key down event for navigation
    fn handle_nav_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool
    where
        Self: Render;
}

/// Check if a key is Tab
pub fn is_tab(event: &KeyDownEvent) -> bool {
    event.keystroke.key.as_str() == "tab"
}

/// Check if a key is Shift+Tab
pub fn is_shift_tab(event: &KeyDownEvent) -> bool {
    event.keystroke.key.as_str() == "tab" && event.keystroke.modifiers.shift
}

/// Check if a key is an arrow key
pub fn is_arrow_key(event: &KeyDownEvent) -> Option<ArrowDirection> {
    match event.keystroke.key.as_str() {
        "up" => Some(ArrowDirection::Up),
        "down" => Some(ArrowDirection::Down),
        "left" => Some(ArrowDirection::Left),
        "right" => Some(ArrowDirection::Right),
        _ => None,
    }
}

/// Check if Enter or Space was pressed (activation key)
pub fn is_activation_key(event: &KeyDownEvent) -> bool {
    matches!(event.keystroke.key.as_str(), "enter" | "space")
}

/// Check if Escape was pressed
pub fn is_escape(event: &KeyDownEvent) -> bool {
    event.keystroke.key.as_str() == "escape"
}

/// Arrow key directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Navigation state for a list/grid of items
#[derive(Debug, Clone, Copy)]
pub struct ListNavigationState {
    /// Currently focused item index (None means no focus)
    pub focused_index: Option<usize>,
    /// Total number of items
    pub item_count: usize,
    /// Number of columns (for grid navigation)
    pub columns: usize,
}

impl ListNavigationState {
    pub fn new(item_count: usize, columns: usize) -> Self {
        Self {
            focused_index: None,
            item_count,
            columns: columns.max(1),
        }
    }

    pub fn with_focused(item_count: usize, columns: usize, focused: usize) -> Self {
        Self {
            focused_index: Some(focused.min(item_count.saturating_sub(1))),
            item_count,
            columns: columns.max(1),
        }
    }

    /// Focus the first item
    pub fn focus_first(&mut self) {
        if self.item_count > 0 {
            self.focused_index = Some(0);
        }
    }

    /// Focus the last item
    pub fn focus_last(&mut self) {
        if self.item_count > 0 {
            self.focused_index = Some(self.item_count - 1);
        }
    }

    /// Move focus up (for grids)
    pub fn move_up(&mut self) -> bool {
        if let Some(current) = self.focused_index {
            let new_index = current.saturating_sub(self.columns);
            if new_index != current {
                self.focused_index = Some(new_index);
                return true;
            }
        }
        false
    }

    /// Move focus down (for grids)
    pub fn move_down(&mut self) -> bool {
        if let Some(current) = self.focused_index {
            let new_index = (current + self.columns).min(self.item_count.saturating_sub(1));
            if new_index != current {
                self.focused_index = Some(new_index);
                return true;
            }
        }
        false
    }

    /// Move focus left
    pub fn move_left(&mut self) -> bool {
        if let Some(current) = self.focused_index
            && current > 0
        {
            self.focused_index = Some(current - 1);
            return true;
        }
        false
    }

    /// Move focus right
    pub fn move_right(&mut self) -> bool {
        if let Some(current) = self.focused_index
            && current < self.item_count.saturating_sub(1)
        {
            self.focused_index = Some(current + 1);
            return true;
        }
        false
    }

    /// Get the current focused index or default to 0
    pub fn current_or_default(&self) -> usize {
        self.focused_index.unwrap_or(0)
    }

    /// Check if an index is currently focused
    pub fn is_focused(&self, index: usize) -> bool {
        self.focused_index == Some(index)
    }
}

impl Default for ListNavigationState {
    fn default() -> Self {
        Self::new(0, 1)
    }
}

/// Extension trait for applying focus styles
pub trait FocusStyleExt {
    fn focus_ring(self, is_focused: bool) -> Self;
}

impl FocusStyleExt for Div {
    fn focus_ring(self, is_focused: bool) -> Self {
        if is_focused {
            self.bg(gpui::rgb(0x2a2a2a))
        } else {
            self
        }
    }
}
