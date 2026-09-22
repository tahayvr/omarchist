---
outline: deep
---

# Themes

The **Themes** page (<kbd>Ctrl</kbd> + <kbd>1</kbd>) shows every theme on your system. **All Themes** lists Omarchy's and yours; **Omarchist Themes** lists only the ones made here. **Apply** switches your desktop to the theme. The <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu edits it, opens its folder, or deletes it.

<img src="/images/themes-light.webp" alt="Themes page" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Themes page" class="screenshot dark-only">

## Create a theme

Choose **Create New Theme** from the **Themes** menu in the title bar, or press <kbd>Ctrl</kbd> + <kbd>N</kbd>.

<img src="/images/create-theme-light.webp" alt="Create New Theme dialog" class="screenshot light-only">
<img src="/images/create-theme-dark.webp" alt="Create New Theme dialog" class="screenshot dark-only">

- **Select Image** builds a palette from a picture and copies the picture in as the wallpaper. The same thing from a terminal: `omarchist theme from-image <picture>`.
- **Create Manually** starts from a default palette.

Either way the Theme Designer opens. Changes save as you make them; there is no Save button.

## Theme Designer

Omarchy Quattro builds your terminals, window borders, the bar, notifications, the launcher, and editor themes from one `colors.toml` file. The designer edits that file, plus the few per-app overrides Omarchy accepts.

<img src="/images/designer-colors-light.webp" alt="Theme Designer, Colors tab" class="screenshot light-only">
<img src="/images/designer-colors-dark.webp" alt="Theme Designer, Colors tab" class="screenshot dark-only">

| Tab | What it sets |
| --- | --- |
| **General** | The name (with **Rename**), the author, and **Light Theme**, saved as `mode` in `colors.toml`. |
| **Colors** | Accent, background, foreground, selection, and the 16 ANSI colors. Everything else is generated from these. |
| **File Manager** | The Yaru icon color for Nautilus. |
| **Editor** | Optional `neovim.lua` and `vscode.json` overrides. Empty means Omarchy generates them from the palette. |
| **Overrides** | Window borders (any Hyprland color, gradients included), the Chromium color, the lock screen, and btop. Off means Omarchy generates them. |
| **Backgrounds** | Wallpapers, copied into the theme folder. |

The copy icon <span class="icon-inline icon-inline-copy" aria-hidden="true"></span> next to a color copies its value so you can paste it into another field. **Apply Theme** (<kbd>Ctrl</kbd> + <kbd>S</kbd>) switches your desktop to the theme you are editing.

::: warning
Themes that ship with Omarchy cannot be edited. Create your own instead.
:::

## Theme folder

```
~/.config/omarchy/themes/my-theme/
├── omarchist.json      # marks the theme as made by Omarchist; do not edit
├── colors.toml         # the palette Omarchy generates everything from
├── icons.theme         # icon theme
├── backgrounds/        # wallpapers
├── btop.theme          # optional override
├── chromium.theme      # optional override
├── shell.lock.toml     # optional override
├── neovim.lua          # optional override
└── vscode.json         # optional override
```

Terminal configs, border colors, and the rest never appear here; Omarchy generates them when the theme is applied.
