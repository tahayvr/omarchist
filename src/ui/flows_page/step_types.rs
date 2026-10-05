//! Every kind of step a flow can hold, as the step picker lists them: a
//! name, an icon, and the group it sits in.
use gpui::{App, Hsla};
use gpui_component::ActiveTheme;

use std::sync::LazyLock;

use crate::system::flows::StepKind;
use crate::system::flows::actions::{self, ACTIONS, ActionDef, ActionGroup};
use crate::system::keybinds::action::ActionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepChoice {
    Action(ActionKind),
    Wait,
    Notify,
    Ask,
    Choose,
    Confirm,
    PickFile,
    PickFolder,
    If,
    Repeat,
    Each,
    Menu,
    Stop,
    /// A ready-made action.
    Do(&'static ActionDef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepGroup {
    Apps,
    Desktop,
    Ask,
    Logic,
    Text,
    Capture,
    Web,
    Script,
}

impl StepGroup {
    pub const ALL: [StepGroup; 8] = [
        StepGroup::Apps,
        StepGroup::Desktop,
        StepGroup::Ask,
        StepGroup::Logic,
        StepGroup::Text,
        StepGroup::Capture,
        StepGroup::Web,
        StepGroup::Script,
    ];

    pub fn label(self) -> &'static str {
        match self {
            StepGroup::Apps => "Apps",
            StepGroup::Desktop => "Desktop",
            StepGroup::Ask => "Ask and notify",
            StepGroup::Logic => "Logic",
            StepGroup::Text => "Text and clipboard",
            StepGroup::Capture => "Capture",
            StepGroup::Web => "Web",
            StepGroup::Script => "Script",
        }
    }

    /// A word for the group's filter in the step list.
    pub fn short_label(self) -> &'static str {
        match self {
            StepGroup::Ask => "Ask",
            StepGroup::Text => "Text",
            other => other.label(),
        }
    }

    /// The colour the group's steps wear in the picker and the step list,
    /// taken from the theme so it follows the palette.
    pub fn accent(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            StepGroup::Apps => theme.blue,
            StepGroup::Desktop => theme.cyan,
            StepGroup::Ask => theme.magenta,
            StepGroup::Logic => theme.yellow,
            StepGroup::Text => theme.green,
            StepGroup::Capture => theme.red,
            StepGroup::Web => theme.blue_light,
            StepGroup::Script => theme.muted_foreground,
        }
    }

    pub fn of_action(group: ActionGroup) -> Self {
        match group {
            ActionGroup::Text => StepGroup::Text,
            ActionGroup::Apps => StepGroup::Apps,
            ActionGroup::Desktop => StepGroup::Desktop,
            ActionGroup::Capture => StepGroup::Capture,
            ActionGroup::Web => StepGroup::Web,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepType {
    pub choice: StepChoice,
    pub label: &'static str,
    pub icon: &'static str,
    pub group: StepGroup,
    /// Extra words the search matches.
    pub keywords: &'static str,
}

macro_rules! step_types {
    ($($choice:expr, $label:literal, $icon:literal, $group:ident, $keywords:literal;)*) => {
        &[$(StepType {
            choice: $choice,
            label: $label,
            icon: $icon,
            group: StepGroup::$group,
            keywords: $keywords,
        },)*]
    };
}

/// The kinds with a form of their own.
const BUILT_IN: &[StepType] = step_types! {
    StepChoice::Action(ActionKind::App), "Open an app", "icons/app-window.svg", Apps, "launch start focus program";
    StepChoice::Action(ActionKind::WebApp), "Open a web app", "icons/globe.svg", Apps, "site url browser link";
    StepChoice::Action(ActionKind::Terminal), "Run in a terminal", "icons/square-terminal.svg", Apps, "tui shell console";

    StepChoice::Action(ActionKind::Omarchy), "Omarchy action", "logo/omarchy-icon.svg", Desktop, "menu panel screenshot volume brightness lock media capture";
    StepChoice::Action(ActionKind::Window), "Window action", "icons/layout-grid.svg", Desktop, "workspace hyprland focus move close fullscreen float resize";

    StepChoice::Ask, "Ask for text", "icons/text-cursor-input.svg", Ask, "input prompt question type";
    StepChoice::Choose, "Choose from a list", "icons/list-checks.svg", Ask, "menu pick select options";
    StepChoice::Confirm, "Confirm", "icons/circle-question-mark.svg", Ask, "continue cancel sure yes no";
    StepChoice::PickFile, "Pick a file", "icons/file.svg", Ask, "choose open browse path";
    StepChoice::PickFolder, "Pick a folder", "icons/folder.svg", Ask, "choose directory browse path";
    StepChoice::Notify, "Notify", "icons/bell.svg", Ask, "notification message show result toast copy open";

    StepChoice::If, "If", "icons/split.svg", Logic, "condition otherwise else when check branch";
    StepChoice::Repeat, "Repeat", "icons/repeat.svg", Logic, "loop times again";
    StepChoice::Each, "Repeat with each", "icons/repeat-2.svg", Logic, "loop for every line item list";
    StepChoice::Menu, "Choose from a menu", "icons/list-tree.svg", Logic, "branch options pick select";
    StepChoice::Stop, "Stop this flow", "icons/octagon-x.svg", Logic, "end exit quit return";
    StepChoice::Wait, "Wait", "icons/hourglass.svg", Logic, "pause sleep delay";
    StepChoice::Action(ActionKind::Flow), "Run a flow", "icons/workflow.svg", Logic, "nested another";

    StepChoice::Action(ActionKind::Command), "Run a command", "icons/terminal.svg", Script, "shell exec script bash";
};

/// Every kind of step, in the order the picker lists them: group by
/// group, the kinds with their own form first, then the ready-made
/// actions of that group.
pub fn step_types() -> &'static [StepType] {
    static ALL: LazyLock<Vec<StepType>> = LazyLock::new(|| {
        let mut all = Vec::new();
        for group in StepGroup::ALL {
            all.extend(BUILT_IN.iter().filter(|t| t.group == group).copied());
            all.extend(
                ACTIONS
                    .iter()
                    .filter(|action| StepGroup::of_action(action.group) == group)
                    .map(|action| StepType {
                        choice: StepChoice::Do(action),
                        label: action.label,
                        icon: action.icon,
                        group,
                        keywords: action.keywords,
                    }),
            );
        }
        all
    });
    &ALL
}

impl StepChoice {
    pub fn info(self) -> &'static StepType {
        step_types()
            .iter()
            .find(|t| t.choice == self)
            .expect("every choice is listed")
    }

    /// The choice that edits `kind`. Commands map onto the action
    /// builder's kinds, which the builder works out itself, so they come
    /// back as a plain command here.
    pub fn of(kind: &StepKind) -> Option<Self> {
        Some(match kind {
            StepKind::Wait { .. } => StepChoice::Wait,
            StepKind::Notify { .. } => StepChoice::Notify,
            StepKind::Ask { .. } => StepChoice::Ask,
            StepKind::Choose { .. } => StepChoice::Choose,
            StepKind::Confirm { .. } => StepChoice::Confirm,
            StepKind::Pick { folder: false, .. } => StepChoice::PickFile,
            StepKind::Pick { folder: true, .. } => StepChoice::PickFolder,
            StepKind::If { .. } => StepChoice::If,
            StepKind::Repeat { .. } => StepChoice::Repeat,
            StepKind::Each { .. } => StepChoice::Each,
            StepKind::Menu { .. } => StepChoice::Menu,
            StepKind::Stop => StepChoice::Stop,
            StepKind::Action { action, .. } => StepChoice::Do(actions::find(action)?),
            StepKind::Exec { .. } | StepKind::Lua { .. } | StepKind::Flow { .. } => return None,
        })
    }

    /// Whether the step has anything to fill in.
    pub fn has_form(self) -> bool {
        match self {
            StepChoice::Stop => false,
            // An action with nothing to set and nothing to save.
            StepChoice::Do(action) => !action.fields.is_empty() || action.has_output(),
            _ => true,
        }
    }

    /// The name a new step of this kind saves its output under, so its
    /// answer is usable without naming it first.
    pub fn default_output(self) -> Option<&'static str> {
        match self {
            StepChoice::Ask => Some("answer"),
            StepChoice::Choose => Some("choice"),
            StepChoice::PickFile => Some("file"),
            StepChoice::PickFolder => Some("folder"),
            StepChoice::Menu => Some("pick"),
            StepChoice::Do(action) if action.has_output() => Some(action.saves_as),
            _ => None,
        }
    }
}

/// How well a step type answers a search; lower is better. `None` when a
/// word of the query is nowhere in its name, keywords or group.
fn rank(step: &StepType, words: &[String]) -> Option<u8> {
    let label = step.label.to_lowercase();
    let rest = format!("{} {}", step.keywords, step.group.label()).to_lowercase();
    let mut worst = 0;
    for word in words {
        let score = if label.starts_with(word.as_str()) {
            0
        } else if label
            .split_whitespace()
            .any(|w| w.starts_with(word.as_str()))
        {
            1
        } else if label.contains(word.as_str()) {
            2
        } else if rest.contains(word.as_str()) {
            3
        } else {
            return None;
        };
        worst = worst.max(score);
    }
    Some(worst)
}

fn words(query: &str) -> Vec<String> {
    query.split_whitespace().map(|w| w.to_lowercase()).collect()
}

/// The step types whose name, keywords or group contain every word of
/// `query`, in list order.
pub fn search(query: &str) -> Vec<&'static StepType> {
    let words = words(query);
    step_types()
        .iter()
        .filter(|t| rank(t, &words).is_some())
        .collect()
}

/// Where the best answer to `query` sits in [`search`]'s result: a name
/// that starts with what was typed beats one that only mentions it, so
/// Enter after "notify" adds Notify, not the first step of its group.
pub fn best_match(query: &str) -> usize {
    let words = words(query);
    search(query)
        .iter()
        .enumerate()
        .min_by_key(|(_, t)| rank(t, &words).unwrap_or(u8::MAX))
        .map_or(0, |(ix, _)| ix)
}

#[cfg(test)]
mod tests {
    use super::{StepChoice, best_match, search, step_types};
    use crate::system::keybinds::action::ActionKind;

    #[test]
    fn every_action_kind_is_listed_once() {
        for kind in ActionKind::ALL {
            let count = step_types()
                .iter()
                .filter(|t| t.choice == StepChoice::Action(kind))
                .count();
            assert_eq!(count, 1, "{kind:?}");
        }
        for step in step_types() {
            let embedded =
                gpui::AssetSource::load(&crate::assets::CombinedAssets::new(), step.icon);
            assert!(
                matches!(embedded, Ok(Some(_))),
                "{} is not embedded",
                step.icon
            );
        }
    }

    #[test]
    fn search_matches_names_groups_and_keywords() {
        let labels = |q: &str| -> Vec<&str> { search(q).iter().map(|t| t.label).collect() };
        assert_eq!(labels("").len(), step_types().len());
        assert_eq!(
            labels("folder"),
            vec!["Open a file or folder", "Pick a folder"]
        );
        assert_eq!(labels("pause"), vec!["Wait"]);
        assert_eq!(labels("else"), vec!["If"]);
        assert!(labels("ask").contains(&"Confirm"), "the group name matches");
        assert!(labels("zzz").is_empty());
    }

    #[test]
    fn the_best_match_is_the_name_that_starts_with_the_query() {
        let best = |q: &str| search(q)[best_match(q)].label;
        assert_eq!(best(""), step_types()[0].label);
        // "notify" is also in the group name of every asking step.
        assert_eq!(best("notify"), "Notify");
        assert_eq!(best("ask"), "Ask for text");
        assert_eq!(best("pick a fol"), "Pick a folder");
        assert_eq!(best("comm"), "Run a command");
        assert_eq!(best("volume"), "Set the volume");
        assert_eq!(best("screenshot"), "Take a screenshot");
        assert_eq!(best("copy"), "Copy to the clipboard");
        assert_eq!(best("repeat"), "Repeat");
        assert_eq!(best("each"), "Repeat with each");
    }
}
