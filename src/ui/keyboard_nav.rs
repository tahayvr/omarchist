// Re-export for use throughout the codebase

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusState {
    pub focused_section: FocusedSection,
    pub sidebar_index: usize,
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

    pub fn next_sidebar_item(&mut self) {
        if self.sidebar_index < self.sidebar_count.saturating_sub(1) {
            self.sidebar_index += 1;
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedSection {
    Sidebar,
    Content,
}

#[derive(Debug, Clone, Copy)]
pub struct ListNavigationState {
    pub focused_index: Option<usize>,
    pub item_count: usize,
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

    pub fn focus_first(&mut self) {
        if self.item_count > 0 {
            self.focused_index = Some(0);
        }
    }

    pub fn focus_last(&mut self) {
        if self.item_count > 0 {
            self.focused_index = Some(self.item_count - 1);
        }
    }

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

    pub fn move_left(&mut self) -> bool {
        if let Some(current) = self.focused_index
            && current > 0
        {
            self.focused_index = Some(current - 1);
            return true;
        }
        false
    }

    pub fn move_right(&mut self) -> bool {
        if let Some(current) = self.focused_index
            && current < self.item_count.saturating_sub(1)
        {
            self.focused_index = Some(current + 1);
            return true;
        }
        false
    }

    pub fn current_or_default(&self) -> usize {
        self.focused_index.unwrap_or(0)
    }

    pub fn is_focused(&self, index: usize) -> bool {
        self.focused_index == Some(index)
    }
}

impl Default for ListNavigationState {
    fn default() -> Self {
        Self::new(0, 1)
    }
}
