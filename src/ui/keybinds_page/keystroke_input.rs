// A Zed-style keystroke recorder for a single Hyprland chord.
//
// Ported from Zed's `KeystrokeInput` (crates/keymap_editor/src/ui_components/
// keystroke_input.rs). Two focus handles carry the state: the outer one
// means the widget is focused, the inner one means it is recording. Focus
// on the inner handle installs an app-wide keystroke interceptor that
// stops propagation, so none of the app's own bindings fire while
// recording, and asks Hyprland to switch into the recording submap so its
// binds don't swallow the chord either. Losing inner focus tears both
// down, which makes the recording state impossible to desync.
//
// Unlike Zed, a Hyprland bind is one chord, so recording stops as soon as
// a complete keystroke (modifiers + key) arrives. Escape is therefore
// recordable; the stop button or clicking elsewhere cancels.
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
};

use crate::system::keybinds::chord::{Chord, ModMask};
use crate::system::keybinds::keymap::{keystroke_to_chord, modifiers_to_modmask};
use crate::system::keybinds::submap;
use crate::ui::keybinds_page::chord_chips::{chord_chips, modifier_chips};

actions!(
    keystroke_input,
    [StartRecording, StopRecording, ClearKeystrokes]
);

pub const KEY_CONTEXT: &str = "KeystrokeInput";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeystrokeInputEvent {
    /// A chord was recorded or cleared.
    Changed(Option<Chord>),
    /// Modifiers held while recording changed (search mode filters on them).
    Pending(Option<ModMask>),
    Started,
    Stopped,
}

pub struct KeystrokeInput {
    chord: Option<Chord>,
    pending: Option<ModMask>,
    /// The current binding, shown greyed until a recording replaces it.
    placeholder: Option<Chord>,
    outer_focus: FocusHandle,
    inner_focus: FocusHandle,
    intercept: Option<Subscription>,
    submap_error: Option<String>,
    search_mode: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<KeystrokeInputEvent> for KeystrokeInput {}

impl Focusable for KeystrokeInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.outer_focus.clone()
    }
}

impl KeystrokeInput {
    pub fn new(
        placeholder: Option<Chord>,
        search_mode: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let outer_focus = crate::ui::focus::tab_stop(cx);
        let inner_focus = cx.focus_handle();

        let subscriptions = vec![
            cx.on_focus_in(&inner_focus, window, Self::on_inner_focus_in),
            cx.on_focus_out(&inner_focus, window, Self::on_inner_focus_out),
            cx.observe_window_activation(window, |this, window, cx| {
                if !window.is_window_active() && this.is_recording(window) {
                    this.stop_recording(window, cx);
                }
            }),
        ];

        Self {
            chord: None,
            pending: None,
            placeholder,
            outer_focus,
            inner_focus,
            intercept: None,
            submap_error: None,
            search_mode,
            _subscriptions: subscriptions,
        }
    }

    pub fn chord(&self) -> Option<&Chord> {
        self.chord.as_ref()
    }

    pub fn pending_modifiers(&self) -> Option<ModMask> {
        self.pending
    }

    /// Sets the chord from outside (the manual text field) without emitting.
    pub fn set_chord(&mut self, chord: Option<Chord>, cx: &mut Context<Self>) {
        if self.chord != chord {
            self.chord = chord;
            cx.notify();
        }
    }

    pub fn is_recording(&self, window: &Window) -> bool {
        self.inner_focus.is_focused(window)
    }

    pub fn start_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.chord = None;
        self.submap_error = None;
        let held = window.modifiers();
        self.pending = held.modified().then(|| modifiers_to_modmask(&held));
        self.inner_focus.focus(window, cx);
        cx.notify();
    }

    pub fn stop_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_recording(window) {
            // Focus-out on the inner handle does the teardown.
            self.outer_focus.focus(window, cx);
        }
        cx.notify();
    }

    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_recording(window, cx);
        if self.chord.take().is_some() {
            cx.emit(KeystrokeInputEvent::Changed(None));
        }
        cx.notify();
    }

    fn on_inner_focus_in(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.intercept.is_none() {
            let listener = cx.listener(|this, event: &KeystrokeEvent, window, cx| {
                this.handle_keystroke(&event.keystroke, window, cx);
            });
            self.intercept = Some(cx.intercept_keystrokes(listener));
        }
        if let Err(e) = submap::enter_recording_submap() {
            self.submap_error = Some(format!(
                "Hyprland may still intercept bound keys: {e}. Use the text field below."
            ));
        }
        cx.emit(KeystrokeInputEvent::Started);
        cx.notify();
    }

    fn on_inner_focus_out(
        &mut self,
        _event: FocusOutEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.intercept.take();
        submap::leave_recording_submap();
        if self.pending.take().is_some() && self.search_mode {
            cx.emit(KeystrokeInputEvent::Pending(None));
        }
        cx.emit(KeystrokeInputEvent::Stopped);
        cx.notify();
    }

    fn handle_keystroke(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        if let Some(chord) = keystroke_to_chord(keystroke) {
            self.chord = Some(chord.clone());
            self.pending = None;
            cx.emit(KeystrokeInputEvent::Changed(Some(chord)));
            self.stop_recording(window, cx);
        }
        cx.notify();
    }

    fn on_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        if !self.is_recording(window) {
            return;
        }
        let pending = event
            .modifiers
            .modified()
            .then(|| modifiers_to_modmask(&event.modifiers));
        if pending != self.pending {
            self.pending = pending;
            if self.search_mode {
                cx.emit(KeystrokeInputEvent::Pending(pending));
            }
            cx.notify();
        }
    }

    fn render_status_pill(&self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        let (label, color) = if self.search_mode {
            ("SEARCH", theme.primary)
        } else {
            ("REC", theme.red)
        };
        h_flex()
            .h(px(18.))
            .px_1p5()
            .gap_1()
            .items_center()
            .rounded_sm()
            .border_1()
            .border_color(theme.border)
            .bg(color.opacity(0.1))
            .child(
                div().size(px(8.)).rounded_full().bg(color).with_animation(
                    "keystroke-input-pulse",
                    Animation::new(Duration::from_millis(1600))
                        .repeat()
                        .with_easing(pulsating_between(0.35, 1.0)),
                    move |this, delta| this.bg(color.opacity(delta)),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(color)
                    .child(label),
            )
            .into_any_element()
    }

    fn render_content(&self, recording: bool, cx: &App) -> AnyElement {
        let theme = cx.theme();
        if let Some(chord) = &self.chord {
            return chord_chips(chord, false, cx);
        }
        if recording {
            if let Some(pending) = self.pending {
                return modifier_chips(pending, cx);
            }
            let hint = if self.search_mode {
                "Press keys to search"
            } else {
                "Press the new key combination"
            };
            return div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child(hint)
                .into_any_element();
        }
        if let Some(placeholder) = &self.placeholder {
            return chord_chips(placeholder, true, cx);
        }
        div()
            .text_sm()
            .text_color(theme.muted_foreground)
            .child(if self.search_mode {
                "Click to search by keystroke"
            } else {
                "Click to record a key combination"
            })
            .into_any_element()
    }
}

impl Render for KeystrokeInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let recording = self.is_recording(window);
        let focused = recording || self.outer_focus.is_focused(window);
        let slot_width = px(72.);
        let record_icon = Icon::new(Icon::empty()).path("icons/circle.svg").size_4();
        let stop_icon = Icon::new(Icon::empty()).path("icons/square.svg").size_4();

        div()
            .w_full()
            .child(
                h_flex()
                    .id("keystroke-input")
                    .track_focus(&self.outer_focus)
                    .key_context(KEY_CONTEXT)
                    .on_action(cx.listener(|this, _: &StartRecording, window, cx| {
                        this.start_recording(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &StopRecording, window, cx| {
                        this.stop_recording(window, cx);
                    }))
                    .on_action(cx.listener(|this, _: &ClearKeystrokes, window, cx| {
                        this.clear(window, cx);
                    }))
                    .w_full()
                    .min_h(px(44.))
                    .px_2()
                    .py_1()
                    .gap_2()
                    .items_center()
                    .justify_between()
                    .rounded(theme.radius)
                    .border_1()
                    .border_color(if focused { theme.ring } else { theme.border })
                    .bg(if recording {
                        theme.primary.opacity(0.08)
                    } else {
                        theme.background
                    })
                    .child(
                        h_flex()
                            .w(slot_width)
                            .flex_none()
                            .justify_start()
                            .when(recording, |this| this.child(self.render_status_pill(cx))),
                    )
                    .child(
                        h_flex()
                            .id("keystroke-input-inner")
                            .track_focus(&self.inner_focus)
                            .on_modifiers_changed(cx.listener(Self::on_modifiers_changed))
                            .on_click(cx.listener(|this, _, window, cx| {
                                if !this.is_recording(window) {
                                    this.start_recording(window, cx);
                                }
                            }))
                            .cursor_pointer()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .py_1()
                            .justify_center()
                            .items_center()
                            .child(self.render_content(recording, cx)),
                    )
                    .child(
                        h_flex()
                            .w(slot_width)
                            .flex_none()
                            .gap_1()
                            .justify_end()
                            .map(|this| {
                                if recording {
                                    this.child(
                                        Button::new("keystroke-input-stop")
                                            .ghost()
                                            .xsmall()
                                            .icon(stop_icon)
                                            .tooltip("Stop recording")
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.stop_recording(window, cx);
                                            })),
                                    )
                                } else {
                                    this.child(
                                        Button::new("keystroke-input-record")
                                            .ghost()
                                            .xsmall()
                                            .icon(record_icon)
                                            .tooltip("Record a key combination")
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.start_recording(window, cx);
                                            })),
                                    )
                                    .when(
                                        self.chord.is_some(),
                                        |this| {
                                            this.child(
                                                Button::new("keystroke-input-clear")
                                                    .ghost()
                                                    .xsmall()
                                                    .icon(IconName::Delete)
                                                    .tooltip("Clear")
                                                    .cursor_pointer()
                                                    .on_click(cx.listener(
                                                        |this, _, window, cx| {
                                                            this.clear(window, cx);
                                                        },
                                                    )),
                                            )
                                        },
                                    )
                                }
                            }),
                    ),
            )
            .when_some(self.submap_error.clone(), |this, error| {
                this.child(
                    div()
                        .mt_1()
                        .text_xs()
                        .text_color(theme.warning)
                        .child(error),
                )
            })
    }
}

impl Drop for KeystrokeInput {
    fn drop(&mut self) {
        if self.intercept.is_some() {
            submap::leave_recording_submap();
        }
    }
}
