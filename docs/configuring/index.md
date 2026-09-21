---
outline: deep
---

# Hyprland

The **Configuration** page (<kbd>Ctrl</kbd> + <kbd>2</kbd>) sets Hyprland options without touching `hyprland.lua`. Changes apply immediately, and the search box filters every section at once.

<img src="/images/config-light.webp" alt="Configuration page" class="screenshot light-only">
<img src="/images/config-dark.webp" alt="Configuration page" class="screenshot dark-only">

| Section | Settings |
| --- | --- |
| **General** | Border size, resize on border, gaps in, gaps out, gaps between workspaces, and the layout (Dwindle or Master). |
| **Appearance** | Corner rounding, active and inactive window opacity, and blur with its size and passes. |
| **Input** | Keyboard layout, repeat rate and delay. Mouse sensitivity, natural scroll, left handed. Touchpad disable while typing, tap to click, natural scroll. |
| **Miscellaneous** | Variable refresh rate. |

## How it works

Omarchist keeps your values in `~/.config/omarchist/hyprland/state.json` and writes them as `hl.config` calls into `~/.config/hypr/omarchist.lua`, which your `hyprland.lua` loads after Omarchy's defaults. Only values that differ from Hyprland's defaults are written, and your own config files are never edited.
