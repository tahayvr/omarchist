use gpui::{Action, actions};

actions!(
    app_menu,
    [
        About,
        Quit,
        NavigateToAbout,
        NavigateToSettings,
        NavigateToOmarchy,
        Copy,
        Paste,
        Cut,
        RefreshTheme,
        ToggleSidebar,
        NewTheme,
        // Page navigation shortcuts
        NavigateToThemes,
        NavigateToConfig,
        NavigateToKeybinds,
        NavigateToFlows,
        // Title-bar menus and the command palette
        NewKeybind,
        SearchKeybindsByKeys,
        NewFlow,
        NewFlowFromTemplate,
        ImportFlow,
        // Theme edit actions
        ThemeEditNextTab,
        ThemeEditPrevTab,
        NavigateBack,
    ]
);
actions!(appearance, [SwitchToLight, SwitchToDark]);

#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = app_menu, no_json)]
pub struct SelectFont(pub i32);

/// Runs the saved flow with this id from any page.
#[derive(Action, Clone, PartialEq, Eq, Debug)]
#[action(namespace = app_menu, no_json)]
pub struct RunFlow(pub String);
