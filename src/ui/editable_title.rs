//! A heading that is edited where it stands: a click, Enter or Space on
//! the title turns it into its field with the text selected; Enter or
//! leaving the field ends the edit. The flow editor's name and the Theme
//! Designer's name are one of these.
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Sizable,
    input::{Input, InputState},
    tooltip::Tooltip,
};
use gpui_kit::TestSupportExt;

use crate::ui::focus;

/// The title: Enter or Space edits it.
pub const TITLE_CONTEXT: &str = "Title";
/// Around the field: Enter ends the edit.
pub const FIELD_CONTEXT: &str = "TitleField";

pub mod title {
    gpui::actions!(title, [Rename]);
}

/// A view's answer to the edit starting or ending.
pub type TitleHandler = Box<dyn Fn(&mut Window, &mut App)>;

/// What one drawing of the title needs from its view.
pub struct Title<'a> {
    /// The title's element id, and the field's.
    pub id: &'static str,
    pub field_id: &'static str,
    pub text: &'a str,
    /// Shown, muted, while the text is empty.
    pub placeholder: &'static str,
    /// The view's own start and end of the edit, so it can act on the
    /// new text.
    pub on_start: TitleHandler,
    pub on_stop: TitleHandler,
}

/// What a view keeps for its editable title.
pub struct TitleState {
    /// The field is shown in place of the title.
    pub renaming: bool,
    /// The title's tab stop.
    pub focus: FocusHandle,
    pub input: Entity<InputState>,
}

impl TitleState {
    pub fn new(input: Entity<InputState>, cx: &mut App) -> Self {
        Self {
            renaming: false,
            focus: focus::tab_stop(cx),
            input,
        }
    }

    /// Shows the field and, once it is drawn, gives it the keyboard with
    /// the text selected (a handle focused before its element is drawn
    /// loses the focus). Until then `page` has it, so the page's shortcuts
    /// keep working.
    pub fn start(&mut self, page: &FocusHandle, window: &mut Window, cx: &mut App) {
        self.renaming = true;
        page.focus(window, cx);
        let input = self.input.clone();
        window.on_next_frame(move |window, cx| {
            input.update(cx, |input, cx| {
                input.focus(window, cx);
                input.select_all(window, cx);
            });
        });
    }

    /// Shows the title again and puts the keyboard on it.
    pub fn stop(&mut self, page: &FocusHandle, window: &mut Window, cx: &mut App) {
        self.renaming = false;
        page.focus(window, cx);
        let title = self.focus.clone();
        window.on_next_frame(move |window, cx| title.focus(window, cx));
    }

    /// Whether the field losing focus ends the edit: the window going
    /// inactive reads as a blur too, with the field still focused, and
    /// that one is not the end.
    pub fn blur_ends(window: &Window) -> bool {
        window.is_window_active()
    }

    /// The title, or the field while renaming.
    pub fn render(&self, title: Title, window: &Window, cx: &App) -> AnyElement {
        let Title {
            id,
            field_id,
            text,
            placeholder,
            on_start,
            on_stop,
        } = title;
        if self.renaming {
            return div()
                .key_context(FIELD_CONTEXT)
                .on_action(move |_: &title::Rename, window, cx| on_stop(window, cx))
                // As wide as a name, not the row.
                .w(px(320.))
                .max_w_full()
                .child(Input::new(&self.input).id(field_id).small())
                .into_any_element();
        }
        let theme = cx.theme();
        let ring = focus::focus_border(self.focus.is_focused(window), theme.transparent, cx);
        let on_click = std::rc::Rc::new(on_start);
        let on_key = on_click.clone();
        div()
            .id(id)
            .test_support()
            .key_context(TITLE_CONTEXT)
            .track_focus(&self.focus)
            .on_action(move |_: &title::Rename, window, cx| on_key(window, cx))
            .min_w_0()
            .px_1p5()
            .py_0p5()
            .rounded(theme.radius)
            .border_1()
            .border_color(ring)
            .font_weight(FontWeight::SEMIBOLD)
            .truncate()
            .when(text.is_empty(), |this| {
                this.text_color(theme.muted_foreground)
            })
            .hover(|this| this.bg(theme.secondary))
            .cursor_pointer()
            .tooltip(|window, cx| Tooltip::new("Rename").build(window, cx))
            .child(if text.is_empty() {
                placeholder.to_string()
            } else {
                text.to_string()
            })
            .on_click(move |_, window, cx| on_click(window, cx))
            .into_any_element()
    }
}
