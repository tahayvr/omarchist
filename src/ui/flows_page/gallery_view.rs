//! The Gallery page: flows other people shared, from the signed catalog.
//! A card opens the flow's details; installing goes through the editor's
//! review screen, so nothing from here is saved or run without a look.
use std::collections::HashMap;

use crate::ui::app_events::{AppEvent, emit};
use crate::ui::heading;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    tag::Tag,
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::config::config_setup::settings;
use crate::system::flows::StepKind;
use crate::system::flows::catalog::{self, CATEGORIES, Catalog, Entry, Standing};
use crate::system::flows::store::load_flows;
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::gallery_detail::{Installed, open_gallery_detail};
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::focus;
use crate::ui::keybinds_page::keybinds_view::{FILTERS_CONTEXT, keybinds_nav};
use crate::ui::menu::app_menu;
use crate::ui::text::selectable;

const KEY_CONTEXT: &str = "FlowGalleryPage";
/// Wraps the search box so Escape clears it before it leaves the page.
pub const SEARCH_CONTEXT: &str = "GallerySearch";

pub mod gallery_nav {
    gpui::actions!(flow_gallery, [ClearSearch]);
}
use gallery_nav::*;

/// Which flows the page lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    All,
    Category(&'static str),
    /// The ones saved on this machine.
    Installed,
    /// The ones published under the user's GitHub name.
    Mine,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Category(name) => name,
            Filter::Installed => "Installed",
            Filter::Mine => "Mine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Popular,
    Newest,
}

/// `1204` as `1.2k`, so a count fits a card.
pub fn count_label(count: u64) -> String {
    let short = |value: f64, unit: &str| {
        let text = format!("{value:.1}");
        format!("{}{unit}", text.trim_end_matches(".0"))
    };
    match count {
        0..=999 => count.to_string(),
        1_000..=999_999 => short(count as f64 / 1_000., "k"),
        _ => short(count as f64 / 1_000_000., "M"),
    }
}

/// The icons of a flow's top-level steps, drawn from their types alone.
pub fn kinds_strip(kinds: &[String], cx: &App) -> impl IntoElement {
    let theme = cx.theme();
    let summaries = SummaryContext {
        apps: &[],
        flows: &[],
    };
    /// More than this and the row says how many are left.
    const SHOWN: usize = 8;
    let mut row = h_flex().gap_1().items_center().flex_wrap();
    for (ix, kind) in kinds
        .iter()
        .filter_map(|tag| StepKind::from_type_tag(tag))
        .take(SHOWN)
        .enumerate()
    {
        if ix > 0 {
            row = row.child(
                Icon::new(Icon::empty())
                    .path("icons/chevron-right.svg")
                    .size_3()
                    .text_color(theme.muted_foreground),
            );
        }
        row = row.child(summaries.summarize(&kind).tile(px(22.), cx));
    }
    if kinds.len() > SHOWN {
        row = row.child(
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(format!("+{}", kinds.len() - SHOWN)),
        );
    }
    row
}

pub struct GalleryView {
    pub focus_handle: FocusHandle,
    search: Entity<InputState>,
    query: String,
    catalog: Option<Catalog>,
    /// Why there is nothing to show, when there is nothing.
    error: Option<String>,
    loading: bool,
    /// The flows on this machine that came from the gallery, by slug.
    installed: HashMap<String, Installed>,
    filter: Filter,
    sort: Sort,
    filter_focus: FocusHandle,
    sort_focus: FocusHandle,
    /// The GitHub name flows are published under, for **Mine**.
    author: String,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl GalleryView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search the gallery"));
        let subscriptions = vec![
            cx.subscribe(&search, |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.query = input.read(cx).value().to_string();
                    cx.notify();
                }
            }),
        ];
        let mut view = Self {
            focus_handle: cx.focus_handle(),
            search,
            query: String::new(),
            catalog: None,
            error: None,
            loading: false,
            installed: HashMap::new(),
            filter: Filter::All,
            sort: Sort::Popular,
            filter_focus: focus::tab_stop(cx),
            sort_focus: focus::tab_stop(cx),
            author: String::new(),
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        };
        view.refresh(cx);
        view
    }

    pub fn focus_entry(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search.update(cx, |input, cx| input.focus(window, cx));
    }

    pub fn catalog(&self) -> Option<&Catalog> {
        self.catalog.as_ref()
    }

    /// Shows the copy kept from the last visit at once, then the gallery
    /// as it is now, both read off the UI thread.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let local = cx
                .background_spawn(async {
                    let installed: HashMap<String, Installed> = load_flows()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|flow| {
                            let (slug, version) = catalog::source_of(&flow)?;
                            Some((
                                slug,
                                Installed {
                                    id: flow.id,
                                    version,
                                },
                            ))
                        })
                        .collect();
                    (installed, catalog::cached(), settings().gallery_author)
                })
                .await;
            this.update(cx, |this, cx| {
                let (installed, cached, author) = local;
                this.installed = installed;
                this.author = author;
                if this.catalog.is_none() {
                    this.catalog = cached;
                }
                cx.notify();
            })
            .ok();
            let loaded = cx.background_spawn(async { catalog::load() }).await;
            this.update(cx, |this, cx| {
                this.loading = false;
                match loaded {
                    Ok(catalog) => {
                        if let Some(notice) = catalog.notice.clone() {
                            emit(cx, AppEvent::Warning(notice));
                        }
                        this.catalog = Some(catalog);
                        this.error = None;
                    }
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn back(&self, cx: &mut Context<Self>) {
        emit(cx, AppEvent::Navigate(ActivePage::Flows));
    }

    /// Escape in the search box: an empty box leaves the page.
    fn clear_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.query.is_empty() {
            self.back(cx);
            return;
        }
        // Setting the value tells nobody, so the query follows by hand.
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.query.clear();
        cx.notify();
    }

    fn filters(&self) -> Vec<Filter> {
        let mut filters = vec![Filter::All];
        // Only the categories that hold something.
        if let Some(catalog) = &self.catalog {
            filters.extend(
                CATEGORIES
                    .iter()
                    .filter(|category| {
                        catalog
                            .index
                            .flows
                            .iter()
                            .any(|entry| entry.yanked.is_none() && entry.category == **category)
                    })
                    .map(|category| Filter::Category(category)),
            );
        }
        if !self.installed.is_empty() {
            filters.push(Filter::Installed);
        }
        if !self.author.is_empty() {
            filters.push(Filter::Mine);
        }
        filters
    }

    fn set_filter(&mut self, filter: Filter, cx: &mut Context<Self>) {
        self.filter = filter;
        self.scroll.set_offset(point(px(0.), px(0.)));
        cx.notify();
    }

    fn cycle_filter(&mut self, delta: isize, cx: &mut Context<Self>) {
        let filters = self.filters();
        let ix = filters
            .iter()
            .position(|filter| *filter == self.filter)
            .unwrap_or(0) as isize;
        let next = (ix + delta).rem_euclid(filters.len() as isize) as usize;
        self.set_filter(filters[next], cx);
    }

    fn set_sort(&mut self, sort: Sort, cx: &mut Context<Self>) {
        self.sort = sort;
        cx.notify();
    }

    fn installs(&self, slug: &str) -> u64 {
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.installs.get(slug).copied())
            .unwrap_or(0)
    }

    fn standing(&self, entry: &Entry) -> Option<Standing> {
        let installed = self.installed.get(&entry.slug)?;
        Some(if entry.yanked.is_some() {
            Standing::Pulled(entry.yanked.clone().unwrap_or_default())
        } else if entry.version > installed.version {
            Standing::Update(entry.version)
        } else {
            Standing::Current
        })
    }

    /// The entries the search, the filter and the sort leave, in order. A
    /// pulled flow is only listed for the people who installed it.
    pub fn shown(&self) -> Vec<&Entry> {
        let Some(catalog) = &self.catalog else {
            return Vec::new();
        };
        let query = self.query.trim().to_lowercase();
        let mut entries: Vec<&Entry> = catalog
            .index
            .flows
            .iter()
            .filter(|entry| entry.yanked.is_none() || self.installed.contains_key(&entry.slug))
            .filter(|entry| match self.filter {
                Filter::All => true,
                Filter::Category(category) => entry.category == category,
                Filter::Installed => self.installed.contains_key(&entry.slug),
                Filter::Mine => entry.author.eq_ignore_ascii_case(&self.author),
            })
            .filter(|entry| {
                query.is_empty()
                    || entry.name.to_lowercase().contains(&query)
                    || entry.description.to_lowercase().contains(&query)
                    || entry.author.to_lowercase().contains(&query)
                    || entry.tags.iter().any(|tag| tag.contains(&query))
            })
            .collect();
        match self.sort {
            Sort::Popular => entries.sort_by(|a, b| {
                self.installs(&b.slug)
                    .cmp(&self.installs(&a.slug))
                    .then_with(|| a.name.cmp(&b.name))
            }),
            Sort::Newest => {
                entries.sort_by(|a, b| b.updated.cmp(&a.updated).then_with(|| a.name.cmp(&b.name)))
            }
        }
        entries
    }

    fn open_detail(&mut self, slug: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(catalog) = &self.catalog else {
            return;
        };
        let Some(entry) = catalog.index.entry(slug) else {
            return;
        };
        open_gallery_detail(
            entry.clone(),
            self.installs(slug),
            catalog.index.is_verified(&entry.author),
            self.installed.get(slug).cloned(),
            window,
            cx,
        );
    }

    fn render_card(&self, ix: usize, entry: &Entry, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let verified = self
            .catalog
            .as_ref()
            .is_some_and(|catalog| catalog.index.is_verified(&entry.author));
        let installs = self.installs(&entry.slug);
        let standing = self.standing(entry);
        let slug = entry.slug.clone();
        let card = Button::new(("gallery-card", ix))
            .outline()
            .flex_1()
            .min_w_0()
            .h_auto()
            .p_4()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| this.open_detail(&slug, window, cx)))
            .child(
                v_flex()
                    .gap_2()
                    .items_start()
                    .text_left()
                    .w_full()
                    .child(
                        h_flex()
                            .w_full()
                            .gap_3()
                            .items_start()
                            .child(icon_tile(&entry.icon, px(36.), cx))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .truncate()
                                            .child(selectable("gallery-name", entry.name.clone())),
                                    )
                                    .child(
                                        h_flex()
                                            .gap_1()
                                            .items_center()
                                            .text_xs()
                                            .font_weight(FontWeight::NORMAL)
                                            .text_color(theme.muted_foreground)
                                            .child(selectable(
                                                "gallery-author",
                                                format!("by {}", entry.author),
                                            ))
                                            .when(verified, |this| {
                                                this.child(
                                                    Icon::new(Icon::empty())
                                                        .path("icons/badge-check.svg")
                                                        .size_3()
                                                        .text_color(theme.primary),
                                                )
                                            })
                                            .child(format!("· {}", entry.category)),
                                    ),
                            )
                            .children(match standing {
                                Some(Standing::Update(_)) => {
                                    Some(Tag::primary().small().child("Update"))
                                }
                                Some(Standing::Pulled(_)) => {
                                    Some(Tag::danger().small().child("Pulled"))
                                }
                                Some(_) => Some(Tag::secondary().small().child("Installed")),
                                None => None,
                            }),
                    )
                    .child(
                        div()
                            .w_full()
                            .whitespace_normal()
                            .text_sm()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.muted_foreground)
                            // Always two lines tall, so the cards of a
                            // row line up.
                            .min_h(rems(2.5))
                            .line_clamp(2)
                            .text_ellipsis()
                            .child(selectable("gallery-description", entry.description.clone())),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .items_center()
                            .justify_between()
                            .child(kinds_strip(&entry.kinds, cx))
                            .when(installs > 0, |this| {
                                this.child(
                                    h_flex()
                                        .flex_shrink_0()
                                        .gap_1()
                                        .items_center()
                                        .text_xs()
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(theme.muted_foreground)
                                        .child(
                                            Icon::new(Icon::empty())
                                                .path("icons/download.svg")
                                                .size_3(),
                                        )
                                        .child(count_label(installs)),
                                )
                            }),
                    ),
            );
        // Each cell, padding included, takes an equal share of its row.
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .child(card)
            .into_any_element()
    }

    fn render_grid(
        &self,
        entries: &[&Entry],
        offset: usize,
        columns: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let columns = columns.max(1);
        let mut cards: Vec<AnyElement> = entries
            .iter()
            .enumerate()
            .map(|(ix, entry)| self.render_card(offset + ix, entry, cx))
            .collect();
        while !cards.len().is_multiple_of(columns) {
            cards.push(div().flex_1().min_w_0().into_any_element());
        }
        let mut rows = Vec::new();
        let mut cards = cards.into_iter();
        loop {
            let row: Vec<AnyElement> = cards.by_ref().take(columns).collect();
            if row.is_empty() {
                break;
            }
            rows.push(h_flex().gap_4().items_stretch().children(row));
        }
        v_flex().gap_4().children(rows)
    }

    fn render_filters(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let ring = focus::focus_border(self.filter_focus.is_focused(window), theme.transparent, cx);
        h_flex()
            .id("gallery-filters")
            .test_support()
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.filter_focus)
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterPrev, _, cx| this.cycle_filter(-1, cx)),
            )
            .on_action(
                cx.listener(|this, _: &keybinds_nav::FilterNext, _, cx| this.cycle_filter(1, cx)),
            )
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_0p5()
            .flex_wrap()
            .children(self.filters().into_iter().map(|filter| {
                let selected = self.filter == filter;
                div()
                    .id(SharedString::from(format!(
                        "gallery-filter-{}",
                        filter.label().to_lowercase()
                    )))
                    .test_support()
                    .px_2()
                    .py_0p5()
                    .rounded(theme.radius)
                    .text_xs()
                    .when(selected, |this| {
                        this.bg(theme.primary).text_color(theme.primary_foreground)
                    })
                    .when(!selected, |this| {
                        this.text_color(theme.muted_foreground)
                            .hover(|this| this.bg(theme.secondary).text_color(theme.foreground))
                    })
                    .cursor_pointer()
                    .child(filter.label())
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.filter_focus.focus(window, cx);
                        this.set_filter(filter, cx);
                    }))
            }))
    }

    fn render_sort(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let ring = focus::focus_border(self.sort_focus.is_focused(window), theme.transparent, cx);
        let option = |sort: Sort, label: &'static str, id: &'static str| {
            let selected = self.sort == sort;
            div()
                .id(id)
                .test_support()
                .px_2()
                .py_0p5()
                .rounded(theme.radius)
                .text_xs()
                .when(selected, |this| {
                    this.bg(theme.secondary).text_color(theme.foreground)
                })
                .when(!selected, |this| {
                    this.text_color(theme.muted_foreground)
                        .hover(|this| this.text_color(theme.foreground))
                })
                .cursor_pointer()
                .child(label)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.sort_focus.focus(window, cx);
                    this.set_sort(sort, cx);
                }))
        };
        let other = |sort: Sort| match sort {
            Sort::Popular => Sort::Newest,
            Sort::Newest => Sort::Popular,
        };
        h_flex()
            .id("gallery-sort")
            .test_support()
            .key_context(FILTERS_CONTEXT)
            .track_focus(&self.sort_focus)
            .on_action(
                cx.listener(move |this, _: &keybinds_nav::FilterPrev, _, cx| {
                    this.set_sort(other(this.sort), cx)
                }),
            )
            .on_action(
                cx.listener(move |this, _: &keybinds_nav::FilterNext, _, cx| {
                    this.set_sort(other(this.sort), cx)
                }),
            )
            .flex_shrink_0()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .p_0p5()
            .gap_0p5()
            .child(option(Sort::Popular, "Popular", "gallery-sort-popular"))
            .child(option(Sort::Newest, "Newest", "gallery-sort-newest"))
    }
}

impl Render for GalleryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let width = window.viewport_size().width;
        let columns = if width < px(700.) {
            1
        } else if width < px(1100.) {
            2
        } else {
            3
        };
        let entries = self.shown();
        // The picks sit on top only while nothing narrows the list.
        let browsing = self.filter == Filter::All && self.query.trim().is_empty();
        let featured: Vec<&Entry> = match &self.catalog {
            Some(catalog) if browsing => catalog
                .index
                .featured
                .iter()
                .filter_map(|slug| entries.iter().find(|entry| &entry.slug == slug).copied())
                .collect(),
            _ => Vec::new(),
        };
        let rest: Vec<&Entry> = entries
            .iter()
            .filter(|entry| !featured.iter().any(|f| f.slug == entry.slug))
            .copied()
            .collect();
        let nothing_loaded = self.catalog.is_none();
        let label = |text: &'static str, cx: &App| heading::section(text, cx);

        v_flex()
            .id("gallery-page")
            .key_context(KEY_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &app_menu::NavigateBack, _, cx| this.back(cx)))
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .flex_wrap()
                    .child(
                        Button::new("gallery-back")
                            .label("Back")
                            .compact()
                            .tooltip_with_action(
                                "Back to Flows",
                                &app_menu::NavigateBack,
                                Some(KEY_CONTEXT),
                            )
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.back(cx))),
                    )
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Gallery"))
                    .child(
                        div()
                            .key_context(SEARCH_CONTEXT)
                            .on_action(cx.listener(|this, _: &ClearSearch, window, cx| {
                                this.clear_search(window, cx)
                            }))
                            .flex_1()
                            .min_w(px(200.))
                            .max_w(px(420.))
                            .child(
                                Input::new(&self.search)
                                    .id("gallery-search")
                                    .small()
                                    .cleanable(true)
                                    .prefix(
                                        Icon::new(Icon::empty())
                                            .path("icons/search.svg")
                                            .size_4()
                                            .text_color(muted),
                                    ),
                            ),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("gallery-reload")
                            .ghost()
                            .compact()
                            .loading(self.loading)
                            .icon(Icon::new(Icon::empty()).path("icons/refresh-cw.svg"))
                            .tooltip_with_action("Reload", &focus::ReloadPage, None)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                    ),
            )
            .when(!nothing_loaded, |this| {
                this.child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .justify_between()
                        .child(self.render_filters(window, cx))
                        .child(self.render_sort(window, cx)),
                )
            })
            // What is still in review is on GitHub, not in the gallery yet.
            .when(self.filter == Filter::Mine, |this| {
                let url = catalog::submissions_url(&self.author);
                this.child(
                    h_flex().child(
                        Button::new("gallery-my-requests")
                            .ghost()
                            .small()
                            .icon(Icon::new(Icon::empty()).path("icons/external-link.svg"))
                            .label("Your pull requests on GitHub")
                            .cursor_pointer()
                            .on_click(move |_, _, cx| cx.open_url(&url)),
                    ),
                )
            })
            .child(
                div()
                    .id("gallery-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .pb_8()
                    .when(nothing_loaded && self.loading, |this| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(muted)
                                .child(selectable("gallery-loading", "Opening the gallery…")),
                        )
                    })
                    .when(nothing_loaded && !self.loading, |this| {
                        this.child(
                            v_flex()
                                .id("gallery-unreachable")
                                .test_support()
                                .gap_3()
                                .items_start()
                                .child(div().text_sm().text_color(muted).child(selectable(
                                    "gallery-error",
                                    self.error.clone().unwrap_or_else(|| {
                                        "The gallery could not be opened".to_string()
                                    }),
                                )))
                                .child(
                                    Button::new("gallery-retry")
                                        .outline()
                                        .small()
                                        .label("Try again")
                                        .cursor_pointer()
                                        .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                                ),
                        )
                    })
                    .when(!nothing_loaded && entries.is_empty(), |this| {
                        this.child(
                            div()
                                .id("gallery-none")
                                .test_support()
                                .text_sm()
                                .text_color(muted)
                                .child(selectable("gallery-no-match", "No flow matches")),
                        )
                    })
                    .when(!entries.is_empty(), |this| {
                        this.child(
                            focus::scroll_area(&self.scroll).child(
                                v_flex()
                                    .gap_8()
                                    .max_w(px(1200.))
                                    .when(!featured.is_empty(), |this| {
                                        this.child(
                                            v_flex()
                                                .gap_3()
                                                .child(label("Featured", cx))
                                                .child(self.render_grid(&featured, 0, columns, cx)),
                                        )
                                    })
                                    .when(!rest.is_empty(), |this| {
                                        this.child(
                                            v_flex()
                                                .gap_3()
                                                .when(!featured.is_empty(), |this| {
                                                    this.child(label("All flows", cx))
                                                })
                                                .child(self.render_grid(
                                                    &rest,
                                                    featured.len(),
                                                    columns,
                                                    cx,
                                                )),
                                        )
                                    }),
                            ),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::count_label;

    #[test]
    fn counts_are_shortened() {
        assert_eq!(count_label(0), "0");
        assert_eq!(count_label(999), "999");
        assert_eq!(count_label(1_000), "1k");
        assert_eq!(count_label(1_204), "1.2k");
        assert_eq!(count_label(15_960), "16k");
        assert_eq!(count_label(2_500_000), "2.5M");
    }
}
