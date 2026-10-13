Omarchist is a desktop app for [Omarchy](https://omarchy.org/) Linux. Design themes, tune Hyprland, change keybinds, and chain actions into flows, all without editing config files by hand.

<img src="/images/themes-light.webp" alt="Omarchist" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Omarchist" class="screenshot dark-only">

## Features

- **[Themes](/theming/)**: design a theme from scratch or from an image. Omarchist writes `colors.toml`; Omarchy generates the rest.
- **[Configuration](/configuring/)**: gaps, borders, blur, keyboard, mouse, and touchpad, applied as you change them.
- **[Keybinds](/configuring/keybinds)**: search and change every Hyprland keybind, recording keys the way an editor does.
- **[Flows](/flows/)**: chain actions like Shortcuts on a Mac and run them from a keybind, the app launcher, at startup, or the command line. Let [automations](/flows/automations) start them, and find more in the [Catalog](/flows/catalog).
- **[Keyboard first](/keyboard)**: every page and dialog works without a mouse.

## Installation

```bash
yay -S omarchist-bin
```

::: tip
Omarchist 2.x requires Omarchy Quattro (v4). Use an Omarchist 1.x release with Omarchy 3.x.
:::

### Uninstall

Omarchist adds a few things outside its package: a `require("hypr.omarchist")` line in `~/.config/hypr/hyprland.lua`, the `~/.config/hypr/omarchist.lua` it generates, the bar widget plugin, launcher entries and startup hooks for flows, the service that runs automations, and its own settings under `~/.config/omarchist`. Remove them first, then the package:

```bash
omarchist uninstall
sudo pacman -R omarchist-bin
```

`omarchist uninstall` lists what it will remove and asks before it does. Themes you made stay in `~/.config/omarchy/themes`; they are ordinary Omarchy themes and keep working. See [Command Line](/cli#uninstall).

## Settings

**Settings** holds the app's own options. It opens over whatever page you are on: press <kbd>Ctrl</kbd> + <kbd>,</kbd>, choose **Settings** in the Omarchist menu or the command palette, or run `omarchist --view settings`. The options live in `~/.config/omarchist/settings.json`, and an Omarchist update adds new options without touching the ones you set.

<img src="/images/settings-light.webp" alt="Settings dialog" class="screenshot light-only">
<img src="/images/settings-dark.webp" alt="Settings dialog" class="screenshot dark-only">

Pick a page from the list on the left, or type in the search box to see every option that matches, on all pages at once. On the list, <kbd>↑</kbd> and <kbd>↓</kbd> move between pages and <kbd>Enter</kbd> takes you to the page's options. Every change saves at once.

| Page | Options |
| --- | --- |
| **General** | Font size. Look: **Omarchy** colors the app with your desktop theme's palette, including its accent and terminal colors; **Light** and **Dark** use Omarchist's own. The gear menu's Appearance switch (<kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>L</kbd> / <kbd>D</kbd>) sets the same option. Open on: the page Omarchist opens on when you start it without `--view` (the Dashboard unless you pick another), or the page you used last. Logs: how long the [log](/logs) is kept, and **Open folder**. |
| **Themes** | Auto-apply theme on edit: apply a theme to your desktop as soon as you open it in the Theme Designer. |
| **Flows** | A desktop notification when a flow run from a keybind or the command line finishes. Run [automations](/flows/automations) in the background. Count your installs in the [Catalog](/flows/catalog#privacy). |
| **Omarchy** | Whether the app checks for Omarchy updates in the background, how often, and whether a found update raises a desktop notification. Show Omarchist in Omarchy's bar; see [Bar Widget](/configuring/bar-widget). |
