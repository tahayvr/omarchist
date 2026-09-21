//! Starter flows offered on an empty Flows page. They only use commands
//! every Omarchy install has.
use crate::system::keybinds::action::{WindowAction, WindowActionKind, WorkspaceTarget};

use super::{Flow, OnError, Step, StepKind, Triggers};

fn exec(command: &str) -> Step {
    Step::new(StepKind::Exec {
        command: command.into(),
        wait: false,
    })
}

fn notify(title: &str, body: &str) -> Step {
    Step::new(StepKind::Notify {
        title: title.into(),
        body: body.into(),
    })
}

fn workspace(n: u8) -> Step {
    let mut action = WindowAction::new(WindowActionKind::SwitchWorkspace);
    action.workspace = WorkspaceTarget::Number(n);
    Step::new(StepKind::Lua { expr: action.lua() })
}

/// The templates, with ids the editor replaces by a unique one on save.
pub fn templates() -> Vec<Flow> {
    vec![
        Flow {
            description: "Opens your browser, a terminal, and music, then says hello.".into(),
            icon: "coffee".into(),
            steps: vec![
                exec("omarchy-launch-browser"),
                exec("omarchy-launch-terminal"),
                exec("omarchy-launch-spotify"),
                notify("Good morning", "Your desk is ready"),
            ],
            on_error: OnError::Continue,
            triggers: Triggers {
                startup: true,
                ..Triggers::default()
            },
            ..Flow::new("morning-start".into(), "Morning start".into())
        },
        Flow {
            description: "Moves to workspace 2 and opens your editor.".into(),
            icon: "target".into(),
            steps: vec![
                workspace(2),
                exec("omarchy-launch-editor"),
                notify("Focus mode", "Everything else can wait"),
            ],
            ..Flow::new("focus-mode".into(), "Focus mode".into())
        },
        Flow {
            description: "Gives you five seconds, then locks the screen.".into(),
            icon: "moon".into(),
            steps: vec![
                notify("Wrapping up", "Locking in five seconds"),
                Step::new(StepKind::Wait { ms: 5000 }),
                exec("omarchy-system-lock"),
            ],
            ..Flow::new("wrap-up".into(), "Wrap up".into())
        },
    ]
}

pub fn template(id: &str) -> Option<Flow> {
    templates().into_iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_valid_and_distinct() {
        let all = templates();
        assert!(all.len() >= 3);
        for flow in &all {
            flow.validate().unwrap();
            assert!(!flow.steps.is_empty());
            assert!(crate::system::flows::ICONS.contains(&flow.icon.as_str()));
        }
        let mut ids: Vec<_> = all.iter().map(|f| f.id.as_str()).collect();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        assert!(template("focus-mode").is_some());
        assert_eq!(
            template("focus-mode").unwrap().steps[0].kind,
            StepKind::Lua {
                expr: "hl.dsp.focus({ workspace = \"2\" })".into()
            }
        );
    }
}
