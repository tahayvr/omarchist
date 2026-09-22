use crate::shell::theme_sh_commands::execute_bash_command;
use crate::system::themes::overrides::{self, OverrideSpec};
use crate::ui::theme_edit_page::shared::{
    error_message, focus_section, form_section, help_text, tab_container,
};
use gpui::*;
use gpui_component::{ActiveTheme, button::Button, h_flex, radio::Radio, v_flex};

struct YaruColor {
    value: &'static str,
    label: &'static str,
    color: u32,
}

const YARU_COLORS: &[YaruColor] = &[
    YaruColor {
        value: "Yaru-red",
        label: "Red",
        color: 0xe92020,
    },
    YaruColor {
        value: "Yaru-blue",
        label: "Blue",
        color: 0x208fe9,
    },
    YaruColor {
        value: "Yaru-olive",
        label: "Olive",
        color: 0x636B2F,
    },
    YaruColor {
        value: "Yaru-yellow",
        label: "Yellow",
        color: 0xe9ba20,
    },
    YaruColor {
        value: "Yaru-purple",
        label: "Purple",
        color: 0x5e2750,
    },
    YaruColor {
        value: "Yaru-magenta",
        label: "Magenta",
        color: 0xFF00FF,
    },
    YaruColor {
        value: "Yaru-sage",
        label: "Sage",
        color: 0x123d18,
    },
];

pub struct FileManagerTab {
    theme_name: String,
    selected_color: String,
    error_message: Option<String>,
    scroll: ScrollHandle,
}

impl FileManagerTab {
    pub fn new(
        theme_name: String,
        scroll: &ScrollHandle,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Self {
        let selected_color = Self::icons_spec()
            .and_then(|spec| overrides::read(&theme_name, spec).ok().flatten())
            .map(|content| content.trim().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Yaru-blue".to_string());

        Self {
            theme_name,
            selected_color,
            error_message: None,
            scroll: scroll.clone(),
        }
    }

    fn icons_spec() -> Option<&'static OverrideSpec> {
        overrides::find("icons.theme")
    }

    fn set_icon_theme(&mut self, name: &str, cx: &mut Context<Self>) {
        self.selected_color = name.to_string();
        self.error_message = Self::icons_spec()
            .and_then(|spec| overrides::write(&self.theme_name, spec, &format!("{name}\n")).err())
            .map(|e| e.to_string());
        cx.notify();
    }

    fn launch_file_manager(&self) {
        let command = "uwsm app -- nautilus --new-window".to_string();
        if let Err(e) = execute_bash_command(command) {
            eprintln!("Failed to launch nautilus: {}", e);
        }
    }

    fn create_color_radio(
        &self,
        yaru_color: &'static YaruColor,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let color_value: SharedString = yaru_color.value.into();
        let is_selected = self.selected_color == yaru_color.value;
        let color_hex = yaru_color.color;

        h_flex()
            .gap_3()
            .items_center()
            .child(
                Radio::new(color_value)
                    .label(yaru_color.label)
                    .checked(is_selected)
                    .on_click(cx.listener(move |this, _checked: &bool, _window, cx| {
                        this.set_icon_theme(yaru_color.value, cx);
                    })),
            )
            .child(div().size_6().bg(gpui::rgb(color_hex)).border_1())
    }
}

impl Render for FileManagerTab {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = v_flex().gap_3();

        for yaru_color in YARU_COLORS {
            container = container.child(self.create_color_radio(yaru_color, cx));
        }

        tab_container()
            .children(
                self.error_message
                    .as_ref()
                    .map(|msg| error_message(msg.clone(), cx)),
            )
            .child(focus_section(
                "file-manager-header",
                &self.scroll,
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(help_text(
                        "Accent color for Nautilus.",
                        cx.theme().muted_foreground,
                    ))
                    .child(
                        Button::new("launch-nautilus")
                            .label("Nautilus")
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                this.launch_file_manager();
                            })),
                    ),
            ))
            .child(focus_section(
                "file-manager-colors",
                &self.scroll,
                form_section().child(container),
            ))
    }
}
