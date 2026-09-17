use crate::types::themes::{ThemeEntry, ThemeOrigin};
use crate::ui::dialogs::create_theme_dialog::open_create_theme_dialog;
use crate::ui::focus;
use crate::ui::themes_page::theme_card::ThemeCard;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, button::Button, h_flex, scroll::ScrollableElement, v_flex};

const BREAKPOINT_SM: f32 = 768.0;
const BREAKPOINT_LG: f32 = 1280.0;
const GRID_GAP: f32 = 16.0;
const PAGE_PADDING_LEFT: f32 = 16.0;
const PAGE_PADDING_RIGHT: f32 = 26.0;
// Card chrome (title row, footer row, borders) above the 16:9 image.
const CARD_CHROME_HEIGHT: f32 = 96.0;

pub const KEY_CONTEXT: &str = "ThemeGrid";

actions!(
    theme_grid,
    [
        Up, Down, Left, Right, First, Last, PageUp, PageDown, Apply, Edit, Delete, OpenFolder,
        LeaveGrid,
    ]
);

// `Only(origin)` restricts display to themes matching that origin.
#[derive(PartialEq, Eq)]
pub enum ThemeFilter {
    All,
    Only(ThemeOrigin),
}

/// Roving index inside the grid: which filtered card has the keyboard.
#[derive(Debug, Clone, Copy)]
struct GridNav {
    focused_index: Option<usize>,
    item_count: usize,
    columns: usize,
}

impl GridNav {
    fn new(item_count: usize, columns: usize) -> Self {
        Self {
            focused_index: None,
            item_count,
            columns: columns.max(1),
        }
    }

    fn set(&mut self, index: usize) -> bool {
        if self.item_count == 0 {
            return false;
        }
        let index = index.min(self.item_count - 1);
        if self.focused_index == Some(index) {
            return false;
        }
        self.focused_index = Some(index);
        true
    }

    fn move_by(&mut self, delta: isize) -> bool {
        let Some(current) = self.focused_index else {
            return self.set(0);
        };
        let target = (current as isize + delta).clamp(0, self.item_count as isize - 1);
        self.set(target as usize)
    }

    fn move_up(&mut self) -> bool {
        match self.focused_index {
            Some(current) if current >= self.columns => self.move_by(-(self.columns as isize)),
            Some(_) => false,
            None => self.set(0),
        }
    }

    fn move_down(&mut self) -> bool {
        match self.focused_index {
            Some(current) if current + self.columns < self.item_count => {
                self.move_by(self.columns as isize)
            }
            Some(current) if current < self.item_count.saturating_sub(1) => {
                self.set(self.item_count - 1)
            }
            Some(_) => false,
            None => self.set(0),
        }
    }
}

pub struct ThemeGrid {
    themes: Vec<ThemeEntry>,
    filter: ThemeFilter,
    cards: Vec<Entity<ThemeCard>>,
    sidebar_collapsed: bool,
    pub focus: FocusHandle,
    nav: GridNav,
    scroll: ScrollHandle,
    rows_per_page: usize,
}

impl ThemeGrid {
    pub fn new(themes: Vec<ThemeEntry>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let item_count = themes.len();
        let cards = Self::make_cards(&themes, cx);
        let focus = focus::tab_stop(cx);

        // Tabbing into the grid lands on the first (or last focused) card.
        cx.on_focus(&focus, window, |this, _, cx| {
            if this.nav.focused_index.is_none() && this.nav.set(0) {
                cx.notify();
            }
        })
        .detach();

        Self {
            themes,
            filter: ThemeFilter::All,
            cards,
            sidebar_collapsed: true,
            focus,
            nav: GridNav::new(item_count, 3),
            scroll: ScrollHandle::new(),
            rows_per_page: 1,
        }
    }

    fn make_cards(themes: &[ThemeEntry], cx: &mut Context<Self>) -> Vec<Entity<ThemeCard>> {
        themes
            .iter()
            .enumerate()
            .map(|(index, theme)| cx.new(|_| ThemeCard::new(theme.clone(), px(200.0), index)))
            .collect()
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool) {
        self.sidebar_collapsed = collapsed;
    }

    pub fn set_filter(&mut self, filter: ThemeFilter) {
        if self.filter != filter {
            self.filter = filter;
            self.nav.focused_index = None;
        }
    }

    pub fn update_themes(&mut self, themes: Vec<ThemeEntry>, cx: &mut Context<Self>) {
        self.themes = themes;
        self.nav.item_count = self.themes.len();
        self.cards = Self::make_cards(&self.themes, cx);
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

    fn focused_card(&self) -> Option<Entity<ThemeCard>> {
        let filtered = self.nav.focused_index?;
        let actual = *self.filtered_indices().get(filtered)?;
        self.cards.get(actual).cloned()
    }

    /// Applies a movement to the roving index and scrolls its row into view.
    fn move_focus(&mut self, moved: bool, cx: &mut Context<Self>) {
        if !moved {
            return;
        }
        if let Some(index) = self.nav.focused_index {
            self.scroll.scroll_to_item(index / self.nav.columns);
        }
        cx.notify();
    }

    fn select_card(&mut self, filtered_index: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.focus.focus(window);
        let moved = self.nav.set(filtered_index);
        self.move_focus(moved, cx);
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
        let row_height = f32::from(image_height) + CARD_CHROME_HEIGHT + GRID_GAP;
        let viewport_height = f32::from(self.scroll.bounds().size.height);
        self.rows_per_page = ((viewport_height / row_height).floor() as usize).max(1);

        self.nav.columns = column_count.max(1);

        let filtered_indices = self.filtered_indices();
        self.nav.item_count = filtered_indices.len();
        if let Some(index) = self.nav.focused_index
            && index >= self.nav.item_count
        {
            self.nav.focused_index = self.nav.item_count.checked_sub(1);
        }

        let grid_focused = self.focus.is_focused(window);
        let focused_filtered_index = self.nav.focused_index;

        for (filtered_idx, &actual_idx) in filtered_indices.iter().enumerate() {
            if let Some(card) = self.cards.get(actual_idx) {
                let is_focused = focused_filtered_index == Some(filtered_idx) && grid_focused;
                card.update(cx, |card, _cx| {
                    card.set_image_height(image_height);
                    card.set_focused(is_focused);
                });
            }
        }

        let is_empty = filtered_indices.is_empty();
        let muted = cx.theme().muted_foreground;

        let rows: Vec<AnyElement> = filtered_indices
            .chunks(column_count)
            .enumerate()
            .map(|(row_ix, row_indices)| {
                let mut row_children: Vec<AnyElement> = row_indices
                    .iter()
                    .enumerate()
                    .filter_map(|(col_ix, &idx)| {
                        let filtered_idx = row_ix * column_count + col_ix;
                        self.cards.get(idx).map(|card| {
                            div()
                                .id(("theme-card", idx))
                                .flex_1()
                                .min_w_0()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.select_card(filtered_idx, window, cx);
                                }))
                                .child(card.clone())
                                .into_any_element()
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
                    .into_any_element()
            })
            .collect();

        div()
            .id("theme-grid")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &Up, _, cx| {
                let moved = this.nav.move_up();
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &Down, _, cx| {
                let moved = this.nav.move_down();
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &Left, _, cx| {
                let moved = this.nav.move_by(-1);
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &Right, _, cx| {
                let moved = this.nav.move_by(1);
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &First, _, cx| {
                let moved = this.nav.set(0);
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &Last, _, cx| {
                let moved = this.nav.set(this.nav.item_count.saturating_sub(1));
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &PageUp, _, cx| {
                let step = (this.rows_per_page * this.nav.columns) as isize;
                let moved = this.nav.move_by(-step);
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &PageDown, _, cx| {
                let step = (this.rows_per_page * this.nav.columns) as isize;
                let moved = this.nav.move_by(step);
                this.move_focus(moved, cx);
            }))
            .on_action(cx.listener(|this, _: &Apply, _, cx| {
                if let Some(card) = this.focused_card() {
                    card.update(cx, |card, _| card.activate());
                }
            }))
            .on_action(cx.listener(|this, _: &Edit, _, cx| {
                if let Some(card) = this.focused_card() {
                    card.update(cx, |card, cx| card.edit(cx));
                }
            }))
            .on_action(cx.listener(|this, _: &Delete, window, cx| {
                if let Some(card) = this.focused_card() {
                    card.update(cx, |card, cx| card.confirm_delete(window, cx));
                }
            }))
            .on_action(cx.listener(|this, _: &OpenFolder, _, cx| {
                if let Some(card) = this.focused_card() {
                    card.update(cx, |card, _| card.open_folder());
                }
            }))
            .relative()
            .size_full()
            .min_w_0()
            .when(is_empty, |this| {
                this.flex().items_center().justify_center().child(
                    v_flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .text_color(muted)
                                .mt_12()
                                .text_sm()
                                .child("You have no themes."),
                        )
                        .child(
                            h_flex().child(
                                Button::new("empty-create-theme-btn")
                                    .label("Create New Theme")
                                    .on_click(|_, window, cx| {
                                        open_create_theme_dialog(window, cx);
                                    }),
                            ),
                        ),
                )
            })
            .when(!is_empty, |this| {
                this.child(
                    div()
                        .id("theme-grid-scroll")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll)
                        .flex()
                        .flex_col()
                        .gap_4()
                        .pb_4()
                        .children(rows),
                )
                .vertical_scrollbar(&self.scroll)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::GridNav;

    #[test]
    fn moves_by_rows_and_clamps_to_the_last_card() {
        let mut nav = GridNav::new(7, 3);
        assert!(nav.set(0));
        assert!(nav.move_down());
        assert_eq!(nav.focused_index, Some(3));
        assert!(nav.move_down());
        assert_eq!(nav.focused_index, Some(6));
        assert!(!nav.move_down());
        assert!(nav.move_up());
        assert_eq!(nav.focused_index, Some(3));
        assert!(nav.move_by(-10));
        assert_eq!(nav.focused_index, Some(0));
        assert!(!nav.move_up());
    }

    #[test]
    fn moving_down_from_a_short_last_row_lands_on_the_last_card() {
        let mut nav = GridNav::new(5, 3);
        nav.set(2);
        assert!(nav.move_down());
        assert_eq!(nav.focused_index, Some(4));
    }

    #[test]
    fn empty_grid_never_focuses() {
        let mut nav = GridNav::new(0, 3);
        assert!(!nav.set(0));
        assert!(!nav.move_down());
        assert_eq!(nav.focused_index, None);
    }
}
