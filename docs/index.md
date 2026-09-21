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

The gear menu in the title bar sets the font size and switches the app between a light and a dark look for the current session (<kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>L</kbd> / <kbd>D</kbd>). The **Settings** page has one option, **Auto-apply theme on edit**, which applies a theme to your desktop as soon as you open it in the Theme Designer.
