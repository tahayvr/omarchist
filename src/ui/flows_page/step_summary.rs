//! How a step is described in the UI: an icon, a short title, and the
//! literal command underneath. Apps get their desktop entry's name and icon
//! when one matches, and flow steps the flow's name.
use std::path::PathBuf;

use gpui::*;
use gpui_component::Icon;

use crate::system::apps::DesktopApp;
use crate::system::flows::actions::{self, FieldKind};
use crate::system::flows::condition::Condition;
use crate::system::flows::{Flow, OnClick, StepKind, format_duration, vars};
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::action::{Action, ActionKind, program_name};
use crate::ui::flows_page::step_types::{StepChoice, StepGroup};
use crate::ui::palette;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepIcon {
    /// A Lucide icon under `assets/icons/`.
    Path(&'static str),
    /// An installed app's icon file.
    Image(PathBuf),
}

impl StepIcon {
    pub fn render(&self, size: Pixels) -> AnyElement {
        match self {
            StepIcon::Path(path) => Icon::new(Icon::empty())
                .path(*path)
                .size(size)
                .flex_shrink_0()
                .into_any_element(),
            StepIcon::Image(path) => img(path.clone())
                .size(size)
                .flex_shrink_0()
                .into_any_element(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepSummary {
    pub icon: StepIcon,
    pub title: String,
    /// The line under the title; empty when the title says it all.
    pub detail: String,
    /// The group the step belongs to, which colours its icon.
    pub group: StepGroup,
}

impl StepSummary {
    /// The step's icon on a square tinted with its group's colour; an
    /// installed app's own icon is drawn as it is.
    pub fn tile(&self, size: Pixels, cx: &App) -> AnyElement {
        match &self.icon {
            StepIcon::Image(_) => div()
                .size(size)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(self.icon.render(size * 0.86))
                .into_any_element(),
            StepIcon::Path(path) => {
                palette::tile(palette::icon(path), self.group.accent(cx), size, cx)
                    .into_any_element()
            }
        }
    }
}

/// `Ask "Name?"`, without the quotes when the text holds a variable: they
/// read oddly around a token.
fn quoted(verb: &str, text: &str) -> String {
    let text = text.trim();
    if vars::references(text).is_empty() {
        format!("{verb} \"{text}\"")
    } else {
        format!("{verb} {text}")
    }
}

/// Names and icons the summaries can draw on.
pub struct SummaryContext<'a> {
    pub apps: &'a [DesktopApp],
    pub flows: &'a [Flow],
}

impl SummaryContext<'_> {
    pub fn summarize(&self, kind: &StepKind) -> StepSummary {
        let detail = kind.text();
        match kind {
            StepKind::Exec { command, wait } => {
                let action = Action::from_dispatcher(&Dispatcher::Exec(command.clone()));
                let mut summary = self.summarize_action(action, detail);
                // The flag changes what the step means; say so in the list.
                if *wait {
                    summary.title.push_str(" · waits");
                }
                summary
            }
            StepKind::Lua { expr } => {
                let action = Action::from_dispatcher(&Dispatcher::Lua(expr.clone()));
                self.summarize_action(action, detail)
            }
            StepKind::Wait { ms } => StepSummary {
                icon: StepIcon::Path("icons/hourglass.svg"),
                title: format!("Wait {}", format_duration(*ms)),
                detail: String::new(),
                group: StepGroup::Logic,
            },
            StepKind::Notify {
                title,
                body,
                on_click,
                ..
            } => {
                let mut detail = body.trim().to_string();
                if let Some(action) = on_click {
                    if !detail.is_empty() {
                        detail.push_str(" · ");
                    }
                    detail.push_str(match action {
                        OnClick::Copy => "click to copy",
                        OnClick::Open => "click to open",
                    });
                }
                StepSummary {
                    icon: StepIcon::Path("icons/bell.svg"),
                    title: quoted("Notify", title),
                    detail,
                    group: StepGroup::Ask,
                }
            }
            StepKind::Flow { id, input } => StepSummary {
                icon: StepIcon::Path(ActionKind::Flow.icon_path()),
                title: if input.trim().is_empty() {
                    format!("Run flow {}", self.flow_name(id))
                } else {
                    format!("Run flow {} with {}", self.flow_name(id), input.trim())
                },
                detail,
                group: StepGroup::Logic,
            },
            StepKind::Ask { prompt } => StepSummary {
                icon: StepIcon::Path(StepChoice::Ask.info().icon),
                title: quoted("Ask", prompt),
                detail: String::new(),
                group: StepGroup::Ask,
            },
            StepKind::Confirm { prompt } => StepSummary {
                icon: StepIcon::Path(StepChoice::Confirm.info().icon),
                title: quoted("Confirm", prompt),
                detail: String::new(),
                group: StepGroup::Ask,
            },
            StepKind::Choose {
                prompt,
                options,
                from,
            } => StepSummary {
                icon: StepIcon::Path(StepChoice::Choose.info().icon),
                title: quoted("Choose", prompt),
                detail: if options.is_empty() {
                    format!("from {}", from.trim())
                } else {
                    options.join(", ")
                },
                group: StepGroup::Ask,
            },
            StepKind::If { condition, not, .. } => StepSummary {
                icon: StepIcon::Path(StepChoice::If.info().icon),
                title: format!("If {}", self.condition(condition, *not)),
                detail: match condition {
                    Condition::Command { command } => command.clone(),
                    _ => String::new(),
                },
                group: StepGroup::Logic,
            },
            StepKind::Repeat { times, .. } => StepSummary {
                icon: StepIcon::Path(StepChoice::Repeat.info().icon),
                title: format!("Repeat {times} time{}", if *times == 1 { "" } else { "s" }),
                detail: String::new(),
                group: StepGroup::Logic,
            },
            StepKind::Each { items, .. } => StepSummary {
                icon: StepIcon::Path(StepChoice::Each.info().icon),
                title: format!("Repeat with each line of {}", items.trim()),
                detail: String::new(),
                group: StepGroup::Logic,
            },
            StepKind::Menu { prompt, choices } => StepSummary {
                icon: StepIcon::Path(StepChoice::Menu.info().icon),
                title: quoted("Menu", prompt),
                detail: choices
                    .iter()
                    .map(|c| c.label.trim())
                    .collect::<Vec<_>>()
                    .join(", "),
                group: StepGroup::Logic,
            },
            StepKind::Stop => StepSummary {
                icon: StepIcon::Path(StepChoice::Stop.info().icon),
                title: "Stop this flow".to_string(),
                detail: String::new(),
                group: StepGroup::Logic,
            },
            StepKind::Action { action, args } => match actions::find(action) {
                Some(def) => StepSummary {
                    icon: StepIcon::Path(def.icon),
                    // An app reads by its name, not its window class.
                    title: def.title_with(args, &|field, value| match field.kind {
                        FieldKind::App => self.app_name_for_class(&value),
                        _ => value,
                    }),
                    detail: String::new(),
                    group: StepGroup::of_action(def.group),
                },
                None => StepSummary {
                    icon: StepIcon::Path("icons/circle-question-mark.svg"),
                    title: format!("An action this Omarchist does not have: {action}"),
                    detail: String::new(),
                    group: StepGroup::Script,
                },
            },
            StepKind::Pick { prompt, folder } => {
                let choice = if *folder {
                    StepChoice::PickFolder
                } else {
                    StepChoice::PickFile
                };
                StepSummary {
                    icon: StepIcon::Path(choice.info().icon),
                    title: choice.info().label.to_string(),
                    detail: prompt.trim().to_string(),
                    group: StepGroup::Ask,
                }
            }
        }
    }

    /// A condition as the rest of the sentence after "If".
    pub fn condition(&self, condition: &Condition, not: bool) -> String {
        match condition {
            Condition::Equals { value, to } => format!(
                "{} {} {}",
                value.trim(),
                if not { "is not" } else { "is" },
                to.trim()
            ),
            Condition::Contains { value, text } => format!(
                "{} {} {}",
                value.trim(),
                if not { "does not contain" } else { "contains" },
                text.trim()
            ),
            Condition::Empty { value } => format!(
                "{} {}",
                value.trim(),
                if not { "is not empty" } else { "is empty" }
            ),
            Condition::Command { .. } => {
                format!("the command {}", if not { "fails" } else { "succeeds" })
            }
            Condition::AppOpen { class } => format!(
                "{} {}",
                self.app_name_for_class(class),
                if not { "is not open" } else { "is open" }
            ),
            Condition::OnBattery => if not { "plugged in" } else { "on battery" }.to_string(),
            Condition::TimeBetween { from, to } => format!(
                "the time is {} {} and {}",
                if not { "outside" } else { "between" },
                from.trim(),
                to.trim()
            ),
        }
    }

    /// The name of the installed app whose windows have this class, or
    /// the class itself.
    pub fn app_name_for_class(&self, class: &str) -> String {
        let class = class.trim();
        self.apps
            .iter()
            .find(|a| !a.is_webapp() && a.wm_class.eq_ignore_ascii_case(class))
            .map(|a| a.name.clone())
            .unwrap_or_else(|| class.to_string())
    }

    fn summarize_action(&self, action: Option<Action>, detail: String) -> StepSummary {
        let Some(action) = action else {
            return StepSummary {
                icon: StepIcon::Path(ActionKind::Window.icon_path()),
                title: "Hyprland dispatcher".to_string(),
                detail,
                group: StepGroup::Desktop,
            };
        };
        match &action {
            Action::App { app, focus } => {
                let entry = self.app_for(&app.exec);
                let name = entry
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| program_name(&app.exec).to_string());
                StepSummary {
                    icon: entry
                        .and_then(|a| a.icon.clone())
                        .map(StepIcon::Image)
                        .unwrap_or(StepIcon::Path(ActionKind::App.icon_path())),
                    title: format!("{} {name}", if *focus { "Open or focus" } else { "Open" }),
                    detail,
                    group: StepGroup::Apps,
                }
            }
            Action::WebApp { url, focus, .. } => {
                let entry = self.webapp_for(url);
                let name = entry
                    .map(|a| a.name.clone())
                    .or_else(|| action.summary())
                    .unwrap_or_default();
                StepSummary {
                    icon: entry
                        .and_then(|a| a.icon.clone())
                        .map(StepIcon::Image)
                        .unwrap_or(StepIcon::Path(ActionKind::WebApp.icon_path())),
                    title: format!("{} {name}", if *focus { "Open or focus" } else { "Open" }),
                    detail,
                    group: StepGroup::Apps,
                }
            }
            Action::Terminal { .. } => StepSummary {
                icon: StepIcon::Path(ActionKind::Terminal.icon_path()),
                title: format!("Run {} in a terminal", action.summary().unwrap_or_default()),
                detail,
                group: StepGroup::Apps,
            },
            Action::Flow(id) => StepSummary {
                icon: StepIcon::Path(ActionKind::Flow.icon_path()),
                title: format!("Run flow {}", self.flow_name(id)),
                detail,
                group: StepGroup::Logic,
            },
            Action::Command(_) => StepSummary {
                icon: StepIcon::Path(ActionKind::Command.icon_path()),
                title: "Run a command".to_string(),
                detail,
                group: StepGroup::Script,
            },
            Action::Omarchy(_) | Action::Window(_) => StepSummary {
                icon: StepIcon::Path(action.kind().icon_path()),
                title: action.summary().unwrap_or_default(),
                detail,
                group: StepGroup::Desktop,
            },
        }
    }

    fn app_for(&self, exec: &str) -> Option<&DesktopApp> {
        let apps = self.apps.iter().filter(|a| !a.is_webapp());
        apps.clone().find(|a| a.exec == exec).or_else(|| {
            let program = program_name(exec);
            apps.clone().find(|a| program_name(&a.exec) == program)
        })
    }

    fn webapp_for(&self, url: &str) -> Option<&DesktopApp> {
        let wanted = url.trim().trim_end_matches('/');
        self.apps.iter().find(|a| {
            a.webapp_url
                .as_deref()
                .is_some_and(|u| u.trim().trim_end_matches('/') == wanted)
        })
    }

    fn flow_name(&self, id: &str) -> String {
        self.flows
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.name.clone())
            .unwrap_or_else(|| id.to_string())
    }
}

#[cfg(test)]
mod tests {
    // `use super::*` would import gpui's `test` attribute macro and shadow `#[test]`.
    use super::{DesktopApp, Flow, PathBuf, StepIcon, StepKind, SummaryContext};

    #[test]
    fn summaries_name_apps_and_flows() {
        let apps = vec![DesktopApp {
            id: "obsidian".into(),
            name: "Obsidian".into(),
            exec: "obsidian --ozone".into(),
            icon: Some(PathBuf::from("/tmp/obsidian.png")),
            wm_class: "obsidian".into(),
            terminal: false,
            webapp_url: None,
        }];
        let flows = vec![Flow::new("focus".into(), "Focus mode".into())];
        let ctx = SummaryContext {
            apps: &apps,
            flows: &flows,
        };

        let launch = ctx.summarize(&StepKind::Exec {
            command: "uwsm-app -- obsidian".into(),
            wait: false,
        });
        assert_eq!(launch.title, "Open Obsidian");
        assert_eq!(
            launch.icon,
            StepIcon::Image(PathBuf::from("/tmp/obsidian.png"))
        );

        let flow = ctx.summarize(&StepKind::flow("focus"));
        assert_eq!(flow.title, "Run flow Focus mode");
        assert_eq!(flow.detail, "omarchist flow run focus");

        let wait = ctx.summarize(&StepKind::Wait { ms: 1500 });
        assert_eq!(wait.title, "Wait 1.5 s");
        assert_eq!(wait.detail, "");

        let ask = ctx.summarize(&StepKind::Ask {
            prompt: "Name?".into(),
        });
        assert_eq!(ask.title, "Ask \"Name?\"");
        let choose = ctx.summarize(&StepKind::Choose {
            prompt: "Size for {{name}}".into(),
            options: vec!["Small".into(), "Large".into()],
            from: String::new(),
        });
        assert_eq!(choose.title, "Choose Size for {{name}}");
        assert_eq!(choose.detail, "Small, Large");
        let notify = ctx.summarize(&StepKind::Notify {
            title: "Uploaded".into(),
            body: "{{url}}".into(),
            on_click: Some(super::OnClick::Open),
            target: String::new(),
        });
        assert_eq!(notify.detail, "{{url}} · click to open");

        let open = ctx.summarize(&StepKind::If {
            condition: super::Condition::AppOpen {
                class: "OBSIDIAN".into(),
            },
            not: true,
            then: Vec::new(),
            otherwise: Vec::new(),
        });
        assert_eq!(open.title, "If Obsidian is not open");
        let battery = ctx.summarize(&StepKind::If {
            condition: super::Condition::OnBattery,
            not: true,
            then: Vec::new(),
            otherwise: Vec::new(),
        });
        assert_eq!(battery.title, "If plugged in");
        let repeat = ctx.summarize(&StepKind::Repeat {
            times: 1,
            steps: Vec::new(),
        });
        assert_eq!(repeat.title, "Repeat 1 time");

        let omarchy = ctx.summarize(&StepKind::Exec {
            command: "omarchy-launch-browser".into(),
            wait: false,
        });
        assert_eq!(omarchy.title, "Browser");
        assert_eq!(omarchy.icon, StepIcon::Path("logo/omarchy-icon.svg"));

        let lua = ctx.summarize(&StepKind::Lua {
            expr: "hl.dsp.focus({ workspace = \"2\" })".into(),
        });
        assert_eq!(lua.title, "Switch to workspace 2");
    }
}
