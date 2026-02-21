use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{Select, SelectState},
    ActiveTheme, Icon, IconName, IndexPath, Sizable,
};

pub struct StatusBarHeader {
    profile_select: Entity<SelectState<Vec<SharedString>>>,
}

impl StatusBarHeader {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let profiles: Vec<SharedString> =
            vec!["Omarchy Default".into(), "Work".into(), "Gaming".into()];

        let profile_select =
            cx.new(|cx| SelectState::new(profiles, Some(IndexPath::default()), window, cx));

        Self { profile_select }
    }
}

impl Render for StatusBarHeader {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        h_flex()
            .w_full()
            .p_4()
            .gap_4()
            .items_center()
            .justify_between()
            .border_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(200.))
                            .child(Select::new(&self.profile_select).small()),
                    )
                    .child(
                        Button::new("add-profile")
                            .icon(Icon::new(IconName::Plus))
                            .ghost()
                            .small()
                            .tooltip("Add profile")
                            .on_click(|_, _, _| {}),
                    ),
            )
            .child(
                Button::new("refresh-status-bar")
                    .icon(Icon::new(IconName::LoaderCircle))
                    .ghost()
                    .small()
                    .tooltip("Refresh status bar")
                    .on_click(|_, _, _| {}),
            )
    }
}
