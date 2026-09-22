---
outline: deep
---

# Keybinds

The **Keybinds** page (<kbd>Ctrl</kbd> + <kbd>3</kbd>) lists every keyboard shortcut Hyprland runs on your system and lets you change them without editing Lua.

<img src="/images/keybinds-light.webp" alt="Keybinds page" class="screenshot light-only">
<img src="/images/keybinds-dark.webp" alt="Keybinds page" class="screenshot dark-only">

Each row shows the bind's description, its keys, the command or dispatcher it runs, and where it comes from: **Default** binds ship with Omarchy, **User** binds are from your `~/.config/hypr/bindings.lua`, and **Omarchist** binds are the ones you changed here, tagged **Modified**, **Custom**, **Disabled**, or **Unbound**. A warning icon marks keys shared by more than one bind; Hyprland runs all of them.

## Searching

Type to match descriptions, commands, or keys (`super k`, `screenshot`). Press the keyboard button or <kbd>Ctrl</kbd> + <kbd>K</kbd> to search by pressing keys instead. The filters (<kbd>Alt</kbd> + <kbd>1</kbd> … <kbd>5</kbd>) narrow the list to modified binds, conflicts, Omarchy's defaults, or your own.

## Changing a keybind

Double-click a row or press <kbd>Enter</kbd> on it. <kbd>Delete</kbd> disables the selected bind; the row menu also resets or disables it.

<img src="/images/keybind-dialog-light.webp" alt="Edit keybind dialog" class="screenshot light-only">
<img src="/images/keybind-dialog-dark.webp" alt="Edit keybind dialog" class="screenshot dark-only">

The **Keys** box records a combination: click it or press <kbd>Enter</kbd>, then press the keys. Recording stops at the first complete combination, so <kbd>Escape</kbd> can be recorded too. Keys that never reach an app, such as mouse buttons, media keys, or Omarchy's `code:` workspace keys, go in the text field in Omarchy's syntax:

```
SUPER + SHIFT + K
SUPER + mouse:272
XF86AudioMute
SUPER + code:10
```

If the keys are already in use, the dialog lists the other binds; press **Save** again to keep both. Binds that run a Lua function inside Omarchy's config cannot be re-bound, only disabled.

**Add keybind** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>), or **Add Keybind…** in the title bar's **Keybinds** menu from any page, makes a new bind the same way. The description fills itself in from the action until you type your own.

## Actions

The **Action** section builds the command for you; the line under it shows exactly what Hyprland will run. Editing a bind opens on the kind that matches its command.

| Kind | Runs |
| --- | --- |
| **App** | An installed app. *Focus the window if it is already open* switches to a running copy instead of starting another. |
| **Web app** | A URL in its own browser window, or an installed web app, with the same focus switch. |
| **Terminal** | A command in a new terminal window, such as `btop`, with the focus switch. |
| **Omarchy** | One of Omarchy's own commands: menus, panels, capture, notifications, media, display, system. |
| **Window** | A Hyprland action: close, float, focus or swap in a direction, workspaces, scratchpad, monitors, groups, resizing. |
| **Flow** | A [flow](/flows/), so one key runs a whole sequence. |
| **Command** | Any shell command, through Hyprland's `exec` dispatcher. |

## Disabling and resetting

**Disable** turns a default bind off; **Reset to default** brings it back. Resetting a bind you added removes it.

## How it works

Omarchy declares binds in Lua, so Omarchist reads your `hyprland.lua` the same way Omarchy's keybinding menu does and records every bind. Your changes are saved to `~/.config/omarchist/hyprland/keybinds.json` and written as `hl.unbind` and `hl.bind` calls into `~/.config/hypr/omarchist.lua`, which loads after your `bindings.lua`. Hyprland reloads immediately, and your own files are never edited.

While recording, Omarchist switches Hyprland into an empty submap so your shortcuts do not fire, and switches back when recording ends. An input method such as fcitx5 keeps its own hotkeys, so type those combinations instead of pressing them. If Omarchist is killed mid-recording and your shortcuts stop responding, run:

```bash
hyprctl dispatch 'hl.dsp.submap("reset")'
```
