use gpui::actions;

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
