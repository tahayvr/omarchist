use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Root, h_flex, v_flex};

use crate::terminal::session::{TerminalSession, TerminalSize};
use crate::terminal::view::TerminalView;

/// A full-page terminal component
pub struct TerminalPage {
    terminal_view: Entity<TerminalView>,
    command: String,
    show_header: bool,
}

impl TerminalPage {
    /// Create a new terminal page with the given command
    pub fn new(command: String, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Create terminal session
        let size = TerminalSize {
            cols: 80,
            rows: 24,
            cell_width: 8,
            cell_height: 16,
        };

        let session = cx.new(|cx| match TerminalSession::new(&command, size, cx) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("Failed to create terminal session: {}", e);
                panic!("Terminal session creation failed");
            }
        });

        let terminal_view = cx.new(|cx| TerminalView::new(session, cx));

        Self {
            terminal_view,
            command,
            show_header: true,
        }
    }

    /// Create a terminal page without header (for dialogs)
    pub fn new_without_header(
        command: String,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let size = TerminalSize {
            cols: 80,
            rows: 24,
            cell_width: 8,
            cell_height: 16,
        };

        let session = cx.new(|cx| match TerminalSession::new(&command, size, cx) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("Failed to create terminal session: {}", e);
                panic!("Terminal session creation failed");
            }
        });

        let terminal_view = cx.new(|cx| TerminalView::new(session, cx));

        Self {
            terminal_view,
            command,
            show_header: false,
        }
    }

    /// Get the command being run
    pub fn command(&self) -> &str {
        &self.command
    }
}

impl Render for TerminalPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        v_flex()
            .size_full()
            .gap_2()
            .child(h_flex().when(self.show_header, |this: gpui::Div| {
                this.p_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(format!("Running: {}", self.command))
            }))
            .child(div().flex_1().size_full().child(self.terminal_view.clone()))
    }
}

/// Create a terminal page root element
pub fn create_terminal_page(
    command: String,
    window: &mut Window,
    cx: &mut Context<TerminalPage>,
) -> AnyView {
    let page = cx.new(|cx| TerminalPage::new(command, window, cx));
    cx.new(|cx| Root::new(page, window, cx)).into()
}
