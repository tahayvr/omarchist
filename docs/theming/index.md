---
outline: deep
---

# Themes

The **Themes** page (<kbd>Ctrl</kbd> + <kbd>2</kbd>) shows every theme on your system. **All themes** lists Omarchy's and yours; **Omarchist themes** lists only the ones made here. **Apply** switches your desktop to the theme. The <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu edits it, opens its folder, or deletes it.

<img src="/images/themes-light.webp" alt="Themes page" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Themes page" class="screenshot dark-only">

## Create a theme

Click **New theme** at the top right of the **Themes** page, choose **New theme** in the command palette, or press <kbd>Ctrl</kbd> + <kbd>N</kbd>.

<img src="/images/create-theme-light.webp" alt="New theme dialog" class="screenshot light-only">
<img src="/images/create-theme-dark.webp" alt="New theme dialog" class="screenshot dark-only">

- **Select image** builds a palette from a picture and copies the picture in as the wallpaper. The same thing from a terminal: `omarchist theme from-image <picture>`.
- **Create manually** starts from a default palette.

Either way the Theme Designer opens. Changes save as you make them; there is no Save button.

## Theme Designer

Omarchy Quattro builds your terminals, window borders, the bar, notifications, the launcher, and editor themes from one `colors.toml` file. The designer edits that file. You never open a text editor: every setting is a color picker, a number, a switch, or a choice.

<img src="/images/designer-colors-light.webp" alt="Theme Designer, Colors tab" class="screenshot light-only">
<img src="/images/designer-colors-dark.webp" alt="Theme Designer, Colors tab" class="screenshot dark-only">

The first three tabs make a complete theme:

| Tab | What it sets |
| --- | --- |
| **General** | The name (with **Rename**), the author, and **Light mode**, saved as `mode` in `colors.toml`. **Customized Apps** lists the apps you customized on the optional tabs; click one to jump to it. |
| **Colors** | Accent, background, foreground, selection, and the 16 ANSI colors, plus the window border colors (any Hyprland color, gradients included). Everything else is generated from these. |
| **Backgrounds** | Wallpapers, copied into the theme folder, and the optional [boot logo](#boot-logo). |

The tabs after **Optional** let you give single apps their own look: **Desktop**, **Terminals**, **Editors**, **Apps**, and **AI Tools**. You never need them. See [Optional Tabs](/theming/optional-tabs).

The copy icon <span class="icon-inline icon-inline-copy" aria-hidden="true"></span> next to a color copies its value so you can paste it into another field. **Apply theme** (<kbd>Ctrl</kbd> + <kbd>S</kbd>) switches your desktop to the theme you are editing.

::: warning
Omarchist edits only the themes it created. Omarchy's own themes and themes you installed from elsewhere are listed, but you cannot open them in the Theme Designer. Create your own theme instead.
:::

### Boot logo

The **Boot Logo** section at the bottom of **Backgrounds** sets the picture on the disk unlock and login screens (`unlock.png`).

1. Click **Choose Logo** and pick a PNG image.
2. Omarchist copies it into the theme and renders the preview Omarchy's boot screen switcher shows (`preview-unlock.png`) from the logo and your background and foreground colors.
3. Open the Omarchy menu, choose **Style** › **Unlock**, and pick your theme. It asks for your password, because the boot screen is a system file.

Applying the theme does not change the boot screen. After you change the theme's colors, click **Refresh preview**. **Remove** deletes the logo and its preview.

## Theme folder

```
~/.config/omarchy/themes/my-theme/
├── omarchist.json      # marks the theme as made by Omarchist; do not edit
├── colors.toml         # the palette Omarchy generates everything from
├── icons.theme         # icon theme
├── backgrounds/        # wallpapers
├── unlock.png          # optional boot logo
├── preview-unlock.png  # its preview for the boot screen switcher
└── …                   # files for the apps you customized
```

Each app you customize on an optional tab adds its file here, for example `btop.theme`, `shell.bar.toml`, or `alacritty.toml`. Omarchy uses a file from the theme folder instead of generating its own. Turn **Customize** off and the file is removed, so Omarchy generates it again.
