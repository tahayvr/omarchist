// The keybind table: a `gpui_component::table::DataTable` delegate over the
// page's filtered rows. The delegate only renders; the page owns the data
// and pushes new rows with `set_rows`.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, h_flex,
    menu::PopupMenu,
    table::{Column, TableDelegate, TableState},
    tag::Tag,
    tooltip::Tooltip,
};

use crate::system::flows::StepKind;
use crate::system::keybinds::{Keybind, Origin};
use crate::ui::explain::explain;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::step_types::StepGroup;
use crate::ui::keybinds_page::chord_chips::chord_chips;
use crate::ui::keybinds_page::keybinds_view::{CopyCommand, DisableRow, EditRow, ResetRow};
use crate::ui::palette::{self, Area};
use crate::ui::text::selectable;
/// How a row relates to the user's overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKind {
    /// An untouched default or user bind.
    Plain,
    /// An Omarchist bind that replaces a default; `original_keys` is the chord it replaced.
    Modified { original_keys: String },
    /// An Omarchist bind the user added from scratch.
    Custom,
    /// A default the user disabled on this page.
    Disabled,
    /// A default removed by `hl.unbind` in the user's own `bindings.lua`.
    UnboundByUser,
    /// An override whose target no longer exists (an Omarchy update changed
    /// the bind's description or command); only Reset applies.
    Stale,
}

#[derive(Debug, Clone)]
pub struct KeybindRow {
    pub bind: Keybind,
    pub kind: RowKind,
    /// Index into `KeybindOverrides::overrides` when an override produced
    /// or affects this row.
    pub override_ix: Option<usize>,
    pub conflict: bool,
    /// Descriptions of the other binds on this chord, for the tooltip.
    pub conflicts_with: Vec<String>,
}

impl KeybindRow {
    pub fn is_greyed(&self) -> bool {
        matches!(
            self.kind,
            RowKind::Disabled | RowKind::UnboundByUser | RowKind::Stale
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyReason {
    Loading,
    NoBinds,
    NoMatches,
    NoConflicts,
    NoModified,
    Error,
}

pub struct KeybindsTableDelegate {
    columns: Vec<Column>,
    /// Below 900 px: no Source column, its tags sit under the action.
    narrow: bool,
    /// The width the columns were last sized for.
    fitted: Pixels,
    rows: Vec<KeybindRow>,
    loading: bool,
    empty_reason: EmptyReason,
    error: Option<String>,
    /// The page's focus handle: context-menu actions are dispatched there
    /// so the page's `on_action` handlers receive them.
    page_focus: FocusHandle,
}

impl KeybindsTableDelegate {
    pub fn new(page_focus: FocusHandle) -> Self {
        let mut delegate = Self {
            page_focus,
            columns: Vec::new(),
            narrow: false,
            fitted: px(0.),
            rows: Vec::new(),
            loading: true,
            empty_reason: EmptyReason::Loading,
            error: None,
        };
        delegate.fit_width(px(1100.));
        delegate
    }

    /// Sizes the columns to the width the table has: Action and Command
    /// share what the fixed columns leave, and below 900 px the Source
    /// column folds into the Action cell so nothing is clipped. `true`
    /// when the columns changed and the header needs a re-layout.
    pub fn fit_width(&mut self, available: Pixels) -> bool {
        let available = available.max(px(480.));
        if self.fitted == available {
            return false;
        }
        self.fitted = available;
        let narrow = available < px(900.);
        let edit = px(36.);
        let keys = if narrow { px(190.) } else { px(230.) };
        let source = if narrow { px(0.) } else { px(230.) };
        // The scrollbar and borders take a little of the width.
        let rest = (available - edit - keys - source - px(24.)).max(px(300.));
        let action = rest * if narrow { 0.45 } else { 0.4 };
        let command = rest - action;
        self.narrow = narrow;
        self.columns = vec![
            Column::new("edit", "").width(edit).resizable(false),
            Column::new("action", "Action").width(action),
            Column::new("keys", "Keystrokes").width(keys),
            Column::new("command", "Command").width(command),
        ];
        if !narrow {
            self.columns.push(
                Column::new("source", "Source")
                    .width(source)
                    .resizable(false),
            );
        }
        true
    }

    pub fn rows(&self) -> &[KeybindRow] {
        &self.rows
    }

    pub fn row(&self, ix: usize) -> Option<&KeybindRow> {
        self.rows.get(ix)
    }

    pub fn set_rows(&mut self, rows: Vec<KeybindRow>, empty_reason: EmptyReason) {
        self.rows = rows;
        self.empty_reason = empty_reason;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    pub fn set_error(&mut self, error: Option<String>) {
        self.error = error;
    }

    fn render_edit_cell(&self, row: &KeybindRow, row_ix: usize, cx: &App) -> AnyElement {
        let theme = cx.theme();
        if !row.bind.is_rebindable() && row.kind != RowKind::UnboundByUser {
            return explain(
                ("kb-fn-why", row_ix),
                h_flex()
                    .id(("kb-fn", row_ix))
                    .h_full()
                    .items_center()
                    .text_color(theme.muted_foreground)
                    .child(Icon::new(Icon::empty()).path("icons/ban.svg").size_4()),
                "Runs a Lua function: the keys cannot be changed, only disabled.",
            )
            .into_any_element();
        }
        if row.kind == RowKind::UnboundByUser {
            return div().into_any_element();
        }
        h_flex()
            .id(("kb-edit", row_ix))
            .h_full()
            .items_center()
            .opacity(0.)
            .group_hover(row_group(row_ix), |style| style.opacity(1.))
            .text_color(theme.muted_foreground)
            .hover(|style| style.text_color(theme.foreground))
            .cursor_pointer()
            .child(Icon::new(Icon::empty()).path("icons/pencil.svg").size_4())
            .tooltip(|window, cx| Tooltip::new("Edit").build(window, cx))
            // On the page's handle, as the context menu dispatches: a
            // window-level dispatch goes to whatever is focused, which can
            // be nothing after a dialog.
            .on_click({
                let page_focus = self.page_focus.clone();
                move |_, window, cx| {
                    page_focus.dispatch_action(&EditRow(row_ix), window, cx);
                }
            })
            .into_any_element()
    }

    fn render_action_cell(&self, row: &KeybindRow, row_ix: usize, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let label = row.bind.label().to_string();
        let undescribed = row.bind.description.is_empty();

        // What the bind does, as the flow editor draws the same action: an
        // app, a window move, a command, a flow, each in its group's colour.
        let kind_tile = match StepKind::from_dispatcher(row.bind.dispatcher.clone(), false) {
            Some(kind) => SummaryContext {
                apps: &[],
                flows: &[],
            }
            .summarize(&kind)
            .tile(px(20.), cx),
            // A Lua function from the user's own config.
            None => palette::tile(
                palette::icon("icons/braces.svg"),
                StepGroup::Script.accent(cx),
                px(20.),
                cx,
            )
            .into_any_element(),
        };
        let cell = h_flex()
            .gap_2()
            .items_center()
            .min_w_0()
            .child(kind_tile)
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .when(undescribed, |this: Div| {
                        this.italic().text_color(theme.muted_foreground)
                    })
                    .child(selectable(("kb-action", row_ix), label)),
            )
            .when(row.conflict, |this| {
                let others = row.conflicts_with.join(", ");
                this.child(explain(
                    ("kb-conflict-why", row_ix),
                    div()
                        .id(("kb-conflict", row_ix))
                        .flex_shrink_0()
                        .text_color(theme.warning)
                        .child(Icon::new(IconName::TriangleAlert).size_4()),
                    format!("Also bound to: {others}. Hyprland runs all of them."),
                ))
            });
        // Rows have a fixed height, so the folded Source tags sit inline
        // after the (truncating) label.
        if self.narrow {
            cell.child(
                div()
                    .flex_shrink_0()
                    .child(self.render_source_cell(row, row_ix, cx)),
            )
            .into_any_element()
        } else {
            cell.into_any_element()
        }
    }

    fn render_command_cell(&self, row: &KeybindRow, row_ix: usize, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let text = row.bind.dispatcher.text().to_string();
        let tooltip_text = text.clone();
        div()
            .id(("kb-cmd", row_ix))
            .min_w_0()
            .w_full()
            .truncate()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(selectable(("kb-cmd-text", row_ix), text))
            .tooltip(move |window, cx| Tooltip::new(tooltip_text.clone()).build(window, cx))
            .into_any_element()
    }

    fn render_source_cell(&self, row: &KeybindRow, row_ix: usize, cx: &App) -> AnyElement {
        // Folded into the Action cell, a Default origin says nothing worth
        // the space; the other origins and every status still show.
        let origin_tag = (!self.narrow || row.bind.origin != Origin::Default).then(|| {
            match row.bind.origin {
                Origin::Default => Tag::secondary(),
                Origin::User => Tag::info(),
                // Made on this page, so it wears the page's colour.
                Origin::Omarchist => {
                    let accent = Area::Keybinds.accent(cx);
                    Tag::custom(accent.opacity(0.16), accent, accent.opacity(0.4))
                }
            }
            .rounded(px(0.))
            .child(row.bind.origin.label())
        });

        let status_tag = match &row.kind {
            RowKind::Plain => None,
            RowKind::Modified { .. } => Some(Tag::success().rounded(px(0.)).child("Modified")),
            RowKind::Custom => Some(Tag::success().rounded(px(0.)).child("Custom")),
            RowKind::Disabled => Some(Tag::danger().rounded(px(0.)).child("Disabled")),
            RowKind::UnboundByUser => Some(Tag::warning().rounded(px(0.)).child("Unbound")),
            RowKind::Stale => Some(Tag::warning().rounded(px(0.)).child("Stale")),
        };

        let note = match &row.kind {
            RowKind::Modified { original_keys } => Some(format!("Default keys: {original_keys}")),
            RowKind::Disabled => Some("Disabled on this page. Reset to bring it back.".to_string()),
            RowKind::UnboundByUser => {
                Some("Removed by hl.unbind in your ~/.config/hypr/bindings.lua".to_string())
            }
            RowKind::Stale => Some(
                "This change targets a keybind Omarchy no longer ships. Reset to remove it."
                    .to_string(),
            ),
            _ => None,
        };

        let tags = h_flex()
            .id(("kb-source", row_ix))
            .gap_1()
            .items_center()
            .children(origin_tag)
            .children(status_tag);
        match note {
            Some(note) => explain(("kb-source-why", row_ix), tags, note).into_any_element(),
            None => tags.into_any_element(),
        }
    }
}

pub fn row_group(row_ix: usize) -> SharedString {
    SharedString::from(format!("kb-row-{row_ix}"))
}

impl TableDelegate for KeybindsTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        // The table may ask for a column from its previous layout during
        // the frame the count shrinks.
        self.columns
            .get(col_ix)
            .cloned()
            .unwrap_or_else(|| Column::new("", "").width(px(0.)))
    }

    fn loading(&self, _cx: &App) -> bool {
        self.loading
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        let tr = div().id(("kb-row", row_ix)).group(row_group(row_ix));
        let Some(row) = self.rows.get(row_ix) else {
            return tr;
        };
        let danger = cx.theme().danger;
        tr.when(row.conflict, |this: Stateful<Div>| {
            this.bg(danger.opacity(0.08))
        })
        .when(row.is_greyed(), |this: Stateful<Div>| this.opacity(0.5))
    }

    fn context_menu(
        &mut self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> PopupMenu {
        let Some(row) = self.rows.get(row_ix) else {
            return menu;
        };
        let editable = !matches!(row.kind, RowKind::UnboundByUser | RowKind::Stale);
        let can_disable = row.bind.origin != Origin::Omarchist && row.kind == RowKind::Plain;
        // A bind the user added is deleted, not reset: there is no default.
        let (reset_label, reset_action): (&str, Box<dyn Action>) = if row.kind == RowKind::Custom {
            ("Delete keybind", Box::new(DisableRow(row_ix)))
        } else {
            ("Reset to default", Box::new(ResetRow(row_ix)))
        };
        menu.action_context(self.page_focus.clone())
            .menu_with_disabled("Edit", Box::new(EditRow(row_ix)), !editable)
            .menu("Copy command", Box::new(CopyCommand(row_ix)))
            .separator()
            .when(row.kind != RowKind::Custom, |menu| {
                menu.menu_with_disabled("Disable", Box::new(DisableRow(row_ix)), !can_disable)
            })
            .menu_with_disabled(reset_label, reset_action, row.override_ix.is_none())
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(row) = self.rows.get(row_ix) else {
            return div().into_any_element();
        };
        match col_ix {
            0 => self.render_edit_cell(row, row_ix, cx),
            1 => self.render_action_cell(row, row_ix, cx),
            2 => chord_chips(&row.bind.chord, row.is_greyed(), cx),
            3 => self.render_command_cell(row, row_ix, cx),
            _ => self.render_source_cell(row, row_ix, cx),
        }
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let (icon, text, is_error) = match (&self.error, self.empty_reason) {
            (Some(error), _) => (IconName::TriangleAlert, error.clone(), true),
            (None, EmptyReason::Loading) => (
                IconName::LoaderCircle,
                "Reading keybinds…".to_string(),
                false,
            ),
            (None, EmptyReason::NoBinds) => (
                IconName::Inbox,
                "No keybinds were found in your Hyprland config".to_string(),
                false,
            ),
            (None, EmptyReason::NoMatches) => (
                IconName::Search,
                "No keybinds match your search".to_string(),
                false,
            ),
            (None, EmptyReason::NoConflicts) => (
                IconName::CircleCheck,
                "No conflicting keybinds".to_string(),
                false,
            ),
            (None, EmptyReason::NoModified) => (
                IconName::Inbox,
                "You have not changed any keybinds yet".to_string(),
                false,
            ),
            (None, EmptyReason::Error) => (
                IconName::TriangleAlert,
                "Keybinds could not be read".to_string(),
                true,
            ),
        };
        let color = if is_error {
            theme.danger
        } else {
            theme.muted_foreground
        };

        h_flex()
            .size_full()
            .p_8()
            .gap_3()
            .items_center()
            .justify_center()
            .text_color(color)
            .child(Icon::new(icon).size_6())
            .child(
                div()
                    .max_w(px(640.))
                    .text_sm()
                    .child(selectable("kb-empty", text)),
            )
            .into_any_element()
    }
}
