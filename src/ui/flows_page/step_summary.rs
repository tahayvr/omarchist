//! How a step is described in the UI: an icon, a short title, and the
//! literal command underneath. Apps get their desktop entry's name and icon
//! when one matches, and flow steps the flow's name.
use std::path::PathBuf;

use gpui::*;
use gpui_component::Icon;

use crate::system::apps::DesktopApp;
use crate::system::flows::{Flow, StepKind, format_duration};
use crate::system::keybinds::Dispatcher;
use crate::system::keybinds::action::{Action, ActionKind, program_name};

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
    pub detail: String,
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
            StepKind::Exec { command, .. } => {
                let action = Action::from_dispatcher(&Dispatcher::Exec(command.clone()));
                self.summarize_action(action, detail)
            }
            StepKind::Lua { expr } => {
                let action = Action::from_dispatcher(&Dispatcher::Lua(expr.clone()));
                self.summarize_action(action, detail)
            }
            StepKind::Wait { ms } => StepSummary {
                icon: StepIcon::Path("icons/hourglass.svg"),
                title: format!("Wait {}", format_duration(*ms)),
                detail,
            },
            StepKind::Notify { title, body } => StepSummary {
                icon: StepIcon::Path("icons/bell.svg"),
                title: format!("Notify \"{}\"", title.trim()),
                detail: if body.trim().is_empty() {
                    detail
                } else {
                    body.trim().to_string()
                },
            },
            StepKind::Flow { id } => StepSummary {
                icon: StepIcon::Path(ActionKind::Flow.icon_path()),
                title: format!("Run flow {}", self.flow_name(id)),
                detail,
            },
        }
    }

    fn summarize_action(&self, action: Option<Action>, detail: String) -> StepSummary {
        let Some(action) = action else {
            return StepSummary {
                icon: StepIcon::Path(ActionKind::Window.icon_path()),
                title: "Hyprland dispatcher".to_string(),
                detail,
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
                }
            }
            Action::Terminal { .. } => StepSummary {
                icon: StepIcon::Path(ActionKind::Terminal.icon_path()),
                title: format!("Run {} in a terminal", action.summary().unwrap_or_default()),
                detail,
            },
            Action::Flow(id) => StepSummary {
                icon: StepIcon::Path(ActionKind::Flow.icon_path()),
                title: format!("Run flow {}", self.flow_name(id)),
                detail,
            },
            Action::Command(_) => StepSummary {
                icon: StepIcon::Path(ActionKind::Command.icon_path()),
                title: "Run a command".to_string(),
                detail,
            },
            Action::Omarchy(_) | Action::Window(_) => StepSummary {
                icon: StepIcon::Path(action.kind().icon_path()),
                title: action.summary().unwrap_or_default(),
                detail,
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

        let flow = ctx.summarize(&StepKind::Flow { id: "focus".into() });
        assert_eq!(flow.title, "Run flow Focus mode");
        assert_eq!(flow.detail, "omarchist flow run focus");

        let wait = ctx.summarize(&StepKind::Wait { ms: 1500 });
        assert_eq!(wait.title, "Wait 1.5 s");

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
