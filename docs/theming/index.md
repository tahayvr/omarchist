---
outline: deep
---

# Themes

The **Themes** page (<kbd>Ctrl</kbd> + <kbd>1</kbd>) shows every theme on your system. **All themes** lists Omarchy's and yours; **Omarchist themes** lists only the ones made here. **Apply** switches your desktop to the theme. The <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu edits it, opens its folder, or deletes it.

<img src="/images/themes-light.webp" alt="Themes page" class="screenshot light-only">
<img src="/images/themes-dark.webp" alt="Themes page" class="screenshot dark-only">

## Create a theme

Click **New theme** at the top right of the **Themes** page, choose **New theme** from the **Themes** menu in the title bar or the command palette, or press <kbd>Ctrl</kbd> + <kbd>N</kbd>.

<img src="/images/create-theme-light.webp" alt="New theme dialog" class="screenshot light-only">
<img src="/images/create-theme-dark.webp" alt="New theme dialog" class="screenshot dark-only">

- **Select image** builds a palette from a picture and copies the picture in as the wallpaper. The same thing from a terminal: `omarchist theme from-image <picture>`.
- **Create manually** starts from a default palette.

Either way the Theme Designer opens. Changes save as you make them; there is no Save button.

## Theme Designer

Omarchy Quattro builds your terminals, window borders, the bar, notifications, the launcher, and editor themes from one `colors.toml` file. The designer edits that file, the wallpapers, and the icon color. You never open a text editor: every setting is a color picker, a switch, or a choice.

<img src="/images/designer-colors-light.webp" alt="Theme Designer, Colors tab" class="screenshot light-only">
<img src="/images/designer-colors-dark.webp" alt="Theme Designer, Colors tab" class="screenshot dark-only">

Four tabs make a complete theme:

| Tab | What it sets |
| --- | --- |
| **General** | The author and **Light mode**, saved as `mode` in `colors.toml`. To rename the theme, click its name in the header. |
| **Colors** | Accent, background, foreground, selection, and the 16 ANSI colors, plus the window border colors (any Hyprland color, gradients included). Everything else is generated from these. |
| **Backgrounds** | Wallpapers, copied into the theme folder, and the optional [boot logo](#boot-logo). |
| **Icons** | The Yaru icon color for GTK apps and the file manager, saved as `icons.theme`. A theme made from an image starts with the color closest to its accent. |

Omarchy generates every app's look from these: terminals, the bar, notifications, btop, Neovim, VS Code, and the rest. You do not set them one by one.

::: tip Want to style single apps?
Omarchist keeps themes simple on purpose. If you want to fine-tune individual apps, try [Aether](https://github.com/bjarneo/aether), a visual theme editor made for Omarchy.
:::

The copy icon <span class="icon-inline icon-inline-copy" aria-hidden="true"></span> next to a color copies its value so you can paste it into another field. **Apply theme** (<kbd>Ctrl</kbd> + <kbd>S</kbd>), to the right of the tabs, switches your desktop to the theme you are editing; once it is the active theme the button becomes a refresh icon <span class="icon-inline icon-inline-refresh" aria-hidden="true"></span> that applies your latest changes again. Next to each **Window Borders** field, a color picker sets the border's first color; the field itself still takes a full Hyprland value, such as a gradient.

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
├── icons.theme         # the Yaru icon color
├── backgrounds/        # wallpapers
├── unlock.png          # optional boot logo
└── preview-unlock.png  # its preview for the boot screen switcher
```

Omarchy generates everything else when you apply the theme. If a theme folder also holds an app's own file, such as `btop.theme` or `alacritty.toml`, Omarchy uses that file instead of generating one. Omarchist never writes these files. Delete one to let Omarchy generate it again.
