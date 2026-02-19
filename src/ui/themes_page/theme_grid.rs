use crate::types::themes::SysTheme;
use crate::ui::keyboard_nav::ListNavigationState;
use crate::ui::themes_page::theme_card::ThemeCard;
use gpui::*;

const BREAKPOINT_SM: f32 = 768.0;
const BREAKPOINT_LG: f32 = 1280.0;
const GRID_GAP: f32 = 16.0;
const PAGE_PADDING_LEFT: f32 = 16.0;
const PAGE_PADDING_RIGHT: f32 = 26.0;

pub enum ThemeFilter {
    Custom,
    System,
}

pub struct ThemeGrid {
    themes: Vec<SysTheme>,
    filter: Option<ThemeFilter>,
    cards: Vec<Entity<ThemeCard>>,
    sidebar_collapsed: bool,
    /// Whether the grid currently has focus
    has_focus: bool,
    /// Navigation state for keyboard control
    nav_state: ListNavigationState,
}

impl ThemeGrid {
    pub fn new(cx: &mut Context<Self>, themes: Vec<SysTheme>, filter: Option<ThemeFilter>) -> Self {
        let item_count = themes.len();
        let cards = themes
            .iter()
            .enumerate()
            .map(|(index, theme)| cx.new(|_| ThemeCard::new(theme.clone(), px(200.0), index)))
            .collect();

        Self {
            themes,
            filter,
            cards,
            sidebar_collapsed: true,
            has_focus: false,
            nav_state: ListNavigationState::new(item_count, 3), // Default to 3 columns
        }
    }

    pub fn set_has_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus && self.nav_state.focused_index.is_none() && self.nav_state.item_count > 0 {
            self.nav_state.focus_first();
        }
    }

    /// Move focus up
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_up() {
            cx.notify();
        }
    }

    /// Move focus down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_down() {
            cx.notify();
        }
    }

    /// Move focus left
    pub fn move_left(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_left() {
            cx.notify();
        }
    }

    /// Move focus right
    pub fn move_right(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_right() {
            cx.notify();
        }
    }

    /// Activate the currently focused item
    pub fn activate_focused(&mut self, cx: &mut Context<Self>) {
        if let Some(filtered_idx) = self.nav_state.focused_index {
            // Get filtered indices to map to actual index
            let filtered_indices: Vec<usize> = match &self.filter {
                Some(ThemeFilter::Custom) => self
                    .themes
                    .iter()
                    .enumerate()
                    .filter(|(_, theme)| theme.is_custom)
                    .map(|(idx, _)| idx)
                    .collect(),
                Some(ThemeFilter::System) => self
                    .themes
                    .iter()
                    .enumerate()
                    .filter(|(_, theme)| !theme.is_custom)
                    .map(|(idx, _)| idx)
                    .collect(),
                None => (0..self.themes.len()).collect(),
            };

            if let Some(&actual_idx) = filtered_indices.get(filtered_idx)
                && let Some(card) = self.cards.get(actual_idx)
            {
                card.update(cx, |card, _cx| {
                    card.activate();
                });
            }
        }
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool) {
        self.sidebar_collapsed = collapsed;
    }

    pub fn set_filter(&mut self, filter: Option<ThemeFilter>) {
        self.filter = filter;
    }

    /// Update themes and recreate cards
    pub fn update_themes(&mut self, themes: Vec<SysTheme>, cx: &mut Context<Self>) {
        self.themes = themes;
        self.nav_state.item_count = self.themes.len();
        // Recreate cards for new themes
        self.cards = self
            .themes
            .iter()
            .enumerate()
            .map(|(index, theme)| cx.new(|_| ThemeCard::new(theme.clone(), px(200.0), index)))
            .collect();
        cx.notify();
    }

    fn get_column_count(&self, width: Pixels) -> usize {
        let width_f32: f32 = width.into();

        if width_f32 < BREAKPOINT_SM {
            1
        } else if width_f32 < BREAKPOINT_LG {
            2
        } else {
            3
        }
    }
}

impl Render for ThemeGrid {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get the actual viewport size and account for sidebar
        let viewport_size = window.viewport_size();
        let sidebar_width = if self.sidebar_collapsed {
            px(48.0) // Collapsed sidebar
        } else {
            px(255.0) // Expanded sidebar
        };
        let width = (viewport_size.width - sidebar_width).max(px(0.0));
        let column_count = self.get_column_count(width);
        let width_f32: f32 = width.into();
        let available_width = (width_f32
            - PAGE_PADDING_LEFT
            - PAGE_PADDING_RIGHT
            - (GRID_GAP * (column_count.saturating_sub(1) as f32)))
            .max(0.0);
        let card_width = if column_count > 0 {
            available_width / column_count as f32
        } else {
            0.0
        };
        let image_height = px((card_width * 9.0 / 16.0).max(0.0));

        // Update navigation state with current column count
        self.nav_state.columns = column_count.max(1);

        // Get filtered card indices
        let filtered_indices: Vec<usize> = match &self.filter {
            Some(ThemeFilter::Custom) => self
                .themes
                .iter()
                .enumerate()
                .filter(|(_, theme)| theme.is_custom)
                .map(|(idx, _)| idx)
                .collect(),
            Some(ThemeFilter::System) => self
                .themes
                .iter()
                .enumerate()
                .filter(|(_, theme)| !theme.is_custom)
                .map(|(idx, _)| idx)
                .collect(),
            None => (0..self.themes.len()).collect(),
        };

        // Update navigation state item count to filtered count
        self.nav_state.item_count = filtered_indices.len();

        // Calculate which filtered index is focused
        let focused_filtered_index = self.nav_state.focused_index;

        // Update image height and focus state for all visible cards
        for (filtered_idx, &actual_idx) in filtered_indices.iter().enumerate() {
            if let Some(card) = self.cards.get(actual_idx) {
                let is_focused = focused_filtered_index == Some(filtered_idx) && self.has_focus;
                card.update(cx, |card, _cx| {
                    card.set_image_height(image_height);
                    card.set_focused(is_focused);
                });
            }
        }

        div().w_full().min_w_0().child(
            div().flex().flex_col().gap_4().w_full().min_w_0().child(
                div().flex().flex_col().gap_4().w_full().min_w_0().children(
                    filtered_indices
                        .chunks(column_count)
                        .map(|row_indices| {
                            let mut row_children: Vec<AnyElement> = row_indices
                                .iter()
                                .filter_map(|&idx| {
                                    self.cards.get(idx).map(|card| {
                                        div().flex_1().child(card.clone()).into_any_element()
                                    })
                                })
                                .collect();

                            let missing = column_count.saturating_sub(row_indices.len());
                            for _ in 0..missing {
                                row_children.push(div().flex_1().into_any_element());
                            }

                            div()
                                .flex()
                                .flex_row()
                                .gap_4()
                                .w_full()
                                .min_w_0()
                                .children(row_children)
                        })
                        .collect::<Vec<_>>(),
                ),
            ),
        )
    }
}
