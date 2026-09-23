use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize, Sizable,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    label::Label,
    v_flex,
};
use gpui_kit::TestSupportExt;

use crate::system::themes::overrides::palette::{self, BaseKey, PaletteBundle};
use crate::system::themes::theme_management::load_theme_for_editing;
use crate::types::themes::ColorsConfig;
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::dialogs::confirm_dialog::{ConfirmDialog, open_confirm_dialog};
use crate::ui::focus::FocusableSwitch;
use crate::ui::theme_edit_page::override_pane::StatusChanged;
use crate::ui::theme_edit_page::shared::{color_picker_with_clipboard, error_message, help_text};

const SAVE_DELAY: Duration = Duration::from_millis(300);

/// Files generated together from the theme's palette with some colors
/// changed, such as every terminal config.
pub struct PalettePane {
    theme_name: String,
    bundle: &'static PaletteBundle,
    installed: bool,
    colors: ColorsConfig,
    /// `Some` while the bundle is on: the colors changed for it.
    overrides: Option<BTreeMap<String, String>>,
    /// The base keys the bundle's files use; `None` while being worked out.
    keys: Option<Vec<&'static BaseKey>>,
    pickers: HashMap<&'static str, Entity<ColorPickerState>>,
    /// Files the theme ships while the bundle is off, from an older Omarchist.
    old_files: Vec<&'static str>,
    busy: bool,
    edit_generation: u64,
    error: Option<String>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<StatusChanged> for PalettePane {}

impl PalettePane {
    pub fn new(
        theme_name: String,
        bundle: &'static PaletteBundle,
        installed: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut pane = Self {
            theme_name,
            bundle,
            installed,
            colors: ColorsConfig::default(),
            overrides: None,
            keys: None,
            pickers: HashMap::new(),
            old_files: Vec::new(),
            busy: false,
            edit_generation: 0,
            error: None,
            _subscriptions: Vec::new(),
        };
        pane.reload(window, cx);
        pane
    }

    pub fn is_custom(&self) -> bool {
        self.overrides.is_some()
    }

    /// Reads the theme's colors and this bundle's changes from disk again,
    /// so colors that follow the palette show its current values.
    pub fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match load_theme_for_editing(&self.theme_name) {
            Ok(theme) => {
                self.colors = theme.colors;
                self.overrides = theme.palettes.get(self.bundle.id).cloned();
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
        self.old_files = if self.overrides.is_some() {
            Vec::new()
        } else {
            palette::existing_files(&self.theme_name, self.bundle)
        };
        self.load_keys(window, cx);
    }

    fn load_keys(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let bundle = self.bundle;
        let colors = self.colors.clone();
        cx.spawn_in(window, async move |this, cx| {
            let keys = cx
                .background_spawn(async move { palette::relevant_keys(bundle, &colors) })
                .await;
            this.update_in(cx, |this, window, cx| {
                match keys {
                    Ok(keys) => {
                        this.keys = Some(keys);
                        this.build_pickers(window, cx);
                    }
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn value(&self, key: &str) -> String {
        self.overrides
            .as_ref()
            .and_then(|overrides| overrides.get(key).cloned())
            .or_else(|| palette::base_value(&self.colors, key))
            .unwrap_or_default()
    }

    fn build_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self._subscriptions.clear();
        self.pickers.clear();
        let keys = self.keys.clone().unwrap_or_default();
        for base in keys {
            let hex = self.value(base.key);
            let picker = cx.new(|cx| {
                let picker = ColorPickerState::new(window, cx);
                match hex_to_hsla(&hex) {
                    Some(color) => picker.default_value(color),
                    None => picker,
                }
            });
            let key = base.key;
            self._subscriptions.push(cx.subscribe(
                &picker,
                move |this: &mut Self, _, event: &ColorPickerEvent, cx| {
                    if let ColorPickerEvent::Change(Some(color)) = event {
                        let hex = color.to_hex();
                        let hex = hex[..hex.len().min(7)].to_lowercase();
                        if this.value(key).eq_ignore_ascii_case(&hex) {
                            return;
                        }
                        if let Some(overrides) = this.overrides.as_mut() {
                            overrides.insert(key.to_string(), hex);
                            this.schedule_save(cx);
                        }
                    }
                },
            ));
            self.pickers.insert(base.key, picker);
        }
    }

    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        self.edit_generation += 1;
        let generation = self.edit_generation;
        let theme = self.theme_name.clone();
        let bundle = self.bundle;
        let overrides = self.overrides.clone();
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_DELAY).await;
            let current = this.update(cx, |this, _| this.edit_generation == generation);
            if !matches!(current, Ok(true)) {
                return;
            }
            let result = cx
                .background_spawn(async move { palette::set(&theme, bundle, overrides) })
                .await;
            this.update(cx, |this, cx| {
                if this.edit_generation == generation {
                    this.error = result.err().map(|e| e.to_string());
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Makes one color follow the palette again.
    fn reset_key(&mut self, key: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(overrides) = self.overrides.as_mut() else {
            return;
        };
        overrides.remove(key);
        self.build_pickers(window, cx);
        self.schedule_save(cx);
    }

    fn set_enabled(&mut self, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.edit_generation += 1;
        let theme = self.theme_name.clone();
        let bundle = self.bundle;
        let overrides = enabled.then(BTreeMap::new);
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { palette::set(&theme, bundle, overrides) })
                .await;
            this.update_in(cx, |this, window, cx| {
                this.busy = false;
                match result {
                    Ok(()) => {
                        this.reload(window, cx);
                        cx.emit(StatusChanged);
                    }
                    Err(e) => this.error = Some(e.to_string()),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn confirm_disable(&self, window: &mut Window, cx: &mut Context<Self>) {
        let pane = cx.entity().downgrade();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Stop customizing?",
                message: format!(
                    "{} will follow the theme's palette again, and the colors you picked for \
                     it are discarded.",
                    self.bundle.app
                ),
                confirm_label: "Discard",
                danger: true,
            },
            move |window, cx| {
                pane.update(cx, |pane, cx| pane.set_enabled(false, window, cx))
                    .ok();
            },
            window,
            cx,
        );
    }

    fn confirm_remove_old_files(&self, window: &mut Window, cx: &mut Context<Self>) {
        let pane = cx.entity().downgrade();
        open_confirm_dialog(
            ConfirmDialog {
                title: "Remove the old files?",
                message: format!(
                    "This deletes {} so Omarchy generates them from the palette again.",
                    self.old_files.join(", ")
                ),
                confirm_label: "Remove",
                danger: true,
            },
            move |window, cx| {
                pane.update(cx, |pane, cx| pane.set_enabled(false, window, cx))
                    .ok();
            },
            window,
            cx,
        );
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let custom = self.is_custom();
        let badge = |text: &'static str, color: Hsla| {
            div()
                .px_2()
                .py_0p5()
                .rounded(theme.radius)
                .border_1()
                .border_color(color.opacity(0.4))
                .text_xs()
                .text_color(color)
                .child(text)
        };

        h_flex()
            .gap_4()
            .items_start()
            .justify_between()
            .flex_wrap()
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .flex_1()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.bundle.app),
                            )
                            .child(if custom {
                                badge("Custom", theme.primary)
                            } else {
                                badge("Generated", theme.muted_foreground)
                            })
                            .when(!self.installed, |row| {
                                row.child(badge("Not installed", theme.muted_foreground))
                            }),
                    )
                    .child(help_text(self.bundle.description, theme.muted_foreground)),
            )
            .child(
                FocusableSwitch::new(SharedString::from(format!("customize-{}", self.bundle.id)))
                    .label("Customize")
                    .checked(custom)
                    .disabled(self.busy)
                    .on_change(cx.listener(|this, checked: &bool, window, cx| {
                        if *checked {
                            this.set_enabled(true, window, cx);
                        } else {
                            this.confirm_disable(window, cx);
                        }
                    })),
            )
    }

    fn render_colors(&self, cx: &mut Context<Self>) -> AnyElement {
        let muted = cx.theme().muted_foreground;
        let Some(keys) = &self.keys else {
            return help_text("Finding the colors these files use…", muted).into_any_element();
        };
        let overrides = self.overrides.clone().unwrap_or_default();

        let mut groups: Vec<(&'static str, Vec<AnyElement>)> = Vec::new();
        for base in keys {
            let Some(picker) = self.pickers.get(base.key) else {
                continue;
            };
            let id = format!("palette-{}-{}", self.bundle.id, base.key);
            let changed = overrides.contains_key(base.key);
            let key = base.key;
            let field = v_flex()
                .gap_1()
                .child(color_picker_with_clipboard(id.clone(), base.label, picker))
                .child(if changed {
                    Button::new(SharedString::from(format!("{id}-reset")))
                        .label("Reset")
                        .xsmall()
                        .ghost()
                        .cursor_pointer()
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.reset_key(key, window, cx)),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child("From palette")
                        .into_any_element()
                })
                .into_any_element();
            match groups.iter_mut().find(|(name, _)| *name == base.group) {
                Some((_, items)) => items.push(field),
                None => groups.push((base.group, vec![field])),
            }
        }

        v_flex()
            .gap_6()
            .child(help_text(
                "Change any color here and only these files use it. Colors you leave alone \
                 follow the theme's palette.",
                muted,
            ))
            .children(groups.into_iter().map(|(name, items)| {
                v_flex()
                    .gap_3()
                    .child(Label::new(name).text_sm().font_weight(FontWeight::MEDIUM))
                    .child(h_flex().gap_x_12().gap_y_4().flex_wrap().children(items))
            }))
            .into_any_element()
    }
}

impl Render for PalettePane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let warning = cx.theme().warning;
        let custom = self.is_custom();

        v_flex()
            .id(SharedString::from(format!(
                "palette-pane-{}",
                self.bundle.id
            )))
            .test_support()
            .gap_4()
            .min_w_0()
            .child(self.render_header(cx))
            .children(self.error.clone().map(|error| error_message(error, cx)))
            .when(!custom && self.old_files.is_empty(), |pane| {
                pane.child(help_text(
                    "Omarchy generates these from your palette. Turn on Customize to pick \
                     different colors for them.",
                    muted,
                ))
            })
            .when(!custom && !self.old_files.is_empty(), |pane| {
                pane.child(help_text(
                    format!(
                        "This theme ships {} from an older Omarchist, which Omarchy uses instead \
                         of generating them. Turn on Customize to replace them with colors you \
                         pick here, or remove them to follow the palette.",
                        self.old_files.join(", ")
                    ),
                    warning,
                ))
                .child(
                    h_flex().child(
                        Button::new(SharedString::from(format!("remove-old-{}", self.bundle.id)))
                            .label("Remove Old Files")
                            .small()
                            .outline()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.confirm_remove_old_files(window, cx)
                            })),
                    ),
                )
            })
            .when(custom, |pane| pane.child(self.render_colors(cx)))
    }
}
