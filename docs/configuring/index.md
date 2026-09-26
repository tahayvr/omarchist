---
outline: deep
---

# Hyprland

The **Configuration** page (<kbd>Ctrl</kbd> + <kbd>2</kbd>) sets Hyprland options without touching `hyprland.lua`. Changes apply immediately, and the search box filters every section at once.

<img src="/images/config-light.webp" alt="Configuration page" class="screenshot light-only">
<img src="/images/config-dark.webp" alt="Configuration page" class="screenshot dark-only">

| Section | Settings |
| --- | --- |
| **General** | Border size and grab area, gaps (in, out, floating, workspaces), the layout (Dwindle, Master, Scrolling, or Monocle), tearing, and floating-window snapping. |
| **Appearance** | Corner rounding, opacity, dimming, blur, shadows, glow, motion blur, and animations. |
| **Layouts** | Single-window aspect ratio, and every option of the Dwindle, Master, and Scrolling layouts. |
| **Keyboard** | Layout, Num Lock on start, keybinds by symbol, repeat rate and delay. |
| **Mouse** | Sensitivity and acceleration, scrolling, and how the pointer changes focus. |
| **Touchpad** | Tapping, clicking, scrolling, and workspace swipe gestures. |
| **Groups** | Window grouping behaviour and the group bar. |
| **Cursor** | Hiding, warping, zooming, and rendering of the pointer. |
| **Windows** | Focus, workspaces, moving focus, keybind and drag behaviour, and window swallowing. |
| **System** | Variable frame rate and adaptive sync, rendering, color management and HDR, display wake, the lock screen, startup notices, and XWayland. |

## Omarchy settings

Below the Hyprland pages, the list continues with Omarchy's own settings. These read and write through Omarchy's scripts and files, so a change here is the same as the matching entry of Omarchy's menu, and there is no reset button because nothing is stored twice.

| Page | Settings |
| --- | --- |
| **Lock & Idle** | Seconds before the screensaver and the lock, stay awake, screensaver on or off, Suspend in the system menu, and a Lock button. |
| **Power** | The power profile on power and on battery, the battery percentage in the bar, and the hybrid GPU switch. |
| **Notifications** | Do not disturb and crash capture. |
| **Default Apps** | The browser, terminal, and editor; only installed apps are offered. |
| **Bar** | Show the bar, its position, and transparency. |
| **Fonts** | The monospace font and the text size for the shell, GTK apps, and terminals. |
| **Displays** | The focused monitor's scale, night light and its temperature, and the laptop display. |
| **Devices** | Touchpad, touchscreen, and Bluetooth. |
| **Network** | The DNS provider and the Wi-Fi band. |
| **Security** | Fingerprint, FIDO2 key, SSH server, sudoless Docker, and passwordless sudo. Each shows whether it is set up and opens Omarchy's terminal to set it up or remove it. |
| **Software** | Everything Omarchy's menu can install or remove, grouped as the menu groups it: browsers, editors, terminals, services, AI tools, games, and development environments. Each row says whether it is installed and opens Omarchy's terminal to install or remove it. |
| **Updates & Resets** | The package channel, firmware updates, timezone, clock sync, and resets of the Hyprland, shell, tmux, and boot screen configs. |

Pages show only what applies to this machine: laptop-only rows appear on laptops, and the fingerprint row when a reader is present. Settings that need your password or a confirmation open in Omarchy's floating terminal, the same as from the menu.

## Settings you changed

Every setting starts at the value Omarchy gives it, or the value your own `looknfeel.lua` and `input.lua` set. When you change one, a reset button <span class="icon-inline icon-inline-rotate-ccw"></span> appears next to it. Its tooltip shows the value Omarchy uses, and clicking it puts the setting back under Omarchy's control. Setting a value back by hand does the same.

Settings you have not changed keep following Omarchy, so an Omarchy update that changes a default reaches you.

## How it works

Omarchist keeps only the settings you changed, in `~/.config/omarchist/hyprland/state.json`, and writes them as `hl.config` calls into `~/.config/hypr/omarchist.lua`, which your `hyprland.lua` loads after Omarchy's defaults and your own files. Nothing else is written, and your own config files are never edited.

To know what a setting goes back to, Omarchist evaluates your `hyprland.lua` with a stub of Hyprland's `hl` API and records every `hl.config` call except its own file, the same way the Keybinds page reads your keybinds. This needs the `lua` interpreter, which Omarchy installs.

Omarchist 1.x stored the whole configuration, which pinned Omarchy's values in `omarchist.lua`. On first start, Omarchist 2 keeps only the settings that differ from what Omarchy and Hyprland set, and leaves the old file next to the new one as `state.json.legacy`.
