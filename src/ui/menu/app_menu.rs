use gpui::{Action, actions};

actions!(
    app_menu,
    [
        About,
        Quit,
        NavigateToAbout,
        NavigateToSettings,
        Copy,
        Paste,
        Cut
    ]
);
actions!(appearance, [SwitchToLight, SwitchToDark]);

/// Action to select a font size
#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = app_menu, no_json)]
pub struct SelectFont(pub i32);
