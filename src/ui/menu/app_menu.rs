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
        // Keybinds page: open the selected row
        ActivateItem,
        // Page navigation shortcuts
        NavigateToThemes,
        NavigateToConfig,
        NavigateToKeybinds,
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
