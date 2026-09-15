// The Keybinds page: every effective Hyprland keybind (Omarchy defaults,
// the user's bindings.lua, Omarchist's overrides) in a searchable table.
//
// Data flow: `refresh` scans the config on a background thread
// (`scan_keybinds`) and loads `keybinds.json`; `rebuild_rows` combines the
// two with conflict detection, the active filter, and the search query, and
// hands the rows to the table delegate.
use std::collections::HashMap;

use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    table::{Table, TableEvent, TableState},
    v_flex,
};

use crate::system::keybinds::conflicts::find_conflicts;
use crate::system::keybinds::overrides::{KeybindOverrides, Override};
use crate::system::keybinds::replay::{ScanResult, scan_keybinds};
use crate::system::keybinds::search::score;
use crate::system::keybinds::store::load_overrides;
use crate::system::keybinds::{BindStatus, Keybind, Origin};
use crate::ui::keybinds_page::keybinds_table::{
    EmptyReason, KeybindRow, KeybindsTableDelegate, RowKind,
};

const KEY_CONTEXT: &str = "KeybindsPage";

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
    filter: KeybindFilter,
    loading: bool,
    error: Option<String>,
    counts: Counts,
    _subscriptions: Vec<Subscription>,
}

impl KeybindsView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| {
            TableState::new(KeybindsTableDelegate::new(), window, cx)
                .row_selectable(true)
                .col_resizable(true)
        });
        let search = cx
            .new(|cx| InputState::new(window, cx).placeholder("Search actions, keys, or commands"));

        let subscriptions = vec![
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
            focus_handle: cx.focus_handle(),
            scan: None,
            overrides: KeybindOverrides::default(),
            table,
            search,
            filter: KeybindFilter::All,
            loading: false,
            error: None,
            counts: Counts::default(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
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

    fn on_row_activated(&mut self, _row_ix: usize, _window: &mut Window, cx: &mut Context<Self>) {
        // Editing arrives with the keybind dialog.
        cx.notify();
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
        } else if !query.trim().is_empty() {
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
        cx.notify();
    }

    fn render_filters(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("keybinds-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
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
                        div().flex_1().min_w(px(220.)).child(
                            Input::new(&self.search)
                                .cleanable(true)
                                .prefix(Icon::new(IconName::Search).size_4()),
                        ),
                    )
                    .child(self.render_filters(cx))
                    .child(
                        Button::new("kb-refresh")
                            .ghost()
                            .small()
                            .icon(IconName::Undo2)
                            .tooltip("Rescan your Hyprland config")
                            .cursor_pointer()
                            .loading(self.loading)
                            .on_click(cx.listener(|this, _, _window, cx| this.refresh(cx))),
                    ),
            )
            .children(self.render_notice(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(Table::new(&self.table).stripe(true).bordered(true)),
            )
            .child(self.render_footer(cx))
    }
}
