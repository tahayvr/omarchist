use crate::ui::color_utils::hex6;
use crate::ui::focus::FocusSection;
use crate::ui::text::selectable;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize,
    clipboard::Clipboard,
    color_picker::{ColorPicker, ColorPickerState},
    h_flex,
    input::{Input, InputState},
    label::Label,
    switch::Switch,
    v_flex,
};

pub struct FormField {
    label: String,
    input: Entity<InputState>,
}

impl FormField {
    pub fn new(
        label: &str,
        initial_value: impl Into<String>,
        placeholder: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial_value.into())
                .placeholder(placeholder.into())
        });

        Self {
            label: label.to_string(),
            input,
        }
    }

    pub fn input(&self) -> &Entity<InputState> {
        &self.input
    }

    pub fn value(&self, cx: &App) -> String {
        self.input.read(cx).value().to_string()
    }
}

impl RenderOnce for FormField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                Label::new(&self.label)
                    .text_sm()
                    .text_color(cx.theme().muted_foreground),
            )
            .child(Input::new(&self.input).cleanable(true))
    }
}

type ToggleChangeCallback = Box<dyn Fn(bool, &mut Window, &mut App)>;

pub struct ToggleField {
    id: String,
    label: String,
    is_checked: bool,
    on_change: Option<ToggleChangeCallback>,
}

impl ToggleField {
    pub fn new(id: &str, label: &str, is_checked: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            is_checked,
            on_change: None,
        }
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ToggleField {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let is_checked = self.is_checked;
        let on_change = self.on_change;
        let id: gpui::SharedString = self.id.into();

        h_flex()
            .gap_4()
            .items_center()
            .child(Label::new(&self.label))
            .child(
                Switch::new(id)
                    .checked(is_checked)
                    .cursor_pointer()
                    .on_click(move |checked, window, cx| {
                        if let Some(ref handler) = on_change {
                            handler(*checked, window, cx);
                        }
                    }),
            )
    }
}

pub fn form_section() -> Div {
    v_flex().gap_2()
}

/// A short message in `color`; `id` must be unique among its siblings.
pub fn help_text(id: impl Into<ElementId>, text: impl Into<SharedString>, color: Hsla) -> Div {
    div()
        .text_sm()
        .text_color(color)
        .child(selectable(id, text))
}

pub trait TabInputHandler: Sized {
    fn on_input_change(
        &mut self,
        field_id: &str,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    );

    fn trigger_save(&mut self, window: &mut Window, cx: &mut Context<Self>);
}

/// Wraps a section so the Designer scrolls it into view when keyboard
/// focus enters it.
pub fn focus_section(
    id: impl Into<ElementId>,
    scroll: &ScrollHandle,
    content: impl IntoElement,
) -> FocusSection {
    FocusSection::new(id, scroll).child(content)
}

pub fn tab_container() -> Div {
    v_flex().gap_6().pt_4().pb_4()
}

/// Lays `cells` out in rows of `columns` equal-width cells. The last row is
/// padded with empty cells so every column keeps its width, which is what
/// lines the fields of one row up with the fields of the next.
pub fn field_grid(columns: usize, cells: Vec<AnyElement>) -> Div {
    let columns = columns.max(1);
    let mut cells = cells;
    let remainder = cells.len() % columns;
    if remainder != 0 {
        for _ in remainder..columns {
            cells.push(div().into_any_element());
        }
    }
    let mut rows: Vec<Vec<AnyElement>> = Vec::new();
    for cell in cells {
        match rows.last_mut() {
            Some(row) if row.len() < columns => row.push(cell),
            _ => rows.push(vec![cell]),
        }
    }
    v_flex().gap_4().children(rows.into_iter().map(|row| {
        h_flex().gap_4().items_start().children(
            row.into_iter()
                .map(|cell| div().flex_1().min_w_0().child(cell)),
        )
    }))
}

/// Columns for a field grid across the whole tab.
pub fn tab_grid_columns(window: &Window) -> usize {
    grid_columns(content_width(window))
}

// The width left for a tab's content: the viewport minus the collapsed
// sidebar and the page padding.
fn content_width(window: &Window) -> f32 {
    let width: f32 = window.viewport_size().width.into();
    width - 88.
}

fn grid_columns(available: f32) -> usize {
    ((available / 136.).floor() as usize).clamp(2, 8)
}

pub fn section_title(text: impl Into<SharedString>) -> Div {
    div()
        .text_base()
        .font_weight(FontWeight::SEMIBOLD)
        .child(text.into())
}

/// A field's label, two lines tall so the controls of a grid row line up
/// whether or not a label wraps; the text sits at the bottom, by its control.
pub fn field_label(text: impl Into<SharedString>, trailing: Option<AnyElement>) -> Div {
    div()
        .h(px(40.))
        .w_full()
        .flex()
        .flex_row()
        .items_end()
        .gap_2()
        .child(div().min_w_0().text_sm().line_clamp(2).child(text.into()))
        .children(trailing.map(|element| div().flex_none().child(element)))
}

pub fn color_picker_with_clipboard(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    picker_state: &Entity<ColorPickerState>,
) -> impl IntoElement {
    let picker_state_clone = picker_state.clone();
    let id: SharedString = id.into();
    let clipboard_id: SharedString = format!("{}-clipboard", id).into();
    let hex_id: SharedString = format!("{}-hex", id).into();
    let clipboard = Clipboard::new(clipboard_id).value_fn(move |_, cx| {
        picker_state_clone
            .read(cx)
            .value()
            .map(|c| hex6(&c.to_hex()))
            .unwrap_or_default()
            .into()
    });
    let picker_state_for_hex = picker_state.clone();

    v_flex()
        .gap_2()
        .child(field_label(label, Some(clipboard.into_any_element())))
        .child(ColorPicker::new(picker_state))
        // The value in the open, selectable (Ctrl+C) without opening the
        // picker.
        .child(div().text_xs().child(HexValue {
            id: hex_id,
            state: picker_state_for_hex,
        }))
}

/// The picker's current value as text, read at render time so it follows
/// every change.
#[derive(IntoElement)]
struct HexValue {
    id: SharedString,
    state: Entity<ColorPickerState>,
}

impl RenderOnce for HexValue {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hex = self
            .state
            .read(cx)
            .value()
            .map(|c| hex6(&c.to_hex()))
            .unwrap_or_default();
        div()
            .text_color(cx.theme().muted_foreground)
            .child(selectable(self.id, hex))
    }
}

/// Extensions the image pickers list, in both cases: the portal's filters
/// are case-sensitive globs, and cameras write `.JPG`.
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "PNG", "jpg", "JPG", "jpeg", "JPEG", "webp", "WEBP", "gif", "GIF", "bmp", "BMP",
];
