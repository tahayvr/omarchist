# Omarchist - Agent Context Document

## Build/Test Commands

```bash
# Build and run
cargo build
cargo check
cargo run
cargo run --profile dev-fast   # what `just run` does: fast rebuilds, optimized deps

# Testing
cargo test                 # Run all tests (unit + headless UI tests)
cargo test <test_name>     # Run single test
cargo test <module>::      # Run tests in module
cargo test --test keyboard_nav   # Headless keyboard-navigation tests only
cargo test --test flows_ui       # Headless flow editor tests only

# Code quality
cargo clippy              # Run linter
cargo fmt                 # Format code

# Adding dependencies
cargo add <crate_name>
```

## Tech Stack

- **Rust Edition:** 2024
- **UI Framework:** GPUI via `gpui-pre` 0.3.x (longbridge's weekly snapshot of Zed's gpui; Zed's crates.io `gpui` stopped at 0.2.2). Renamed in `Cargo.toml` so code keeps `use gpui::*`. `gpui-pre-platform` (as `gpui_platform`, features `wayland` + `x11`) owns `application()`.
- **Components:** GPUI Kit 0.6 — `gpui-component` (styled), `gpui-base` (unstyled behaviour, focus traps), `gpui-kit` (facade; required in the graph because gpui's macros resolve paths through it), `gpui-kit-assets` (Lucide icons, no brand icons)
- **Pinning:** every `gpui*` crate is pinned exactly (`=`) and bumped together; `gpui-pre` patch releases are not semver-stable.
- **Async Runtime:** smol 2.0.2
- **Serialization:** serde + serde_json (app state), toml (flow files)
- **Date/Time:** chrono

## Configuration Directories

Important distinction between two config directories:

- **`~/.config/omarchy`** - Belongs to Omarchy Linux system. Used to store themes created by Omarchist app (the OS reads themes from here)
- **`~/.config/omarchist`** - Belongs to the Omarchist app itself. Used for app operations (settings.json, Hyprland settings state, `flows/*.toml`). `settings.json` is `SettingsSchema` in `src/system/config/config_setup.rs`; every field of `SettingsConfig` has a serde default, a version bump **merges** the new defaults into the user's file (`merge_settings`, keeping their values), and `defaults/omarchist/settings.json` must equal `SettingsConfig::default()` (a test checks). Change settings through `update_settings(|s| ..)`; read them with `settings()`.

Omarchy (Quattro/v4) itself is installed at `$OMARCHY_PATH`, defaulting to `/usr/share/omarchy` — see `src/system/omarchy_paths.rs`, the single canonical source for every Omarchy-related path.

Never confuse these two directories. Themes go in `omarchy/`, app config goes in `omarchist/`.

## Code Style Guidelines

### Module Organization

Use **named parent files** instead of `mod.rs`:
```
src/
├── ui.rs              # NOT ui/mod.rs
├── ui/
│   ├── app_view.rs
│   └── themes_page.rs
├── system.rs          # NOT system/mod.rs
└── types.rs           # NOT types/mod.rs
```

### Imports Order

1. Standard library (`use std::...`)
2. External crates (`use gpui::...`, `use serde::...`)
3. Internal modules (`use crate::...`)

```rust
use std::cell::RefCell;
use std::fs;

use gpui::*;
use gpui_component::button::Button;
use serde::{Deserialize, Serialize};

use crate::types::themes::EditingTheme;
use crate::ui::theme_edit_page::general_tab::GeneralTab;
```

### Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Types (structs/enums) | PascalCase | `ThemeEditPage`, `ActivePage` |
| Functions/variables | snake_case | `load_theme()`, `theme_name` |
| Constants | SCREAMING_SNAKE_CASE | `THEME_FILE` |
| Static/thread_local | snake_case with type | `PENDING_THEME_NAVIGATION` |
| Enum variants | PascalCase | `ThemeEditTab::General` |
| Generic params | PascalCase | `T`, `Entity<T>` |

### Type Definitions

Always derive common traits, use `#[serde(skip)]` for runtime fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTheme {
    pub name: String,
    pub created_at: String,
    #[serde(skip)]
    pub is_light_theme: bool,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            name: String::new(),
            created_at: String::new(),
            is_light_theme: false,
        }
    }
}
```

### Error Handling

Fallible code in `system/` and `shell/` returns `crate::error::Result<T>`, whose error is the `thiserror` enum in `src/error.rs` (`Io`, `Json`, `ThemeNotFound`, `ThemeExists`, `NotOmarchistTheme`, `UnknownDirectory`, `Network`, `Invalid`). Never return `Result<T, String>`.

```rust
use crate::error::{Error, Result};

pub fn create_theme(name: &str) -> Result<String> {
    let dir = get_themes_dir().ok_or(Error::UnknownDirectory("themes"))?;

    fs::write(&path, content).map_err(|e| Error::io("Failed to write colors.toml", e))?;

    Ok(name.to_string())
}
```

UI code displays errors with `to_string()` (`self.error_message = Some(e.to_string())`) and may match on variants when it needs to react differently. In files that `use gpui::*`, spell the alias out as `crate::error::Result<T>` because gpui re-exports anyhow's `Result` under the same name.

### GPUI Actions and Shortcuts

Define actions with `actions!(namespace, [..])` (in a `pub mod` when several components share a namespace, e.g. `focus::tab_strip`) or the derive macro for actions with data:

```rust
#[derive(Clone, PartialEq, Action)]
#[action(namespace = keybinds, no_json)]
pub struct SetFilter(pub usize);
```

Every key binding lives in `src/ui/shortcuts.rs` (`SHORTCUTS`): `main.rs` registers `key_bindings()`, the help dialog renders `help_rows()`, the command palette (`src/ui/dialogs/command_palette.rs`) looks its key hints up from the live keymap, and a test rejects duplicate `(keys, context)` pairs. App-wide commands belong in the palette's `groups()` too. Never call `cx.bind_keys` with app shortcuts anywhere else. Never bind a bare printable key (`space`, letters) in a context that can contain an `Input`: the binding wins over the input and the character is never typed. To override a component's own key (Input binds `tab`, `down`, `escape`, `ctrl-enter`) use a child predicate registered later, e.g. `Some("KeybindsSearch > Input")`.

### State Management

Cross-component requests go through the `AppEvents` global in `src/ui/app_events.rs`, never through `thread_local!` flags polled from `render`:

```rust
use crate::ui::app_events::{emit, emit_async, AppEvent};

// From a click handler or anything with `&mut App`
emit(cx, AppEvent::Navigate(ActivePage::ThemeEdit(theme_name)));

// From a background task with an `AsyncApp`
let _ = emit_async(cx, AppEvent::RefreshThemes);
```

`MainWindowView::new` registers `cx.observe_global_in::<AppEvents>` and drains the queue in `handle_app_event` (events queued before the view existed are drained on the first frame). Feedback goes through `window.push_notification(..)`; `MainWindowView::render` must keep rendering `Root::render_notification_layer` (after the dialog and sheet layers) or every toast is silent — `tests/keyboard_nav.rs::notifications_are_shown` guards it. Add a variant to `AppEvent` and a match arm there for any new request. Local UI state still lives on the entity and is updated with `cx.notify()`.

### UI Patterns

Use `v_flex()` and `h_flex()` for layouts:

```rust
impl Render for MyComponent {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        
        v_flex()
            .id("my-component")
            .size_full()
            .bg(theme.background)
            .gap_4()
            .child(
                h_flex()
                    .gap_4()
                    .items_center()
                    .child(Button::new("btn").label("Click").on_click(...))
            )
    }
}
```

### Selectable Text

Every value the app shows (names, descriptions, ids, paths, commands, step summaries, messages, versions) is rendered with `crate::ui::text::selectable(id, text)` from `src/ui/text.rs`, a `gpui_base::SelectableText` run that joins gpui-component's window selection layer (`Root` renders it; Ctrl+C copies). The run takes its parent `div()`'s text style, so styling stays on the div and only the text child changes. The `id` must be unique among siblings (`("flow-name", ix)` in lists). Button labels, chips, control labels, and headings stay plain. A headless test double-clicks the About page's version and reads the selection back with `TextSelection::selected_text`.

### Responsive Design (REQUIRED)

**ALL pages, tabs, and content MUST be responsive.** Never use fixed widths causing overflow.

**Key Principles:**
1. Use `.flex_wrap()` on horizontal containers
2. Avoid large fixed gaps (`.gap_128()`, `.gap_64()`) - use `.gap_8()`, `.gap_6()`
3. For charts/grids, adjust based on viewport width

```rust
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let viewport_width = window.viewport_size().width;
    
    let column_count = if viewport_width < px(640.0) {
        1
    } else if viewport_width < px(1024.0) {
        2
    } else {
        3
    };
}
```

4. Never use `overflow_x_hidden()` to hide overflow - fix the layout with `.flex_wrap()`, `.min_w_0()`, `.flex_grow()`

**Breakpoints:**
- `< px(640.0)` - Mobile/single column
- `< px(768.0)` - Small tablet
- `< px(1024.0)` - Tablet/2 columns
- `>= px(1024.0)` - Desktop/3+ columns

### Comments

- Use `///` for doc comments on public items
- Use `//` for inline implementation comments
- Use `eprintln!()` for debug output (not in production)

## Architecture Patterns

### Page Navigation

Centralized in `MainWindowView`:

```rust
pub enum ActivePage {
    Themes,
    ThemeEdit(String),
    Configuration,
    Keybinds,
    Flows,
    FlowEdit(String),      // the editor for an existing flow, by id
    FlowNew(Option<String>), // the editor for a new flow, optionally from a template key
    FlowImport(Box<Imported>), // the editor reviewing an imported flow before its first save
    FlowTemplates,         // the Templates page
    Settings,
    About,
    Omarchy,
}
```

### Focus and Keyboard Navigation

GPUI focus is the only source of truth; never keep a shadow "focused index" that is not backed by a `FocusHandle`. Helpers live in `src/ui/focus.rs`:

- Every interactive element is a tab stop (`focus::tab_stop(cx)` or gpui-component's `Button`/`Input`/`Select`/`Radio`/`ColorPicker`). `Switch` is its own tab stop; `FocusableSwitch` wraps it with a clickable label and a row focus ring without adding a second stop.
- Composites (sidebar, tab strips, theme grid, keybinds table and filters, config section list) are one tab stop with a roving index; their arrow keys are actions in their own `key_context`. Tab never stops on individual items inside them.
- Each page exposes `focus_entry(&self, window, cx)`; `MainWindowView::navigate_to` calls it so keyboard users land on the first control. `Escape` (`focus::EscapeToSidebar`) toggles between the sidebar and the page.
- Dialogs wrap their content in `focus::dialog_body(...)` (handles `dialog::Submit` = Ctrl+Enter) and call `focus::focus_first_in(&body_focus, window, cx)` after `open_dialog`. Tab is trapped by gpui-kit itself: every dialog is a `focus_trap`, and `Root`'s Tab handler stays inside the active trap. Where a control's own `tab` binding is overridden (`... > Input`), route it to `focus::FocusNext`/`FocusPrev`, whose handler (`focus_next_trapped`) honours the trap the same way.
- Dialogs are deferred elements of `Root`, but the main view stays their dispatch ancestor, so global shortcuts pressed inside a dialog reach `MainWindowView`. Its page-level handlers (navigation, New Theme, Import, the help dialog, the palette) therefore check `dialog_open(window, cx)` first and do nothing while a dialog is up. The palette closes itself and dispatches the chosen command through the main view's `FocusHandle::dispatch_action`, so the command runs with no dialog open.
- Headless UI tests (`tests/keyboard_nav.rs`) follow the GPUI Kit testing guide (gpui-kit.com/docs/test): `#[gpui_kit::test]`, types from the Kit root (`gpui_kit::{TestAppContext, ..}`, `gpui_kit::component::Root`), `cx.update(gpui_kit::init)`, `cx.open_window(size, ..)` around the production `Root`/`MainWindowView`, `window.render_frame(cx)` before the first query, `TestWindowExt` for `find`/`press`/`input`, `wait_for` for async work, and model state checked next to UI snapshots. Elements that tests query carry a stable `.id(..)` followed by `.test_support()` *before* `.track_focus(..)` (`SidebarNav`, `focus::dialog_body`); `test_support()` is inert in normal builds but wraps the element under `test-support`, so such helpers return `impl ParentElement + StatefulInteractiveElement + ..` rather than `Stateful<Div>`. The window must be free of network tasks (the Omarchy update watcher starts from `main.rs`) and background work runs on gpui's executor (`cx.background_spawn`, not `smol::unblock`) so the test scheduler can drive it.
- The UI tests share `tests/common/mod.rs`: `open` builds the production window in a home directory of its own (`common::home` points `HOME` at a temp folder once per test binary, so a test never reads or writes the real config; `write_flow` seeds a flow file), `settle` runs the executor and delivers next-frame work (`Window::simulate_next_frame`: a test window has no frame loop, so `window.on_next_frame` callbacks, such as a dialog taking focus, never fire otherwise), and `wait_real` waits in real time for work on an OS thread (a flow run). `tests/flows_ui.rs` drives the flow editor: each test acts through `click`/`input`/`press` on ids and checks the screen next to `common::edited_flow` (the flow the editor holds). Give a new control a stable id (`Input::new(..).id("..")`, `.id(..).test_support()` on a div) when a test should reach it.
- Scrolling content wraps sections in `FocusSection` so a focused section scrolls into view, and wraps its focusable content in `focus::scroll_area(&handle)` (inside the element that tracks the handle): its `ScrollArea` key context gives Up/Down, PgUp/PgDn and Ctrl+Home/End to the scroll, and any deeper control that binds those keys keeps them. A scroll list with nothing focusable in it (the shortcuts dialog) makes the `scroll_area` itself the tab stop and focuses it. Grids that are one tab stop scroll their focused row into view with `ScrollHandle::scroll_to_item`, which needs the rows to be the scroll container's direct children. A headless test presses Page Down and Up in the shortcuts dialog.
- Focus rings come from `handle.is_focused(window)` and `focus::focus_border`.

### Entity Pattern

Components that need state use the Entity pattern:

```rust
let view = cx.new(|cx| MyComponent::new(cx));
let root = cx.new(|cx| Root::new(view, window, cx)).into();
```

### Auto-save Theme Editing

Changes persist automatically (no save button):
1. User edits field
2. Call `theme_management::save_theme_data()`
3. Writes `colors.toml` (the theme's source of truth); every app's file is template-generated by Omarchy itself from it (see "Theme System (Quattro)" below).

## GPUI / gpui-component Gotchas

- **`dirs::config_dir()` is wrong on macOS** — returns `~/Library/Application Support`. Always use `dirs::home_dir().join(".config")` throughout the entire codebase.
- **Rust 2024 `impl Trait` lifetime capturing** — returning `impl IntoElement` from a method that also borrows `cx` causes borrow conflicts. Return `AnyElement` from free functions instead of methods.
- **`ContextMenuExt`** is a trait from `gpui_component::menu` — import it to get `.context_menu(|menu, _, _| {...})` on any `ParentElement + Styled`. It handles right-click automatically; no manual mouse button checking needed.
- **`ButtonVariants` trait** must be in scope for `.ghost()` on `Button`.
- **`Tooltip`** lives at `gpui_component::tooltip::Tooltip`, not `gpui_component::Tooltip`.
- **`.when()` on a div** requires `use gpui::prelude::FluentBuilder` in scope; the closure parameter needs an explicit type annotation: `|this: gpui::Div|`.
- **Drag and drop API** — `on_drag` takes a value + ghost-view constructor, `on_drop` takes `Fn(&T, &mut Window, &mut App)`. Use `drag_over::<T>` for highlight styling on drop targets.
- **Stateless vs stateful components** — stateless: `#[derive(IntoElement)] + impl RenderOnce`; stateful (holding GPUI state): full `impl Render` struct owned as `Entity<T>`.
- **Tab stops** — only the `FocusHandle`'s own `tab_stop`/`tab_index` count; `Div::tab_index()` does not write them to a tracked handle. Set them on the handle (`cx.focus_handle().tab_stop(true)`).
- **`actions!` does not create a module** — the namespace is only the action name. Wrap it in `pub mod name { gpui::actions!(name, [...]); }` when you want `name::Action` paths.
- **`use super::*` in a test module** of a file that has `use gpui::*` imports gpui's `test` attribute macro and breaks `#[test]` (recursion limit). Import the specific items instead.
- **gpui-component `Settings`** keeps its page selection in private keyed state and cannot be driven from the keyboard; the Configuration page renders its own section list and `GroupBox`es from a declarative table instead.
- **`SidebarMenuItem` is not an element on its own** — it implements `SidebarItem`; render it with `.render(id, window, cx)` outside a `Sidebar`/`SidebarMenu`. It is `Styled`, so focus rings go on the item itself. `Sidebar::new(id).side(..)`; `.suffix()` takes a builder closure. It has no tooltip API, so `SidebarNav` lays the items out itself and wraps each collapsed one in a `div().tooltip(..)` naming the page and its shortcut.
- **Wording**: sentence case for every button, menu item, tab, and dialog title ("New theme", "Add keybind", "Reset to generated", "Delete this flow?"); errors read "Could not <verb> the <thing>: {e}"; creation toasts read "Created '<name>'"; `…` rather than `...`. The title bar's page menus (Themes: New theme, Re-apply theme; Keybinds: Add keybind…; Flows: New flow, New from template, Import flow…, Run) mirror the pages and the palette; the Omarchist menu holds the palette, shortcuts, Settings, About, and Quit. No disabled placeholder items.
- **`Select` renders a full-width root** — box it in a fixed-width `flex_none` div or it squeezes its siblings.
- **`h_flex()` centres its children** (`items_center`), and gpui has no `items_stretch`; use `div().flex().flex_row()` when a child must fill the row's height (e.g. a scroll container).
- **`font_family("monospace")` does not resolve through fontconfig** and falls back to a proportional face; the app font is already JetBrainsMono, so never set a generic family.
- **`DataTable` rows have a fixed height and its column groups are cached**: a delegate that changes its column count (`KeybindsTableDelegate::fit_width`, which drops the Source column below 900 px and folds its tags into the Action cell) must call `TableState::refresh`, and its `column()` must tolerate an index from the previous layout. The main window pushes `set_sidebar_collapsed` into the Themes and Keybinds pages so they can size to the width they get.
- **`AlertDialog`** can only be opened from its own trigger element (`with_base_alert_dialog` is crate-private), so programmatic confirmations use `dialogs::confirm_dialog`.
- **`FocusHandle::focus`, `Window::focus_next/prev` take `cx`**; `Entity::update` on an `AsyncApp` is infallible (it panics once the app is gone, which cannot happen to a foreground task); `ScrollHandle::max_offset()` is a `Point`.
- **Key contexts:** the data table's context is `DataTable` (not `Table`); `Input` binds `tab`, `up`, `down`, `escape`; `Root` binds `tab`, `shift-tab`, `ctrl-c`.
- **`InputState` steps by itself.** It is created with a step of 1, and while it has a step it changes its own text on the stepper buttons and Up/Down (clamped to `.min()`/`.max()`) and emits `InputEvent::Change`; `NumberInputEvent::Step` is only emitted after `set_step(None)`. Give every `NumberInput` its real step and range (`.step(..)`/`.step_by(..)`, `.min(..)`, `.max(..)`) and act on `Change`. The Configuration page tells a step from typing with a `step_by` hook, commits typed text on Enter or blur, and ignores the blur a `hyprctl reload` causes (the field is still `is_focused` then), so a half-typed `0.` never reaches the compositor.
- **Library dialogs bind Enter to `Confirm`**, which closes them before a focused button gets its keyboard click. `focus::dialog_body` stops the action and `shortcuts::key_bindings` adds a `NoAction` Enter binding in `DialogBody`, so Enter reaches the focused control; `Input` and `Select` keep their own deeper Enter bindings (`Select` re-propagates `Confirm` after opening).
- **`gpui_kit_assets` embeds only the component default icon subset** (`default-icons.txt`, about 100 icons); any other Lucide icon is copied into `assets/icons/` and used through `Icon::new(Icon::empty()).path(..)`.
- **Inline code in `TextView::markdown` (gpui-base 0.6.1)** is drawn over the neighbouring words of its line. Fixed upstream after 0.6.1 (gpui-kit PRs #3038 and #3046); goes away with the next pinned bump.

## Theme System (Quattro)

Omarchist targets Omarchy Quattro (v4) only — see `tahayvr/omarchist#39`. Quattro replaced Waybar/Mako/Walker/hyprlock/SwayOSD/hypridle/swaybg with one Quickshell-based desktop shell, and collapsed theme folders down to a handful of files driven by a semantic `colors.toml`. Confirmed against the real `omacom/omarchy` source (not docs):

- **`colors.toml` is the theme's sole source of truth.** `ColorsConfig` (`src/types/themes.rs`) writes `mode`, `accent`, `cursor`, `foreground`, `background`, `selection_foreground`/`selection_background`, and `color0`-`color15`. Omarchy's own `omarchy-theme-color` resolver derives everything else (shades, named hues, mode auto-detection) from this set — Omarchist does not need to compute or store those derived values.
- **Omarchist writes only `colors.toml`, `icons.theme`, the backgrounds, and the boot logo.** Terminal configs, window borders, the bar, notifications, btop, editors and every other app's file are generated by Omarchy's `omarchy-theme-set-templates` from `colors.toml` after `omarchy-theme-set` runs, unless the theme folder ships that file itself. Omarchist has no per-app editors (the optional tabs, override files, and palette bundles were removed in 2.0; the docs point people who want per-app styling to Aether). Never reimplement Omarchy's template engine or write a template-generated file.
- **Hyprland border colors** are the two optional `colors.toml` keys Quattro's `hyprland.lua.tpl` reads: `hyprland_active_border` and `hyprland_inactive_border` (`ColorsConfig`, edited as free text on the Colors tab because they accept gradients). There is no per-theme `hyprland.lua` written by Omarchist.
- **Tabs each hold a snapshot of the theme.** They must save manifest and `colors.toml` fields through `theme_management::update_theme(name, |theme| ...)`, which reloads from disk and applies only the fields that tab owns, never `save_theme_data` with the tab's whole snapshot. `icons.theme` goes through `themes::icons::write`, which accepts only the Yaru variants in `ICON_THEMES`.
- **No source editing in the GUI.** Omarchist is for people who do not want to touch text editors, so every theme setting is a color picker, a switch, or a radio group; there are no code editors, Source views, or file previews.
- **No explainer text in the UI.** Pages show labels, controls, and at most a one-line identifier; how a feature works is documented under `docs/`, never in help text on the page. Forms lay their fields out with `shared::field_grid` (rows of equal-width cells, the last row padded, columns from `tab_grid_columns`) and label them with `shared::field_label` (a fixed two-line box so a wrapped label never pushes its control out of line with the row); the Colors tab uses eight columns so each bright color sits under its normal one.
- **Theme Designer tabs** (`ThemeEditTab::ALL`): General (name, author, light mode), Colors (palette plus the Hyprland border keys), Backgrounds (images and the boot logo), and Icons (`icons_tab.rs`, one radio per Yaru color, written to `icons.theme` on pick). Every color picker shows its hex value under the swatch (`shared::color_picker_with_clipboard`); Colors edits save 300 ms after the last change, off the UI thread, and `ThemeEditPage::flush_pending_saves` runs before Apply and Back so nothing pending is lost. Picker values are written through `ui::color_utils::hex6` (`#rrggbb`, never the alpha byte). `update_theme` holds a per-theme lock so tabs on different threads never overwrite each other. The designer reads the palette from `colors.toml` (`colors::read_colors_toml`, aliases honoured, unknown keys kept in `ColorsConfig::extra`) and only falls back to the manifest for missing keys. Every Apply goes through `ui::theme_apply` (background, toast on success or failure). A theme made from an image gets the Yaru color closest to its accent (`icons::closest_to`).
- **Boot logo**: `unlock.png` is the Plymouth/SDDM logo, applied only through `omarchy-plymouth-switcher` (sudo), never by `omarchy-theme-set`. The switcher lists a theme only if it has `preview-unlock.png`, so the Backgrounds tab renders that with `omarchy-plymouth-preview` whenever a logo is set (`theme_file_ops::render_boot_preview`); the script ends by opening the result in a fullscreen `imv`, so it runs with a PATH shim whose `imv` exits at once (`theme_sh_commands::plymouth_preview`).
- **`light.mode` is legacy.** Omarchy's `omarchy-theme-color` only consults it as a fallback behind the `mode` key in `colors.toml`. Omarchist always writes `mode`, never writes `light.mode`, and deletes a stale one on save; it is still honored on load for themes written by Omarchist 1.x.
- **The Status Bar / Waybar feature was removed entirely** (not ported) — Quattro's own `omarchy bar` tooling and shell now own bar editing natively, so Omarchist no longer needs to.

- **Omarchist edits only its own themes**: a folder under `~/.config/omarchy/themes` with an `omarchist.json`. `theme_file_ops::omarchist_theme_dir` is the gate every theme write goes through (`save_theme_data`, `update_colors_toml`, `rename_theme`, `icons::write`, backgrounds, the boot logo) and returns `Error::NotOmarchistTheme` for anything else; `load_theme_for_editing` never falls back to a default manifest. `MainWindowView::navigate_to` turns a `ThemeEdit` of such a theme into the Themes page with a notification.

### Hyprland Config — Lua Ownership

Quattro moved Hyprland's config from hyprlang text to real Lua (`~/.config/hypr/hyprland.lua`). Omarchist **never parses the user's Lua** — confirmed mechanism from the real `config/hypr/hyprland.lua`/`default/hypr/bootstrap.lua`:

- **Only the keys the user changed are stored**, as a sparse JSON object in the shape of `HyprlandConfig` (`~/.config/omarchist/hyprland/state.json`, `{"version": 2, "overrides": {...}}`, see `src/system/hyprland_config/manager.rs`). Every other key follows Omarchy. The UI reads and writes values by dotted option path (`general.gaps_in`); `HyprlandConfigManager::set_value` drops an override that equals the baseline, `reset` drops it outright, and each changed setting shows a reset button with Omarchy's value in its tooltip.
- **The Configuration page's content is data** (`src/ui/config_page/pages.rs`, `HYPRLAND_PAGES` + `OMARCHY_PAGES`, iterated by `pages()`/`items()`): sub-pages of groups of items. An item's `Source` is `Hyprland(path)` (one option by dotted path), `Omarchy(Backing)`, or `None` (actions). Control kinds: `Number`, `Pair` for a `Vec2` component, `Switch`, `Dropdown` for strings, `Choice` for named integers, `KeyboardLayout`, `DynamicDropdown` (choices found at runtime), `Action` (a button running a command, in Omarchy's floating terminal when `terminal`), `DynamicDropdown` with more than `LONG_LIST` (40) choices renders as a searchable `Select` (the timezone list), and `Feature` (a status read plus setup/remove commands). `when: Some(&["omarchy-hw-laptop"])` hides an item when the command fails. Adding a Hyprland setting is adding a field to `HyprlandConfig` and an item; tests check every item's path and type against the model, its range and choice values against `hyprctl descriptions -j`, and that every command an Omarchy item names exists. Colors and gradients are not on the page (the Theme Designer owns borders).
- **There is no Software page** on Configuration; a separate Software section is planned. `src/system/software_catalog.rs` (parses Omarchy's `omarchy-menu.jsonc` plus the user's extensions file into install/remove pairs, evaluates each `when` with `bash -c`, runs actions with `bash -lc`) is kept for it and is not wired to any page. Never hardcode an install list.
- **Omarchy settings** (`src/system/omarchy_settings.rs`) are `Backing { read, write }` pairs: `Read::{ShellJson, ShellWidget, Flag, StateFile, Command{argv, parse}}` and `Write::{ShellJson, Bool{on, off}, ToggleIfDifferent, Command, CommandAnd, CommandJson, Terminal, X11Keymap}` (`X11Keymap` passes the system's model, variant and options back to `localectl`, which clears whatever it is not given). A `Terminal` write is read back with `read_until` (the terminal has only just opened) and the row says "Running in the terminal…"; a toggle that reloads Hyprland (`Bool`, `ToggleIfDifferent`) re-reads the baseline afterwards (`reload_baseline`); every terminal action waits for Omarchy's terminal window to close (`wait_for_terminal_to_close`) and then re-reads the Omarchy values, as does every visit to the page (`refresh_omarchy_values`). Every write goes through Omarchy's own script for the setting (`omarchy-toggle <flag> on|off`, `omarchy bar position`, `omarchy-powerprofiles-set`, ...) or edits `shell.json` and calls `omarchy-shell shell reloadConfig`; never reimplement what a script does. The view reads an Omarchy page's values in the background when it opens (`load_omarchy_page`) and writes in the background with an optimistic update and a read-back. Scripts with no arguments often *toggle* (`omarchy-toggle-idle`, `omarchy-toggle-nightlight`): read state only with their `status`/`--status` forms.
- A `state.json` that does not parse makes `HyprlandConfigManager::load` fail (never "no overrides"): the page shows why, disables every Hyprland control (`is_unavailable`) and refuses to save, so the file is never replaced; without a reachable compositor (`compositor_reachable`) the page says so instead of showing Hyprland's stock defaults as current. After every save the page reloads Hyprland with `reload_hyprland_checked` and shows `hyprctl configerrors` above the scrolling column.
- **The baseline** (`baseline.rs`) is what Hyprland runs without `omarchist.lua`: the compositor's live values (`hyprctl_reader.rs`, one batch of `hyprctl getoption` for every key of the flattened model, typed by the model's default, `[[EMPTY]]` read as empty), except that overridden keys take the value recorded by scanning the user's `hyprland.lua` with the keybind scanner's stubbed `hl` (`scan.lua` emits a `config` record per `hl.config` leaf, skipping `omarchist.lua`), or Hyprland's default when no file sets them. `HyprlandConfig::default()` must therefore equal Hyprland's defaults; a test audits it against `hyprctl descriptions -j` whenever a compositor is running. A 1.x state file (the whole config) is migrated on load to the keys that differ from Hyprland's defaults, the 1.x model's wrong defaults, and Omarchy's scanned values, and kept as `state.json.legacy`.
- On every save, `src/system/hyprland_config/lua_writer.rs` generates a write-only `~/.config/hypr/omarchist.lua` containing `hl.config({ <section> = {...} })` calls — Hyprland's own real Lua config API, confirmed against `config/hypr/looknfeel.lua`/`input.lua` — rendered straight from the overrides, with the Lua API's key names (`input.touchpad.tap_to_click`, not hyprlang's `tap-to-click`; `debug.vfr`, not the retired `misc.vfr`; `decoration.shadow.offset` as a `{x, y}` sequence).
- `src/system/config/hypr_setup.rs` appends an idempotent `require("hypr.omarchist")` line to the user's `hyprland.lua`, after the existing `require("hypr.autostart")` line — this is Omarchy's own sanctioned convention for user config modules (`package.path` already includes `~/.config/?.lua`).
- `src/system/hyprland_config/hyprctl_reader.rs` still live-introspects settings via `hyprctl getoption` — untouched, version-agnostic.
- The pre-Quattro `~/.config/omarchist/hyprland/hyprland.conf` (hyprlang, sourced by glob) is gone; `config_setup.rs` deletes a stale copy on startup.

### Keybinds — Lua Scanner + Overrides

Omarchy declares every keybind in Lua (`o.bind(keys, description, dispatcher, opts)` over Hyprland's `hl.bind`), and `hyprctl binds` cannot describe them (every dispatcher is `__lua` with an opaque id; `code:N` keys come back empty). The Keybinds page therefore works like Omarchy's own `omarchy-menu-keybindings`:

- `src/system/keybinds/scan.lua` (embedded with `include_str!`) is run by the system `lua` interpreter against `~/.config/hypr/hyprland.lua` with a stubbed `hl` table. It prints one tab-separated record per `hl.bind`/`hl.unbind` call, with the declaring file (skipping Omarchy's `helpers.lua` frames), the keys, description, dispatcher (`exec` command, `lua` expression reconstructed as source text, or `fn`), and flags. `hl.get_*` getters answer `nil`; everything else is a noop.
- `replay.rs` turns those events into the effective list with Hyprland's semantics: binds on one chord stack, and `hl.unbind` removes every earlier bind on that chord. Origin is classified from the path: `$OMARCHY_PATH/default/` → Default, `omarchist.lua` → Omarchist, otherwise User.
- User changes are overrides in `~/.config/omarchist/hyprland/keybinds.json` (`overrides.rs`: Rebind / Disable / Add, each targeting a bind by chord + description + dispatcher, with the sibling binds to restore captured at save time; `upsert` drops a newly targeted bind from every other override's restore list, `restored_by` says which override keeps a sibling alive so the page shows the original row as Plain instead of a phantom copy, and `stale` lists overrides whose target Omarchy no longer ships, shown as `RowKind::Stale` rows that only Reset applies to). The page's `commit` always loads the file fresh (the Flows page writes it too), refuses to write while the file cannot be parsed, and every visit rescans. Dialog events name the row they were opened for (`Save { replaces }`, `Disable(identity)`, `Reset(ix)`), never the table's selection. After a save the page runs `manager::reload_hyprland_checked` and reports `hyprctl configerrors`. `emit_keybinds_lua` renders them as `hl.unbind(...)` + `hl.bind(...)` lines that `manager::write_omarchist_lua` appends to `omarchist.lua` after the settings section, so the Configuration page and the Keybinds page can never drop each other's block. Omarchist never edits the user's `bindings.lua`. `validate()` only accepts Lua dispatchers of the form `hl.dsp.<path>(<literals>)`: `is_dsp_call` parses the argument list and allows only string, number, boolean and `nil` literals and tables of those (everything the scanner reconstructs and the action builder produces), so neither chaining after the call nor a call inside the arguments (`hl.dsp.focus(os.execute(..))`) passes; `write_omarchist_lua` renders only `valid_only()` overrides, so a hand-edited json cannot inject code. `write_omarchist_lua` also re-adds the `require("hypr.omarchist")` line to `hyprland.lua` when it is gone (Omarchy's reset scripts replace that file) and returns whether it had to, which the pages report.
- `omarchist.lua` always defines the `omarchist-recording` submap (one inert switch bind, because Hyprland only registers submaps that contain a bind, plus `SUPER + SHIFT + ESCAPE` → `submap("reset")` so a user is never locked in). Because release builds abort on panic, `submap::install_panic_hook` resets the submap before the abort and `submap::reset_on_startup` resets it on every launch. The recorder (`keystroke_input.rs`, ported from Zed's `KeystrokeInput`) switches Hyprland into it with `hyprctl dispatch 'hl.dsp.submap("omarchist-recording")'` while recording, so bound chords reach the app, and resets it on stop, blur, drop, and app quit (`submap.rs`). On this Hyprland `hyprctl dispatch` takes Lua syntax; the old `submap name` form is rejected.
- Recording uses `cx.intercept_keystrokes` + `cx.stop_propagation()` installed on inner-focus-in and dropped on focus-out, so `is_recording()` is derived from focus and cannot desync. A Hyprland bind is one chord, so recording stops after the first complete keystroke.
- GPUI key names are mapped to Hyprland keysym names in `keymap.rs` (shifted symbols back to base key + SHIFT, Omarchy's spellings such as `RETURN`/`comma`, XF86 names canonicalised via `xkbcommon`). Chords are parsed and compared in `chord.rs` (`code:10` ≡ `1`).

### Flows — Sequences Triggered From Anywhere

A flow (`src/system/flows.rs`) is a named list of steps run in order. Steps reuse the keybind dispatcher vocabulary (`StepKind::Exec`/`Lua` ↔ `Dispatcher`, so the same `ActionBuilder` edits both) plus the flow-only `Wait`, `Notify` (optionally with `on_click` copy/open of a `target`), `Flow` (nesting), the steps that ask (`Ask`, `Choose`, `Confirm`, `Pick`), and the blocks, which hold steps: `If` (a `Condition` from `condition.rs`, `not`, `then`, `otherwise`), `Repeat` (`times`), `Each` (one round per line of `items`), `Menu` (a `MenuChoice` per branch), plus `Stop`. In TOML a block's steps are tables under it (`[[step.then]]`, `[[step.otherwise]]`, `[[step.do]]`, `[[step.choice]]` with `[[step.choice.do]]`). A `StepPath` addresses a step in that tree (`[2, 1, 0]`: third step, second branch, first step; an even-length path names a list), `Flow::walk` lists every step as written (which is also the order of step numbers in messages), `flows::move_step` carries a step one place through the tree (into and out of blocks), and `StepKind::adopt_branches` keeps a block's steps when the dialog edits it (a menu's by choice name). Each `Step` has a runtime-only `uid` (not serialized, not part of equality) so editor state follows a step when it moves. Each flow is one TOML file `~/.config/omarchist/flows/<id>.toml` (`store.rs`; TOML rather than JSON because flows are meant to be shared and hand-edited, and single-quoted TOML strings carry Lua and shell text without escaping); the id is a slug fixed at creation because keybinds, desktop entries, and the CLI refer to it. `validate()` applies the same `hl.dsp.*(...)` guard as keybind overrides.

- `runner.rs` runs steps one at a time: `Exec` through `setsid sh -c` (execs in place, so the command outlives the flow) with a 300 ms grace in which an exit counts as failure (`sh` reporting a missing program), or `sh -c` when `wait` is set (Omarchy's launch scripts `exec setsid`, which does not fork under a plain child, so the wait lasts until the window closes) with the last stderr lines in the error; `hyprctl dispatch` for Lua; `notify-send --`. It reports `RunEvent`s (a step's `StepPath` and, when it ends, its `StepStatus`: `Done(output)`, `Failed(error)` or `Cancelled`), honours `OnError`, refuses nested loops via a call stack, and stops between steps (or kills a waited command, or cuts a `Wait` short) when its `Cancel` is set. `run_in_thread` validates and runs a flow on a thread of its own (a run blocks for as long as the flow takes); the editor runs on a thread too, tags messages with a run id, shows each failed step's error under it, offers Stop, and refuses step edits while running. A dispatcher of the form `omarchist flow run <id>` (quoted or not) always becomes `StepKind::Flow` (`from_dispatcher`), so nesting is checked in process. `Runner::with_loader` takes a flow loader so tests never touch disk.
- Ready-made actions (`actions.rs`): `StepKind::Action { action, args }` is a row of `ACTIONS` (an `ActionDef`: id, label, group, icon, `fields`, `title` template, `requires`, `saves_as`, `run`) with its fields flattened next to it in TOML (`action = "volume.set"`, `level = 40`). Adding an action is adding a row: the picker lists it (`step_types()`), the form is built from its `Field`s (`FieldKind::{Text, Number, Choice, App, Theme}`), and tests check that every title marker and every `$ARG_<KEY>` names a field and that every script parses (`sh -n`). `Run::Shell` scripts are fixed text that read their fields from `$ARG_<KEY>` environment variables; `Run::Lua` fills fields into string literals through `actions::lua_call` (escaped, then `is_dsp_call`); `Run::Native` is Rust. Never build an action's command from a field's value. An action that produces output (`saves_as` not empty) gets that name pre-filled in the form. Actions set a state rather than toggle one, so a flow run twice ends up the same.
- Blocks in the runner: `run_list` runs one list of steps and recurses through `run_branch`/`run_rounds`; events carry the nested step's path, and a loop sends `RunEvent::Round` before each round. `Stop` ends the current flow as finished (`Halt::Stop`), a failure inside a block follows the flow's `OnError`, and a switched-off block runs nothing it holds. Loops set `{{index}}` (and `{{item}}`) for their steps and restore what those names held (`vars::LOOP_NAMES`, reserved like the built-ins); `Flow::names_at(list, index)` says what a step at a place can use. An `if` checks the machine through the `Probe` trait (`Machine` for real, a fake via `Runner::probe` in tests). A name no step has set is an error wherever a value is used (so a command never runs with a hole in it) and reads as empty only in the `empty` check (`Vars::text_or_empty`).
- Asking (`prompt.rs`): the runner asks through the `Prompter` trait; the `Desktop` one runs Omarchy's own menu scripts (`omarchy-menu-input`, `omarchy-menu-select` with the options on stdin, a longer `OMARCHY_SHELL_IPC_TIMEOUT` and one retry while the shell is not ready) and `rfd`'s portal file chooser, and tests pass a scripted one with `Runner::prompter`. The answer is the step's output. A dismissed prompt (or Cancel in a `Confirm`) is `StepStatus::Cancelled`: the flow ends there with `Outcome::cancelled`, no failure and no notification, and it ends a parent flow too. A clickable notification goes through `omarchy-notification-send --exec <argv>` so its target is only ever one argument. Never run a flow that asks from a test or the headless harness without a scripted prompter: the menu opens on the user's screen.
- Variables (`vars.rs`): a step with `output = "name"` saves its output for later steps (a waited `Exec`'s stdout, a nested `Flow`'s `Outcome::last_output`, or an asking step's answer; `StepKind::has_output`), and `{{name}}` uses it in commands, notification text and `Lua` actions; built-ins (`BUILTINS`: clipboard, selection, date, time, window, app, workspace) are read on first use. Substitution never makes a value code: `Vars::shell` replaces each reference with a quoted `${OMARCHIST_VAR_n}` expansion fitted to the shell quoting around it and passes the values as environment variables; `Vars::lua` only fills references inside Lua string literals, escapes the value, and re-checks `is_dsp_call`; `Vars::text` is plain for notifications. `Flow::validate` refuses a reference no earlier step saves (built-ins excepted), a bad or built-in name, an output on a step that has none, and a Lua reference outside quotes. A flow is written with the lowest format that holds it (`Flow::required_format`: 2 only when it uses something 2.0.0 does not know, such as variables or the asking steps, so plain flows stay readable by 2.0.0); `FORMAT` is the newest format read. `RunEvent::Finished` carries each step's output, which the editor shows under the step; the step dialog has "Save output as" (waited Command, Flow) and a `StepVariables` row (one tab stop, arrows, Enter/Space) that appends `{{name}}` to the field being edited. Outside text inputs a variable is never shown as raw `{{name}}`: `ui::flows_page::var_token` draws it as a token (icon + readable name, `describe`), and `rich_text` renders step titles, details and previews with tokens inline.
- Triggers: `omarchist flow run <id-or-name>` (`cli.rs`, handled in `main()` before the window opens); a keybind via `Action::Flow` (dispatcher `omarchist flow run <id>`, unquoted; offered as the **Flow** kind in the action builder; `run_command_id` also recognises the launcher's `uwsm-app -- omarchist flow run <id>` form); `launcher.rs` writes a `.desktop` entry (`~/.local/share/applications/omarchist-flow-<id>.desktop`, icon SVG under `~/.local/share/omarchist/flows/`) and a `post-boot.d` hook script for startup, both naming the binary from `system::binary::omarchist_binary` (the installed one on PATH wins over a `target/` build; never a `(deleted)` path) and refreshed at startup by `launcher::refresh_all`. Entries carry `X-Omarchist-Flow`, which the App picker skips so a flow can never be added to itself. `store::save_flow` keeps those files in sync with `flow.triggers`; `save_new_flow` refuses an id whose file exists; `delete_flow` removes them and any keybind override that ran the flow. `load_flows_with_broken` lists files that failed to parse so the page names them; `existing_ids` counts every file, readable or not.
- Templates (`templates.rs`): `TemplateSource::{BuiltIn, User(path)}`; user templates are `~/.config/omarchist/templates/<slug>.flow.toml` with keys `user:<slug>`, added by copying an exported file in (no UI writes them yet). A template never carries triggers (they belong to the machine; a test checks). A marketplace is a further source, never a new format. `requirements.rs` names the program of each command step and lists what is missing from `PATH`; the editor shows that as warnings, never as errors.
- Sharing (`share.rs`): `export_toml` strips the id and triggers; `read_import` reads a file or `https://` URL (64 KB cap, redirects followed but never off https, `meta.source` set for URLs) into an `Imported` flow with no id and no triggers, since both belong to the receiving machine. Import never saves or runs: the UI opens the editor as `FlowEditSource::Imported` with a review banner, and the CLI prints the steps and asks unless `--yes`. Keep it that way; it is the safety gate for third-party flows.
- UI: `src/ui/flows_page/` — `flows_view.rs` (card grid, one tab stop with a roving index; unreadable files named above it), `flow_edit_view.rs` (details, triggers, step list as one tab stop, live run states through a `smol::channel`; the dirty check compares against a `baseline` that is what the editor opened with or last saved, so a saved new flow reopens clean and a template is not dirty until edited; Ctrl+Q and closing the window go through the same check via `MainWindowView::request_quit`), `step_types.rs` (every kind of step as the picker lists it, `step_types()`: the kinds with their own form plus one `StepChoice::Do(&ActionDef)` per ready-made action, each with label, icon, `StepGroup`, search keywords; a group's `accent` colours its steps' icon tiles everywhere), `step_picker.rs` (the first screen of Add step: a search box whose Enter takes `best_match`, a row of group filters, and a grouped grid; each is one tab stop), `step_dialog.rs` (picker, then form; editing opens the form, and Change goes back to the picker) + `step_builder.rs` (the form: hosts `ActionBuilder` with its kind strip hidden via `set_kind_strip(false)`, and has the forms of the flow-only kinds; a variable picked from the Insert row goes where the cursor was in the field focused last; asking steps start with a default output name from `StepChoice::default_output`), `step_summary.rs` (titles, icons and group for steps, resolving installed apps and flow names; `StepSummary::tile` draws the tinted icon), `step_list.rs` (the editor's step list as a tree: `rows()` flattens the steps into lines (`RowKey::Step`, `Branch` headings for "Otherwise" and menu choices, an `Add` line per branch), drawn with a rail per level; the list is one tab stop whose selection is a `RowKey`, Enter edits a step or adds to a branch, Left/Right fold a block (`collapsed` holds block `uid`s), and run marks are kept by path), `app_picker.rs` (installed apps as a searchable list that yields a window class), `option_picker.rs` (a searchable list loaded in the background, used for themes), `templates.rs` (starter flows: `.flow.toml` files without an id, embedded from `defaults/flows/` through `DefaultAssets` and parsed with the same code as saved flows; a test validates every one, so adding a template is adding a file). Every new Flows feature ships with at least one built-in template that uses it (variables: Search selection, Daily note, Clipboard log, Where am I; asking: Quick note, Close an app, Wallpaper from a file, Empty the trash; blocks: Open copied link, Power by charger, Leave, Open my sites, Four pomodoros; actions: Look up text on screen, Weather now, Tidy copied text, Pick a color, and the older templates rewritten with named actions), so the Templates page doubles as a tour of what flows can do. Assigning a keybind opens `KeybindDialog` in `DialogMode::AddPreset`.

### Bar Widget and Single Instance

- **The bar widget** is a Quattro shell plugin in `defaults/plugin/tahayvr.omarchist/` (manifest + `BarWidget.qml`, embedded through `DefaultAssets`). `src/system/bar_widget.rs` installs it into `~/.config/omarchy/plugins/tahayvr.omarchist/` (plus a `command` file holding `current_exe()`), enables it with `omarchy plugin enable`, and refreshes the files at startup when the embedded version or the binary path changed (`ensure_current`, run from `main.rs` when `settings.bar_widget` is on). Enabled state is read from `shell.json`'s `bar.layout`, never stored twice. The shell keeps a widget instance until it restarts (plugin file changes and `reloadConfig` do not recreate it), which is why the binary path goes through a watched file and not a layout setting. The widget is a `Panel` + `KeyboardPanel` like Omarchy's first-party panels, and draws every icon as a font glyph so theme colors apply: Material Design Nerd Font glyphs, and `\ue900` in the `omarchy` font for the Omarchy logo. Flow icons map to glyphs through `flows::icon_glyph` (a test covers every `ICONS` entry), which `omarchist flow list --json` emits as `glyph`; the widget opens pages with `omarchist --view`. Bump the manifest version when the QML changes so `ensure_current` refreshes installed copies.
- `OMARCHIST_APP_ID` overrides the window's app id (class). Only the headless harness sets it (`.claude/skills/run-omarchist/headless.sh` with `APP_HOME`), to run a sandboxed second instance whose window rules and keys never reach the user's own window.
- **Single instance** (`src/system/instance.rs`): `main.rs` forwards `--view`/`--theme` as one JSON line over `$XDG_RUNTIME_DIR/omarchist.sock` to a running instance (which answers `ok`, navigates through `AppEvent::Navigate`, and activates its window) and exits; otherwise it binds the socket *before* the GPUI app starts (a launch that loses the bind forwards instead of opening a second window) and serves it from a background task. `accept` answers `Rejected` for a peer that sends nothing usable (the line is capped at 4 KB) so the loop keeps serving; only a listener error ends it. The socket is removed on quit only by the process that bound it; a stale one nobody answers is replaced.

## Key File Locations

- **Navigation:** `src/ui/app_view.rs`
- **Atomic writes:** `src/system/fs.rs` (`write_atomic`, used for every state file Omarchist owns: `settings.json`, `state.json`, `keybinds.json`, `omarchist.lua`, `hyprland.lua`, flow TOML, `omarchist.json`, `colors.toml`, `icons.theme`)
- **Selectable text:** `src/ui/text.rs`
- **Keyboard:** `src/ui/shortcuts.rs` (every binding), `src/ui/focus.rs` (tab stops, trap-aware focus moves, focusable switch, scroll-into-view), `src/ui/dialogs/shortcuts_dialog.rs`, `src/ui/dialogs/command_palette.rs`, `tests/keyboard_nav.rs`
- **Theme Creation:** `src/ui/dialogs/create_theme_dialog.rs`
- **Theme Editing:** `src/ui/theme_edit_page/theme_edit_view.rs`
- **Theme Management:** `src/system/themes/theme_management.rs`
- **Type Definitions:** `src/types/themes.rs`
- **Omarchy Paths:** `src/system/omarchy_paths.rs`
- **Omarchy version and updates:** `src/system/omarchy/updates.rs` shells out to `omarchy-version` and `omarchy-update-available` (never the stale `$OMARCHY_PATH/version` file or GitHub tags, which do not reflect the user's package channel); `src/ui/omarchy_page/updates.rs` is the one `OmarchyUpdates` model, owned by `MainTitleBar`, that the badge and the Omarchy page both observe. GitHub is used only for release notes.
- **Hyprland Config:** `src/system/hyprland_config/` (`manager.rs`, `lua_writer.rs`, `hyprctl_reader.rs`)
- **Keybinds:** `src/system/keybinds/` (`scan.lua`, `scanner.rs`, `replay.rs`, `overrides.rs`, `store.rs`, `submap.rs`) and `src/ui/keybinds_page/` (`keybinds_view.rs`, `keybinds_table.rs`, `keystroke_input.rs`, `keybind_dialog.rs`)
- **Flows:** `src/system/flows/` (`store.rs`, `runner.rs`, `actions.rs`, `condition.rs`, `prompt.rs`, `vars.rs`, `launcher.rs`, `templates.rs`) and `src/ui/flows_page/` (`flows_view.rs`, `flow_edit_view.rs`, `step_list.rs`, `step_dialog.rs`, `step_picker.rs`, `step_types.rs`, `step_builder.rs`, `step_summary.rs`), tests in `tests/flows_ui.rs`
- **App settings:** `src/system/config/config_setup.rs`, `src/ui/settings_page/settings_view.rs`
- **Bar widget and single instance:** `src/system/bar_widget.rs`, `defaults/plugin/tahayvr.omarchist/`, `src/system/instance.rs`

## CLI Handling

The app supports command-line arguments via `clap`. CLI args are parsed at startup and determine the initial page.

```rust
// src/cli.rs
#[derive(Parser)]
pub struct CliArgs {
    #[arg(short, long)]
    pub view: Option<ViewOption>,  // themes, config, keybinds, flows, settings, about, omarchy
    
    #[arg(short, long, requires = "view")]
    pub theme: Option<String>,     // For editing specific theme
}
```

**Usage examples:**
```bash
omarchist --view config              # Open Hyprland configuration
omarchist --view themes --theme foo  # Open theme editor for "foo"
omarchist --view settings            # Open settings page
omarchist --view keybinds            # Open the Keybinds page
omarchist --view flows               # Open the Flows page
omarchist flow run morning-start     # Run a flow without opening the window
omarchist flow list
```

Subcommands (`CliArgs::command`) are handled by `cli::run_command` in `main()` before the GPUI app starts, so they never open a window. `--theme` is refused unless `--view themes` is given (`CliArgs::parse_args`).

`omarchist uninstall` (`src/system/uninstall.rs`) lists and removes everything the app puts outside its package, in an order that never leaves a dangling `require`: the require line and `omarchist.lua`, the bar plugin, flow launcher files, `~/.config/omarchist`, `~/.local/share/omarchist`. Themes stay. It refuses while a window is running (`instance::is_running`) and stops at the first failure. Anything new that writes outside `~/.config/omarchist` needs a `Step` there.

Startup checks `omarchy_paths::is_quattro_installed()` (`$OMARCHY_PATH/default/hypr/bootstrap.lua`): on anything else the Hyprland hook and the bar plugin are not touched and the window shows a notification. The About page's "Copy debug info" (`about_view::debug_info`) gathers the versions a bug report needs.

To extend: add variants to `ViewOption` enum and handle them in `cli_args_to_active_page()`.

## Dependencies

Always use `cargo add` to add dependencies. Key crates:
- `gpui` (= `gpui-pre`), `gpui_platform` (= `gpui-pre-platform`), `gpui-component`, `gpui-base`, `gpui-kit`, `gpui-kit-assets` - UI framework (GPUI Kit; keep the pins in step)
- `smol` - Async runtime
- `serde`, `serde_json` - Serialization
- `dirs` - Cross-platform directories
- `chrono` - Date/time
- `clap` - CLI argument parsing

## External Resources

- [GPUI Kit](https://github.com/longbridge/gpui-kit) and [gpui-kit.com](https://gpui-kit.com) (component, base, and headless-testing docs)
- [GPUI docs](https://github.com/zed-industries/zed/tree/main/crates/gpui)

## Rules for Omarchist Documentation Site

path: /docs

- Documentation is written in second person, present tense. example: "You do this", "You can do that".
- Use active voice. example: "You create a file", not "A file is created".
- Use consistent terminology.
- Use simple, clear language.
- Use short sentences and paragraphs.
- Use lists and tables to organize information.
- Use headings and subheadings to break up text.
- Use code blocks for code snippets.
- Use links to reference other documentation.
- Use images and diagrams to illustrate concepts.
- Use examples to clarify complex concepts.
- Use a friendly and approachable tone.
- Use the `<span class="icon-inline icon-inline-copy"></span>` pattern for inline icons display so the icon inherits the current text color.
