//! One flow of the gallery, opened from its card: who made it, what it
//! needs, what a reader should know, and every step. Installing hands the
//! flow to the editor's review screen; an update opens the installed flow
//! with the new steps, unsaved, after showing what changes.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Disableable, Icon, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    tag::Tag,
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::apps::{DesktopApp, installed_apps};
use crate::system::flows::catalog::{self, Change, DiffRow, Entry};
use crate::system::flows::requirements::is_installed;
use crate::system::flows::risks::{self, Level};
use crate::system::flows::share::Imported;
use crate::system::flows::store::load_flow;
use crate::system::flows::{Flow, Step};
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::gallery_view::count_label;
use crate::ui::flows_page::share_ui::warning_banner;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::var_token;
use crate::ui::focus;
use crate::ui::text::selectable;

/// A flow on this machine that came from the gallery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The saved flow's id.
    pub id: String,
    /// The gallery version it was installed from.
    pub version: u32,
}

enum Loaded {
    Loading,
    Ready {
        imported: Box<Imported>,
        /// What an update changes in the installed flow's steps.
        diff: Option<Vec<DiffRow>>,
    },
    Failed(String),
}

pub struct GalleryDetail {
    entry: Entry,
    installs: u64,
    verified: bool,
    installed: Option<Installed>,
    loaded: Loaded,
    apps: Vec<DesktopApp>,
    /// Programs the flow needs that are not on this machine.
    missing: Vec<String>,
    body_focus: FocusHandle,
    list_focus: FocusHandle,
    scroll: ScrollHandle,
}

impl GalleryDetail {
    fn new(
        entry: Entry,
        installs: u64,
        verified: bool,
        installed: Option<Installed>,
        cx: &mut Context<Self>,
    ) -> Self {
        let (fetch_entry, local_id) = (entry.clone(), installed.as_ref().map(|i| i.id.clone()));
        cx.spawn(async move |this, cx| {
            let loaded = cx
                .background_spawn(async move {
                    let missing: Vec<String> = fetch_entry
                        .requires
                        .iter()
                        .filter(|program| !is_installed(program))
                        .cloned()
                        .collect();
                    let local: Option<Flow> = local_id.and_then(|id| load_flow(&id).ok());
                    (
                        catalog::fetch_flow(&fetch_entry),
                        local,
                        missing,
                        installed_apps(),
                    )
                })
                .await;
            this.update(cx, |this, cx| {
                let (fetched, local, missing, apps) = loaded;
                this.missing = missing;
                this.apps = apps;
                this.loaded = match fetched {
                    Ok(imported) => {
                        let diff = local
                            .filter(|_| this.update_version().is_some())
                            .map(|local| catalog::diff_steps(&local, &imported.flow));
                        Loaded::Ready {
                            imported: Box::new(imported),
                            diff,
                        }
                    }
                    Err(e) => Loaded::Failed(e.to_string()),
                };
                cx.notify();
            })
            .ok();
        })
        .detach();
        Self {
            entry,
            installs,
            verified,
            installed,
            loaded: Loaded::Loading,
            apps: Vec::new(),
            missing: Vec::new(),
            body_focus: cx.focus_handle(),
            list_focus: focus::tab_stop(cx),
            scroll: ScrollHandle::new(),
        }
    }

    /// The version an installed copy can move up to.
    fn update_version(&self) -> Option<u32> {
        let installed = self.installed.as_ref()?;
        (self.entry.yanked.is_none() && self.entry.version > installed.version)
            .then_some(self.entry.version)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.loaded, Loaded::Ready { .. })
    }

    /// The dialog's main button: install, update, or open what is there.
    fn act(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let updating = self.update_version().is_some();
        let page = match (&self.installed, &self.loaded) {
            (Some(installed), Loaded::Ready { imported, .. }) if updating => {
                ActivePage::FlowUpdate(installed.id.clone(), imported.clone())
            }
            // The new version is still on its way: Ctrl+Enter must not
            // open the old one in its place.
            (Some(_), _) if updating => return,
            (Some(installed), _) => ActivePage::FlowEdit(installed.id.clone()),
            (None, Loaded::Ready { imported, .. }) if self.entry.yanked.is_none() => {
                ActivePage::FlowImport(imported.clone())
            }
            _ => return,
        };
        window.close_dialog(cx);
        emit(cx, AppEvent::Navigate(page));
    }

    fn render_step(
        ix: usize,
        depth: usize,
        step: &Step,
        change: Change,
        comparing: bool,
        summaries: &SummaryContext,
        cx: &App,
    ) -> AnyElement {
        let theme = cx.theme();
        let summary = summaries.summarize(&step.kind);
        let (mark, tint) = match change {
            Change::Same => ("", theme.transparent),
            Change::Added => ("+", theme.success),
            Change::Removed => ("\u{2212}", theme.danger),
        };
        h_flex()
            .gap_2()
            .items_start()
            .pl(px(8. + 18. * depth as f32))
            .pr_2()
            .py_1()
            .rounded(theme.radius)
            .when(change != Change::Same, |this| this.bg(tint.opacity(0.10)))
            .when(change == Change::Removed, |this| this.opacity(0.75))
            // The column of marks is only there when something changes.
            .when(comparing, |this| {
                this.child(
                    div()
                        .w_3()
                        .flex_shrink_0()
                        .text_sm()
                        .text_color(tint)
                        .child(mark),
                )
            })
            .child(
                div()
                    .opacity(if step.enabled { 1. } else { 0.4 })
                    .child(summary.tile(px(24.), cx)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .text_sm()
                    .child(var_token::rich_text(
                        &format!("gallery-step-{ix}"),
                        &summary.title,
                        cx,
                    ))
                    .when(!summary.detail.is_empty(), |this| {
                        this.child(div().text_xs().text_color(theme.muted_foreground).child(
                            var_token::rich_text(
                                &format!("gallery-step-detail-{ix}"),
                                &summary.detail,
                                cx,
                            ),
                        ))
                    }),
            )
            .into_any_element()
    }

    fn render_steps(&self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let summaries = SummaryContext {
            apps: &self.apps,
            flows: &[],
        };
        let note = |id: &'static str, text: String| {
            div()
                .py_4()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(selectable(id, text))
                .into_any_element()
        };
        match &self.loaded {
            Loaded::Loading => note("gallery-steps-loading", "Fetching the steps…".to_string()),
            Loaded::Failed(error) => note("gallery-steps-error", error.clone()),
            Loaded::Ready {
                diff: Some(rows), ..
            } => v_flex()
                .gap_0p5()
                .children(rows.iter().enumerate().map(|(ix, row)| {
                    Self::render_step(ix, row.depth, &row.step, row.change, true, &summaries, cx)
                }))
                .into_any_element(),
            Loaded::Ready { imported, .. } => {
                v_flex()
                    .gap_0p5()
                    .children(imported.flow.walk().into_iter().enumerate().map(
                        |(ix, (path, step))| {
                            Self::render_step(
                                ix,
                                path.len() / 2,
                                step,
                                Change::Same,
                                false,
                                &summaries,
                                cx,
                            )
                        },
                    ))
                    .into_any_element()
            }
        }
    }
}

impl Render for GalleryDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let entry = self.entry.clone();
        let update = self.update_version();
        let ready = self.is_ready();
        let list_focused = self.list_focus.is_focused(window);
        let ring = focus::focus_border(list_focused, theme.border, cx);
        let view = cx.entity();
        let heading = |text: &'static str| {
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted)
                .child(text)
        };
        let mut facts = vec![
            entry.category.clone(),
            format!("Version {}", entry.version),
            format!("Updated {}", entry.updated),
        ];
        if self.installs > 0 {
            facts.insert(2, format!("{} installs", count_label(self.installs)));
        }
        // What the main button does, or nothing when there is nothing to do.
        let action: Option<&'static str> = match (&self.installed, update) {
            (Some(_), Some(_)) => Some("Update"),
            (Some(_), None) => Some("Open"),
            (None, _) if entry.yanked.is_none() => Some("Install"),
            (None, _) => None,
        };

        focus::dialog_body("gallery-detail", &self.body_focus, move |window, cx| {
            view.update(cx, |this, cx| this.act(window, cx));
        })
        .child(
            v_flex()
                .gap_4()
                .child(
                    h_flex()
                        .gap_3()
                        .items_start()
                        .child(icon_tile(&entry.icon, px(44.), cx))
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .items_center()
                                        .text_sm()
                                        .child(selectable(
                                            "gallery-detail-author",
                                            format!("by {}", entry.author),
                                        ))
                                        .when(self.verified, |this| {
                                            this.child(
                                                Icon::new(Icon::empty())
                                                    .path("icons/badge-check.svg")
                                                    .size_4()
                                                    .text_color(theme.primary),
                                            )
                                        }),
                                )
                                .child(
                                    div().text_xs().text_color(muted).child(selectable(
                                        "gallery-detail-facts",
                                        facts.join(" · "),
                                    )),
                                ),
                        )
                        .children(self.installed.as_ref().map(|installed| {
                            Tag::secondary()
                                .small()
                                .child(format!("Installed: version {}", installed.version))
                        })),
                )
                .child(div().text_sm().child(selectable(
                    "gallery-detail-description",
                    entry.description.clone(),
                )))
                .when(!entry.tags.is_empty(), |this| {
                    this.child(
                        h_flex().gap_1().flex_wrap().children(
                            entry
                                .tags
                                .iter()
                                .map(|tag| Tag::secondary().small().child(tag.clone())),
                        ),
                    )
                })
                .when_some(entry.yanked.clone(), |this, reason| {
                    this.child(warning_banner(
                        "gallery-detail-pulled",
                        if reason.is_empty() {
                            "Pulled from the gallery".to_string()
                        } else {
                            format!("Pulled from the gallery: {reason}")
                        },
                        cx,
                    ))
                })
                .when(!entry.requires.is_empty(), |this| {
                    this.child(
                        v_flex().gap_1p5().child(heading("NEEDS")).child(
                            h_flex()
                                .id("gallery-detail-needs")
                                .test_support()
                                .gap_1()
                                .flex_wrap()
                                .children(entry.requires.iter().map(|program| {
                                    if self.missing.contains(program) {
                                        Tag::danger()
                                            .small()
                                            .child(format!("{program} (not installed)"))
                                    } else {
                                        Tag::secondary().small().child(program.clone())
                                    }
                                })),
                        ),
                    )
                })
                .when(!entry.risks.is_empty(), |this| {
                    this.child(
                        v_flex()
                            .id("gallery-detail-risks")
                            .test_support()
                            .gap_1p5()
                            .child(heading("WORTH KNOWING"))
                            .children(entry.risks.iter().enumerate().map(|(ix, what)| {
                                let danger = risks::level_of(what) == Level::Danger;
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .text_sm()
                                    .child(
                                        Icon::new(Icon::empty())
                                            .path("icons/shield-alert.svg")
                                            .size_4()
                                            .flex_shrink_0()
                                            .text_color(if danger {
                                                theme.danger
                                            } else {
                                                theme.warning
                                            }),
                                    )
                                    .child(selectable(("gallery-risk", ix), what.clone()))
                            })),
                    )
                })
                .child(
                    v_flex()
                        .gap_1p5()
                        .child(heading(if update.is_some() && self.installed.is_some() {
                            "WHAT CHANGES"
                        } else {
                            "STEPS"
                        }))
                        .child(
                            div()
                                .id("gallery-detail-steps")
                                .max_h(focus::dialog_height(300., window))
                                .overflow_y_scroll()
                                .track_scroll(&self.scroll)
                                .rounded(theme.radius)
                                .border_1()
                                .border_color(ring)
                                .child(
                                    focus::scroll_area(&self.scroll)
                                        .id("gallery-detail-list")
                                        .test_support()
                                        .track_focus(&self.list_focus)
                                        .p_1()
                                        .child(self.render_steps(cx)),
                                ),
                        ),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .gap_2()
                        .child(
                            Button::new("gallery-report")
                                .ghost()
                                .small()
                                .icon(Icon::new(Icon::empty()).path("icons/flag.svg"))
                                .label("Report")
                                .cursor_pointer()
                                .on_click({
                                    let url = catalog::report_url(&entry);
                                    move |_, _, cx| cx.open_url(&url)
                                }),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("gallery-close")
                                        .outline()
                                        .small()
                                        .label("Close")
                                        .cursor_pointer()
                                        .on_click(|_, window, cx| window.close_dialog(cx)),
                                )
                                .children(action.map(|label| {
                                    Button::new("gallery-act")
                                        .primary()
                                        .small()
                                        .label(label)
                                        // Opening what is installed needs
                                        // nothing from the network.
                                        .disabled(!ready && label != "Open")
                                        .cursor_pointer()
                                        .on_click(
                                            cx.listener(|this, _, window, cx| this.act(window, cx)),
                                        )
                                })),
                        ),
                ),
        )
    }
}

/// Opens the details of one gallery flow. `installed` names the copy of
/// it on this machine, when there is one.
pub fn open_gallery_detail(
    entry: Entry,
    installs: u64,
    verified: bool,
    installed: Option<Installed>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<GalleryDetail> {
    let title: SharedString = entry.name.clone().into();
    let dialog = cx.new(|cx| GalleryDetail::new(entry, installs, verified, installed, cx));
    let view = dialog.clone();
    let list_focus = dialog.read(cx).list_focus.clone();
    window.open_dialog(cx, move |d, window, _| {
        d.title(title.clone())
            .w(focus::dialog_width(680., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .child(view.clone())
    });
    list_focus.focus(window, cx);
    dialog
}
