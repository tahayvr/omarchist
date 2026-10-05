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
        assert!(window.find("step-type-ask-for-text").visible());
        assert!(window.find("step-type-run-a-command").visible());
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
        // Down leaves the search box for the grid, on its first tile.
        window.press("down", cx);
        window.render_frame(cx);
        assert_eq!(window.find(GRID).focused(), Some(true));
        assert_eq!(window.find(SEARCH).focused(), Some(false));

        // Apps, then Desktop, then the Ask group; Right moves along its row.
        window.press("down", cx);
        window.press("down", cx);
        window.press("right", cx);
        window.press("enter", cx);
    });
    settle(cx, handle);

    with(cx, handle, |window, _| {
        assert!(
            window.find("choose-source").visible(),
            "Enter opened the form of Choose from a list"
        );
        assert_eq!(window.find(PROMPT).focused(), Some(true));
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
        window.click("step-type-confirm", cx)
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
        assert!(window.find(("flow-step", 0usize)).visible());
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
fn choose_takes_its_options_from_a_list(cx: &mut TestAppContext) {
    let (handle, view) = open(cx, ActivePage::FlowNew(None));
    open_add_step(cx, handle);
    pick_step_type(cx, handle, "choose");

    with(cx, handle, |window, cx| {
        window.input("Which size?", cx);
        window.click("step-options", cx);
        window.input("Small", cx);
        window.press("enter", cx);
        window.input("Large", cx);
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
        window.click(("flow-step", 0usize), cx);
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
        window.click(("flow-step", 1usize), cx);
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
        assert!(window.try_find(("flow-step", 3usize)).is_none());
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
        window.click(("flow-step", 0usize), cx);
        window.press("delete", cx);
        window.render_frame(cx);
    });
    let flow = edited_flow(cx, &view);
    assert_eq!(flow.steps.len(), 2);
    assert!(flow.validate_content().is_err());
    with(cx, handle, |window, _| {
        assert!(window.find(("step-unknown-variable", 1usize)).visible());
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
        assert!(window.try_find(("step-result", 0usize)).is_none());
        window.click("flow-run", cx);
    });
    // The run is on a thread of its own, outside the test executor.
    wait_real(cx, handle, Duration::from_secs(10), |window, _| {
        window.try_find("flow-run").is_some() && window.try_find(("step-result", 2usize)).is_some()
    });

    with(cx, handle, |window, _| {
        assert!(window.find(("step-result", 0usize)).visible());
        assert!(
            window.find(("step-failure", 1usize)).visible(),
            "the failed step says why"
        );
        assert!(
            window.try_find(("step-result", 1usize)).is_none(),
            "a failed step has no result"
        );
        assert!(
            window.try_find(("step-failure", 2usize)).is_none(),
            "the value saved by the first step reached the third"
        );
    });
}
