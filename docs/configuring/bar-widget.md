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

- **Click** the widget to open the dropdown. The top lists your flows with their icons. Click one to run it.
- Below the flows, the **Open Omarchist** buttons open the app on that page. If Omarchist is already running, its window comes forward on that page instead of a second window opening.
- **Right-click** the widget to open the Flows page.

The list updates as you add, rename, or delete flows in Omarchist.

## Change the icon

The widget shows a Nerd Font glyph. Change it with the bar's settings for the widget:

```bash
omarchy bar set tahayvr.omarchist icon "󰐊"
```

## How it works

The widget is a `BarWidget` from Omarchy's `qs.Ui`, so it takes the bar's colors and font. It runs `omarchist flow list --json` to read your flows and `omarchist flow run <id>` to run one. Omarchist writes the path of its own binary into a `command` file in the widget's folder when it installs the widget, and the widget watches that file, so it works even when `omarchist` is not on your `PATH` and follows the binary when Omarchist moves. When you update Omarchist, the app refreshes the widget's files on its next start; the shell loads new widget files when it restarts.
