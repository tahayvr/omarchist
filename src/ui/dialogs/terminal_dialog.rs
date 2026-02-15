use gpui::*;
use gpui_component::{ActiveTheme, button::Button};

use crate::ui::terminal_page::terminal_page::TerminalPage;

/// A modal dialog containing an embedded terminal
pub struct TerminalDialog {
    terminal_page: Entity<TerminalPage>,
    on_complete: Option<Box<dyn FnOnce(i32)>>,
    command: String,
}

impl TerminalDialog {
    /// Create a new terminal dialog
    pub fn new(
        command: String,
        on_complete: Option<Box<dyn FnOnce(i32)>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let terminal_page =
            cx.new(|cx| TerminalPage::new_without_header(command.clone(), window, cx));

        Self {
            terminal_page,
            on_complete,
            command,
        }
    }

    /// Close the dialog
    fn close_dialog(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        window.remove_window();
    }
}

impl Render for TerminalDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .w(px(900.0))
            .h(px(700.0))
            .bg(theme.background)
            .rounded_md()
            .shadow_lg()
            .flex()
            .flex_col()
            .child(
                // Header
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground)
                            .child(format!("Terminal: {}", self.command)),
                    ),
            )
            .child(
                // Terminal content
                div().flex_1().p_2().child(self.terminal_page.clone()),
            )
            .child(
                // Footer
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_end()
                    .p_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .child(
                        Button::new("terminal-done")
                            .label("Done")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                if let Some(on_complete) = this.on_complete.take() {
                                    on_complete(0);
                                }
                                this.close_dialog(window, cx);
                            })),
                    ),
            )
    }
}
