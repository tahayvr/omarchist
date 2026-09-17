// The edit / create dialog for one keybind.
//
// The recorder is the primary way to pick keys; a plain text field in
// Omarchy syntax (`SUPER + mouse:272`) is kept in sync with it for chords
// that cannot be typed into the app (mouse buttons, media keys the
// compositor keeps, `code:N`). Conflicts are shown live and, like Zed,
// saving onto an occupied chord asks for a second press of Save.
//
// The dialog only validates and emits; the page owns the overrides and
// writes them (`KeybindsView::handle_dialog_event`).
use std::rc::Rc;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::Disableable;
use gpui_component::{
    ActiveTheme, Icon, IconName, Sizable, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::system::keybinds::chord::Chord;
use crate::system::keybinds::conflicts::binds_on_chord;
use crate::system::keybinds::overrides::{
    BindSpec, Override, restore_specs, unrestorable_siblings,
};
use crate::system::keybinds::{Dispatcher, Keybind, Origin};
use crate::ui::focus;
use crate::ui::keybinds_page::action_builder::{ActionBuilder, ActionBuilderEvent};
use crate::ui::keybinds_page::keybinds_table::{KeybindRow, RowKind};
use crate::ui::keybinds_page::keystroke_input::{KeystrokeInput, KeystrokeInputEvent};

pub enum DialogMode {
    Edit {
        row: Box<KeybindRow>,
        /// The override that produced or affects the row, if any.
        existing: Option<Override>,
    },
    Add,
    /// A new bind whose action is already chosen (a flow assigning its keybind).
    AddPreset {
        dispatcher: Dispatcher,
        description: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeybindDialogEvent {
    Save(Override),
    Disable,
    Reset,
    Cancel,
}

pub struct KeybindDialog {
    mode: DialogMode,
    binds: Rc<Vec<Keybind>>,
    recorder: Entity<KeystrokeInput>,
    keys_text: Entity<InputState>,
    description: Entity<InputState>,
    /// Absent only for Lua-function binds, which cannot be re-bound.
    builder: Option<Entity<ActionBuilder>>,
    /// The user typed a description, so the builder stops suggesting one.
    description_edited: bool,
    chord: Option<Chord>,
    chord_error: Option<String>,
    syncing: bool,
    conflicts: Vec<String>,
    lost_siblings: Vec<String>,
    confirm_pending: bool,
    error: Option<String>,
    /// Focus scope of the dialog body.
    body_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<KeybindDialogEvent> for KeybindDialog {}

impl KeybindDialog {
    pub fn new(
        mode: DialogMode,
        binds: Rc<Vec<Keybind>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let original = match &mode {
            DialogMode::Edit { row, .. } => Some(&row.bind),
            DialogMode::Add | DialogMode::AddPreset { .. } => None,
        };
        let chord = original.map(|b| b.chord.clone());
        let keys_value = chord
            .as_ref()
            .map(Chord::to_omarchy_string)
            .unwrap_or_default();
        let (preset_dispatcher, preset_description) = match &mode {
            DialogMode::AddPreset {
                dispatcher,
                description,
            } => (Some(dispatcher), description.clone()),
            _ => (None, String::new()),
        };
        let description_value = original
            .map(|b| b.description.clone())
            .unwrap_or(preset_description);
        let builder = match original.map(|b| &b.dispatcher).or(preset_dispatcher) {
            Some(Dispatcher::Function) => None,
            dispatcher => Some(cx.new(|cx| ActionBuilder::new(dispatcher, window, cx))),
        };

        let recorder = cx.new(|cx| KeystrokeInput::new(chord.clone(), false, window, cx));
        let keys_text = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("SUPER + SHIFT + K")
                .default_value(keys_value)
        });
        let description = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("What this keybind does")
                .default_value(description_value.clone())
        });

        let mut subscriptions = vec![
            cx.subscribe_in(
                &recorder,
                window,
                |this, _, event: &KeystrokeInputEvent, window, cx| {
                    if let KeystrokeInputEvent::Changed(chord) = event {
                        this.chord = chord.clone();
                        this.chord_error = None;
                        this.sync_keys_text(window, cx);
                        this.recompute(cx);
                    }
                },
            ),
            cx.subscribe_in(
                &keys_text,
                window,
                |this, input, event: &InputEvent, _window, cx| {
                    if !matches!(event, InputEvent::Change) || this.syncing {
                        return;
                    }
                    let text = input.read(cx).value().to_string();
                    this.set_chord_from_text(&text, cx);
                },
            ),
            cx.subscribe_in(
                &description,
                window,
                |this, _, event: &InputEvent, _window, cx| {
                    if matches!(event, InputEvent::Change) {
                        if !this.syncing {
                            this.description_edited = true;
                        }
                        this.recompute(cx);
                    }
                },
            ),
        ];
        if let Some(builder) = &builder {
            subscriptions.push(cx.subscribe_in(
                builder,
                window,
                |this, builder, event: &ActionBuilderEvent, window, cx| {
                    let ActionBuilderEvent::Changed = event;
                    this.suggest_description(builder, window, cx);
                    this.recompute(cx);
                },
            ));
        }

        let mut dialog = Self {
            mode,
            binds,
            recorder,
            keys_text,
            description,
            builder,
            description_edited: !description_value.is_empty(),
            chord,
            chord_error: None,
            syncing: false,
            conflicts: Vec::new(),
            lost_siblings: Vec::new(),
            confirm_pending: false,
            error: None,
            body_focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        };
        dialog.recompute(cx);
        dialog
    }

    fn original(&self) -> Option<&Keybind> {
        match &self.mode {
            DialogMode::Edit { row, .. } => Some(&row.bind),
            DialogMode::Add | DialogMode::AddPreset { .. } => None,
        }
    }

    fn row(&self) -> Option<&KeybindRow> {
        match &self.mode {
            DialogMode::Edit { row, .. } => Some(row.as_ref()),
            DialogMode::Add | DialogMode::AddPreset { .. } => None,
        }
    }

    fn is_rebindable(&self) -> bool {
        self.original().is_none_or(Keybind::is_rebindable)
    }

    fn sync_keys_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self
            .chord
            .as_ref()
            .map(Chord::to_omarchy_string)
            .unwrap_or_default();
        self.syncing = true;
        self.keys_text
            .update(cx, |input, cx| input.set_value(text, window, cx));
        self.syncing = false;
    }

    /// Fills the description from the chosen action until the user types one.
    fn suggest_description(
        &mut self,
        builder: &Entity<ActionBuilder>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.description_edited
            || !matches!(self.mode, DialogMode::Add | DialogMode::AddPreset { .. })
        {
            return;
        }
        let Some(suggestion) = builder.read(cx).suggested_description(cx) else {
            return;
        };
        self.syncing = true;
        self.description
            .update(cx, |input, cx| input.set_value(suggestion, window, cx));
        self.syncing = false;
    }

    fn set_chord_from_text(&mut self, text: &str, cx: &mut Context<Self>) {
        if text.trim().is_empty() {
            self.chord = None;
            self.chord_error = None;
        } else {
            match Chord::parse(text) {
                Ok(chord) => {
                    self.chord = Some(chord);
                    self.chord_error = None;
                }
                Err(e) => {
                    self.chord = None;
                    self.chord_error = Some(e.to_string());
                }
            }
        }
        let chord = self.chord.clone();
        self.recorder
            .update(cx, |recorder, cx| recorder.set_chord(chord, cx));
        self.recompute(cx);
    }

    /// Refreshes the conflict notices; any edit also clears a pending
    /// "save anyway" confirmation.
    fn recompute(&mut self, cx: &mut Context<Self>) {
        self.confirm_pending = false;
        self.error = None;

        let exclude = self.original().map(Keybind::identity);
        let release = self.original().is_some_and(|b| b.options.release);
        self.conflicts = match &self.chord {
            Some(chord) => binds_on_chord(&self.binds, chord, release, exclude.as_ref())
                .into_iter()
                .filter(|b| {
                    b.origin != Origin::Omarchist
                        || self.original().is_none_or(|o| o.identity() != b.identity())
                })
                .map(|b| b.label().to_string())
                .collect(),
            None => Vec::new(),
        };

        self.lost_siblings = match self.original() {
            Some(original) if original.origin != Origin::Omarchist => {
                unrestorable_siblings(original, &self.binds)
                    .into_iter()
                    .map(|b| b.label().to_string())
                    .collect()
            }
            _ => Vec::new(),
        };
        cx.notify();
    }

    fn build_override(&self, cx: &App) -> Result<Override, String> {
        let Some(chord) = &self.chord else {
            return Err(self
                .chord_error
                .clone()
                .unwrap_or_else(|| "Record or type a key combination".to_string()));
        };
        let description = self.description.read(cx).value().trim().to_string();
        let dispatcher = match &self.builder {
            Some(builder) => builder.read(cx).dispatcher(cx)?,
            None => return Err("This keybind runs a Lua function and cannot be re-bound".into()),
        };
        if self.original().is_none() && description.is_empty() {
            return Err("Enter a description so you can find the keybind later".into());
        }

        let bind = BindSpec {
            keys: chord.to_omarchy_string(),
            description,
            dispatcher,
            options: self.original().map(|b| b.options).unwrap_or_default(),
        };

        let override_ = match &self.mode {
            DialogMode::Add | DialogMode::AddPreset { .. } => Override::Add { bind },
            DialogMode::Edit { existing, row } => match existing {
                Some(Override::Rebind {
                    target, restore, ..
                }) => Override::Rebind {
                    target: target.clone(),
                    bind,
                    restore: restore.clone(),
                },
                Some(Override::Add { .. }) => Override::Add { bind },
                _ => Override::Rebind {
                    target: row.bind.identity(),
                    bind,
                    restore: restore_specs(&row.bind, &self.binds),
                },
            },
        };
        Ok(override_)
    }

    fn on_save(&mut self, cx: &mut Context<Self>) {
        match self.build_override(cx) {
            Err(message) => {
                self.error = Some(message);
                cx.notify();
            }
            Ok(override_) => {
                if !self.conflicts.is_empty() && !self.confirm_pending {
                    self.confirm_pending = true;
                    cx.notify();
                    return;
                }
                cx.emit(KeybindDialogEvent::Save(override_));
            }
        }
    }

    fn render_field(
        &self,
        label: &'static str,
        hint: Option<String>,
        content: AnyElement,
        cx: &App,
    ) -> AnyElement {
        let theme = cx.theme();
        v_flex()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted_foreground)
                    .child(label),
            )
            .child(content)
            .when_some(hint, |this, hint| {
                this.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(hint),
                )
            })
            .into_any_element()
    }

    fn render_notice(&self, text: String, color: Hsla, cx: &App) -> AnyElement {
        let theme = cx.theme();
        h_flex()
            .gap_2()
            .items_start()
            .px_3()
            .py_2()
            .rounded(theme.radius)
            .border_1()
            .border_color(color.opacity(0.4))
            .bg(color.opacity(0.08))
            .text_sm()
            .child(
                Icon::new(IconName::TriangleAlert)
                    .size_4()
                    .flex_shrink_0()
                    .text_color(color),
            )
            .child(div().min_w_0().child(text))
            .into_any_element()
    }

    fn render_action(&self, cx: &App) -> AnyElement {
        let theme = cx.theme();
        match &self.builder {
            Some(builder) => self.render_field(
                "Action",
                None,
                builder.clone().into_any_element(),
                cx,
            ),
            None => self.render_field(
                "Action",
                None,
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Runs a Lua function from Omarchy's config. Its keys cannot be changed here, but the keybind can be disabled.")
                    .into_any_element(),
                cx,
            ),
        }
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let row = self.row();
        let has_override = row.is_some_and(|r| r.override_ix.is_some());
        let can_disable = row.is_some_and(|r| {
            r.bind.origin != Origin::Omarchist && matches!(r.kind, RowKind::Plain)
        });
        let save_label = if self.confirm_pending {
            "Save anyway"
        } else {
            "Save"
        };

        h_flex()
            .justify_between()
            .items_center()
            .pt_2()
            .border_t_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .gap_2()
                    .when(has_override, |this| {
                        this.child(
                            Button::new("kb-dialog-reset")
                                .ghost()
                                .small()
                                .icon(Icon::new(Icon::empty()).path("icons/rotate-ccw.svg"))
                                .label("Reset to default")
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.emit(KeybindDialogEvent::Reset);
                                })),
                        )
                    })
                    .when(can_disable, |this| {
                        this.child(
                            Button::new("kb-dialog-disable")
                                .ghost()
                                .small()
                                .icon(Icon::new(Icon::empty()).path("icons/ban.svg"))
                                .label("Disable")
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.emit(KeybindDialogEvent::Disable);
                                })),
                        )
                    }),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("kb-dialog-cancel")
                            .outline()
                            .small()
                            .label("Cancel")
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, cx| {
                                cx.emit(KeybindDialogEvent::Cancel);
                            })),
                    )
                    .child(
                        Button::new("kb-dialog-save")
                            .primary()
                            .small()
                            .label(save_label)
                            .disabled(!self.is_rebindable() || self.chord_error.is_some())
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.on_save(cx))),
                    ),
            )
            .into_any_element()
    }
}

impl Render for KeybindDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let rebindable = self.is_rebindable();
        let view = cx.entity();

        focus::dialog_body("keybind-dialog", &self.body_focus, move |_, cx| {
            view.update(cx, |this, cx| this.on_save(cx));
        })
        .child(
        v_flex()
            .gap_4()
            .when(rebindable, |this| {
                this.child(self.render_field(
                    "Keys",
                    None,
                    self.recorder.clone().into_any_element(),
                    cx,
                ))
                .child(self.render_field(
                    "Or type the keys",
                    Some(
                        "Omarchy syntax: modifiers SUPER, SHIFT, CTRL, ALT joined with +, then a key name such as K, RETURN, comma, F9, XF86AudioMute, mouse:272 or code:10"
                            .to_string(),
                    ),
                    Input::new(&self.keys_text).into_any_element(),
                    cx,
                ))
            })
            .when(!rebindable, |this| {
                this.child(self.render_field(
                    "Keys",
                    None,
                    self.recorder.clone().into_any_element(),
                    cx,
                ))
            })
            .when_some(self.chord_error.clone(), |this, error| {
                this.child(div().text_xs().text_color(theme.danger).child(error))
            })
            .child(self.render_field(
                "Description",
                None,
                Input::new(&self.description).into_any_element(),
                cx,
            ))
            .child(self.render_action(cx))
            .when(!self.conflicts.is_empty(), |this| {
                let list = self.conflicts.join(", ");
                let text = format!(
                    "These keys are already used by: {list}. Hyprland runs every keybind on a chord, so all of them would fire.{}",
                    if self.confirm_pending {
                        " Press Save again to keep both."
                    } else {
                        ""
                    }
                );
                this.child(self.render_notice(text, theme.warning, cx))
            })
            .when(!self.lost_siblings.is_empty(), |this| {
                let list = self.lost_siblings.join(", ");
                let text = format!(
                    "Changing this keybind also removes {list} from the same keys, and those run Lua functions that cannot be restored."
                );
                this.child(self.render_notice(text, theme.warning, cx))
            })
            .when_some(self.error.clone(), |this, error| {
                this.child(self.render_notice(error, theme.danger, cx))
            })
            .child(self.render_footer(cx)),
        )
    }
}

/// Opens the dialog and returns its entity so the caller can subscribe.
pub fn open_keybind_dialog(
    mode: DialogMode,
    binds: Rc<Vec<Keybind>>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<KeybindDialog> {
    let title = match &mode {
        DialogMode::Edit { .. } => "Edit keybind",
        DialogMode::Add | DialogMode::AddPreset { .. } => "New keybind",
    };
    let dialog = cx.new(|cx| KeybindDialog::new(mode, binds, window, cx));
    let view = dialog.clone();
    let body_focus = dialog.read(cx).body_focus.clone();
    window.open_dialog(cx, move |d, _, _| {
        let on_close_view = view.clone();
        d.title(title)
            .w(px(640.))
            .overlay(true)
            .keyboard(true)
            .close_button(true)
            .overlay_closable(false)
            .on_close(move |_, _, cx| {
                on_close_view.update(cx, |_, cx| cx.emit(KeybindDialogEvent::Cancel));
            })
            .child(view.clone())
    });
    // The first tab stop inside the body is the recorder.
    focus::focus_first_in(&body_focus, window, cx);
    dialog
}
