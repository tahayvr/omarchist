use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::scroll::ScrollableElement;

use crate::terminal::session::{TerminalSession, TerminalSize};

/// GPUI component that renders a terminal session
pub struct TerminalView {
    session: Entity<TerminalSession>,

    /// Current scroll offset (for scrollback)
    #[allow(dead_code)]
    scroll_offset: usize,

    /// Whether terminal has focus
    focused: bool,

    /// Font settings
    font_family: SharedString,
    #[allow(dead_code)]
    font_size: Pixels,

    /// Cell dimensions (calculated from font)
    cell_width: Pixels,
    cell_height: Pixels,

    /// Blink state for cursor
    cursor_visible: bool,
}

impl TerminalView {
    pub fn new(session: Entity<TerminalSession>, cx: &mut Context<Self>) -> Self {
        // Set up cursor blink timer
        cx.spawn(async move |this, cx| {
            loop {
                smol::Timer::after(std::time::Duration::from_millis(530)).await;
                this.update(cx, |this, _cx| {
                    this.cursor_visible = !this.cursor_visible;
                })
                .ok();
            }
        })
        .detach();

        Self {
            session,
            scroll_offset: 0,
            focused: false,
            font_family: SharedString::from("JetBrains Mono"),
            font_size: px(14.0),
            cell_width: px(8.0),
            cell_height: px(18.0),
            cursor_visible: true,
        }
    }

    /// Send input to the terminal
    fn send_input(&self, data: Vec<u8>, cx: &mut Context<Self>) {
        self.session.update(cx, |session, _cx| {
            session.input(&data);
        });
    }

    /// Handle key down events
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(input) = convert_key_event(event) {
            self.send_input(input, cx);
        }
    }

    /// Calculate cell dimensions based on font
    fn calculate_cell_size(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.cell_width = px(8.0);
        self.cell_height = px(16.0);

        // Update session size based on available space
        let terminal_size = TerminalSize {
            cols: 80,
            rows: 24,
            cell_width: 8,
            cell_height: 16,
        };

        self.session.update(cx, |session, cx| {
            session.resize(terminal_size, cx);
        });
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Calculate cell dimensions first
        self.calculate_cell_size(window, cx);

        // Read the session to get the command info
        let session_ref = self.session.read(cx);
        let command = session_ref.command.clone();
        let is_running = session_ref.is_running();

        // Get theme values
        let bg_color = cx.theme().background;
        let font_family = self.font_family.clone();

        // Clone session for the child closure

        div()
            .id("terminal-view")
            .size_full()
            .font_family(font_family)
            .bg(bg_color)
            .flex()
            .flex_col()
            .overflow_hidden()
            .on_key_down(cx.listener(|this, event, window, cx| {
                this.handle_key_down(event, window, cx);
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.focused = true;
                    cx.notify();
                }),
            )
            .child(
                // Terminal content area
                div().flex_1().overflow_y_scrollbar().child(
                    div()
                        .flex()
                        .flex_col()
                        .p_4()
                        .child("Terminal output...")
                        .child(format!("Command: {}", command))
                        .child(format!("Running: {}", is_running)),
                ),
            )
    }
}

/// Convert GPUI key events to terminal input sequences
fn convert_key_event(event: &KeyDownEvent) -> Option<Vec<u8>> {
    // Handle modifiers
    let mods = event.keystroke.modifiers;
    let has_ctrl = mods.control;
    let has_alt = mods.alt;

    // Get the key string representation
    let key_str = event.keystroke.key.to_string();

    match key_str.as_str() {
        // Special keys
        "return" | "enter" => Some(vec![b'\r']),
        "escape" | "esc" => Some(vec![0x1b]),
        "backspace" => Some(vec![0x7f]),
        "tab" => Some(vec![b'\t']),
        "up" => Some(vec![0x1b, b'[', b'A']),
        "down" => Some(vec![0x1b, b'[', b'B']),
        "right" => Some(vec![0x1b, b'[', b'C']),
        "left" => Some(vec![0x1b, b'[', b'D']),
        "home" => Some(vec![0x1b, b'[', b'H']),
        "end" => Some(vec![0x1b, b'[', b'F']),
        "pageup" => Some(vec![0x1b, b'[', b'5', b'~']),
        "pagedown" => Some(vec![0x1b, b'[', b'6', b'~']),
        "delete" => Some(vec![0x1b, b'[', b'3', b'~']),
        "space" => Some(vec![b' ']),

        // Function keys
        "f1" => Some(vec![0x1b, b'O', b'P']),
        "f2" => Some(vec![0x1b, b'O', b'Q']),
        "f3" => Some(vec![0x1b, b'O', b'R']),
        "f4" => Some(vec![0x1b, b'O', b'S']),
        "f5" => Some(vec![0x1b, b'[', b'1', b'5', b'~']),
        "f6" => Some(vec![0x1b, b'[', b'1', b'7', b'~']),
        "f7" => Some(vec![0x1b, b'[', b'1', b'8', b'~']),
        "f8" => Some(vec![0x1b, b'[', b'1', b'9', b'~']),
        "f9" => Some(vec![0x1b, b'[', b'2', b'0', b'~']),
        "f10" => Some(vec![0x1b, b'[', b'2', b'1', b'~']),
        "f11" => Some(vec![0x1b, b'[', b'2', b'3', b'~']),
        "f12" => Some(vec![0x1b, b'[', b'2', b'4', b'~']),

        // Regular characters
        key if key.len() == 1 => {
            let c = key.chars().next()?;

            if has_ctrl {
                // Ctrl + letter: send control character
                if c.is_ascii_lowercase() {
                    Some(vec![(c as u8) - b'a' + 1])
                } else if c.is_ascii_uppercase() {
                    Some(vec![(c as u8) - b'A' + 1])
                } else {
                    Some(vec![c as u8])
                }
            } else if has_alt {
                // Alt/Meta: prefix with ESC
                Some(vec![0x1b, c as u8])
            } else {
                Some(vec![c as u8])
            }
        }

        _ => None,
    }
}
