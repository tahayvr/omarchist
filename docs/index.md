Omarchist is a desktop app for [Omarchy](https://omarchy.org/) Linux. Design themes, tune Hyprland, change keybinds, and chain actions into flows, all without editing config files by hand.

<img src="/images/themes-light.webp" alt="Omarchist" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Omarchist" class="screenshot dark-only">

## Features

- **[Themes](/theming/)**: design a theme from scratch or from an image. Omarchist writes `colors.toml`; Omarchy generates the rest.
- **[Configuration](/configuring/)**: gaps, borders, blur, keyboard, mouse, and touchpad, applied as you change them.
- **[Keybinds](/configuring/keybinds)**: search and change every Hyprland keybind, recording keys the way an editor does.
- **[Flows](/flows/)**: chain actions like Shortcuts on a Mac and run them from a keybind, the app launcher, at startup, or the command line.
- **[Keyboard first](/keyboard)**: every page and dialog works without a mouse.

## Installation

```bash
yay -S omarchist-bin
```

::: tip
Omarchist 2.x requires Omarchy Quattro (v4). Use an Omarchist 1.x release with Omarchy 3.x.
:::

## Settings

The **Settings** page (<kbd>Ctrl</kbd> + <kbd>,</kbd>) holds the app's own options. They live in `~/.config/omarchist/settings.json`, and an Omarchist update adds new options without touching the ones you set.

| Section | Options |
| --- | --- |
| **Appearance** | Font size. Look: follow the desktop theme's light or dark mode, or force one. The gear menu's light and dark switch (<kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>L</kbd> / <kbd>D</kbd>) sets the same option. |
| **Startup** | The page Omarchist opens on when you start it without `--view`, or the page you used last. |
| **Omarchy Updates** | Whether the app checks for Omarchy updates in the background, how often, and whether a found update raises a desktop notification. |
| **Theme Designer** | Auto-apply theme on edit: apply a theme to your desktop as soon as you open it in the Theme Designer. |
| **Flows** | A desktop notification when a flow run from a keybind or the command line finishes. |
