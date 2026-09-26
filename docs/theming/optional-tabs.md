---
outline: deep
---

# Optional Tabs

Your palette already themes every app on your desktop. The tabs after **Optional** in the Theme Designer let you give one app its own colors or sizes. You never need them: a theme made on the first three tabs is complete.

| Tab | Apps |
| --- | --- |
| **Desktop** | The bar, notifications, launcher, menus, popups, tooltips, password prompt, lock screen, image picker, controls, spacing, font sizes, terminal menus, screen share picker, icons, and keyboard lighting. |
| **Terminals** | Alacritty, Kitty, Ghostty, and Foot, set together. |
| **Editors** | Neovim, VS Code, Helix, and Obsidian. |
| **Apps** | btop and your browser. |
| **AI Tools** | Claude Code, Pi, Hermes, and T3 Code. |

## Customize an app

1. Open an optional tab and pick an app from the list on the left.
2. Turn on **Customize**. The app starts with exactly the colors your palette gives it, so nothing changes yet.
3. Change what you want. Changes save as you make them.

In the list, a dot marks the apps you customized. An app you have not installed is dimmed; you can still customize it for people who use the theme.

To start over, click **Reset to Generated**. To go back to the palette, turn **Customize** off. Omarchist asks before it discards your changes.

::: tip
The **General** tab lists every app you customized under **Customized Apps**. Click one to jump to it.
:::

## Two kinds of apps

**Apps that follow your palette.** Terminal Colors, Terminal Menus, Screen Share Picker, and Obsidian show the palette colors the app uses. Change a color and only that app uses the new one. Every color you leave alone keeps following the palette, even when you change the palette later. **Reset** under a color hands it back to the palette.

**Apps with their own colors.** Every other app keeps the colors you set for it. When you change the palette later, those apps keep their colors. Click **Reset to Generated** to pick up the palette again.

## Desktop

The Omarchy shell draws the bar, notifications, launcher, menus, popups, tooltips, password prompt, lock screen, image picker, and controls. Each one has its own entry, with the settings Omarchy offers for it:

- **Colors** open a color picker.
- **Opacity** goes from `0` (invisible) to `1` (solid). Use <kbd>−</kbd> and <kbd>+</kbd> or type a number.
- **Sizes** are in pixels.
- **Borders** follow the window border you set on the **Colors** tab. Choose **Window border**, **Border, else text color**, or **Custom color**.

**Spacing** and **Font Sizes** scale the whole shell. Their per-size fields are empty until you fill one in; an empty field uses the shell's own size.

| Entry | What it themes |
| --- | --- |
| **Terminal Menus** | Omarchy's menus and prompts in the terminal. |
| **Screen Share Picker** | The window and screen picker shown when an app starts sharing your screen. |
| **Icons** | The Yaru icon color for GTK apps and the file manager. |
| **Keyboard Lighting** | The backlight color of supported RGB keyboards (ASUS ROG and Framework 16). |

## Terminals

**Terminal Colors** sets one palette for Alacritty, Kitty, Ghostty, and Foot. Pick the colors once and Omarchist writes the config of all four terminals. It shows only the colors terminals use.

## Editors

| Entry | What it themes |
| --- | --- |
| **Neovim** | **Colors** changes the colors of Omarchy's own Neovim theme. **Plugin** uses a colorscheme plugin instead: enter its GitHub repository, such as `folke/tokyonight.nvim`, and the colorscheme name you would pass to `:colorscheme`. |
| **VS Code Extension** | A theme from the VS Code Marketplace. Enter the extension id, such as `enkia.tokyo-night`, and the theme's name. VS Code, VSCodium, and Cursor install it when you apply the theme. |
| **VS Code Theme** | The colors of the theme Omarchy installs into VS Code, VSCodium, and Cursor. |
| **Helix** | Helix's colors. |
| **Obsidian** | The theme Omarchy copies into your Obsidian vaults. |

Customizing **VS Code Extension** replaces **VS Code Theme**: the Marketplace theme wins.

## Apps

| Entry | What it themes |
| --- | --- |
| **btop** | btop's boxes, graphs, meters, and text. |
| **Browsers** | The toolbar color of Chromium, Chrome, Edge, and Brave. |

## AI Tools

**Claude Code**, **Pi**, **Hermes**, and **T3 Code** each get the theme Omarchy makes for them.

## Colors by key or by color

Apps with many colors, such as the AI tools and the editors, show two views:

- **By Key** gives every color its own picker, named the way the app names it, for example `diffAdded`.
- **By Color** gives one picker per color the app uses. Change it and every place that uses the color changes with it. The VS Code theme uses hundreds of entries, so it opens **By Color**.

## Sharing a theme through git

When someone installs a theme with `omarchy theme install`, Omarchy skips the parts that could run code: Neovim, Terminal Menus, Terminal Colors, and **VS Code Extension**. Everything else works for everyone who installs your theme.

## Keyboard

| Key | Action |
| --- | --- |
| <kbd>↑</kbd> / <kbd>↓</kbd> | Previous or next app |
| <kbd>Home</kbd> / <kbd>End</kbd> | First or last app |
| <kbd>Enter</kbd> or <kbd>→</kbd> | Go to the app's settings |
| <kbd>Tab</kbd>, then <kbd>Space</kbd> | Move to **Customize** and turn it on or off |
