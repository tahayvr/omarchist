// Headless UI tests for the Flows editor: the Add step dialog, the step
// forms, the step list, and a run, driven through gpui-kit's
// `TestWindowExt` against the production window. Every test checks what
// is on screen next to the flow the editor holds.
mod common;

use std::time::Duration;

use common::{
    edited_flow, open, open_add_step, pick_step_type, settle, wait_real, with, write_flow,
};
use gpui_kit::TestAppContext;
use gpui_kit::component::WindowExt;
use gpui_kit::test::TestWindowExt;
use omarchist::ActivePage;
use omarchist::system::flows::{OnClick, StepKind};

/// The step dialog's body (`dialog_body` in `step_dialog.rs`).
const STEP_DIALOG: &str = "step-dialog";
/// The search box and grid of the list of step types.
const SEARCH: &str = "step-search";
const GRID: &str = "step-picker-grid";
/// Fields of the step forms.
const PROMPT: &str = "step-prompt";
const OUTPUT_NAME: &str = "step-output-name";

// MARK: The list of step types

#[gpui_kit::test]
fn add_step_starts_on_a_searchable_list_of_step_types(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.find(STEP_DIALOG).visible());
        assert_eq!(
            window.find(SEARCH).focused(),
            Some(true),
            "the search box has the keyboard"
        );
        assert!(window.find("step-type-open-an-app").visible());
        assert!(window.find("step-type-set-the-volume").visible());
        assert!(
            window.try_find("step-save").is_none(),
            "there is nothing to add until a type is picked"
        );

        window.input("folder", cx);
        window.render_frame(cx);
        assert!(window.find("step-type-pick-a-folder").visible());
        assert!(
            window.try_find("step-type-ask-for-text").is_none(),
            "the search narrows the list"
        );
        assert_eq!(window.find(SEARCH).value(), Some("folder"));
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

#[gpui_kit::test]
fn a_search_with_no_match_adds_nothing(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "zzzz");

    with(cx, handle, |window, _| {
        assert!(window.find(SEARCH).visible(), "still on the list");
        assert!(window.try_find(PROMPT).is_none());
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

#[gpui_kit::test]
fn the_arrow_keys_move_through_the_step_types(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);

    with(cx, handle, |window, cx| {
        // Narrowed to one group, the grid is small enough to count in:
        // Ask for text, Choose from a list, Confirm / Pick a file, ...
        window.click("step-group-ask", cx);
        window.click(SEARCH, cx);
        // Down leaves the search box for the grid, on its first tile.
        window.press("down", cx);
        window.render_frame(cx);
        assert_eq!(window.find(GRID).focused(), Some(true));
        assert_eq!(window.find(SEARCH).focused(), Some(false));

        window.press("right", cx);
        window.press("down", cx);
        window.press("left", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, _| {
        assert_eq!(
            window.find(OUTPUT_NAME).value(),
            Some("file"),
            "Enter opened the form of Pick a file"
        );
        assert_eq!(window.find(PROMPT).focused(), Some(true));
    });
}

#[gpui_kit::test]
fn the_group_filters_narrow_the_list_from_the_keyboard(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);

    with(cx, handle, |window, cx| {
        window.press("tab", cx);
        window.render_frame(cx);
        assert_eq!(window.find("step-picker-groups").focused(), Some(true));
        // All, Apps, Desktop, Ask.
        for _ in 0..3 {
            window.press("right", cx);
        }
        window.render_frame(cx);
        assert!(window.find("step-type-confirm").visible());
        assert!(window.try_find("step-type-open-an-app").is_none());

        // Typing looks in every group again.
        window.click(SEARCH, cx);
        window.input("app", cx);
    });
    // The picker hears about the typing once this update is over.
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert_eq!(window.find(SEARCH).value(), Some("app"));
        assert!(window.find("step-type-open-an-app").visible());
        assert!(window.try_find("step-type-confirm").is_none());
    });
}

#[gpui_kit::test]
fn up_from_the_first_row_returns_to_the_search(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);

    with(cx, handle, |window, cx| {
        window.press("down", cx);
        window.render_frame(cx);
        assert_eq!(window.find(GRID).focused(), Some(true));
        window.press("up", cx);
        window.render_frame(cx);
        assert_eq!(window.find(SEARCH).focused(), Some(true));
    });
}

#[gpui_kit::test]
fn clicking_a_step_type_opens_its_form_and_change_goes_back(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);

    with(cx, handle, |window, cx| {
        window.click("step-group-ask", cx);
        window.render_frame(cx);
        window.click("step-type-confirm", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find(PROMPT).focused(), Some(true));
        assert!(
            window.try_find(OUTPUT_NAME).is_none(),
            "a confirmation has no answer to save"
        );
        window.click("step-change-type", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert_eq!(window.find(SEARCH).focused(), Some(true));
        assert!(window.try_find(PROMPT).is_none());
    });
}

// MARK: The asking steps

#[gpui_kit::test]
fn an_ask_step_saves_its_answer_under_a_default_name(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "ask for");

    with(cx, handle, |window, cx| {
        assert_eq!(window.find(PROMPT).focused(), Some(true));
        assert_eq!(window.find(OUTPUT_NAME).value(), Some("answer"));
        window.input("What is it called?", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 1);
    assert_eq!(
        flow.steps[0].kind,
        StepKind::Ask {
            prompt: "What is it called?".into()
        }
    );
    assert_eq!(flow.steps[0].output.as_deref(), Some("answer"));
    with(cx, handle, |window, cx| {
        assert!(!window.has_active_dialog(cx), "the dialog closed");
        assert!(window.find(("flow-step", 1usize)).visible());
    });

    // A second one must not reuse the name.
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "ask for");
    with(cx, handle, |window, cx| {
        assert_eq!(window.find(OUTPUT_NAME).value(), Some("answer 2"));
        window.input("And then?", cx);
        window.press("ctrl-enter", cx);
    });
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 2, "Ctrl+Enter adds the step");
    assert_eq!(flow.steps[1].output.as_deref(), Some("answer 2"));
    assert_eq!(flow.outputs_before(2), vec!["answer", "answer 2"]);
}

#[gpui_kit::test]
fn a_step_without_its_question_is_not_added(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "confirm");

    with(cx, handle, |window, cx| window.click("step-save", cx));
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx), "the dialog stays open");
        assert!(window.find(PROMPT).visible());
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

#[gpui_kit::test]
fn a_refused_save_is_one_error_toast(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "choose");
    with(cx, handle, |window, cx| {
        assert_eq!(window.notifications(cx).len(), 0);
        window.press("ctrl-enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx), "the dialog stays open");
        assert_eq!(window.notifications(cx).len(), 1);
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

#[gpui_kit::test]
fn choose_takes_its_options_from_a_list(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "choose");

    with(cx, handle, |window, cx| {
        window.input("Which size?", cx);
        // Two empty fields to start with; Enter adds one after the first.
        window.click(("step-option", 0usize), cx);
        window.input("Small", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find(("step-option", 1usize)).focused(), Some(true));
        window.input("Medium", cx);
        window.click(("step-option", 2usize), cx);
        window.input("Large", cx);
        // The middle one goes.
        window.click(("step-option-remove", 1usize), cx);
        window.render_frame(cx);
        assert!(window.try_find(("step-option", 2usize)).is_none());
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(
        flow.steps[0].kind,
        StepKind::Choose {
            prompt: "Which size?".into(),
            options: vec!["Small".into(), "Large".into()],
            from: String::new(),
        }
    );
    assert_eq!(flow.steps[0].output.as_deref(), Some("choice"));

    // Editing shows one field per option, and the last one cannot be
    // removed.
    with(cx, handle, |window, cx| {
        window.double_click(("flow-step", 1usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find(("step-option", 0usize)).value(), Some("Small"));
        assert_eq!(window.find(("step-option", 1usize)).value(), Some("Large"));
        window.click(("step-option-remove", 0usize), cx);
        window.click(("step-option-remove", 0usize), cx);
        window.render_frame(cx);
        assert_eq!(window.find(("step-option", 0usize)).value(), Some("Large"));
        window.press("escape", cx);
    });
}

#[gpui_kit::test]
fn choose_takes_its_options_from_a_variable(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "choose");

    with(cx, handle, |window, cx| {
        window.input("Which one?", cx);
        // The second choice of the Options row: From a variable.
        window.click(("choose-source", 1usize), cx);
        window.render_frame(cx);
        assert!(window.try_find("step-options").is_none());
        // A variable picked from the Insert row lands in the field.
        window.click("step-variable-clipboard", cx);
        window.render_frame(cx);
        assert_eq!(window.find("step-from").value(), Some("{{clipboard}}"));
        assert_eq!(window.find("step-from").focused(), Some(true));
        window.click("step-save", cx);
    });
    settle(cx, handle);

    assert_eq!(
        edited_flow(cx, &view).steps[0].kind,
        StepKind::Choose {
            prompt: "Which one?".into(),
            options: Vec::new(),
            from: "{{clipboard}}".into(),
        }
    );
}

#[gpui_kit::test]
fn a_variable_goes_where_the_cursor_was(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "notify");

    with(cx, handle, |window, cx| {
        assert_eq!(window.find("notify-title").focused(), Some(true));
        window.input("On  now", cx);
        for _ in 0..4 {
            window.press("left", cx);
        }
        window.click("step-variable-date", cx);
        window.render_frame(cx);
        assert_eq!(window.find("notify-title").value(), Some("On {{date}} now"));
        window.click("step-save", cx);
    });
    settle(cx, handle);

    assert_eq!(
        edited_flow(cx, &view).steps[0].kind,
        StepKind::notify("On {{date}} now", "")
    );
}

#[gpui_kit::test]
fn a_notification_can_copy_when_clicked(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "notify");

    with(cx, handle, |window, cx| {
        window.input("Link ready", cx);
        assert!(
            window.try_find("notify-target").is_none(),
            "nothing to act on until a click action is chosen"
        );
        // Nothing, Copy, Open.
        window.click(("notify-click", 1usize), cx);
        window.render_frame(cx);
        window.click("notify-target", cx);
        window.input("https://omarchist.com", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    assert_eq!(
        edited_flow(cx, &view).steps[0].kind,
        StepKind::Notify {
            title: "Link ready".into(),
            body: String::new(),
            on_click: Some(OnClick::Copy),
            target: "https://omarchist.com".into(),
        }
    );
}

#[gpui_kit::test]
fn a_step_cannot_use_a_variable_nothing_saves(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "ask for");

    with(cx, handle, |window, cx| {
        window.input("Rename {{missing}} to?", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx))
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

// MARK: The step list

const THREE_STEPS: &str = r#"
format = 2
id = "ui-three-steps"
name = "UI three steps"

[[step]]
type = "ask"
prompt = "Name?"
output = "name"

[[step]]
type = "wait"
ms = 250

[[step]]
type = "notify"
title = "Hello {{name}}"
"#;

fn step_titles(flow: &omarchist::system::flows::Flow) -> Vec<String> {
    flow.steps.iter().map(|s| s.kind.text()).collect()
}

#[gpui_kit::test]
fn enter_on_a_step_opens_its_form_with_its_values(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));

    with(cx, handle, |window, cx| {
        window.click(("flow-step", 1usize), cx);
        window.render_frame(cx);
        assert_eq!(window.find("flow-steps").focused(), Some(true));
        window.press("enter", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(
            window.try_find(SEARCH).is_none(),
            "editing opens the form, not the list of types"
        );
        assert_eq!(window.find(PROMPT).value(), Some("Name?"));
        assert_eq!(window.find(PROMPT).focused(), Some(true));
        assert_eq!(window.find(OUTPUT_NAME).value(), Some("name"));
        window.input(" Really?", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(
        flow.steps[0].kind,
        StepKind::Ask {
            prompt: "Name? Really?".into()
        }
    );
    assert_eq!(flow.steps.len(), 3, "editing replaces the step in place");
    with(cx, handle, |window, _| {
        assert_eq!(
            window.find("flow-steps").focused(),
            Some(true),
            "the keyboard returns to the step list"
        );
    });
}

#[gpui_kit::test]
fn the_keyboard_reorders_toggles_duplicates_and_removes_steps(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));
    let original = step_titles(&edited_flow(cx, &view));

    with(cx, handle, |window, cx| {
        window.click(("flow-step", 2usize), cx);
        // The Wait step moves below the notification, and back.
        window.press("alt-down", cx);
    });
    let moved = step_titles(&edited_flow(cx, &view));
    assert_eq!(
        moved,
        vec![
            original[0].clone(),
            original[2].clone(),
            original[1].clone()
        ]
    );
    with(cx, handle, |window, cx| window.press("alt-up", cx));
    assert_eq!(step_titles(&edited_flow(cx, &view)), original);

    with(cx, handle, |window, cx| window.press("space", cx));
    assert!(
        !edited_flow(cx, &view).steps[1].enabled,
        "Space turns the step off"
    );
    with(cx, handle, |window, cx| window.press("space", cx));
    assert!(edited_flow(cx, &view).steps[1].enabled);

    with(cx, handle, |window, cx| window.press("ctrl-d", cx));
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 4);
    assert_eq!(flow.steps[1], flow.steps[2], "Ctrl+D duplicates the step");

    with(cx, handle, |window, cx| {
        window.press("delete", cx);
        window.render_frame(cx);
        assert!(window.try_find(("flow-step", 4usize)).is_none());
    });
    assert_eq!(step_titles(&edited_flow(cx, &view)), original);
}

#[gpui_kit::test]
fn a_step_that_lost_its_variable_is_flagged(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));

    // Removing the step that saves `name` leaves the notification using a
    // name nothing saves, which the flow refuses to be saved with.
    with(cx, handle, |window, cx| {
        window.click(("flow-step", 1usize), cx);
        window.press("delete", cx);
        window.render_frame(cx);
    });
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 2);
    assert!(flow.validate_content().is_err());
    with(cx, handle, |window, _| {
        assert!(window.find(("step-unknown-variable", 2usize)).visible());
    });
}

// MARK: Running

const PRINTS: &str = r#"
format = 2
id = "ui-prints"
name = "UI prints"
on_error = "continue"

[[step]]
type = "exec"
command = "printf 'from the first step'"
wait = true
output = "text"

[[step]]
type = "exec"
command = "echo oops >&2; exit 3"
wait = true

[[step]]
type = "exec"
command = "test {{text}} = 'from the first step'"
wait = true
"#;

#[gpui_kit::test]
fn a_run_shows_each_steps_result_under_it(cx: &mut TestAppContext) {
    write_flow("ui-prints", PRINTS);
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-prints".into()));

    with(cx, handle, |window, cx| {
        assert!(window.try_find(("step-result", 1usize)).is_none());
        window.click("flow-run", cx);
    });
    // The run is on a thread of its own, outside the test executor.
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", 3usize)).is_some()
    });

    with(cx, handle, |window, _| {
        assert!(window.find(("step-result", 1usize)).visible());
        assert!(
            window.find(("step-failure", 2usize)).visible(),
            "the failed step says why"
        );
        assert!(
            window.try_find(("step-result", 2usize)).is_none(),
            "a failed step has no result"
        );
        assert!(
            window.try_find(("step-failure", 3usize)).is_none(),
            "the value saved by the first step reached the third"
        );
    });
}

// MARK: Steps that hold steps

use omarchist::system::flows::condition::Condition;
use omarchist::system::flows::{Flow, Step};

/// The times of the Wait steps, in the order the flow is written: a
/// compact picture of where steps sit.
fn waits(flow: &Flow) -> Vec<u64> {
    flow.walk()
        .into_iter()
        .filter_map(|(_, step)| match step.kind {
            StepKind::Wait { ms } => Some(ms),
            _ => None,
        })
        .collect()
}

fn branches(step: &Step) -> Vec<Vec<u64>> {
    step.kind
        .branches()
        .into_iter()
        .map(|branch| {
            branch
                .iter()
                .filter_map(|s| match s.kind {
                    StepKind::Wait { ms } => Some(ms),
                    _ => None,
                })
                .collect()
        })
        .collect()
}

const NESTED: &str = r#"
format = 2
id = "ui-nested"
name = "UI nested"

[[step]]
type = "wait"
ms = 1

[[step]]
type = "if"
check = "on_battery"

[[step.then]]
type = "wait"
ms = 2

[[step.otherwise]]
type = "wait"
ms = 3

[[step]]
type = "wait"
ms = 4
"#;

#[gpui_kit::test]
fn an_if_step_gets_a_branch_for_each_outcome(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "if");

    with(cx, handle, |window, cx| {
        assert_eq!(
            window.find("if-value").focused(),
            Some(true),
            "a new If checks a text"
        );
        window.input("{{clipboard}}", cx);
        // is, is not, contains, ...
        window.click(("if-op", 2usize), cx);
        window.render_frame(cx);
        window.click("if-other", cx);
        window.input("https://", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(
        flow.steps[0].kind,
        StepKind::If {
            condition: Condition::Contains {
                value: "{{clipboard}}".into(),
                text: "https://".into(),
            },
            not: false,
            then: Vec::new(),
            otherwise: Vec::new(),
        }
    );
    with(cx, handle, |window, cx| {
        assert!(window.find(("flow-step", 1usize)).visible());
        assert!(
            window.find("flow-add-1-0").visible(),
            "a place to add to 'then'"
        );
        assert!(
            window.find("flow-branch-1-1").visible(),
            "the Otherwise heading"
        );
        // Add to "otherwise" from its own line.
        window.click("flow-add-1-1", cx);
    });
    settle(cx, handle);
    pick_step_type(cx, handle, "stop");

    let flow = edited_flow(cx, &view);
    let StepKind::If {
        then, otherwise, ..
    } = &flow.steps[0].kind
    else {
        panic!("still an If");
    };
    assert!(then.is_empty());
    assert_eq!(
        otherwise.iter().map(|s| &s.kind).collect::<Vec<_>>(),
        vec![&StepKind::Stop],
        "Stop has nothing to fill in, so picking it adds it"
    );
    with(cx, handle, |window, cx| {
        assert!(!window.has_active_dialog(cx));
        assert!(window.find(("flow-step", 2usize)).visible());
    });
}

#[gpui_kit::test]
fn the_negated_comparisons_and_other_checks_are_saved(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "if");

    with(cx, handle, |window, cx| {
        // Text, Command, App, Power, Time.
        window.click(("if-kind", 3usize), cx);
        window.render_frame(cx);
        assert!(window.try_find("if-value").is_none());
        // on battery, plugged in.
        window.click(("if-op", 1usize), cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);
    assert_eq!(
        edited_flow(cx, &view).steps[0].kind,
        StepKind::If {
            condition: Condition::OnBattery,
            not: true,
            then: Vec::new(),
            otherwise: Vec::new(),
        }
    );

    open_add_step(cx, handle);
    pick_step_type(cx, handle, "if");
    with(cx, handle, |window, cx| {
        window.click(("if-kind", 4usize), cx);
        window.render_frame(cx);
        assert_eq!(window.find("if-from").value(), Some("09:00"));
        window.click("if-to", cx);
        window.press("ctrl-a", cx);
        window.input("25:00", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx), "25:00 is not a time");
        window.click("if-to", cx);
        window.press("ctrl-a", cx);
        window.input("12:30", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);
    assert_eq!(
        edited_flow(cx, &view).steps[1].kind,
        StepKind::If {
            condition: Condition::TimeBetween {
                from: "09:00".into(),
                to: "12:30".into(),
            },
            not: false,
            then: Vec::new(),
            otherwise: Vec::new(),
        }
    );
}

#[gpui_kit::test]
fn editing_a_block_keeps_the_steps_inside_it(cx: &mut TestAppContext) {
    write_flow("ui-nested", NESTED);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-nested".into()));

    with(cx, handle, |window, cx| {
        window.double_click(("flow-step", 2usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("if-kind").visible());
        // on battery -> plugged in.
        window.click(("if-op", 1usize), cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert!(matches!(flow.steps[1].kind, StepKind::If { not: true, .. }));
    assert_eq!(branches(&flow.steps[1]), vec![vec![2], vec![3]]);
}

#[gpui_kit::test]
fn a_new_step_goes_after_the_selected_one_or_into_the_selected_branch(cx: &mut TestAppContext) {
    write_flow("ui-nested", NESTED);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-nested".into()));

    // With the step inside "then" selected, a new step follows it there.
    with(cx, handle, |window, cx| {
        window.click(("flow-step", 3usize), cx)
    });
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "wait");
    with(cx, handle, |window, cx| window.press("ctrl-enter", cx));
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(branches(&flow.steps[1]), vec![vec![2, 1000], vec![3]]);
    with(cx, handle, |window, _| {
        assert_eq!(window.find("flow-steps").focused(), Some(true));
    });

    // With the Otherwise heading selected, it goes to the end of that branch.
    with(cx, handle, |window, cx| window.click("flow-branch-2-1", cx));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "wait");
    with(cx, handle, |window, cx| window.press("ctrl-enter", cx));
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(branches(&flow.steps[1]), vec![vec![2, 1000], vec![3, 1000]]);

    // The button under the list always adds at the end of the flow.
    with(cx, handle, |window, cx| window.click("flow-add-step", cx));
    settle(cx, handle);
    pick_step_type(cx, handle, "wait");
    with(cx, handle, |window, cx| window.press("ctrl-enter", cx));
    settle(cx, handle);
    assert_eq!(
        waits(&edited_flow(cx, &view)),
        vec![1, 2, 1000, 3, 1000, 4, 1000]
    );
}

#[gpui_kit::test]
fn alt_arrows_carry_a_step_into_and_out_of_a_block(cx: &mut TestAppContext) {
    write_flow("ui-nested", NESTED);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-nested".into()));

    with(cx, handle, |window, cx| {
        window.click(("flow-step", 1usize), cx);
        window.press("alt-down", cx);
    });
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 2);
    assert_eq!(branches(&flow.steps[0]), vec![vec![1, 2], vec![3]]);

    with(cx, handle, |window, cx| {
        window.press("alt-down", cx);
        window.press("alt-down", cx);
    });
    assert_eq!(
        branches(&edited_flow(cx, &view).steps[0]),
        vec![vec![2], vec![1, 3]],
        "past the last step of 'then' is the top of 'otherwise'"
    );

    with(cx, handle, |window, cx| {
        for _ in 0..3 {
            window.press("alt-up", cx);
        }
    });
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 3, "and back out above the If");
    assert_eq!(waits(&flow), vec![1, 2, 3, 4]);
}

#[gpui_kit::test]
fn left_and_right_fold_a_block(cx: &mut TestAppContext) {
    write_flow("ui-nested", NESTED);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-nested".into()));

    with(cx, handle, |window, cx| {
        window.click(("flow-step", 2usize), cx);
        assert!(window.find(("flow-step", 3usize)).visible());
        window.press("left", cx);
        window.render_frame(cx);
        assert!(
            window.try_find(("flow-step", 3usize)).is_none(),
            "the steps inside are hidden"
        );
        assert!(window.try_find("flow-add-2-0").is_none());
        assert!(window.find(("flow-step", 5usize)).visible(), "numbers stay");

        // Down from the folded block goes straight to the step after it.
        window.press("down", cx);
        window.press("delete", cx);
    });
    let flow = edited_flow(cx, &view);
    assert_eq!(
        waits(&flow),
        vec![1, 2, 3],
        "the step after the block was removed"
    );

    with(cx, handle, |window, cx| {
        window.click(("flow-step", 2usize), cx);
        window.press("right", cx);
        window.render_frame(cx);
        assert!(window.find(("flow-step", 3usize)).visible());
        // Left from a step inside goes to its block.
        window.click(("flow-step", 3usize), cx);
        window.press("left", cx);
        window.press("delete", cx);
    });
    assert_eq!(
        waits(&edited_flow(cx, &view)),
        vec![1],
        "removing a block removes what it holds"
    );
}

#[gpui_kit::test]
fn a_menu_gets_a_branch_per_choice_and_keeps_them_by_name(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "menu");

    with(cx, handle, |window, cx| {
        assert_eq!(window.find(OUTPUT_NAME).value(), Some("pick"));
        window.input("Power", cx);
        window.click(("step-choice", 0usize), cx);
        window.input("Lock", cx);
        window.click(("step-choice", 1usize), cx);
        window.input("Sleep", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.find("flow-branch-1-0").visible());
        assert!(window.find("flow-branch-1-1").visible());
        // A step for Sleep.
        window.click("flow-add-1-1", cx);
    });
    settle(cx, handle);
    pick_step_type(cx, handle, "wait");
    with(cx, handle, |window, cx| window.press("ctrl-enter", cx));
    settle(cx, handle);

    // Put a choice between the two; Sleep keeps its step.
    with(cx, handle, |window, cx| {
        window.double_click(("flow-step", 1usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.click(("step-choice", 0usize), cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.input("Off", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    let StepKind::Menu { prompt, choices } = &flow.steps[0].kind else {
        panic!("a menu");
    };
    assert_eq!(prompt, "Power");
    assert_eq!(
        choices
            .iter()
            .map(|c| (c.label.as_str(), c.steps.len()))
            .collect::<Vec<_>>(),
        vec![("Lock", 0), ("Off", 0), ("Sleep", 1)]
    );
}

#[gpui_kit::test]
fn the_icon_is_picked_in_a_dialog(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));
    with(cx, handle, |window, cx| {
        window.click("flow-icon-button", cx)
    });
    settle(cx, handle);

    // The search has the keyboard; Enter takes the first match.
    with(cx, handle, |window, cx| {
        assert!(window.find("icon-dialog").visible());
        assert_eq!(window.find("icon-search").focused(), Some(true));
        window.input("rock", cx);
        window.render_frame(cx);
        assert!(window.try_find("icon-sun").is_none());
        window.press("enter", cx);
    });
    settle(cx, handle);
    assert_eq!(edited_flow(cx, &view).icon, "rocket");
    with(cx, handle, |window, cx| {
        assert!(!window.has_active_dialog(cx))
    });

    // From the search, Down reaches the icons; the arrows move and a
    // click picks.
    with(cx, handle, |window, cx| {
        window.click("flow-icon-button", cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.press("down", cx);
        window.render_frame(cx);
        assert_eq!(window.find("icon-grid").focused(), Some(true));
        window.press("right", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
    // The current icon, rocket, was highlighted; Right moved one on.
    assert_eq!(edited_flow(cx, &view).icon, "sparkles");

    with(cx, handle, |window, cx| {
        window.click("flow-icon-button", cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| window.click("icon-sun", cx));
    settle(cx, handle);
    assert_eq!(edited_flow(cx, &view).icon, "sun");
}

#[gpui_kit::test]
fn the_name_is_edited_in_the_header(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));
    // A named flow opens on its title; Enter edits it in place.
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("flow-title").focused(), Some(true));
        assert!(window.try_find("flow-name").is_none());
        window.press("enter", cx);
    });
    settle(cx, handle);
    // The whole name is selected, so typing replaces it.
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("flow-name").focused(), Some(true));
        window.input("Renamed", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert!(window.try_find("flow-name").is_none());
        assert_eq!(window.find("flow-title").focused(), Some(true));
    });
    assert_eq!(edited_flow(cx, &view).name, "Renamed");

    // A click does the same.
    with(cx, handle, |window, cx| window.click("flow-title", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("flow-name").focused(), Some(true));
        window.input("Clicked", cx);
        window.press("enter", cx);
    });
    assert_eq!(edited_flow(cx, &view).name, "Clicked");

    // A new flow starts by asking for its name.
    let (handle, _) = open(cx, ActivePage::FlowNew(None));
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert_eq!(window.find("flow-name").focused(), Some(true));
    });
}

const LOOPS: &str = r#"
format = 2
id = "ui-loops"
name = "UI loops"

[[step]]
type = "exec"
command = "printf 'a\nb\n'"
wait = true
output = "letters"

[[step]]
type = "each"
items = "{{letters}}"

[[step.do]]
type = "exec"
command = "printf '%s%s' {{index}} {{item}}"
wait = true
"#;

#[gpui_kit::test]
fn a_loops_names_are_offered_only_inside_it(cx: &mut TestAppContext) {
    write_flow("ui-loops", LOOPS);
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-loops".into()));

    with(cx, handle, |window, cx| window.click("flow-add-2-0", cx));
    settle(cx, handle);
    pick_step_type(cx, handle, "notify");
    with(cx, handle, |window, cx| {
        assert!(window.find("step-variable-item").visible());
        assert!(window.find("step-variable-index").visible());
        assert!(window.find("step-variable-letters").visible());
        window.click("step-cancel", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| window.click("flow-add-step", cx));
    settle(cx, handle);
    pick_step_type(cx, handle, "notify");
    with(cx, handle, |window, _| {
        assert!(window.find("step-variable-letters").visible());
        assert!(
            window.try_find("step-variable-item").is_none(),
            "after the loop its names are gone"
        );
    });
}

#[gpui_kit::test]
fn a_run_marks_the_steps_inside_a_loop(cx: &mut TestAppContext) {
    write_flow("ui-loops", LOOPS);
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-loops".into()));

    with(cx, handle, |window, cx| window.click("flow-run", cx));
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", 3usize)).is_some()
    });
    with(cx, handle, |window, _| {
        assert!(window.find(("step-result", 1usize)).visible());
        assert!(
            window.find(("step-result", 3usize)).visible(),
            "the step inside the loop shows its last round's result"
        );
        assert!(window.try_find(("step-failure", 2usize)).is_none());
    });
}

#[gpui_kit::test]
fn repeat_asks_how_many_times(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "repeat");

    with(cx, handle, |window, cx| {
        window.press("ctrl-a", cx);
        window.input("5", cx);
        window.press("ctrl-enter", cx);
    });
    settle(cx, handle);
    assert_eq!(
        edited_flow(cx, &view).steps[0].kind,
        StepKind::Repeat {
            times: 5,
            steps: Vec::new(),
        }
    );
    with(cx, handle, |window, _| {
        assert!(window.find("flow-add-1-0").visible());
    });
}

// MARK: Ready-made actions

use omarchist::system::flows::actions::Arg;

fn action(flow: &Flow, index: usize) -> (String, Vec<(String, Arg)>) {
    match &flow.steps[index].kind {
        StepKind::Action { action, args } => (
            action.clone(),
            args.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ),
        other => panic!("not an action: {other:?}"),
    }
}

#[gpui_kit::test]
fn an_action_is_a_form_over_its_fields(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "volume");

    with(cx, handle, |window, cx| {
        // The level field has the keyboard, with its default in it.
        window.press("ctrl-a", cx);
        window.input("45", cx);
        window.press("ctrl-enter", cx);
    });
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(
        action(&flow, 0),
        (
            "volume.set".to_string(),
            vec![("level".to_string(), Arg::Number(45))]
        )
    );
    assert_eq!(flow.steps[0].kind.text(), "Set the volume to 45%");

    // Out of range is refused in the form.
    with(cx, handle, |window, cx| {
        window.double_click(("flow-step", 1usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.press("ctrl-a", cx);
        window.input("450", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx))
    });
    assert_eq!(
        edited_flow(cx, &view).steps[0].kind.text(),
        "Set the volume to 45%"
    );
}

#[gpui_kit::test]
fn an_actions_choices_and_text_fields_are_saved(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "change case");

    with(cx, handle, |window, cx| {
        assert_eq!(window.find("action-text").focused(), Some(true));
        assert_eq!(
            window.find(OUTPUT_NAME).value(),
            Some("text"),
            "its result is saved under a name"
        );
        window.click("step-variable-clipboard", cx);
        // UPPERCASE, lowercase, Title Case.
        window.click(("action-to", 2usize), cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(
        action(&flow, 0),
        (
            "text.case".to_string(),
            vec![
                ("text".to_string(), Arg::Text("{{clipboard}}".into())),
                ("to".to_string(), Arg::Text("title".into())),
            ]
        )
    );
    assert_eq!(flow.steps[0].output.as_deref(), Some("text"));
}

#[gpui_kit::test]
fn an_action_with_nothing_to_set_is_added_at_once(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "next wallpaper");

    let flow = edited_flow(cx, &view);
    assert_eq!(action(&flow, 0), ("wallpaper.next".to_string(), Vec::new()));
    with(cx, handle, |window, cx| {
        assert!(!window.has_active_dialog(cx))
    });
}

const TEXT_ACTIONS: &str = r#"
format = 2
id = "ui-text-actions"
name = "UI text actions"

[[step]]
type = "action"
action = "text"
text = "hello, world"
output = "greeting"

[[step]]
type = "action"
action = "text.case"
text = "{{greeting}}"
to = "upper"
output = "loud"

[[step]]
type = "exec"
command = "test {{loud}} = 'HELLO, WORLD'"
wait = true
"#;

#[gpui_kit::test]
fn actions_pass_their_results_on_in_a_run(cx: &mut TestAppContext) {
    write_flow("ui-text-actions", TEXT_ACTIONS);
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-text-actions".into()));

    with(cx, handle, |window, cx| window.click("flow-run", cx));
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", 3usize)).is_some()
    });
    with(cx, handle, |window, _| {
        assert!(window.find(("step-result", 2usize)).visible());
        assert!(
            window.try_find(("step-failure", 3usize)).is_none(),
            "the upper-cased text reached the command"
        );
    });
}

// MARK: Input

use omarchist::system::flows::InputFallback;

#[gpui_kit::test]
fn a_flow_that_uses_its_input_says_where_it_comes_from(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));

    with(cx, handle, |window, _| {
        assert!(
            window.try_find("flow-input-fallback").is_none(),
            "nothing to choose for a flow that reads no input"
        );
    });
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "notify");
    with(cx, handle, |window, cx| {
        window.click("step-variable-input", cx);
        window.click("step-save", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.find("flow-input-fallback").visible());
        // Nothing, Selected text, Clipboard, Ask.
        window.click(("flow-input-fallback", 1usize), cx);
    });
    assert_eq!(edited_flow(cx, &view).input, InputFallback::Selection);

    with(cx, handle, |window, cx| {
        window.click(("flow-input-fallback", 2usize), cx)
    });
    assert_eq!(edited_flow(cx, &view).input, InputFallback::Clipboard);
}

#[gpui_kit::test]
fn the_files_menu_is_a_trigger(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));

    assert!(!edited_flow(cx, &view).triggers.files);
    with(cx, handle, |window, cx| {
        window.click("flow-trigger-files", cx)
    });
    let flow = edited_flow(cx, &view);
    assert!(flow.triggers.files);
    with(cx, handle, |window, cx| {
        window.render_frame(cx);
        assert!(
            window.find("flow-input-fallback").visible(),
            "a flow run on files gets input"
        );
    });
}

#[gpui_kit::test]
fn run_a_flow_can_hand_it_input(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "run a flow");

    with(cx, handle, |window, cx| {
        assert!(window.find("flow-input").visible());
        window.click("flow-input", cx);
        window.input("hello", cx);
    });
    settle(cx, handle);
    // Without a flow picked there is nothing to add.
    with(cx, handle, |window, cx| window.click("step-save", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx))
    });
    assert!(edited_flow(cx, &view).steps.is_empty());
}

// MARK: Automations

use omarchist::system::flows::automations::{Day, Event};

#[gpui_kit::test]
fn an_automation_is_added_from_its_dialog(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));

    with(cx, handle, |window, cx| {
        assert!(window.try_find("automations-service-off").is_none());
        window.click("flow-add-automation", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("automation-dialog").visible());
        // A new automation waits for a time of day; pick Monday and Friday.
        assert_eq!(window.find("automation-time").value(), Some("09:00"));
        window.click(("automation-day", 0usize), cx);
        window.click(("automation-day", 4usize), cx);
        window.click("automation-time", cx);
        window.press("ctrl-a", cx);
        window.input("07:30", cx);
        window.click("automation-save", cx);
    });
    settle(cx, handle);

    let flow = edited_flow(cx, &view);
    assert_eq!(flow.triggers.automations.len(), 1);
    let automation = &flow.triggers.automations[0];
    assert_eq!(
        automation.event,
        Event::Time {
            at: "07:30".into(),
            days: vec![Day::Mon, Day::Fri],
        }
    );
    assert!(automation.enabled && !automation.ask);
    with(cx, handle, |window, cx| {
        assert!(!window.has_active_dialog(cx));
        assert!(window.find(("flow-automation", 0usize)).visible());
        assert!(
            window.find("automations-service-off").visible(),
            "with an automation and no service, the card says so"
        );
        // Its switch turns it off without removing it.
        window.click(("automation-enabled", 0usize), cx);
    });
    assert!(!edited_flow(cx, &view).triggers.automations[0].enabled);
    with(cx, handle, |window, cx| {
        window.click(("automation-remove", 0usize), cx)
    });
    assert!(edited_flow(cx, &view).triggers.automations.is_empty());
}

#[gpui_kit::test]
fn a_time_that_is_not_one_is_refused(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    with(cx, handle, |window, cx| {
        window.click("flow-add-automation", cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.click("automation-time", cx);
        window.press("ctrl-a", cx);
        window.input("9am", cx);
        window.press("ctrl-enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.has_active_dialog(cx))
    });
    assert!(edited_flow(cx, &view).triggers.automations.is_empty());
}

#[gpui_kit::test]
fn what_an_automation_waits_for_decides_its_fields(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    with(cx, handle, |window, cx| {
        window.click("flow-add-automation", cx)
    });
    settle(cx, handle);

    // Open the list of events, search it, and take the match.
    with(cx, handle, |window, cx| {
        window.click("automation-kind", cx);
        window.render_frame(cx);
        window.input("battery", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| window.press("enter", cx));
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        assert!(window.try_find("automation-time").is_none());
        assert!(window.find("automation-percent").visible());
        window.click("automation-ask", cx);
        window.click("automation-save", cx);
    });
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(
        flow.triggers.automations[0].event,
        Event::BatteryBelow { percent: 20 }
    );
    assert!(flow.triggers.automations[0].ask);

    // Editing opens the same dialog on what is there.
    with(cx, handle, |window, cx| {
        window.click(("automation-edit", 0usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("automation-percent").visible());
        window.click("automation-ask", cx);
        window.click("automation-save", cx);
    });
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.triggers.automations.len(), 1, "edited in place");
    assert!(!flow.triggers.automations[0].ask);
}

// MARK: Run history

use omarchist::system::flows::history::{self, Run, RunResult, StepResult, StepRun};
use omarchist::ui::flows_page::flows_view::FlowHistory;

fn past_run(trigger: &str, result: RunResult, steps: Vec<StepRun>) -> Run {
    Run {
        started: chrono::Local::now().timestamp() - 120,
        ms: 400,
        trigger: trigger.to_string(),
        result,
        summary: String::new(),
        steps,
    }
}

#[gpui_kit::test]
fn a_run_is_added_to_the_flows_history(cx: &mut TestAppContext) {
    write_flow("ui-history", &PRINTS.replace("ui-prints", "ui-history"));
    history::clear("ui-history").unwrap();
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-history".into()));

    with(cx, handle, |window, cx| window.click("flow-run", cx));
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", 3usize)).is_some()
    });

    let runs = history::load("ui-history");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].trigger, "Editor");
    assert_eq!(runs[0].result, RunResult::Failed, "its second step fails");
    assert_eq!(
        runs[0].steps.iter().map(|s| s.result).collect::<Vec<_>>(),
        vec![StepResult::Done, StepResult::Failed, StepResult::Done]
    );

    // The toast of the run covers the History button; the shortcut is
    // not in its way.
    with(cx, handle, |window, cx| window.press("ctrl-shift-h", cx));
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert!(window.find("history-dialog").visible());
        assert_eq!(
            window.find("history-runs").focused(),
            Some(true),
            "the list of runs has the keyboard"
        );
        assert!(window.find(("history-run", 0usize)).visible());
        assert!(
            window.find(("history-steps", 0usize)).visible(),
            "the newest run starts opened"
        );
        assert!(window.try_find(("history-run", 1usize)).is_none());
    });
}

#[gpui_kit::test]
fn the_history_opens_a_run_from_the_keyboard_and_can_be_cleared(cx: &mut TestAppContext) {
    write_flow("ui-past", &PRINTS.replace("ui-prints", "ui-past"));
    history::clear("ui-past").unwrap();
    let step = StepRun {
        number: 1,
        depth: 0,
        title: "Wait 1 s".into(),
        result: StepResult::Done,
        detail: String::new(),
        ms: 1000,
    };
    // Oldest first, as runs are recorded.
    history::record(
        "ui-past",
        &past_run("Keybind", RunResult::Finished, vec![step.clone()]),
    )
    .unwrap();
    history::record(
        "ui-past",
        &past_run("Launcher", RunResult::Stopped, vec![step]),
    )
    .unwrap();
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-past".into()));

    with(cx, handle, |window, cx| window.click("flow-history", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find(("history-run", 0usize)).visible());
        assert!(window.find(("history-run", 1usize)).visible());
        assert!(window.find(("history-steps", 0usize)).visible());
        assert!(window.try_find(("history-steps", 1usize)).is_none());

        // Down to the older run, Enter opens it in place of the newest.
        window.press("down", cx);
        window.press("enter", cx);
        window.render_frame(cx);
        assert!(window.find(("history-steps", 1usize)).visible());
        assert!(window.try_find(("history-steps", 0usize)).is_none());

        window.click("history-clear", cx);
        window.render_frame(cx);
        assert!(window.try_find(("history-run", 0usize)).is_none());
        assert!(window.find("history-dialog").visible(), "the dialog stays");
    });
    assert!(history::load("ui-past").is_empty());
}

#[gpui_kit::test]
fn a_flow_that_was_never_saved_has_no_history(cx: &mut TestAppContext) {
    let (handle, _view) = open(cx, ActivePage::FlowNew(None));
    with(cx, handle, |window, cx| window.press("ctrl-shift-h", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.try_find("history-dialog").is_none());
        assert_eq!(window.notifications(cx).len(), 1);
    });
}

#[gpui_kit::test]
fn a_flows_card_says_when_it_last_ran(cx: &mut TestAppContext) {
    write_flow("ui-ran", &PRINTS.replace("ui-prints", "ui-ran"));
    write_flow("ui-idle", &PRINTS.replace("ui-prints", "ui-idle"));
    history::clear("ui-ran").unwrap();
    history::record("ui-ran", &past_run("Keybind", RunResult::Finished, vec![])).unwrap();
    let (handle, _view) = open(cx, ActivePage::Flows);
    settle(cx, handle);

    // Other tests keep flows in the same home; the search leaves one card.
    with(cx, handle, |window, cx| window.input("ui-ran", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find(("flow-last-run", 0usize)).visible());
        window.dispatch_action(Box::new(FlowHistory(0)), cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find(("history-run", 0usize)).visible());
        window.close_dialog(cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, cx| {
        window.press("ctrl-a", cx);
        window.input("ui-idle", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert!(
            window.try_find(("flow-last-run", 0usize)).is_none(),
            "a flow that never ran says nothing"
        );
    });
}

// MARK: One step alone

#[gpui_kit::test]
fn a_step_runs_alone_with_what_the_last_run_saved(cx: &mut TestAppContext) {
    write_flow("ui-alone", &PRINTS.replace("ui-prints", "ui-alone"));
    history::clear("ui-alone").unwrap();
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-alone".into()));
    let finished = |window: &mut gpui_kit::Window, number: usize| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", number)).is_some()
    };

    // The first step uses no variable: it runs at once.
    with(cx, handle, |window, cx| {
        window.click(("step-test", 1usize), cx)
    });
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        finished(window, 1)
    });
    with(cx, handle, |window, cx| {
        assert!(window.try_find("test-step-dialog").is_none());
        assert!(
            window.try_find(("step-failure", 2usize)).is_none()
                && window.try_find(("step-result", 3usize)).is_none(),
            "the other steps did not run"
        );
        window.clear_notifications(cx);
    });

    // The third uses what the first saves, so it asks, offering what the
    // first just produced.
    with(cx, handle, |window, cx| {
        window.click(("flow-step", 3usize), cx);
        window.press("shift-enter", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("test-step-dialog").visible());
        assert!(window.find("test-value-text").visible());
        window.click("test-step-run", cx);
    });
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        finished(window, 3)
    });
    with(cx, handle, |window, _| {
        assert!(window.try_find("test-step-dialog").is_none());
        assert!(
            window.try_find(("step-failure", 3usize)).is_none(),
            "the value of the first step reached the third"
        );
        assert!(
            window.find(("step-result", 1usize)).visible(),
            "the first step keeps its result"
        );
    });
    assert!(
        history::load("ui-alone").is_empty(),
        "trying a step is not a run of the flow"
    );
}

#[gpui_kit::test]
fn a_step_alone_fails_on_a_value_typed_for_it(cx: &mut TestAppContext) {
    write_flow(
        "ui-alone-typed",
        &PRINTS.replace("ui-prints", "ui-alone-typed"),
    );
    let (handle, _view) = open(cx, ActivePage::FlowEdit("ui-alone-typed".into()));

    with(cx, handle, |window, cx| {
        window.click(("step-test", 3usize), cx)
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        // The first field has the keyboard.
        window.input("something else", cx);
        window.press("ctrl-enter", cx);
    });
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-failure", 3usize)).is_some()
    });
    with(cx, handle, |window, _| {
        assert!(window.try_find("test-step-dialog").is_none());
        assert!(window.try_find(("step-result", 1usize)).is_none());
    });
}

// MARK: Undo

#[gpui_kit::test]
fn changes_to_the_flow_are_undone_and_redone(cx: &mut TestAppContext) {
    write_flow("ui-three-steps", THREE_STEPS);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-three-steps".into()));
    let original = edited_flow(cx, &view);
    let editor = common::editor(cx, &view);
    let can = |cx: &mut TestAppContext| cx.update(|cx| editor.read(cx).can_undo_redo());
    assert_eq!(can(cx), (false, false));

    // Three changes: a step removed, a step switched off, a switch.
    with(cx, handle, |window, cx| {
        window.click(("flow-step", 2usize), cx);
        window.press("delete", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.click(("flow-step", 1usize), cx);
        window.press("space", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| window.click("flow-on-error", cx));
    settle(cx, handle);
    let changed = edited_flow(cx, &view);
    assert_eq!(changed.steps.len(), 2);
    assert!(!changed.steps[0].enabled);
    assert_ne!(changed.on_error, original.on_error);

    // Back one at a time, newest first.
    assert_eq!(can(cx), (true, false));
    with(cx, handle, |window, cx| window.press("ctrl-z", cx));
    settle(cx, handle);
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.on_error, original.on_error);
    assert!(!flow.steps[0].enabled);
    with(cx, handle, |window, cx| {
        window.press("ctrl-z", cx);
        window.press("ctrl-z", cx);
    });
    settle(cx, handle);
    assert_eq!(edited_flow(cx, &view), original);
    with(cx, handle, |window, _| {
        assert!(window.find(("flow-step", 3usize)).visible());
    });
    assert_eq!(can(cx), (false, true));
    assert!(
        !cx.update(|cx| editor.read(cx).is_dirty(cx)),
        "back at what was saved, nothing is unsaved"
    );

    // And forward again, by the button and by the key.
    with(cx, handle, |window, cx| window.click("flow-redo", cx));
    settle(cx, handle);
    assert_eq!(edited_flow(cx, &view).steps.len(), 2);
    with(cx, handle, |window, cx| {
        window.press("ctrl-shift-z", cx);
        window.press("ctrl-y", cx);
    });
    settle(cx, handle);
    assert_eq!(edited_flow(cx, &view), changed);

    // A new change after an undo leaves nothing to redo.
    with(cx, handle, |window, cx| window.press("ctrl-z", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.click("flow-trigger-launcher", cx)
    });
    settle(cx, handle);
    assert_eq!(can(cx), (true, false));
    assert!(edited_flow(cx, &view).triggers.launcher);
}

// MARK: Templates of your own

use omarchist::system::flows::templates::template;
use omarchist::ui::flows_page::flow_edit_view::flow_edit_nav::SaveAsTemplate;

#[gpui_kit::test]
fn a_flow_becomes_a_template_and_the_template_can_be_deleted(cx: &mut TestAppContext) {
    let toml = THREE_STEPS
        .replace("ui-three-steps", "ui-keeper")
        .replace("UI three steps", "UI keeper");
    assert!(
        toml.contains("UI keeper"),
        "the test flow is named in its file"
    );
    write_flow("ui-keeper", &toml);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-keeper".into()));

    with(cx, handle, |window, cx| {
        window.dispatch_action(Box::new(SaveAsTemplate), cx);
    });
    settle(cx, handle);
    let saved = template("user:ui-keeper").expect("the template was written");
    assert!(saved.flow.id.is_empty(), "a template has no id");
    assert_eq!(saved.flow.steps, edited_flow(cx, &view).steps);

    // The Templates page lists it first, under the search.
    with(cx, handle, |window, cx| {
        window.clear_notifications(cx);
        omarchist::ui::app_events::emit(
            cx,
            omarchist::ui::app_events::AppEvent::Navigate(ActivePage::FlowTemplates),
        );
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("templates-search").focused(), Some(true));
        window.input("ui keeper", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find(("user-template", 0usize)).visible());
        assert!(
            window.try_find(("built-in-template", 0usize)).is_none(),
            "the search narrows both groups"
        );
        window.click(("template-delete", 0usize), cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("confirm-dialog").visible());
        window.click("confirm-ok", cx);
    });
    settle(cx, handle);
    assert!(template("user:ui-keeper").is_none());
    with(cx, handle, |window, cx| {
        assert!(window.try_find(("user-template", 0usize)).is_none());
        assert!(window.find("templates-none").visible());
        // Escape clears the search, and the built-in ones are back.
        window.press("escape", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, _| {
        assert!(window.find(("built-in-template", 0usize)).visible());
    });
    common::assert_page(cx, &view, ActivePage::FlowTemplates);
}

// MARK: The gallery

use omarchist::system::flows::catalog;
use omarchist::system::flows::store::load_flow;
use omarchist::ui::flows_page::flow_edit_view::flow_edit_nav::Publish;

/// The Gallery page with the tests' gallery loaded.
fn open_gallery(
    cx: &mut TestAppContext,
) -> (
    gpui_kit::WindowHandle<gpui_kit::component::Root>,
    gpui_kit::Entity<omarchist::MainWindowView>,
) {
    common::gallery();
    let (handle, view) = open(cx, ActivePage::FlowGallery);
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find(("gallery-card", 0usize)).is_some()
    });
    (handle, view)
}

/// The slugs the Gallery page lists, in its order.
fn listed(
    cx: &mut TestAppContext,
    view: &gpui_kit::Entity<omarchist::MainWindowView>,
) -> Vec<String> {
    cx.update(|cx| {
        let gallery = view.read(cx).flow_gallery().expect("the gallery page");
        gallery
            .read(cx)
            .shown()
            .iter()
            .map(|entry| entry.slug.clone())
            .collect()
    })
}

#[gpui_kit::test]
fn the_gallery_lists_flows_and_narrows_them(cx: &mut TestAppContext) {
    let (handle, view) = open_gallery(cx);
    // The most installed first.
    assert_eq!(listed(cx, &view), vec!["pause", "breathe", "careful"]);
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("gallery-search").focused(), Some(true));
        assert!(window.find(("gallery-card", 2usize)).visible());
        assert!(window.try_find("gallery-notice").is_none());
        // Only the categories that hold a flow are offered.
        assert!(window.find("gallery-filter-focus").visible());
        assert!(window.try_find("gallery-filter-web").is_none());
        window.click("gallery-sort-newest", cx);
    });
    settle(cx, handle);
    assert_eq!(listed(cx, &view), vec!["breathe", "careful", "pause"]);

    with(cx, handle, |window, cx| {
        window.click("gallery-filter-focus", cx)
    });
    settle(cx, handle);
    assert_eq!(listed(cx, &view), vec!["pause"]);
    // The arrow keys move through the filters, and round again.
    with(cx, handle, |window, cx| window.press("left", cx));
    settle(cx, handle);
    assert_eq!(listed(cx, &view).len(), 3);

    // The search reads names, descriptions and authors.
    with(cx, handle, |window, cx| {
        window.click("gallery-search", cx);
        window.input("grace", cx);
    });
    settle(cx, handle);
    assert_eq!(listed(cx, &view), vec!["breathe", "careful"]);
    with(cx, handle, |window, cx| window.input(" nothing", cx));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        assert!(window.find("gallery-none").visible());
        window.press("escape", cx);
    });
    settle(cx, handle);
    assert_eq!(listed(cx, &view).len(), 3);
    common::assert_page(cx, &view, ActivePage::FlowGallery);
    // Escape in the empty box leaves.
    with(cx, handle, |window, cx| window.press("escape", cx));
    settle(cx, handle);
    common::assert_page(cx, &view, ActivePage::Flows);
}

#[gpui_kit::test]
fn a_gallery_flow_is_installed_through_the_review_screen(cx: &mut TestAppContext) {
    let (handle, view) = open_gallery(cx);
    // "Careful" is the last card: the least installed, by name.
    with(cx, handle, |window, cx| {
        window.click(("gallery-card", 2usize), cx)
    });
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("gallery-detail-risks").is_some()
    });
    with(cx, handle, |window, _| {
        assert!(window.find("gallery-detail").visible());
        assert!(window.find("gallery-detail-needs").visible());
        assert_eq!(
            window.find("gallery-detail-list").focused(),
            Some(true),
            "the steps have the keyboard, to scroll through"
        );
    });
    // Ctrl+Enter is the dialog's main button once the steps are fetched.
    wait_real(cx, handle, Duration::from_secs(10), |window, cx| {
        window.press("ctrl-enter", cx);
        window.try_find("gallery-detail").is_none()
    });
    settle(cx, handle);

    // Nothing is saved: the flow is in the editor, to be read first.
    let flow = edited_flow(cx, &view);
    assert!(flow.id.is_empty());
    assert_eq!(flow.meta.source, "catalog:careful@1");
    assert!(load_flow("careful").is_err());
    with(cx, handle, |window, cx| {
        assert!(
            window.find("import-risks").visible(),
            "the step that runs as administrator is pointed out"
        );
        window.press("ctrl-s", cx);
    });
    settle(cx, handle);
    let saved = load_flow("careful").expect("saved after the review");
    assert_eq!(catalog::source_of(&saved), Some(("careful".to_string(), 1)));
    assert!(saved.triggers.is_empty());
    std::fs::remove_file(common::home().join(".config/omarchist/flows/careful.toml")).unwrap();
}

#[gpui_kit::test]
fn an_update_from_the_gallery_is_one_change_to_undo(cx: &mut TestAppContext) {
    common::gallery();
    // Installed as version 1, which waited a second and said nothing.
    write_flow(
        "ui-pause",
        r#"
format = 2
id = "ui-pause"
name = "My pause"
icon = "moon"

[meta]
author = "ada"
version = "1"
source = "catalog:pause@1"
category = "Focus"
license = "CC0-1.0"

[triggers]
launcher = true

[[step]]
type = "wait"
ms = 1000
"#,
    );
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-pause".into()));
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        window.click("flow-gallery-update", cx)
    });
    // The dialog compares the saved steps with the new version's.
    wait_real(cx, handle, Duration::from_secs(10), |window, cx| {
        window.press("ctrl-enter", cx);
        window.try_find("gallery-detail").is_none()
    });
    settle(cx, handle);

    let editor = common::editor(cx, &view);
    let updated = edited_flow(cx, &view);
    assert_eq!(waits(&updated), vec![500]);
    assert_eq!(updated.steps.len(), 2);
    assert_eq!(updated.meta.source, "catalog:pause@2");
    // What is this machine's stays.
    assert_eq!(
        (updated.id.as_str(), updated.name.as_str()),
        ("ui-pause", "My pause")
    );
    assert_eq!(updated.icon, "moon");
    assert!(updated.triggers.launcher);
    assert!(cx.update(|cx| editor.read(cx).is_dirty(cx)));
    assert_eq!(
        waits(&load_flow("ui-pause").unwrap()),
        vec![1000],
        "not saved yet"
    );

    with(cx, handle, |window, cx| {
        assert!(window.try_find("flow-gallery-update").is_none());
        window.click("flow-steps", cx);
        window.press("ctrl-z", cx);
    });
    settle(cx, handle);
    let back = edited_flow(cx, &view);
    assert_eq!(waits(&back), vec![1000]);
    assert_eq!(back.meta.source, "catalog:pause@1");
}

#[gpui_kit::test]
fn publishing_makes_the_flow_ready_and_hands_it_to_github(cx: &mut TestAppContext) {
    common::gallery();
    let toml = THREE_STEPS
        .replace("ui-three-steps", "ui-shared")
        .replace("UI three steps", "UI shared");
    write_flow("ui-shared", &toml);
    let (handle, view) = open(cx, ActivePage::FlowEdit("ui-shared".into()));
    with(cx, handle, |window, cx| {
        window.click("flow-description", cx);
        window.input("Three steps, shared by a test.", cx);
        window.dispatch_action(Box::new(Publish), cx);
    });
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("publish-dialog").is_some()
    });
    settle(cx, handle);

    // Without the license, nothing leaves.
    with(cx, handle, |window, cx| {
        assert_eq!(window.find("publish-author").focused(), Some(true));
        window.input("ada", cx);
        window.click("publish-category-text", cx);
        window.click("publish-tags", cx);
        window.input("Test, sharing", cx);
        window.click("publish-submit", cx);
    });
    settle(cx, handle);
    with(cx, handle, |window, cx| {
        // Refused with an error toast; the dialog stays.
        assert!(window.try_find("publish-dialog").is_some());
        assert_eq!(window.notifications(cx).len(), 1);
        window.clear_notifications(cx);
        window.click("publish-license", cx);
        window.click("publish-submit", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, _| {
        assert!(window.try_find("publish-dialog").is_none());
    });
    let url = cx.opened_url().expect("GitHub opens in the browser");
    assert!(
        url.starts_with(
            "https://github.com/tahayvr/omarchist-flows/new/main?filename=flows%2Fui%2Dshared%2Eflow%2Etoml"
        ),
        "{url}"
    );
    let copied = cx
        .read_from_clipboard()
        .and_then(|item| item.text())
        .expect("the flow's text is on the clipboard");
    let report = catalog::check_text("ui-shared.flow.toml", &copied, true);
    assert!(report.ok, "{:?}", report.errors);
    assert_eq!(report.author, "ada");
    assert_eq!(report.category, "Text");
    assert_eq!(report.tags, vec!["test", "sharing"]);
    assert!(
        !copied.contains("ui-shared\""),
        "the id stays on this machine"
    );
    // The flow in the editor is as it was.
    assert!(edited_flow(cx, &view).meta.author.is_empty());
}
