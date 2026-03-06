---
outline: deep
---

# Theming with Omarchist

Omarchist gives you a Theme Designer to create and customize themes for your desktop environment.

You find your themes in the **Custom Themes** tab on the Themes page. System themes and other non-omarchist themes appear in the **System Themes** tab.

## Create a Theme

Click the **Create New Theme** button on the Themes page. Enter a name for your theme and click **Create**. The Theme Designer opens automatically.

## Theme Designer

The Theme Designer lets you customize every part of your desktop. It contains tabs for different components.

::: warning
You cannot edit system themes.
:::

### General

Set basic information about your theme.

- **Theme Name**: The name appears in your theme list.
- **Author**: Enter your name or handle.
- **Light Mode**: Toggle this if you create a light theme. This ensures proper contrast and text colors.
- **Accent Color**: This color is used by Omarchy to make themes for various apps.

### Terminal

Set the color palette for your terminal emulators. Omarchist generates configurations for **Alacritty**, **Ghostty**, and **Kitty** automatically.

- **Primary Colors**: Background and foreground colors.
- **Cursor**: Cursor color and text color.
- **Normal/Bright Colors**: The 8 standard ANSI colors for both normal and bright variants.

### Browser

Set the theme color for **Chromium**.

- **Theme Color**: One color generates a complete Chromium theme.

### Waybar (Status Bar)

Customize **Waybar**, the bar at the top of your screen.

- **Background/Foreground**: Base colors for the bar.

### Windows (Hyprland)

Configure window appearance in **Hyprland**.

- **Active/Inactive Border**: Border colors for focused and unfocused windows.
- **Border Size**: Border thickness in pixels.
- **Gaps**: Space between windows and screen edges.
- **Rounding**: Corner rounding for windows.

### Menu (Walker)

Customize **Walker**, the application launcher.

- **Background**: Main menu background color.
- **Base**: Search bar background.
- **Border**: Menu border color.
- **Foreground**: Text color.
- **Selected Text**: Highlighted item color.

### Lock Screen (Hyprlock)

Customize **Hyprlock**, the screen locker.

- **Main Color**: Input field color.
- **Inner/Outer Color**: Border colors.
- **Font Color**: Text color.
- **Check Color**: Success indicator color.

### Notifications (Mako)

Style notifications with **Mako**.

- **Background**: Notification bubble background.
- **Text Color**: Notification text color.
- **Border Color**: Border color.

### SwayOSD

Customize on-screen popups for volume and brightness changes.

- **Background**: Popup background color.
- **Border**: Popup border color.
- **Label/Image/Progress**: Colors for text, icons, and progress bars.

### Btop

Set colors for the **Btop** system monitor.

- **Main Colors**: Background, text, and title colors.
- **Box Colors**: Colors for CPU, memory, network, and process boxes.
- **Gradient Colors**: Temperature, CPU, memory, and network gradients.

### File Manager

Select the icon theme for **Nautilus**.

- **Yaru Colors**: Choose from Red, Blue, Olive, Yellow, Purple, Magenta, or Sage variants.

### Editor

Edit configuration files for **Neovim** and **VSCode:**.

- **Neovim**: Edit the `neovim.lua` file directly.
- **VSCode:**: Edit the `vscode.json` file directly.

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

A complete theme folder contains:

```
~/.config/omarchy/themes/my-theme/
├── omarchist.json          # Theme manifest (do not edit)
├── colors.toml             # Color definitions
├── alacritty.toml          # Alacritty terminal config
├── ghostty.conf            # Ghostty terminal config
├── kitty.conf              # Kitty terminal config
├── hyprland.conf           # Hyprland window settings
├── hyprlock.conf           # Hyprlock screen lock
├── waybar.css              # Waybar styling
├── walker.css              # Walker launcher styling
├── mako.ini                # Mako notifications
├── btop.theme              # Btop system monitor
├── swayosd.css             # SwayOSD styling
├── icons.theme             # Icon theme reference
├── neovim.lua              # Neovim configuration
├── vscode.json             # VSCode: theme reference
├── chromium.theme          # Chromium theme color
└── backgrounds/            # Wallpaper images
    └── *.png
```
