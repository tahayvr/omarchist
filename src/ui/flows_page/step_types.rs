//! Every kind of step a flow can hold, as the step picker lists them: a
//! name, an icon, and the group it sits in.
use gpui::{App, Hsla};
use gpui_component::ActiveTheme;

use crate::system::flows::StepKind;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepGroup {
    Apps,
    Desktop,
    Ask,
    Logic,
    Script,
}

impl StepGroup {
    pub const ALL: [StepGroup; 5] = [
        StepGroup::Apps,
        StepGroup::Desktop,
        StepGroup::Ask,
        StepGroup::Logic,
        StepGroup::Script,
    ];

    pub fn label(self) -> &'static str {
        match self {
            StepGroup::Apps => "Apps",
            StepGroup::Desktop => "Desktop",
            StepGroup::Ask => "Ask and notify",
            StepGroup::Logic => "Logic",
            StepGroup::Script => "Script",
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
            StepGroup::Script => theme.green,
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

/// In the order the picker lists them.
pub const STEP_TYPES: &[StepType] = step_types! {
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

impl StepChoice {
    pub fn info(self) -> &'static StepType {
        STEP_TYPES
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
            StepKind::Exec { .. } | StepKind::Lua { .. } | StepKind::Flow { .. } => return None,
        })
    }

    /// Whether the step has anything to fill in.
    pub fn has_form(self) -> bool {
        self != StepChoice::Stop
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
    STEP_TYPES
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
    use super::{STEP_TYPES, StepChoice, best_match, search};
    use crate::system::keybinds::action::ActionKind;

    #[test]
    fn every_action_kind_is_listed_once() {
        for kind in ActionKind::ALL {
            let count = STEP_TYPES
                .iter()
                .filter(|t| t.choice == StepChoice::Action(kind))
                .count();
            assert_eq!(count, 1, "{kind:?}");
        }
        for step in STEP_TYPES {
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
        assert_eq!(labels("").len(), STEP_TYPES.len());
        assert_eq!(labels("folder"), vec!["Pick a folder"]);
        assert_eq!(labels("pause"), vec!["Wait"]);
        assert_eq!(labels("else"), vec!["If"]);
        assert!(labels("ask").contains(&"Confirm"), "the group name matches");
        assert!(labels("zzz").is_empty());
    }

    #[test]
    fn the_best_match_is_the_name_that_starts_with_the_query() {
        let best = |q: &str| search(q)[best_match(q)].label;
        assert_eq!(best(""), STEP_TYPES[0].label);
        // "notify" is also in the group name of every asking step.
        assert_eq!(best("notify"), "Notify");
        assert_eq!(best("ask"), "Ask for text");
        assert_eq!(best("fold"), "Pick a folder");
        assert_eq!(best("comm"), "Run a command");
        assert_eq!(best("repeat"), "Repeat");
        assert_eq!(best("each"), "Repeat with each");
    }
}
