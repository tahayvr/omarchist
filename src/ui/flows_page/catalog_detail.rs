//! One flow of the catalog, opened from its card: who made it, what it
//! needs, what a reader should know, and every step with the command it
//! runs in full. This dialog is where a catalog flow is read before it
//! reaches the machine: Install writes the flow file and opens the editor
//! on the installed flow. An update opens the installed flow with the new
//! steps, unsaved, after showing what changes.
use crate::ui::app_events::{AppEvent, emit};
use crate::ui::heading;
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
use crate::system::config::config_setup::settings;
use crate::system::flows::catalog::{self, Change, DiffRow, Entry};
use crate::system::flows::condition::Condition;
use crate::system::flows::requirements::{is_installed, programs_of};
use crate::system::flows::risks::{self, Level, Risk};
use crate::system::flows::share::Imported;
use crate::system::flows::store::{existing_ids, load_flow, save_new_flow};
use crate::system::flows::{Flow, InputFallback, OnError, Step, StepKind, unique_id};
use crate::ui::app_view::ActivePage;
use crate::ui::flows_page::catalog_view::count_label;
use crate::ui::flows_page::flow_card::icon_tile;
use crate::ui::flows_page::share_ui::warning_banner;
use crate::ui::flows_page::step_summary::SummaryContext;
use crate::ui::flows_page::var_token;
use crate::ui::focus;
use crate::ui::notify;
use crate::ui::text::{selectable, title_case};

/// A flow on this machine that came from the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    /// The saved flow's id.
    pub id: String,
    /// The catalog version it was installed from.
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

pub struct CatalogDetail {
    entry: Entry,
    installs: u64,
    verified: bool,
    installed: Option<Installed>,
    loaded: Loaded,
    apps: Vec<DesktopApp>,
    /// What the flow needs: the index's word until the flow is fetched,
    /// then what this Omarchist reads in the steps themselves.
    needs: Vec<String>,
    /// Programs the flow needs that are not on this machine.
    missing: Vec<String>,
    /// What this Omarchist's own heuristics find in the fetched steps,
    /// which may know more than the ones that built the index.
    risks: Option<Vec<Risk>>,
    body_focus: FocusHandle,
    list_focus: FocusHandle,
    scroll: ScrollHandle,
}

impl CatalogDetail {
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
                    let fetched = catalog::fetch_flow(&fetch_entry);
                    // Read from the flow itself once it is here: the index
                    // says what the catalog's build saw, this says what
                    // this Omarchist sees.
                    let (needs, risks) = match &fetched {
                        Ok(imported) => {
                            (needs_of(&imported.flow), Some(risks::risks(&imported.flow)))
                        }
                        Err(_) => (fetch_entry.requires.clone(), None),
                    };
                    let missing: Vec<String> = needs
                        .iter()
                        .filter(|program| !is_installed(program))
                        .cloned()
                        .collect();
                    let local: Option<Flow> = local_id.and_then(|id| load_flow(&id).ok());
                    (fetched, local, needs, missing, risks, installed_apps())
                })
                .await;
            this.update(cx, |this, cx| {
                let (fetched, local, needs, missing, risks, apps) = loaded;
                this.needs = needs;
                this.missing = missing;
                this.risks = risks;
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
        let needs = entry.requires.clone();
        Self {
            entry,
            installs,
            verified,
            installed,
            loaded: Loaded::Loading,
            apps: Vec::new(),
            needs,
            missing: Vec::new(),
            risks: None,
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
                let imported = imported.clone();
                self.install(&imported, window, cx);
                return;
            }
            _ => return,
        };
        window.close_dialog(cx);
        emit(cx, AppEvent::Navigate(page));
    }

    /// Writes the flow to this machine, with no triggers and an id of its
    /// own, and opens the editor on it. The flow has been read here, so
    /// from now on it is the person's: it can be run, changed and saved
    /// like any other.
    fn install(&mut self, imported: &Imported, window: &mut Window, cx: &mut Context<Self>) {
        let mut flow = imported.flow.clone();
        flow.id = unique_id(&flow.name, &existing_ids());
        if let Err(e) = save_new_flow(&flow) {
            notify::error(window, format!("Could not install the flow: {e}"), cx);
            return;
        }
        if settings().catalog_count_installs
            && let Some((slug, version)) = catalog::source_of(&flow)
        {
            cx.background_spawn(async move { catalog::count_install(&slug, version) })
                .detach();
        }
        window.close_dialog(cx);
        notify::success(
            window,
            format!("Installed '{}'", title_case(&flow.name)),
            cx,
        );
        emit(cx, AppEvent::Navigate(ActivePage::FlowEdit(flow.id)));
    }

    /// The text a step hands to a shell or the compositor: what a reader
    /// of somebody else's flow has to see in full.
    fn code_of(kind: &StepKind) -> Option<&str> {
        match kind {
            StepKind::Exec { command, .. } => Some(command),
            StepKind::Lua { expr } => Some(expr),
            StepKind::If {
                condition: Condition::Command { command },
                ..
            } => Some(command),
            _ => None,
        }
    }

    /// How the flow behaves as a whole: on a failing step, and where its
    /// input comes from.
    fn render_behaviour(flow: &Flow, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let mut facts = vec![match flow.on_error {
            OnError::Stop => "Stops at the first step that fails",
            OnError::Continue => "Keeps going when a step fails",
        }];
        match flow.input {
            InputFallback::None => {}
            InputFallback::Selection => facts.push("Starts with the text selected on screen"),
            InputFallback::Clipboard => facts.push("Starts with what is on the clipboard"),
            InputFallback::Ask => facts.push("Asks for its input when started"),
        }
        div()
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(selectable("catalog-detail-behaviour", facts.join(" · ")))
            .into_any_element()
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
        let code = Self::code_of(&step.kind);
        // A command step's detail is its command, shown in full below.
        let detail = (!summary.detail.is_empty() && code != Some(summary.detail.as_str()))
            .then(|| summary.detail.clone());
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
            // The column of marks is only there when something changes;
            // otherwise the steps are numbered as the editor numbers them.
            .child(if comparing {
                div()
                    .w_3()
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(tint)
                    .child(mark)
            } else {
                div()
                    .w_5()
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child(format!("{}", ix + 1))
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
                        &format!("catalog-step-{ix}"),
                        &summary.title,
                        cx,
                    ))
                    .when_some(detail, |this, detail| {
                        this.child(div().text_xs().text_color(theme.muted_foreground).child(
                            var_token::rich_text(&format!("catalog-step-detail-{ix}"), &detail, cx),
                        ))
                    })
                    // The command itself, whole, where it cannot be missed.
                    .when_some(code, |this, code| {
                        this.child(
                            div()
                                .id(ElementId::Name(format!("catalog-step-code-{ix}").into()))
                                .test_support()
                                .mt_1()
                                .px_2()
                                .py_1()
                                .rounded(theme.radius)
                                .bg(theme.secondary)
                                .text_sm()
                                .child(var_token::rich_text(
                                    &format!("catalog-step-command-{ix}"),
                                    code,
                                    cx,
                                )),
                        )
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
            Loaded::Loading => note("catalog-steps-loading", "Fetching the steps…".to_string()),
            Loaded::Failed(error) => note("catalog-steps-error", error.clone()),
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

impl Render for CatalogDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        let entry = self.entry.clone();
        let update = self.update_version();
        let ready = self.is_ready();
        let list_focused = self.list_focus.is_focused(window);
        let ring = focus::focus_border(list_focused, theme.border, cx);
        let view = cx.entity();
        let heading = |text: &'static str, cx: &App| heading::section(text, cx);
        let mut facts = vec![
            entry.category.clone(),
            format!("Version {}", entry.version),
            format!("Updated {}", entry.updated),
        ];
        if self.installs > 0 {
            facts.insert(2, format!("{} installs", count_label(self.installs)));
        }
        // The index's risks until the steps are here, then this
        // Omarchist's own reading of them, step by step.
        let worth_knowing: Vec<(Option<usize>, String)> = match &self.risks {
            Some(found) => found
                .iter()
                .map(|risk| (Some(risk.step), risk.what.to_string()))
                .collect(),
            None => entry
                .risks
                .iter()
                .map(|what| (None, what.clone()))
                .collect(),
        };
        // What the main button does, or nothing when there is nothing to do.
        let action: Option<&'static str> = match (&self.installed, update) {
            (Some(_), Some(_)) => Some("Update"),
            (Some(_), None) => Some("Open"),
            (None, _) if entry.yanked.is_none() => Some("Install"),
            (None, _) => None,
        };

        focus::dialog_body("catalog-detail", &self.body_focus, move |window, cx| {
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
                                            "catalog-detail-author",
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
                                        "catalog-detail-facts",
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
                    "catalog-detail-description",
                    entry.description.clone(),
                )))
                .children(match &self.loaded {
                    Loaded::Ready { imported, .. } => {
                        Some(Self::render_behaviour(&imported.flow, cx))
                    }
                    _ => None,
                })
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
                        "catalog-detail-pulled",
                        if reason.is_empty() {
                            "Pulled from the catalog".to_string()
                        } else {
                            format!("Pulled from the catalog: {reason}")
                        },
                        cx,
                    ))
                })
                .when(!self.needs.is_empty(), |this| {
                    this.child(
                        v_flex().gap_1p5().child(heading("Needs", cx)).child(
                            h_flex()
                                .id("catalog-detail-needs")
                                .test_support()
                                .gap_1()
                                .flex_wrap()
                                .children(self.needs.iter().map(|program| {
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
                .when(!worth_knowing.is_empty(), |this| {
                    this.child(
                        v_flex()
                            .id("catalog-detail-risks")
                            .test_support()
                            .gap_1p5()
                            .child(heading("Worth knowing", cx))
                            .children(worth_knowing.iter().enumerate().map(
                                |(ix, (step, what))| {
                                    let danger = risks::level_of(what) == Level::Danger;
                                    let what = match step {
                                        Some(step) => format!("Step {step}: {what}"),
                                        None => what.to_string(),
                                    };
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
                                        .child(selectable(("catalog-risk", ix), what))
                                },
                            )),
                    )
                })
                .child(
                    v_flex()
                        .gap_1p5()
                        .child(heading(
                            if update.is_some() && self.installed.is_some() {
                                "What changes"
                            } else {
                                "Steps"
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .id("catalog-detail-steps")
                                .max_h(focus::dialog_height(380., window))
                                .overflow_y_scroll()
                                .track_scroll(&self.scroll)
                                .rounded(theme.radius)
                                .border_1()
                                .border_color(ring)
                                .child(
                                    focus::scroll_area(&self.scroll)
                                        .id("catalog-detail-list")
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
                            Button::new("catalog-report")
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
                                    Button::new("catalog-close")
                                        .outline()
                                        .small()
                                        .label("Close")
                                        .cursor_pointer()
                                        .on_click(|_, window, cx| window.close_dialog(cx)),
                                )
                                .children(action.map(|label| {
                                    Button::new("catalog-act")
                                        .primary()
                                        .small()
                                        .when(label == "Install", |this| {
                                            this.icon(
                                                Icon::new(Icon::empty()).path("icons/download.svg"),
                                            )
                                        })
                                        .label(label)
                                        // Opening what is installed needs
                                        // nothing from the network.
                                        .disabled(!ready && label != "Open")
                                        .when(!ready && label != "Open", |this| {
                                            this.tooltip(match &self.loaded {
                                                Loaded::Failed(_) => {
                                                    "The flow could not be fetched"
                                                }
                                                _ => "Fetching the flow…",
                                            })
                                        })
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

/// Every program the flow's enabled steps start and what its author says
/// it needs, each once, in order.
fn needs_of(flow: &Flow) -> Vec<String> {
    let mut needs: Vec<String> = Vec::new();
    let steps = flow.walk();
    let from_steps = steps
        .iter()
        .filter(|(_, step)| step.enabled)
        .flat_map(|(_, step)| programs_of(&step.kind));
    for program in from_steps.chain(flow.meta.requires.iter().cloned()) {
        if !needs.contains(&program) {
            needs.push(program);
        }
    }
    needs
}

/// Opens the details of one catalog flow. `installed` names the copy of
/// it on this machine, when there is one.
pub fn open_catalog_detail(
    entry: Entry,
    installs: u64,
    verified: bool,
    installed: Option<Installed>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<CatalogDetail> {
    let title: SharedString = title_case(&entry.name).into();
    let dialog = cx.new(|cx| CatalogDetail::new(entry, installs, verified, installed, cx));
    let view = dialog.clone();
    let list_focus = dialog.read(cx).list_focus.clone();
    window.open_dialog(cx, move |d, window, _| {
        d.title(title.clone())
            .w(focus::dialog_width(720., window))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .child(view.clone())
    });
    list_focus.focus(window, cx);
    dialog
}
