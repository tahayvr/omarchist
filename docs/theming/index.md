---
outline: deep
---

# Theming with Omarchist

Omarchist gives you a Theme Designer to create and customize themes for your desktop environment.

You find your themes in the **Custom Themes** tab on the Themes page. System themes and other non-omarchist themes appear in the **System Themes** tab.

## Create a Theme

Click the **Create New Theme** button on the Themes page. Enter a name for your theme and click **Create**. The Theme Designer opens automatically.

You can also create a theme from an image. Omarchist extracts a color palette from the picture, builds the theme from it, and copies the image into the theme's backgrounds folder.

## How Omarchy Uses Your Theme

Omarchy Quattro builds most app configs from a single `colors.toml` file. When you apply a theme, Omarchy generates the terminal configs, window border colors, the bar, notifications, the launcher, a Neovim colorscheme, and a VS Code theme from that palette.

Omarchist writes `colors.toml` for you. It only adds extra files for the few apps that accept a per-theme override, such as btop, Chromium, and the lock screen.

## Theme Designer

The Theme Designer lets you customize every part of your desktop. It contains tabs for different components.

::: warning
You cannot edit system themes.
:::

### General

Set basic information about your theme.

- **Theme Name**: The name appears in your theme list.
- **Author**: Enter your name or handle.
- **Light Mode**: Toggle this if you create a light theme. Omarchist writes it as the `mode` key in `colors.toml`.
- **Accent Color**: Omarchy uses this color for window borders, the bar, and other highlights.

### Colors

Set the full palette in `colors.toml`. Omarchy generates configurations for **Alacritty**, **Ghostty**, **Kitty**, and **Foot** from these values.

- **Primary Colors**: Background and foreground colors.
- **Cursor and Selection**: Cursor color plus selection foreground and background.
- **Normal/Bright Colors**: The 8 standard ANSI colors for both normal and bright variants.
- **Window Borders**: Optional Hyprland border colors. Omarchy uses the accent color for the active border and a neutral grey for inactive ones. Any Hyprland color works, including gradients such as `rgba(26a269ee) rgba(2ec27eee) 45deg`. Leave a field blank for the default.

### File Manager

Select the icon theme for **Nautilus**.

- **Yaru Colors**: Choose from Red, Blue, Olive, Yellow, Purple, Magenta, or Sage variants.

### Editor

Optionally override the editor themes Omarchy generates.

- **Neovim**: Edit the `neovim.lua` file directly. Leave it empty to use the colorscheme Omarchy generates from your palette.
- **VS Code**: Edit the `vscode.json` file directly. Leave it empty to use the VS Code theme Omarchy generates from your palette.

Clearing a field removes the override file.

### Overrides

Omarchy generates these app configs from your palette every time you apply the theme. Turn an override on only when an app needs colors that differ from the generated ones. Turning it off removes the file so the app follows the palette again.

Each override starts from the same colors Omarchy would generate, so you only change what you need.

- **Browser**: The Chromium theme color. Omarchy uses the theme background by default.
- **Lock Screen**: Text, placeholder, error, and border colors for the lock screen input.
- **Btop**: Main, selection, status, box outline, and gradient colors for the btop system monitor.

### Backgrounds

Set wallpapers for your desktop.

- **Select Image**: Choose from your local files. Omarchist copies the image to your theme directory.

## Tips

### Copy Colors

Reuse the same color across different components.

Click the **Copy** icon <span class="icon-inline icon-inline-copy" aria-hidden="true"></span> next to any color field to copy its value. Paste it into another field for consistency.

### Auto-Save

Omarchist saves your theme automatically when you make changes. You do not need to click a Save button.

### View Themes

After saving, your theme appears in the **Custom Themes** tab on the Themes page. Click the theme card to apply it to your desktop.

## Theme Manifest

Every theme created with Omarchist contains an `omarchist.json` file. This file serves as a manifest and identifies the theme as an Omarchist-created theme.

### What the Manifest Contains

The manifest stores:

- **Version**: The manifest format version
- **Name**: Theme name
- **Created At**: Creation timestamp
- **Modified At**: Last modification timestamp
- **Author**: Theme creator name
- **Colors**: Color palette definitions
- **App Configurations**: Settings for individual applications

Themes with an `omarchist.json` file are considered Omarchist-managed.

::: warning Do Not Edit Manually
Never edit the `omarchist.json` file directly. Use the Theme Designer to make changes. Manual edits may corrupt the theme.
:::

### Theme Structure

A theme folder contains:

```
~/.config/omarchy/themes/my-theme/
├── omarchist.json          # Theme manifest (do not edit)
├── colors.toml             # Color palette — Omarchy generates the rest from this
├── btop.theme              # Btop system monitor (optional override)
├── chromium.theme          # Chromium theme color (optional override)
├── shell.lock.toml         # Lock screen colors (optional override)
├── icons.theme             # Icon theme reference
├── neovim.lua              # Neovim configuration (optional override)
├── vscode.json             # VS Code theme reference (optional override)
└── backgrounds/            # Wallpaper images
    └── *.png
```

Omarchy generates terminal configs, window border colors, and the rest of the desktop from `colors.toml` when you apply the theme, so those files never appear in the theme folder.
