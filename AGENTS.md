# Omarchist - Agent Context Document

## Build/Test Commands

```bash
# Build and run
cargo build
cargo check
cargo run

# Testing
cargo test                 # Run all tests (unit + headless UI tests)
cargo test <test_name>     # Run single test
cargo test <module>::      # Run tests in module
cargo test --test keyboard_nav   # Headless keyboard-navigation tests only

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
- **Serialization:** serde + serde_json
- **Date/Time:** chrono

## Configuration Directories

Important distinction between two config directories:

- **`~/.config/omarchy`** - Belongs to Omarchy Linux system. Used to store themes created by Omarchist app (the OS reads themes from here)
- **`~/.config/omarchist`** - Belongs to the Omarchist app itself. Used for app operations (settings.json, Hyprland settings state)

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

Fallible code in `system/` and `shell/` returns `crate::error::Result<T>`, whose error is the `thiserror` enum in `src/error.rs` (`Io`, `Json`, `ThemeNotFound`, `ThemeExists`, `UnknownDirectory`, `Network`, `Invalid`). Never return `Result<T, String>`.

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

`MainWindowView::new` registers `cx.observe_global_in::<AppEvents>` and drains the queue in `handle_app_event`. Add a variant to `AppEvent` and a match arm there for any new request. Local UI state still lives on the entity and is updated with `cx.notify()`.

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
    Settings,
    About,
    Omarchy,
}
```

### Focus and Keyboard Navigation

GPUI focus is the only source of truth; never keep a shadow "focused index" that is not backed by a `FocusHandle`. Helpers live in `src/ui/focus.rs`:

- Every interactive element is a tab stop (`focus::tab_stop(cx)` or gpui-component's `Button`/`Input`/`Select`/`Radio`/`ColorPicker`). `Switch` is mouse-only: use `FocusableSwitch`.
- Composites (sidebar, tab strips, theme grid, keybinds table and filters, config section list) are one tab stop with a roving index; their arrow keys are actions in their own `key_context`. Tab never stops on individual items inside them.
- Each page exposes `focus_entry(&self, window, cx)`; `MainWindowView::navigate_to` calls it so keyboard users land on the first control. `Escape` (`focus::EscapeToSidebar`) toggles between the sidebar and the page.
- Dialogs wrap their content in `focus::dialog_body(...)` (handles `dialog::Submit` = Ctrl+Enter) and call `focus::focus_first_in(&body_focus, window, cx)` after `open_dialog`. Tab is trapped by gpui-kit itself: every dialog is a `focus_trap`, and `Root`'s Tab handler stays inside the active trap. Where a control's own `tab` binding is overridden (`... > Input`), route it to `focus::FocusNext`/`FocusPrev`, whose handler (`focus_next_trapped`) honours the trap the same way.
- Commands dispatched from inside a dialog (the palette) never reach `MainWindowView`'s handlers, because the dialog is rendered by `Root` outside the main view's element path. Close the dialog and dispatch through the main view's `FocusHandle::dispatch_action` instead.
- Headless UI tests (`tests/keyboard_nav.rs`) follow the GPUI Kit testing guide (gpui-kit.com/docs/test): `#[gpui_kit::test]`, types from the Kit root (`gpui_kit::{TestAppContext, ..}`, `gpui_kit::component::Root`), `cx.update(gpui_kit::init)`, `cx.open_window(size, ..)` around the production `Root`/`MainWindowView`, `window.render_frame(cx)` before the first query, `TestWindowExt` for `find`/`press`/`input`, `wait_for` for async work, and model state checked next to UI snapshots. Elements that tests query carry a stable `.id(..)` followed by `.test_support()` *before* `.track_focus(..)` (`SidebarNav`, `focus::dialog_body`); `test_support()` is inert in normal builds but wraps the element under `test-support`, so such helpers return `impl ParentElement + StatefulInteractiveElement + ..` rather than `Stateful<Div>`. The window must be free of network tasks (the Omarchy update watcher starts from `main.rs`) and background work runs on gpui's executor (`cx.background_spawn`, not `smol::unblock`) so the test scheduler can drive it.
- Scrolling content wraps sections in `FocusSection` so a focused section scrolls into view.
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
3. Writes `colors.toml` (the theme's source of truth) plus any opted-in advanced overrides (`btop.theme`, `chromium.theme`, `shell.lock.toml`). Terminal configs, window border colors, the bar, and notifications are template-generated by Omarchy itself from `colors.toml` — Omarchist does not write those files directly (see "Theme System (Quattro)" below).

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
- **`SidebarMenuItem` is not an element on its own** — it implements `SidebarItem`; render it with `.render(id, window, cx)` outside a `Sidebar`/`SidebarMenu`. It is `Styled`, so focus rings go on the item itself. `Sidebar::new(id).side(..)`; `.suffix()` takes a builder closure.
- **`Select` renders a full-width root** — box it in a fixed-width `flex_none` div or it squeezes its siblings.
- **`h_flex()` centres its children** (`items_center`), and gpui has no `items_stretch`; use `div().flex().flex_row()` when a child must fill the row's height (e.g. a scroll container).
- **`AlertDialog`** can only be opened from its own trigger element (`with_base_alert_dialog` is crate-private), so programmatic confirmations use `dialogs::confirm_dialog`.
- **`FocusHandle::focus`, `Window::focus_next/prev` take `cx`**; `Entity::update` on an `AsyncApp` is infallible (it panics once the app is gone, which cannot happen to a foreground task); `ScrollHandle::max_offset()` is a `Point`.
- **Key contexts:** the data table's context is `DataTable` (not `Table`); `Input` binds `tab`, `up`, `down`, `escape`; `Root` binds `tab`, `shift-tab`, `ctrl-c`.
- **Library dialogs bind Enter to `Confirm`**, which closes them before a focused button gets its keyboard click. `focus::dialog_body` stops the action and `shortcuts::key_bindings` adds a `NoAction` Enter binding in `DialogBody`, so Enter reaches the focused control; `Input` and `Select` keep their own deeper Enter bindings (`Select` re-propagates `Confirm` after opening).
- **`gpui_kit_assets` embeds only the component default icon subset** (`default-icons.txt`, about 100 icons); any other Lucide icon is copied into `assets/icons/` and used through `Icon::new(Icon::empty()).path(..)`.
- **Inline code in `TextView::markdown` (gpui-base 0.6.1)** is drawn over the neighbouring words of its line. Fixed upstream after 0.6.1 (gpui-kit PRs #3038 and #3046); goes away with the next pinned bump.

## Theme System (Quattro)

Omarchist targets Omarchy Quattro (v4) only — see `tahayvr/omarchist#39`. Quattro replaced Waybar/Mako/Walker/hyprlock/SwayOSD/hypridle/swaybg with one Quickshell-based desktop shell, and collapsed theme folders down to a handful of files driven by a semantic `colors.toml`. Confirmed against the real `omacom/omarchy` source (not docs):

- **`colors.toml` is the theme's sole source of truth.** `ColorsConfig` (`src/types/themes.rs`) writes `mode`, `accent`, `cursor`, `foreground`, `background`, `selection_foreground`/`selection_background`, and `color0`-`color15`. Omarchy's own `omarchy-theme-color` resolver derives everything else (shades, named hues, mode auto-detection) from this set — Omarchist does not need to compute or store those derived values.
- **Omarchist never hand-writes template-generated files** — terminal configs (`alacritty.toml`/`foot.ini`/`kitty.conf`/`ghostty.conf`), window border colors, the bar, and notifications are all generated by Omarchy's `omarchy-theme-set-templates` from `colors.toml` after `omarchy-theme-set` runs. Reimplementing that template engine would be a permanently-drifting duplicate of logic Omarchy already owns.
- **Optional override files** `btop.theme` (`BtopConfig`), `chromium.theme` (`BrowserConfig`), `shell.lock.toml` (`LockScreenConfig`) are all template-generated by Quattro from `colors.toml` when absent (`default/themed/*.tpl`, and the `[lock]` section of `shell.toml.tpl`). `apps.{btop,chromium,lock}` is therefore `None` by default and means "no file, Omarchy generates it"; the Overrides tab (`overrides_tab.rs`) seeds a config from the palette via `default_{btop,browser,lock}_config` when switched on, and `save_theme_data` **deletes the file** when it goes back to `None`. On load, the file on disk is the source of truth for whether an override is on.
- **Hyprland border colors** are the two optional `colors.toml` keys Quattro's `hyprland.lua.tpl` reads: `hyprland_active_border` and `hyprland_inactive_border` (`ColorsConfig`, edited as free text on the Colors tab because they accept gradients). There is no per-theme `hyprland.lua` written by Omarchist.
- **Tabs each hold a snapshot of the theme.** They must save through `theme_management::update_theme(name, |theme| ...)`, which reloads from disk and applies only the fields that tab owns, never `save_theme_data` with the tab's whole snapshot.
- **Opaque pass-through, never parsed**: `neovim.lua` and `vscode.json` are edited as raw text (`editor_tab.rs`) since `neovim.lua` is real Lua — never parse or structurally rewrite it. Both are *optional overrides*: Quattro's `default/themed/neovim.lua.tpl` and `vscode-theme.json.tpl` generate palette-matched editor themes from `colors.toml` whenever the theme folder lacks them, so new themes ship neither file and a blank editor deletes the override.
- **`light.mode` is legacy.** Omarchy's `omarchy-theme-color` only consults it as a fallback behind the `mode` key in `colors.toml`. Omarchist always writes `mode`, never writes `light.mode`, and deletes a stale one on save; it is still honored on load for themes written by Omarchist 1.x.
- **The Status Bar / Waybar feature was removed entirely** (not ported) — Quattro's own `omarchy bar` tooling and shell now own bar editing natively, so Omarchist no longer needs to.

### Hyprland Config — Lua Ownership

Quattro moved Hyprland's config from hyprlang text to real Lua (`~/.config/hypr/hyprland.lua`). Omarchist **never parses the user's Lua** — confirmed mechanism from the real `config/hypr/hyprland.lua`/`default/hypr/bootstrap.lua`:

- Omarchist's own settings round-trip through `~/.config/omarchist/hyprland/state.json` (plain `serde_json`, see `src/system/hyprland_config/manager.rs`) — this is the only file ever read back.
- On every save, `src/system/hyprland_config/lua_writer.rs` generates a write-only `~/.config/hypr/omarchist.lua` containing `hl.config({ <section> = {...} })` calls — Hyprland's own real Lua config API, confirmed against `config/hypr/looknfeel.lua`/`input.lua`. It diffs the config against `HyprlandConfig::default()` via JSON (`serde_json::to_value`) and only emits changed keys, rather than re-encoding every field's default a second time by hand.
- `src/system/config/hypr_setup.rs` appends an idempotent `require("hypr.omarchist")` line to the user's `hyprland.lua`, after the existing `require("hypr.autostart")` line — this is Omarchy's own sanctioned convention for user config modules (`package.path` already includes `~/.config/?.lua`).
- `src/system/hyprland_config/hyprctl_reader.rs` still live-introspects settings via `hyprctl getoption` — untouched, version-agnostic.
- The pre-Quattro `~/.config/omarchist/hyprland/hyprland.conf` (hyprlang, sourced by glob) is gone; `config_setup.rs` deletes a stale copy on startup.

### Keybinds — Lua Scanner + Overrides

Omarchy declares every keybind in Lua (`o.bind(keys, description, dispatcher, opts)` over Hyprland's `hl.bind`), and `hyprctl binds` cannot describe them (every dispatcher is `__lua` with an opaque id; `code:N` keys come back empty). The Keybinds page therefore works like Omarchy's own `omarchy-menu-keybindings`:

- `src/system/keybinds/scan.lua` (embedded with `include_str!`) is run by the system `lua` interpreter against `~/.config/hypr/hyprland.lua` with a stubbed `hl` table. It prints one tab-separated record per `hl.bind`/`hl.unbind` call, with the declaring file (skipping Omarchy's `helpers.lua` frames), the keys, description, dispatcher (`exec` command, `lua` expression reconstructed as source text, or `fn`), and flags. `hl.get_*` getters answer `nil`; everything else is a noop.
- `replay.rs` turns those events into the effective list with Hyprland's semantics: binds on one chord stack, and `hl.unbind` removes every earlier bind on that chord. Origin is classified from the path: `$OMARCHY_PATH/default/` → Default, `omarchist.lua` → Omarchist, otherwise User.
- User changes are overrides in `~/.config/omarchist/hyprland/keybinds.json` (`overrides.rs`: Rebind / Disable / Add, each targeting a bind by chord + description + dispatcher, with the sibling binds to restore captured at save time). `emit_keybinds_lua` renders them as `hl.unbind(...)` + `hl.bind(...)` lines that `manager::write_omarchist_lua` appends to `omarchist.lua` after the settings section, so the Configuration page and the Keybinds page can never drop each other's block. Omarchist never edits the user's `bindings.lua`. `validate()` only accepts Lua dispatchers of the form `hl.dsp.*(...)` so a hand-edited json cannot inject code.
- `omarchist.lua` always defines the `omarchist-recording` submap (one inert switch bind, because Hyprland only registers submaps that contain a bind). The recorder (`keystroke_input.rs`, ported from Zed's `KeystrokeInput`) switches Hyprland into it with `hyprctl dispatch 'hl.dsp.submap("omarchist-recording")'` while recording, so bound chords reach the app, and resets it on stop, blur, drop, and app quit (`submap.rs`). On this Hyprland `hyprctl dispatch` takes Lua syntax; the old `submap name` form is rejected.
- Recording uses `cx.intercept_keystrokes` + `cx.stop_propagation()` installed on inner-focus-in and dropped on focus-out, so `is_recording()` is derived from focus and cannot desync. A Hyprland bind is one chord, so recording stops after the first complete keystroke.
- GPUI key names are mapped to Hyprland keysym names in `keymap.rs` (shifted symbols back to base key + SHIFT, Omarchy's spellings such as `RETURN`/`comma`, XF86 names canonicalised via `xkbcommon`). Chords are parsed and compared in `chord.rs` (`code:10` ≡ `1`).

## Key File Locations

- **Navigation:** `src/ui/app_view.rs`
- **Keyboard:** `src/ui/shortcuts.rs` (every binding), `src/ui/focus.rs` (tab stops, trap-aware focus moves, focusable switch, scroll-into-view), `src/ui/dialogs/shortcuts_dialog.rs`, `src/ui/dialogs/command_palette.rs`, `tests/keyboard_nav.rs`
- **Theme Creation:** `src/ui/dialogs/create_theme_dialog.rs`
- **Theme Editing:** `src/ui/theme_edit_page/theme_edit_view.rs`
- **Theme Management:** `src/system/themes/theme_management.rs`
- **Type Definitions:** `src/types/themes.rs`
- **Omarchy Paths:** `src/system/omarchy_paths.rs`
- **Hyprland Config:** `src/system/hyprland_config/` (`manager.rs`, `lua_writer.rs`, `hyprctl_reader.rs`)
- **Keybinds:** `src/system/keybinds/` (`scan.lua`, `scanner.rs`, `replay.rs`, `overrides.rs`, `store.rs`, `submap.rs`) and `src/ui/keybinds_page/` (`keybinds_view.rs`, `keybinds_table.rs`, `keystroke_input.rs`, `keybind_dialog.rs`)

## CLI Handling

The app supports command-line arguments via `clap`. CLI args are parsed at startup and determine the initial page.

```rust
// src/cli.rs
#[derive(Parser)]
pub struct CliArgs {
    #[arg(short, long)]
    pub view: Option<ViewOption>,  // themes, config, keybinds, settings, about, omarchy
    
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
```

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
