Omarchist is a desktop app for [Omarchy](https://omarchy.org/) Linux. Design themes, tune Hyprland, change keybinds, and chain actions into flows, all without editing config files by hand.

<img src="/images/themes-light.webp" alt="Omarchist" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Omarchist" class="screenshot dark-only">

## Features

- **[Themes](/theming/)**: design a theme from scratch or from an image. Omarchist writes `colors.toml`; Omarchy generates the rest.
- **[Configuration](/configuring/)**: gaps, borders, blur, keyboard, mouse, and touchpad, applied as you change them.
- **[Keybinds](/configuring/keybinds)**: search and change every Hyprland keybind, recording keys the way an editor does.
- **[Flows](/flows/)**: chain actions like Shortcuts on a Mac and run them from a keybind, the app launcher, at startup, or the command line. Let [automations](/flows/automations) start them, and find more in the [Gallery](/flows/gallery).
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

The **Settings** page (<kbd>Ctrl</kbd> + <kbd>,</kbd>) holds the app's own options. They live in `~/.config/omarchist/settings.json`, and an Omarchist update adds new options without touching the ones you set.

<img src="/images/settings-light.webp" alt="Settings page" class="screenshot light-only">
<img src="/images/settings-dark.webp" alt="Settings page" class="screenshot dark-only">

| Section | Options |
| --- | --- |
| **Appearance** | Font size. Look: follow the desktop theme's light or dark mode, or force one. The gear menu's light and dark switch (<kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>L</kbd> / <kbd>D</kbd>) sets the same option. |
| **Startup** | The page Omarchist opens on when you start it without `--view`, or the page you used last. |
| **Omarchy Updates** | Whether the app checks for Omarchy updates in the background, how often, and whether a found update raises a desktop notification. |
| **Theme Designer** | Auto-apply theme on edit: apply a theme to your desktop as soon as you open it in the Theme Designer. |
| **Flows** | A desktop notification when a flow run from a keybind or the command line finishes. Run [automations](/flows/automations) in the background. Count your installs in the [Gallery](/flows/gallery#privacy). |
| **Bar** | Show Omarchist in Omarchy's bar. See [Bar Widget](/configuring/bar-widget). |
