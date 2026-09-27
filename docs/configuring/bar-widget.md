---
outline: deep
---

# Bar Widget

Omarchist ships a widget for Omarchy's bar. It lists your flows so you can run one with a click, and it opens Omarchist on any page.

## Turn it on

Open **Settings** (<kbd>Ctrl</kbd> + <kbd>,</kbd>) and switch on **Show Omarchist in the Bar**. Omarchist installs the widget as a shell plugin under `~/.config/omarchy/plugins/tahayvr.omarchist/`, asks the shell to load it, and puts it in the right section of the bar.

Move it like any other widget with the bar's layout tools, for example:

```bash
omarchy bar move tahayvr.omarchist --section left --index 1
```

Switch the setting off to take the widget off the bar. The files stay, so switching it back on is instant. To delete them too, run `omarchy plugin remove tahayvr.omarchist`.

## Use it

- **Click** the widget to open its panel.
- The row of icons opens Omarchist on a page: Themes, Configuration, Keybinds, Flows, Omarchy, and Settings. Hover an icon to see its name. If Omarchist is already running, its window comes forward on that page instead of a second window opening.
- Below the icons, **Flows** lists your flows with their icons and step counts. Click one to run it.
- **Right-click** the widget to open Omarchist.

The panel works from the keyboard too. Use the arrow keys to move between the icons and the flows, <kbd>Enter</kbd> to open or run, and <kbd>Esc</kbd> to close.

The list updates as you add, rename, or delete flows in Omarchist.

## Change the icon

The widget shows a Nerd Font glyph. Change it with the bar's settings for the widget:

```bash
omarchy bar set tahayvr.omarchist icon "󰐊"
```

## How it works

The widget is a panel built from Omarchy's own `qs.Ui` parts, like the Power and Agents panels. Every icon is a Nerd Font glyph, so it takes the bar's colors and font and follows your theme. It runs `omarchist flow list --json` to read your flows and `omarchist flow run <id>` to run one. Omarchist writes the path of its own binary into a `command` file in the widget's folder when it installs the widget, and the widget watches that file, so it works even when `omarchist` is not on your `PATH` and follows the binary when Omarchist moves. When you update Omarchist, the app refreshes the widget's files on its next start; the shell loads new widget files when it restarts.
