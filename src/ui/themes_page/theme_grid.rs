use crate::types::themes::{ThemeEntry, ThemeOrigin};
use crate::ui::keyboard_nav::ListNavigationState;
use crate::ui::themes_page::theme_card::ThemeCard;
use gpui::*;

const BREAKPOINT_SM: f32 = 768.0;
const BREAKPOINT_LG: f32 = 1280.0;
const GRID_GAP: f32 = 16.0;
const PAGE_PADDING_LEFT: f32 = 16.0;
const PAGE_PADDING_RIGHT: f32 = 26.0;

// `Only(origin)` restricts display to themes matching that origin.
pub enum ThemeFilter {
    All,
    Only(ThemeOrigin),
}

pub struct ThemeGrid {
    themes: Vec<ThemeEntry>,
    filter: ThemeFilter,
    cards: Vec<Entity<ThemeCard>>,
    sidebar_collapsed: bool,
    has_focus: bool,
    nav_state: ListNavigationState,
}

impl ThemeGrid {
    pub fn new(cx: &mut Context<Self>, themes: Vec<ThemeEntry>) -> Self {
        let item_count = themes.len();
        let cards = themes
            .iter()
            .enumerate()
            .map(|(index, theme)| cx.new(|_| ThemeCard::new(theme.clone(), px(200.0), index)))
            .collect();

        Self {
            themes,
            filter: ThemeFilter::All,
            cards,
            sidebar_collapsed: true,
            has_focus: false,
            nav_state: ListNavigationState::new(item_count, 3),
        }
    }

    pub fn set_has_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus && self.nav_state.focused_index.is_none() && self.nav_state.item_count > 0 {
            self.nav_state.focus_first();
        }
    }

    // Move focus up — returns true if focus actually moved, false if already at top
    pub fn move_up(&mut self, cx: &mut Context<Self>) -> bool {
        if self.nav_state.move_up() {
            cx.notify();
            true
        } else {
            false
        }
    }

    // Move focus down
    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_down() {
            cx.notify();
        }
    }

    // Move focus left
    pub fn move_left(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_left() {
            cx.notify();
        }
    }

    // Move focus right
    pub fn move_right(&mut self, cx: &mut Context<Self>) {
        if self.nav_state.move_right() {
            cx.notify();
        }
    }

    // Activate the currently focused item
    pub fn activate_focused(&mut self, cx: &mut Context<Self>) {
        if let Some(filtered_idx) = self.nav_state.focused_index {
            let filtered_indices = self.filtered_indices();
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

    /// Clear keyboard focus from the grid (no item highlighted)
    pub fn clear_focus(&mut self) {
        self.nav_state.focused_index = None;
    }

    pub fn set_filter(&mut self, filter: ThemeFilter) {
        self.filter = filter;
    }

    pub fn update_themes(&mut self, themes: Vec<ThemeEntry>, cx: &mut Context<Self>) {
        self.themes = themes;
        self.nav_state.item_count = self.themes.len();
        self.cards = self
            .themes
            .iter()
            .enumerate()
            .map(|(index, theme)| cx.new(|_| ThemeCard::new(theme.clone(), px(200.0), index)))
            .collect();
        cx.notify();
    }

    fn filtered_indices(&self) -> Vec<usize> {
        match &self.filter {
            ThemeFilter::All => (0..self.themes.len()).collect(),
            ThemeFilter::Only(origin) => self
                .themes
                .iter()
                .enumerate()
                .filter(|(_, t)| &t.origin == origin)
                .map(|(i, _)| i)
                .collect(),
        }
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
        let viewport_size = window.viewport_size();
        let sidebar_width = if self.sidebar_collapsed {
            px(48.0)
        } else {
            px(255.0)
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

        self.nav_state.columns = column_count.max(1);

        let filtered_indices = self.filtered_indices();
        self.nav_state.item_count = filtered_indices.len();

        let focused_filtered_index = self.nav_state.focused_index;

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
