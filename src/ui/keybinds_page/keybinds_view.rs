// The Keybinds page: every effective Hyprland keybind (Omarchy defaults,
// the user's bindings.lua, Omarchist's overrides) in a searchable table.
//
// Data flow: `refresh` scans the config on a background thread
// (`scan_keybinds`) and loads `keybinds.json`; `rebuild_rows` combines the
// two with conflict detection, the active filter, and the search query, and
// hands the rows to the table delegate.
use std::collections::HashMap;
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::WindowExt;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    table::{DataTable, TableDelegate, TableEvent, TableState},
    v_flex,
};

use crate::system::keybinds::chord::{Chord, ModMask};
use crate::system::keybinds::conflicts::find_conflicts;
use crate::system::keybinds::overrides::{KeybindOverrides, Override, restore_specs};
use crate::system::keybinds::replay::{ScanResult, scan_keybinds};
use crate::system::keybinds::search::score;
use crate::system::keybinds::store::{load_overrides, save_overrides};
use crate::system::keybinds::{BindIdentity, BindStatus, Keybind, Origin};
use crate::ui::focus;
use crate::ui::keybinds_page::keybind_dialog::{
    DialogMode, KeybindDialog, KeybindDialogEvent, open_keybind_dialog,
};
use crate::ui::keybinds_page::keybinds_table::{
    EmptyReason, KeybindRow, KeybindsTableDelegate, RowKind,
};
use crate::ui::keybinds_page::keystroke_input::{KeystrokeInput, KeystrokeInputEvent};

const KEY_CONTEXT: &str = "KeybindsPage";
/// Wraps the search box (text or keystroke) so Down/Escape can hand off to
/// the table from inside the input.
pub const SEARCH_CONTEXT: &str = "KeybindsSearch";
/// Wraps the filter buttons: one tab stop, left/right cycle.
pub const FILTERS_CONTEXT: &str = "KeybindsFilters";
/// Wraps the table: row actions and Home/End/PageUp/PageDown.
pub const TABLE_CONTEXT: &str = "KeybindsTable";
/// Rows moved by PageUp/PageDown when the table has not reported its
/// visible range yet.
const FALLBACK_PAGE_ROWS: usize = 12;

pub mod keybinds_nav {
    gpui::actions!(
        keybinds,
        [
            FocusSearch,
            FocusTable,
            ClearSearch,
            ToggleChordSearch,
            AddKeybind,
            EditSelected,
            DisableSelected,
            CopySelectedCommand,
            FilterPrev,
            FilterNext,
            TableFirst,
            TableLast,
            TablePageUp,
            TablePageDown,
        ]
    );

    #[derive(gpui::Action, Clone, PartialEq, Eq, Debug)]
    #[action(namespace = keybinds, no_json)]
    pub struct SetFilter(pub usize);
}
use keybinds_nav::*;

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = keybinds, no_json)]
pub struct EditRow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = keybinds, no_json)]
pub struct DisableRow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = keybinds, no_json)]
pub struct ResetRow(pub usize);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = keybinds, no_json)]
pub struct CopyCommand(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindFilter {
    All,
    Modified,
    Conflicts,
    Default,
    User,
}

impl KeybindFilter {
    const ALL: [KeybindFilter; 5] = [
        KeybindFilter::All,
        KeybindFilter::Modified,
        KeybindFilter::Conflicts,
        KeybindFilter::Default,
        KeybindFilter::User,
    ];

    fn label(self) -> &'static str {
        match self {
            KeybindFilter::All => "All",
            KeybindFilter::Modified => "Modified",
            KeybindFilter::Conflicts => "Conflicts",
            KeybindFilter::Default => "Omarchy",
            KeybindFilter::User => "Mine",
        }
    }

    fn accepts(self, row: &KeybindRow) -> bool {
        match self {
            KeybindFilter::All => true,
            KeybindFilter::Modified => matches!(
                row.kind,
                RowKind::Modified { .. } | RowKind::Custom | RowKind::Disabled
            ),
            KeybindFilter::Conflicts => row.conflict,
            KeybindFilter::Default => row.bind.origin == Origin::Default,
            KeybindFilter::User => row.bind.origin != Origin::Default,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Counts {
    total: usize,
    modified: usize,
    conflicts: usize,
}

pub struct KeybindsView {
    pub focus_handle: FocusHandle,
    scan: Option<ScanResult>,
    overrides: KeybindOverrides,
    table: Entity<TableState<KeybindsTableDelegate>>,
    search: Entity<InputState>,
    chord_search: Entity<KeystrokeInput>,
    chord_search_on: bool,
    chord_query: Option<Chord>,
    mods_query: Option<ModMask>,
    filter: KeybindFilter,
    /// The filter strip is one tab stop; left/right cycle the filter.
    filters_focus: FocusHandle,
    loading: bool,
    error: Option<String>,
    counts: Counts,
    dialog: Option<(Entity<KeybindDialog>, Subscription)>,
    /// Row to select once the rescan after a save completes.
    pending_reselect: Option<BindIdentity>,
    _subscriptions: Vec<Subscription>,
}

impl KeybindsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let table = cx.new(|cx| {
            TableState::new(KeybindsTableDelegate::new(focus_handle.clone()), window, cx)
                .row_selectable(true)
                .col_resizable(true)
        });
        let search = cx
            .new(|cx| InputState::new(window, cx).placeholder("Search actions, keys, or commands"));

        let chord_search = cx.new(|cx| KeystrokeInput::new(None, true, window, cx));

        // The table is a tab stop; tabbing into it selects the first row.
        let table_focus = table.read(cx).focus_handle(cx).tab_stop(true);

        let subscriptions = vec![
            cx.on_focus(&table_focus, window, |this, _, cx| {
                if this.table.read(cx).selected_row().is_none()
                    && this.table.read(cx).delegate().rows_count(cx) > 0
                {
                    this.table
                        .update(cx, |table, cx| table.set_selected_row(0, cx));
                }
            }),
            cx.subscribe_in(
                &chord_search,
                window,
                |this, _, event: &KeystrokeInputEvent, _window, cx| match event {
                    KeystrokeInputEvent::Changed(chord) => {
                        this.chord_query = chord.clone();
                        this.mods_query = None;
                        this.rebuild_rows(cx);
                    }
                    KeystrokeInputEvent::Pending(mods) => {
                        this.mods_query = *mods;
                        this.rebuild_rows(cx);
                    }
                    KeystrokeInputEvent::Started | KeystrokeInputEvent::Stopped => cx.notify(),
                },
            ),
            cx.subscribe_in(
                &search,
                window,
                |this, _, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::Change) {
                        this.rebuild_rows(cx);
                    }
                },
            ),
            cx.subscribe_in(&table, window, |this, _, event: &TableEvent, window, cx| {
                if let TableEvent::DoubleClickedRow(row_ix) = event {
                    this.on_row_activated(*row_ix, window, cx);
                }
            }),
        ];

        let mut view = Self {
            focus_handle,
            scan: None,
            overrides: KeybindOverrides::default(),
            table,
            search,
            chord_search,
            chord_search_on: false,
            chord_query: None,
            mods_query: None,
            filter: KeybindFilter::All,
            filters_focus: focus::tab_stop(cx),
            loading: false,
            error: None,
            counts: Counts::default(),
            dialog: None,
            pending_reselect: None,
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    /// Focuses the search box.
    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    /// Rescans the Hyprland config and reloads the overrides file.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        self.loading = true;
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_loading(true);
            cx.notify();
        });
        cx.notify();

        cx.spawn(async move |this, cx| {
            let scanned = cx
                .background_executor()
                .spawn(async { (scan_keybinds(), load_overrides()) })
                .await;
            this.update(cx, |this, cx| this.apply_scan(scanned, cx))
                .ok();
        })
        .detach();
    }

    fn apply_scan(
        &mut self,
        (scan, overrides): (
            crate::error::Result<ScanResult>,
            crate::error::Result<KeybindOverrides>,
        ),
        cx: &mut Context<Self>,
    ) {
        self.loading = false;
        match scan {
            Ok(result) => {
                self.scan = Some(result);
                self.error = None;
            }
            Err(e) => {
                self.scan = None;
                self.error = Some(e.to_string());
            }
        }
        match overrides {
            Ok(overrides) => self.overrides = overrides,
            Err(e) => {
                self.error
                    .get_or_insert_with(|| format!("Keybind overrides could not be read: {e}"));
            }
        }
        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_loading(false);
            table.delegate_mut().set_error(self.error.clone());
            cx.notify();
        });
        self.rebuild_rows(cx);
    }

    fn set_filter(&mut self, filter: KeybindFilter, cx: &mut Context<Self>) {
        if self.filter != filter {
            self.filter = filter;
            self.rebuild_rows(cx);
        }
    }

    fn toggle_chord_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.chord_search_on = !self.chord_search_on;
        if self.chord_search_on {
            self.chord_search
                .update(cx, |input, cx| input.start_recording(window, cx));
        } else {
            self.chord_search
                .update(cx, |input, cx| input.clear(window, cx));
            self.chord_query = None;
            self.mods_query = None;
            self.search.update(cx, |input, cx| input.focus(window, cx));
        }
        self.rebuild_rows(cx);
    }

    /// Keystroke search: an exact chord, or every chord holding the
    /// modifiers currently pressed.
    fn matches_chord_query(&self, row: &KeybindRow) -> bool {
        if !self.chord_search_on {
            return true;
        }
        if let Some(chord) = &self.chord_query {
            return row.bind.chord.same_as(chord);
        }
        match self.mods_query {
            Some(mods) => row.bind.chord.mods.contains(mods),
            None => true,
        }
    }

    fn on_row_activated(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open_edit(row_ix, window, cx);
    }

    fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.chord_search_on {
            self.chord_search.update(cx, |input, cx| {
                input.focus_handle(cx).focus(window, cx);
            });
        } else {
            self.search.update(cx, |input, cx| input.focus(window, cx));
        }
    }

    fn focus_table(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focus = self.table.read(cx).focus_handle(cx);
        focus.focus(window, cx);
    }

    /// Escape in the search box: clear it, or move to the table when it is
    /// already empty.
    fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.chord_search_on {
            self.toggle_chord_search(window, cx);
            return;
        }
        if self.search.read(cx).value().is_empty() {
            self.focus_table(window, cx);
        } else {
            self.search
                .update(cx, |input, cx| input.set_value("", window, cx));
        }
    }

    fn selected_row(&self, cx: &App) -> Option<usize> {
        self.table.read(cx).selected_row()
    }

    fn select_row(&mut self, row_ix: usize, cx: &mut Context<Self>) {
        let count = self.table.read(cx).delegate().rows_count(cx);
        if count == 0 {
            return;
        }
        let row_ix = row_ix.min(count - 1);
        self.table.update(cx, |table, cx| {
            table.set_selected_row(row_ix, cx);
            table.scroll_to_row(row_ix, cx);
        });
    }

    fn page_rows(&self, cx: &App) -> usize {
        let visible = self.table.read(cx).visible_range().rows().len();
        if visible > 1 {
            visible - 1
        } else {
            FALLBACK_PAGE_ROWS
        }
    }

    fn cycle_filter(&mut self, delta: isize, cx: &mut Context<Self>) {
        let current = KeybindFilter::ALL
            .iter()
            .position(|f| *f == self.filter)
            .unwrap_or(0) as isize;
        let len = KeybindFilter::ALL.len() as isize;
        let next = (current + delta).rem_euclid(len) as usize;
        self.set_filter(KeybindFilter::ALL[next], cx);
    }

    fn row(&self, row_ix: usize, cx: &App) -> Option<KeybindRow> {
        self.table.read(cx).delegate().row(row_ix).cloned()
    }

    fn snapshot(&self) -> Rc<Vec<Keybind>> {
        Rc::new(
            self.scan
                .as_ref()
                .map(|s| s.binds.clone())
                .unwrap_or_default(),
        )
    }

    fn open_dialog(&mut self, mode: DialogMode, window: &mut Window, cx: &mut Context<Self>) {
        let dialog = open_keybind_dialog(mode, self.snapshot(), window, cx);
        let subscription = cx.subscribe_in(
            &dialog,
            window,
            |this, _, event: &KeybindDialogEvent, window, cx| {
                this.handle_dialog_event(event.clone(), window, cx);
            },
        );
        self.dialog = Some((dialog, subscription));
    }

    fn open_edit(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.row(row_ix, cx) else {
            return;
        };
        if row.kind == RowKind::UnboundByUser {
            window.push_notification(
                "This keybind was removed in your ~/.config/hypr/bindings.lua; edit it there",
                cx,
            );
            return;
        }
        let existing = row
            .override_ix
            .and_then(|ix| self.overrides.overrides.get(ix).cloned());
        self.open_dialog(
            DialogMode::Edit {
                row: Box::new(row),
                existing,
            },
            window,
            cx,
        );
    }

    fn open_add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_dialog(DialogMode::Add, window, cx);
    }

    fn close_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialog.take().is_some() {
            window.close_dialog(cx);
        }
    }

    fn handle_dialog_event(
        &mut self,
        event: KeybindDialogEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let row_ix = self.table.read(cx).selected_row();
        let row = row_ix.and_then(|ix| self.row(ix, cx));
        match event {
            KeybindDialogEvent::Cancel => self.close_dialog(window, cx),
            KeybindDialogEvent::Save(override_) => {
                let reselect = override_.bind().map(|spec| spec.identity());
                let replace_ix = row
                    .as_ref()
                    .filter(|_| matches!(override_, Override::Add { .. }))
                    .and_then(|r| r.override_ix);
                self.commit(
                    |overrides| match replace_ix {
                        Some(ix) if ix < overrides.overrides.len() => {
                            overrides.overrides[ix] = override_;
                        }
                        _ => overrides.upsert(override_),
                    },
                    "Keybind saved",
                    reselect,
                    window,
                    cx,
                );
            }
            KeybindDialogEvent::Disable => {
                if let Some(ix) = row_ix {
                    self.disable_row(ix, window, cx);
                }
            }
            KeybindDialogEvent::Reset => {
                if let Some(ix) = row_ix {
                    self.reset_row(ix, window, cx);
                }
            }
        }
    }

    fn disable_row(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.row(row_ix, cx) else {
            return;
        };
        if row.bind.origin == Origin::Omarchist || row.kind != RowKind::Plain {
            return;
        }
        let binds = self.snapshot();
        let target = row.bind.identity();
        let override_ = Override::Disable {
            target: target.clone(),
            restore: restore_specs(&row.bind, &binds),
        };
        self.commit(
            |overrides| overrides.upsert(override_),
            "Keybind disabled",
            Some(target),
            window,
            cx,
        );
    }

    fn reset_row(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.row(row_ix, cx) else {
            return;
        };
        let Some(ix) = row.override_ix else {
            return;
        };
        let reselect = self
            .overrides
            .overrides
            .get(ix)
            .and_then(|o| o.target().cloned());
        self.commit(
            |overrides| {
                overrides.remove(ix);
            },
            "Keybind reset to default",
            reselect,
            window,
            cx,
        );
    }

    fn copy_command(&mut self, row_ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(row) = self.row(row_ix, cx) {
            cx.write_to_clipboard(ClipboardItem::new_string(
                row.bind.dispatcher.text().to_string(),
            ));
            window.push_notification("Command copied", cx);
        }
    }

    /// Applies `mutate` to a copy of the overrides, saves them (json +
    /// omarchist.lua + hyprctl reload), then rescans so the table shows
    /// what Hyprland now runs.
    fn commit(
        &mut self,
        mutate: impl FnOnce(&mut KeybindOverrides),
        success: &'static str,
        reselect: Option<BindIdentity>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut overrides = self.overrides.clone();
        mutate(&mut overrides);
        match save_overrides(&overrides) {
            Ok(()) => {
                self.overrides = overrides;
                self.close_dialog(window, cx);
                window.push_notification(success, cx);
                self.pending_reselect = reselect;
                self.refresh(cx);
            }
            Err(e) => {
                window.push_notification(format!("Could not save keybind: {e}"), cx);
            }
        }
    }

    fn reselect_pending(&mut self, cx: &mut Context<Self>) {
        let Some(identity) = self.pending_reselect.take() else {
            return;
        };
        let row_ix = self
            .table
            .read(cx)
            .delegate()
            .rows()
            .iter()
            .position(|row| row.bind.identity() == identity);
        if let Some(row_ix) = row_ix {
            self.table.update(cx, |table, cx| {
                table.set_selected_row(row_ix, cx);
                table.scroll_to_row(row_ix, cx);
            });
        }
    }

    /// Every displayable row, before filtering and search.
    fn all_rows(&self) -> Vec<KeybindRow> {
        let Some(scan) = &self.scan else {
            return Vec::new();
        };
        let binds = &scan.binds;

        let mut conflicts_of: HashMap<usize, Vec<usize>> = HashMap::new();
        for conflict in find_conflicts(binds) {
            for &ix in &conflict.rows {
                let others = conflict.rows.iter().copied().filter(|&o| o != ix).collect();
                conflicts_of.insert(ix, others);
            }
        }

        binds
            .iter()
            .enumerate()
            .filter_map(|(ix, bind)| {
                let (kind, override_ix) = self.classify(bind)?;
                let others = conflicts_of.remove(&ix).unwrap_or_default();
                Some(KeybindRow {
                    conflict: !others.is_empty(),
                    conflicts_with: others
                        .iter()
                        .map(|&o| binds[o].label().to_string())
                        .collect(),
                    bind: bind.clone(),
                    kind,
                    override_ix,
                })
            })
            .collect()
    }

    /// Decides how a scanned bind shows up, or `None` to hide it (a default
    /// that an Omarchist rebind replaced is represented by the new row).
    fn classify(&self, bind: &Keybind) -> Option<(RowKind, Option<usize>)> {
        if bind.origin == Origin::Omarchist {
            let identity = bind.identity();
            let produced_by = self
                .overrides
                .overrides
                .iter()
                .position(|o| o.bind().is_some_and(|spec| spec.identity() == identity));
            let kind = match produced_by.map(|ix| &self.overrides.overrides[ix]) {
                Some(Override::Rebind { target, .. }) => RowKind::Modified {
                    original_keys: target.keys.clone(),
                },
                Some(Override::Add { .. }) => RowKind::Custom,
                _ => RowKind::Custom,
            };
            return Some((kind, produced_by));
        }

        let override_ix = self.overrides.find_for_target(&bind.identity());
        let override_ = override_ix.map(|ix| &self.overrides.overrides[ix]);
        let kind = match (bind.status, override_) {
            (BindStatus::Active, _) => RowKind::Plain,
            (
                BindStatus::Unbound {
                    by_origin: Origin::Omarchist,
                    ..
                },
                Some(Override::Rebind { .. }),
            ) => return None,
            (
                BindStatus::Unbound {
                    by_origin: Origin::Omarchist,
                    ..
                },
                _,
            ) => RowKind::Disabled,
            (BindStatus::Unbound { .. }, _) => RowKind::UnboundByUser,
        };
        Some((kind, override_ix))
    }

    fn rebuild_rows(&mut self, cx: &mut Context<Self>) {
        let all = self.all_rows();
        self.counts = Counts {
            total: all.iter().filter(|r| !r.is_greyed()).count(),
            modified: all
                .iter()
                .filter(|r| KeybindFilter::Modified.accepts(r))
                .count(),
            conflicts: all.iter().filter(|r| r.conflict).count(),
        };

        let query = self.search.read(cx).value().to_string();
        let filter = self.filter;
        let mut rows: Vec<(u32, KeybindRow)> = all
            .into_iter()
            .filter(|row| filter.accepts(row))
            .filter(|row| self.matches_chord_query(row))
            .filter_map(|row| score(&query, &row.bind).map(|s| (s, row)))
            .collect();
        if !query.trim().is_empty() {
            rows.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        }
        let rows: Vec<KeybindRow> = rows.into_iter().map(|(_, row)| row).collect();

        let empty_reason = if self.error.is_some() {
            EmptyReason::Error
        } else if self.loading {
            EmptyReason::Loading
        } else if self.scan.as_ref().is_none_or(|s| s.binds.is_empty()) {
            EmptyReason::NoBinds
        } else if !query.trim().is_empty() || self.chord_query.is_some() {
            EmptyReason::NoMatches
        } else {
            match filter {
                KeybindFilter::Conflicts => EmptyReason::NoConflicts,
                KeybindFilter::Modified => EmptyReason::NoModified,
                _ => EmptyReason::NoMatches,
            }
        };

        self.table.update(cx, |table, cx| {
            table.delegate_mut().set_rows(rows, empty_reason);
            table.refresh(cx);
            cx.notify();
        });
        if !self.loading {
            self.reselect_pending(cx);
        }
        cx.notify();
    }

    fn render_filters(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.filters_focus.is_focused(window);
        let ring = focus::focus_border(focused, cx.theme().transparent, cx);
        h_flex()
            .id("kb-filters")
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.filters_focus)
            .on_action(cx.listener(|this, _: &FilterPrev, _, cx| this.cycle_filter(-1, cx)))
            .on_action(cx.listener(|this, _: &FilterNext, _, cx| this.cycle_filter(1, cx)))
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_1()
            .flex_wrap()
            .children(KeybindFilter::ALL.iter().enumerate().map(|(ix, &filter)| {
                let count = match filter {
                    KeybindFilter::Modified => Some(self.counts.modified),
                    KeybindFilter::Conflicts => Some(self.counts.conflicts),
                    _ => None,
                };
                let label = match count {
                    Some(n) if n > 0 => format!("{} ({n})", filter.label()),
                    _ => filter.label().to_string(),
                };
                let button = Button::new(("kb-filter", ix))
                    .label(label)
                    .small()
                    .tab_stop(false)
                    .cursor_pointer();
                let button = if self.filter == filter {
                    button.primary()
                } else {
                    button.ghost()
                };
                button.on_click(cx.listener(move |this, _, _window, cx| {
                    this.set_filter(filter, cx);
                }))
            }))
    }

    fn render_notice(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let scan = self.scan.as_ref()?;
        if scan.warnings.is_empty() {
            return None;
        }
        let theme = cx.theme();
        let text = if scan.complete {
            format!(
                "{} keybind{} could not be read: {}",
                scan.warnings.len(),
                if scan.warnings.len() == 1 { "" } else { "s" },
                scan.warnings[0]
            )
        } else {
            format!(
                "Your hyprland.lua stopped with an error, so this list may be incomplete: {}",
                scan.warnings.last().map(String::as_str).unwrap_or_default()
            )
        };
        Some(
            h_flex()
                .gap_2()
                .items_center()
                .px_3()
                .py_2()
                .rounded(theme.radius)
                .border_1()
                .border_color(theme.warning.opacity(0.4))
                .bg(theme.warning.opacity(0.08))
                .text_sm()
                .text_color(theme.foreground)
                .child(
                    Icon::new(IconName::TriangleAlert)
                        .size_4()
                        .text_color(theme.warning),
                )
                .child(div().min_w_0().truncate().child(text))
                .into_any_element(),
        )
    }

    fn render_footer(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let summary = if self.loading {
            "Reading keybinds…".to_string()
        } else if let Some(error) = &self.error {
            error.clone()
        } else {
            let mut parts = vec![format!("{} keybinds", self.counts.total)];
            if self.counts.modified > 0 {
                parts.push(format!("{} modified", self.counts.modified));
            }
            if self.counts.conflicts > 0 {
                parts.push(format!("{} conflicts", self.counts.conflicts));
            }
            parts.join(" · ")
        };
        h_flex()
            .justify_between()
            .items_center()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(summary)
            .child("Changes are written to ~/.config/hypr/omarchist.lua and applied immediately")
    }
}

impl Render for KeybindsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("keybinds-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.focus_search(window, cx);
            }))
            .on_action(cx.listener(|this, _: &FocusTable, window, cx| {
                this.focus_table(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ClearSearch, window, cx| {
                this.clear_search(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleChordSearch, window, cx| {
                this.toggle_chord_search(window, cx);
            }))
            .on_action(cx.listener(|this, _: &AddKeybind, window, cx| {
                if this.dialog.is_none() {
                    this.open_add(window, cx);
                }
            }))
            .on_action(cx.listener(|this, action: &SetFilter, _, cx| {
                if let Some(filter) = KeybindFilter::ALL.get(action.0).copied() {
                    this.set_filter(filter, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &EditSelected, window, cx| {
                if this.dialog.is_none()
                    && let Some(row_ix) = this.selected_row(cx)
                {
                    this.open_edit(row_ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &DisableSelected, window, cx| {
                if this.dialog.is_none()
                    && let Some(row_ix) = this.selected_row(cx)
                {
                    this.disable_row(row_ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &CopySelectedCommand, window, cx| {
                if let Some(row_ix) = this.selected_row(cx) {
                    this.copy_command(row_ix, window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &TableFirst, _, cx| this.select_row(0, cx)))
            .on_action(cx.listener(|this, _: &TableLast, _, cx| {
                this.select_row(usize::MAX, cx);
            }))
            .on_action(cx.listener(|this, _: &TablePageUp, _, cx| {
                let page = this.page_rows(cx);
                let current = this.selected_row(cx).unwrap_or(0);
                this.select_row(current.saturating_sub(page), cx);
            }))
            .on_action(cx.listener(|this, _: &TablePageDown, _, cx| {
                let page = this.page_rows(cx);
                let current = this.selected_row(cx).unwrap_or(0);
                this.select_row(current + page, cx);
            }))
            .on_action(cx.listener(|this, action: &EditRow, window, cx| {
                this.open_edit(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &DisableRow, window, cx| {
                this.disable_row(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &ResetRow, window, cx| {
                this.reset_row(action.0, window, cx);
            }))
            .on_action(cx.listener(|this, action: &CopyCommand, window, cx| {
                this.copy_command(action.0, window, cx);
            }))
            .size_full()
            .p_4()
            .gap_3()
            .child(
                h_flex()
                    .w_full()
                    .flex_wrap()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .key_context(SEARCH_CONTEXT)
                            .flex_1()
                            .min_w(px(220.))
                            .map(|this| {
                                if self.chord_search_on {
                                    this.child(self.chord_search.clone())
                                } else {
                                    this.child(
                                        Input::new(&self.search)
                                            .cleanable(true)
                                            .prefix(Icon::new(IconName::Search).size_4()),
                                    )
                                }
                            }),
                    )
                    .child({
                        let button = Button::new("kb-chord-search")
                            .small()
                            .icon(Icon::new(Icon::empty()).path("icons/keyboard.svg"))
                            .tooltip_with_action(
                                if self.chord_search_on {
                                    "Back to text search"
                                } else {
                                    "Search by pressing keys"
                                },
                                &ToggleChordSearch,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer();
                        let button = if self.chord_search_on {
                            button.primary()
                        } else {
                            button.ghost()
                        };
                        button.on_click(cx.listener(|this, _, window, cx| {
                            this.toggle_chord_search(window, cx);
                        }))
                    })
                    .child(self.render_filters(window, cx))
                    .child(
                        Button::new("kb-add")
                            .primary()
                            .small()
                            .icon(IconName::Plus)
                            .label("Add keybind")
                            .tooltip_with_action("Add a keybind", &AddKeybind, Some(KEY_CONTEXT))
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| this.open_add(window, cx))),
                    )
                    .child(
                        Button::new("kb-refresh")
                            .ghost()
                            .small()
                            .icon(IconName::Undo2)
                            .tooltip_with_action(
                                "Rescan your Hyprland config",
                                &focus::ReloadPage,
                                None,
                            )
                            .cursor_pointer()
                            .loading(self.loading)
                            .on_click(cx.listener(|this, _, _window, cx| this.refresh(cx))),
                    ),
            )
            .children(self.render_notice(cx))
            .child(
                div()
                    .key_context(TABLE_CONTEXT)
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(DataTable::new(&self.table).stripe(true).bordered(true)),
            )
            .child(self.render_footer(cx))
    }
}
