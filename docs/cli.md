---
outline: deep
---

# Command Line

```bash
omarchist [--view <page>] [--theme <name>]
omarchist theme from-image <image> [--name <name>] [--apply]
omarchist flow <run | list | export | import> ...
```

## Open a page

`--view` (`-v`) opens Omarchist on a page: `themes`, `config`, `keybinds`, `flows`, `settings`, `about`, or `omarchy`. `--theme` (`-t`) with `--view themes` opens that theme in the Theme Designer.

```bash
omarchist --view keybinds
omarchist --view themes --theme my-theme
```

## Themes

Runs without opening the window.

| Command | What it does |
| --- | --- |
| `theme from-image <image> [--name <name>] [--apply]` | Makes a theme from a picture, the same way **Select Image** does: extracts the palette and copies the picture in as the wallpaper. The name defaults to the file name. `--apply` switches to it right away. |

```bash
omarchist theme from-image ~/Pictures/dunes.jpg
omarchist theme from-image ~/Pictures/dunes.jpg --name "Sahara" --apply
```

## Flows

These run without opening the window.

| Command | What it does |
| --- | --- |
| `flow run <name or id>` | Runs the flow and prints each step. Exits with status 1 and sends a notification if a step fails, so a keybind never fails silently. |
| `flow list` | Every flow with its id and step count. |
| `flow export <name or id> [--output <path>]` | Writes the flow as a shareable `.flow.toml` file, to stdout or to a file or directory. |
| `flow import <file or https URL> [--yes]` | Prints the flow's steps and saves it after you confirm. `--yes` skips the question. |

```bash
omarchist flow run "Morning start"
omarchist flow export morning-start --output ~/Downloads
omarchist flow import https://example.com/focus.flow.toml
```

See [Flows](/flows/) for what a flow is and [Sharing flows](/flows/#sharing-flows) for the file format.
