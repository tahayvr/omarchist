use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Colorize, Disableable, Sizable,
    button::{Button, ButtonVariants},
    color_picker::{ColorPickerEvent, ColorPickerState},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::system::themes::overrides::OverrideSpec;
use crate::system::themes::overrides::color_map::{self, ColorEntry, ColorUse};
use crate::ui::color_utils::hex_to_hsla;
use crate::ui::theme_edit_page::override_editors::ContentChanged;
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, field_grid, field_label, group_title, help_text, pane_grid_columns,
};

/// Above this many entries a picker per entry is too many to use, and only
/// the by-color view is offered.
const MAX_KEYED_ENTRIES: usize = 150;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    ByKey,
    ByColor,
}

/// A picker for every hex color in the file, either one per entry or one per
/// distinct color (changing it recolors every entry that uses it).
pub struct ColorMapForm {
    file: &'static str,
    content: String,
    entries: Vec<ColorEntry>,
    /// Fixed while the by-color view is shown, so pickers keep their meaning
    /// when two colors become equal.
    uses: Vec<ColorUse>,
    mode: Mode,
    pickers: Vec<Entity<ColorPickerState>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ContentChanged> for ColorMapForm {}

impl ColorMapForm {
    pub fn new(
        spec: &'static OverrideSpec,
        content: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let entries = color_map::scan(content);
        let mode = if entries.len() <= MAX_KEYED_ENTRIES {
            Mode::ByKey
        } else {
            Mode::ByColor
        };
        let mut form = Self {
            file: spec.file,
            content: content.to_string(),
            uses: color_map::uses(&entries),
            entries,
            mode,
            pickers: Vec::new(),
            _subscriptions: Vec::new(),
        };
        form.build_pickers(window, cx);
        form
    }

    fn build_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self._subscriptions.clear();
        let values: Vec<String> = match self.mode {
            Mode::ByKey => self.entries.iter().map(|e| e.value.clone()).collect(),
            Mode::ByColor => self.uses.iter().map(|u| u.value.clone()).collect(),
        };
        self.pickers = values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let picker = cx.new(|cx| {
                    let picker = ColorPickerState::new(window, cx);
                    match hex_to_hsla(&value[..7]) {
                        Some(color) => picker.default_value(color),
                        None => picker,
                    }
                });
                self._subscriptions.push(cx.subscribe(
                    &picker,
                    move |this: &mut Self, _, event: &ColorPickerEvent, cx| {
                        if let ColorPickerEvent::Change(Some(color)) = event {
                            this.set(index, &color.to_hex(), cx);
                        }
                    },
                ));
                picker
            })
            .collect();
    }

    fn set_mode(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.uses = color_map::uses(&self.entries);
        self.build_pickers(window, cx);
        cx.notify();
    }

    fn set(&mut self, index: usize, hex: &str, cx: &mut Context<Self>) {
        let ranges: Vec<_> = match self.mode {
            Mode::ByKey => self.entries.get(index).map(|e| vec![e.range.clone()]),
            Mode::ByColor => self.uses.get(index).map(|u| {
                u.entries
                    .iter()
                    .map(|&i| self.entries[i].range.clone())
                    .collect()
            }),
        }
        .unwrap_or_default();
        if ranges.is_empty() {
            return;
        }
        self.content = color_map::replace(&self.content, &ranges, hex);
        self.entries = color_map::scan(&self.content);
        if let Some(color_use) = self
            .uses
            .get_mut(index)
            .filter(|_| self.mode == Mode::ByColor)
        {
            color_use.value = hex[..hex.len().min(7)].to_ascii_lowercase();
        }
        cx.emit(ContentChanged(self.content.clone()));
    }

    fn render_by_key(&self, columns: usize, cx: &App) -> Vec<AnyElement> {
        let mut groups: Vec<(&str, Vec<usize>)> = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            match groups.iter_mut().find(|(name, _)| *name == entry.group) {
                Some((_, items)) => items.push(index),
                None => groups.push((&entry.group, vec![index])),
            }
        }

        groups
            .into_iter()
            .map(|(group, indices)| {
                // A name every entry shares, like a Neovim plugin's, belongs
                // in the heading rather than on each label.
                let first = &self.entries[indices[0]].name;
                let shared =
                    first.is_some() && indices.iter().all(|&i| &self.entries[i].name == first);
                let heading = match (shared.then_some(first.as_deref()).flatten(), group) {
                    (Some(name), "") => name.to_string(),
                    (Some(name), group) => format!("{name} › {group}"),
                    (None, "") => "General".to_string(),
                    (None, group) => group.to_string(),
                };
                let items = indices.into_iter().filter_map(|index| {
                    let entry = &self.entries[index];
                    let picker = self.pickers.get(index)?;
                    let label = if shared && !entry.key.is_empty() {
                        words(&entry.key)
                    } else {
                        words(&entry.label())
                    };
                    let id = format!("color-map-{}-{index}", self.file);
                    Some(color_picker_with_clipboard(id, label, picker).into_any_element())
                });
                v_flex()
                    .gap_1()
                    .child(group_title(heading, cx))
                    .child(field_grid(columns, items.collect()))
                    .into_any_element()
            })
            .collect()
    }

    fn render_by_color(&self, columns: usize) -> Vec<AnyElement> {
        let items =
            self.uses
                .iter()
                .zip(&self.pickers)
                .enumerate()
                .map(|(index, (color_use, picker))| {
                    let count = color_use.entries.len();
                    let label = format!(
                        "{} · {count} {}",
                        color_use.value,
                        if count == 1 { "use" } else { "uses" }
                    );
                    color_picker_with_clipboard(
                        format!("color-use-{}-{index}", self.file),
                        label,
                        picker,
                    )
                    .into_any_element()
                });
        vec![field_grid(columns, items.collect()).into_any_element()]
    }
}

impl Render for ColorMapForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        if self.entries.is_empty() {
            return v_flex().child(help_text("no-colors", "No colors to pick.", muted));
        }

        let columns = pane_grid_columns(window);
        let keyed_allowed = self.entries.len() <= MAX_KEYED_ENTRIES;
        let mode_button = |id: &'static str, label: &'static str, mode: Mode, this: &Self| {
            Button::new(id)
                .label(label)
                .small()
                .map(|b| {
                    if this.mode == mode {
                        b.primary()
                    } else {
                        b.ghost()
                    }
                })
                .cursor_pointer()
        };

        v_flex()
            .gap_6()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        mode_button("color-map-by-key", "By Key", Mode::ByKey, self)
                            .disabled(!keyed_allowed)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.set_mode(Mode::ByKey, window, cx)
                            })),
                    )
                    .child(
                        mode_button("color-map-by-color", "By Color", Mode::ByColor, self)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.set_mode(Mode::ByColor, window, cx)
                            })),
                    ),
            )
            .children(match self.mode {
                Mode::ByKey => self.render_by_key(columns, cx),
                Mode::ByColor => self.render_by_color(columns),
            })
    }
}

// MARK: Neovim plugin

/// `neovim.lua` as a colorscheme plugin for LazyVim: the plugin's repository
/// and the colorscheme name it provides.
pub struct NeovimPluginForm {
    repo: Entity<InputState>,
    colorscheme: Entity<InputState>,
    /// The file is a color map, not a plugin spec: the fields are empty and
    /// nothing is written until the user asks for a plugin.
    empty: bool,
    _subscriptions: Vec<Subscription>,
}

/// The plugin the "Use a plugin" button fills in.
const DEFAULT_PLUGIN: (&str, &str) = ("tahayvr/sunset-drive.nvim", "sunsetdrive");

/// The colorscheme plugin of Omarchy's own generated `neovim.lua`.
const OMARCHY_PLUGIN: &str = "bjarneo/aether.nvim";

/// Whether `neovim.lua` names a plugin of the user's own (rather than being
/// Omarchy's generated color map), so the pane opens on the Plugin view.
pub fn is_user_plugin_spec(content: &str) -> bool {
    let (repo, _) = parse_plugin(content);
    !repo.is_empty() && repo != OMARCHY_PLUGIN
}

impl EventEmitter<ContentChanged> for NeovimPluginForm {}

impl NeovimPluginForm {
    pub fn new(content: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let empty = !is_user_plugin_spec(content);
        let (repo, colorscheme) = if empty {
            (String::new(), String::new())
        } else {
            parse_plugin(content)
        };
        let repo = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_PLUGIN.0)
                .default_value(repo)
        });
        let colorscheme = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_PLUGIN.1)
                .default_value(colorscheme)
        });
        let on_change = |this: &mut Self, _, event: &InputEvent, cx: &mut Context<Self>| {
            if let InputEvent::Change = event {
                let repo = this.repo.read(cx).value().trim().to_string();
                let colorscheme = this.colorscheme.read(cx).value().trim().to_string();
                if let Some(content) = plugin_lua(&repo, &colorscheme) {
                    cx.emit(ContentChanged(content));
                }
            }
        };
        let subscriptions = vec![
            cx.subscribe(&repo, on_change),
            cx.subscribe(&colorscheme, on_change),
        ];
        Self {
            repo,
            colorscheme,
            empty,
            _subscriptions: subscriptions,
        }
    }

    /// Replaces the color map with the default plugin spec. The only way a
    /// plugin spec is written without the user typing one, so looking at
    /// this view never changes the file.
    fn use_default_plugin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.repo.update(cx, |input, cx| {
            input.set_value(DEFAULT_PLUGIN.0, window, cx);
        });
        self.colorscheme.update(cx, |input, cx| {
            input.set_value(DEFAULT_PLUGIN.1, window, cx);
        });
        self.empty = false;
        if let Some(content) = plugin_lua(DEFAULT_PLUGIN.0, DEFAULT_PLUGIN.1) {
            cx.emit(ContentChanged(content));
        }
        cx.notify();
    }
}

fn is_plugin_repo(repo: &str) -> bool {
    let allowed = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    };
    repo.split_once('/')
        .is_some_and(|(owner, name)| allowed(owner) && allowed(name))
}

fn is_colorscheme_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// The plugin and colorscheme of a LazyVim plugin spec, as Omarchy's own
/// themes write it.
pub fn parse_plugin(content: &str) -> (String, String) {
    let quoted = |text: &str| -> Vec<String> {
        text.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect()
    };
    let repo = quoted(content)
        .into_iter()
        .find(|s| is_plugin_repo(s) && s != "LazyVim/LazyVim")
        .unwrap_or_default();
    let colorscheme = content
        .lines()
        .find(|line| line.trim_start().starts_with("colorscheme"))
        .and_then(|line| quoted(line).into_iter().next())
        .unwrap_or_default();
    (repo, colorscheme)
}

/// The `neovim.lua` for a plugin and colorscheme, or `None` while either is
/// not a plain name that is safe to write into Lua.
pub fn plugin_lua(repo: &str, colorscheme: &str) -> Option<String> {
    (is_plugin_repo(repo) && is_colorscheme_name(colorscheme)).then(|| {
        format!(
            "return {{\n  {{\n    \"{repo}\",\n    priority = 1000,\n  }},\n  {{\n    \
             \"LazyVim/LazyVim\",\n    opts = {{\n      colorscheme = \"{colorscheme}\",\n    \
             }},\n  }},\n}}\n"
        )
    })
}

impl Render for NeovimPluginForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let input = |label: &'static str, state: &Entity<InputState>| {
            v_flex()
                .gap_2()
                .child(field_label(label, None))
                .child(Input::new(state))
                .into_any_element()
        };
        v_flex()
            .gap_4()
            .when(self.empty, |this| {
                this.child(
                    Button::new("use-plugin")
                        .label("Use a plugin instead of the colors")
                        .small()
                        .on_click(
                            cx.listener(|this, _, window, cx| this.use_default_plugin(window, cx)),
                        ),
                )
            })
            .child(field_grid(
                (pane_grid_columns(window) / 2).max(1),
                vec![
                    input("Plugin", &self.repo),
                    input("Colorscheme", &self.colorscheme),
                ],
            ))
    }
}

/// A file's key as words: `diffAddedDimmed` and `dark_bg` become
/// `Diff added dimmed` and `Dark bg`, which wrap inside a grid cell.
fn words(key: &str) -> String {
    let mut out = String::with_capacity(key.len() + 4);
    let mut previous_lower = false;
    for c in key.chars() {
        if matches!(c, '_' | '-' | '.') {
            out.push(' ');
            previous_lower = false;
            continue;
        }
        if c.is_uppercase() && previous_lower {
            out.push(' ');
        }
        previous_lower = c.is_lowercase() || c.is_ascii_digit();
        out.push(c);
    }
    let mut chars = out.chars();
    let first: String = chars
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default();
    let rest: String = chars
        .map(|c| {
            if c.is_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c
            }
        })
        .collect();
    first + &rest
}

#[cfg(test)]
mod tests {
    use super::{parse_plugin, plugin_lua, words};

    #[test]
    fn keys_become_words() {
        assert_eq!(words("diffAddedDimmed"), "Diff added dimmed");
        assert_eq!(words("dark_bg"), "Dark bg");
        assert_eq!(words("Comment · foreground"), "Comment · foreground");
        assert_eq!(words("editor.background"), "Editor background");
    }

    #[test]
    fn plugin_round_trip() {
        let lua = plugin_lua("folke/tokyonight.nvim", "tokyonight-night").unwrap();
        assert_eq!(
            parse_plugin(&lua),
            (
                "folke/tokyonight.nvim".to_string(),
                "tokyonight-night".to_string()
            )
        );
        let shipped = "return {\n\t{ \"ellisonleao/gruvbox.nvim\" },\n\t{\n\t\t\"LazyVim/LazyVim\",\n\t\topts = {\n\t\t\tcolorscheme = \"gruvbox\",\n\t\t},\n\t},\n}\n";
        assert_eq!(
            parse_plugin(shipped),
            (
                "ellisonleao/gruvbox.nvim".to_string(),
                "gruvbox".to_string()
            )
        );
    }

    #[test]
    fn rejects_names_that_would_break_the_lua() {
        assert!(plugin_lua("a/b\", evil()", "x").is_none());
        assert!(plugin_lua("a/b", "x\"").is_none());
        assert!(plugin_lua("nobody", "x").is_none());
    }
}
