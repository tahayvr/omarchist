---
outline: deep
---

# Status Bar

The Status Bar page lets you visually manage your [Waybar](https://github.com/Alexays/Waybar) configuration. You find it in the left sidebar under **Status Bar**.

You can rearrange modules, add or remove them, edit their settings, adjust the bar's global properties, and manage multiple named profiles — all without editing a config file by hand.

## Profiles

A profile is a complete Waybar configuration stored in its own directory. Each profile contains a `config.jsonc` file (the module layout and bar settings) and a `style.css` file (the visual styling).

Profile files live at:

```
~/.config/omarchist/waybar/profiles/<profile-name>/
├── config.jsonc
└── style.css
```

### Switching profiles

Use the dropdown at the top of the page to switch between profiles. Omarchist reloads the bar preview and all editor panels immediately.

### Creating a profile

Click the **+** button next to the profile dropdown to open the **New Waybar Profile** dialog. Enter a name and click **Create**. The new profile is a copy of the default Omarchy profile and becomes the active profile.

### Renaming, duplicating, and deleting

Click the **···** button next to the **+** button to open the profile options menu.

| Action | What it does |
|---|---|
| **Rename profile** | Renames the current profile's directory. The bar reloads under the new name. |
| **Duplicate profile** | Copies the current profile into a new directory. The copy becomes the active profile. |
| **Delete profile** | Permanently removes the profile directory. Omarchist switches to the next available profile. Disabled when only one profile exists. |

## Module Layout

The bar preview in the center of the page shows your three zones — **Left**, **Center**, and **Right** — as draggable chips. Each chip represents one Waybar module.

### Reordering modules

Drag a chip and drop it onto another chip or into a zone to reorder or move it. The config saves automatically on every drop.

### Adding modules

Click **Add Module** to open the module library panel. It lists all supported Waybar modules grouped by category.

| Category | Modules |
|---|---|
| System | CPU, Memory, Battery, Temperature, Disk, Backlight |
| Time | Clock |
| Audio | PulseAudio, WirePlumber |
| Network | Network, Bluetooth |
| Hyprland | Workspaces, Window, Submap, Language |
| Utilities | Tray, Keyboard State, Idle Inhibitor |

Each row shows the module's icon, name, and a short description. Select a zone from the dropdown on the right and click **Add** to append the module to that zone. The chip appears in the preview immediately.

### Removing modules

Right-click any chip and choose **Remove from bar**. The module is removed from the zone and the config saves immediately.

## Editing a Module

Right-click any chip and choose **Edit** to open the module editor panel for that module.

The editor shows the most useful fields for the selected module:

| Field | Description |
|---|---|
| **Format** | The display string. Use tokens like `{usage}`, `{icon}`, `{capacity}` specific to that module. |
| **Interval** | How often the module updates, in seconds. |
| **Tooltip Format** | Text shown on hover. |
| **On Click** | Shell command to run when you click the module. |
| **Max Length** | Maximum number of characters before the output is truncated. |

Changes save automatically as you type. Close the editor with the **✕** button in the panel header.

## Bar Settings

Click **Bar Settings** at the bottom of the page to expand the bar-level configuration panel.

| Field | Description |
|---|---|
| **Position** | Where the bar appears: `top`, `bottom`, `left`, or `right`. |
| **Layer** | Compositor layer: `top`, `bottom`, `overlay`, or `background`. |
| **Height** | Bar height in pixels. |
| **Spacing** | Space between modules in pixels. |
| **Output** | Monitor the bar appears on (leave blank for all monitors). |
| **Margin** | Outer margins in pixels — top, right, bottom, left. |

All changes save immediately to `config.jsonc`.

## Restarting Waybar

Click the **restart** button (the circular arrow icon) in the top-right corner of the header to restart Waybar and apply your changes.

## Auto-Save

Every action — reordering, adding, removing, editing a module, changing a bar setting — saves immediately to disk. There is no Save button.

## Quick Access

You can open the Status Bar page directly from the command line:

```bash
omarchist --view status-bar
```
