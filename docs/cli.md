---
outline: deep
---

# Command Line

```bash
omarchist [--view <page>] [--theme <name>]
omarchist theme from-image <image> [--name <name>] [--apply]
omarchist flow <run | stop | history | list | check | export | import> ...
omarchist automations <on | off | status | run>
omarchist uninstall [--yes]
```

## Open a page

`--view` (`-v`) opens Omarchist on a page: `themes`, `config`, `keybinds`, `flows`, `settings`, `about`, or `omarchy`. `--theme` (`-t`) with `--view themes` opens that theme in the Theme Designer.

Omarchist runs one window at a time. If it is already running, a new `omarchist --view <page>` brings that window forward on the page and exits, so a keybind or the [bar widget](/configuring/bar-widget) never opens a second window. Without `--view`, Omarchist opens on the page chosen in Settings.

```bash
omarchist --view keybinds
omarchist --view themes --theme my-theme
```

## Themes

Runs without opening the window.

| Command | What it does |
| --- | --- |
| `theme from-image <image> [--name <name>] [--apply]` | Makes a theme from a picture, the same way **Select image** does: extracts the palette and copies the picture in as the wallpaper. The name defaults to the file name. `--apply` switches to it right away. |

```bash
omarchist theme from-image ~/Pictures/dunes.jpg
omarchist theme from-image ~/Pictures/dunes.jpg --name "Sahara" --apply
```

## Flows

These run without opening the window.

| Command | What it does |
| --- | --- |
| `flow run <name or id> [-- <input>...]` | Runs the flow and prints each step. The words after `--`, or text piped in, are the flow's [input](/flows/input). Exits with status 1 and sends a notification if a step fails (a program that cannot start counts, even for a step the flow does not wait for), so a keybind never fails silently. |
| `automations <on \| off \| status>` | Turns the background service for [automations](/flows/automations) on or off, or says whether it runs and what it waits for. |
| `flow stop <name or id>` | Ends the flow's runs, whatever started them. See [Run history](/flows/history#stop-a-running-flow). |
| `flow history <name or id> [--json]` | The flow's last runs: how each ended, when, how long it took, and what started it. `--json` prints every kept run with its steps. |
| `flow list` | Every flow with its id and step count. `--json` prints an array of `{id, name, icon, glyph, steps, running}` for scripts and the bar widget; `glyph` is the icon as a Nerd Font character. |
| `flow check <file>... [--json] [--catalog]` | Says whether each file is a valid flow, lists its steps and the programs it needs, and points at steps worth a closer look, such as one that runs as administrator. `--catalog` also applies the [Catalog](/flows/catalog)'s rules. Exits with status 1 if a file fails. |
| `flow export <name or id> [--output <path>]` | Writes the flow as a shareable `.flow.toml` file, to stdout or to a file or directory. |
| `flow import <file or https URL> [--yes]` | Prints the flow's steps and saves it after you confirm. `--yes` skips the question. |

```bash
omarchist flow run "Morning start"
omarchist flow run search-selection -- rust traits
omarchist flow export morning-start --output ~/Downloads
omarchist flow import https://example.com/focus.flow.toml
```

See [Flows](/flows/) for what a flow is and [Sharing flows](/flows/#sharing-flows) for the file format.

## Uninstall

`omarchist uninstall` removes everything Omarchist added outside its package, in this order, and stops at the first failure so nothing is left half done:

1. The `require("hypr.omarchist")` line in `~/.config/hypr/hyprland.lua` and `~/.config/hypr/omarchist.lua` (Hyprland is reloaded, so your settings and keybinds go back to Omarchy's).
2. The bar widget, taken off the bar and deleted from `~/.config/omarchy/plugins`.
3. The service that runs [automations](/flows/automations), stopped and removed.
4. Every flow's launcher entry, icon, startup hook, and Files menu script.
5. `~/.config/omarchist` (settings, Hyprland state, keybind overrides, flows, templates), `~/.local/share/omarchist`, `~/.local/state/omarchist` (the flows' run history), and `~/.cache/omarchist` (the copy of the Catalog).

It prints that list and asks first; `--yes` skips the question. Quit Omarchist before you run it, because a running window would write its files back on the next save. Themes you made stay in `~/.config/omarchy/themes`. Remove the package afterwards with `sudo pacman -R omarchist-bin`.
