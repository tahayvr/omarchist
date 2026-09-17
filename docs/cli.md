---
outline: deep
---

# Command Line Interface

Omarchist supports command-line arguments to control the initial view when launching the application, and a `flow` command that runs [flows](/flows/) without opening the window.

## Usage

```bash
omarchist [OPTIONS]
omarchist flow <run <NAME> | list>
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
| `keybinds` | Keybinds page |
| `flows` | Flows page |
| `about` | About page |
| `omarchy` | Omarchy page |

## Examples

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

### Open the Keybinds Page

```bash
omarchist --view keybinds
```

## Flows

`omarchist flow run` runs a flow made on the Flows page by its id or its name, prints each step as it runs, and exits with status 1 if a step fails (a desktop notification reports the failure too, so a keybind never fails silently). `omarchist flow list` prints every flow with its id.

```bash
omarchist flow run 'morning-start'
omarchist flow run "Morning start"
omarchist flow list
```
