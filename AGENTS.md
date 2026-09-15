# Omarchist - Agent Context Document

## Build/Test Commands

```bash
# Build and run
cargo build
cargo check
cargo run

# Testing
cargo test                 # Run all tests
cargo test <test_name>     # Run single test
cargo test <module>::      # Run tests in module

# Code quality
cargo clippy              # Run linter
cargo fmt                 # Format code

# Adding dependencies
cargo add <crate_name>
```

## Tech Stack

- **Rust Edition:** 2024
- **UI Framework:** GPUI 0.2.2 (GPU-accelerated)
- **Components:** gpui-component 0.5.1
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

### GPUI Actions

Define actions with derive macro:

```rust
#[derive(Clone, PartialEq, Action)]
#[action(no_json)]
pub struct NavigateToThemes;
```

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
    Settings,
    About,
    Omarchy,
}
```

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

## Key File Locations

- **Navigation:** `src/ui/app_view.rs`
- **Theme Creation:** `src/ui/dialogs/create_theme_dialog.rs`
- **Theme Editing:** `src/ui/theme_edit_page/theme_edit_view.rs`
- **Theme Management:** `src/system/themes/theme_management.rs`
- **Type Definitions:** `src/types/themes.rs`
- **Omarchy Paths:** `src/system/omarchy_paths.rs`
- **Hyprland Config:** `src/system/hyprland_config/` (`manager.rs`, `lua_writer.rs`, `hyprctl_reader.rs`)

## CLI Handling

The app supports command-line arguments via `clap`. CLI args are parsed at startup and determine the initial page.

```rust
// src/cli.rs
#[derive(Parser)]
pub struct CliArgs {
    #[arg(short, long)]
    pub view: Option<ViewOption>,  // themes, config, settings, about, omarchy
    
    #[arg(short, long, requires = "view")]
    pub theme: Option<String>,     // For editing specific theme
}
```

**Usage examples:**
```bash
omarchist --view config              # Open Hyprland configuration
omarchist --view themes --theme foo  # Open theme editor for "foo"
omarchist --view settings            # Open settings page
```

To extend: add variants to `ViewOption` enum and handle them in `cli_args_to_active_page()`.

## Dependencies

Always use `cargo add` to add dependencies. Key crates:
- `gpui`, `gpui-component` - UI framework
- `smol` - Async runtime
- `serde`, `serde_json` - Serialization
- `dirs` - Cross-platform directories
- `chrono` - Date/time
- `clap` - CLI argument parsing

## External Resources

- [gpui-component](https://github.com/longbridge/gpui-component)
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
