---
outline: deep
---

# Command Line Interface

Omarchist supports command-line arguments to control the initial view when launching the application.

## Usage

```bash
omarchist [OPTIONS]
```

## Options

| Option | Short | Description |
|--------|-------|-------------|
| `--view <VIEW>` | `-v` | Open a specific page on startup |
| `--theme <NAME>` | `-t` | Specify a theme to edit (requires `--view`) |

## View Options

You can open Omarchist directly to any page:

| View | Description |
|------|-------------|
| `themes` | Themes page |
| `settings` | Settings page |
| `config` | Hyprland Configuration |
| `about` | About page |
| `omarchy` | Omarchy page |

## Examples

<!--### Open System Monitor

```bash
omarchist --view system
```-->

### Open Themes Page

```bash
omarchist --view themes
```

### Edit a Specific Theme

```bash
omarchist --view themes --theme my-custom-theme
```

### Open Hyprland Configuration

```bash
omarchist --view config
```

<!--## Use Cases

- **Keyboard shortcuts**: Bind a key combination to `omarchist --view system` for quick system monitoring
- **Automation**: Launch Omarchist to specific pages from scripts-->
