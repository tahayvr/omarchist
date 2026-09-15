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
        // Themes page keyboard navigation (see themes_view.rs)
        NextItem,
        PrevItem,
        ActivateItem,
        EscapeFocus,
        SelectNext,
        SelectPrev,
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
