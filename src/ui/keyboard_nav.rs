// Re-export for use throughout the codebase

/// Global focus state for the main window (sidebar vs. content section)
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
