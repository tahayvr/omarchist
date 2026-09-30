---
outline: deep
---

# Keybinds

The **Keybinds** page (<kbd>Ctrl</kbd> + <kbd>4</kbd>) lists every keyboard shortcut Hyprland runs on your system and lets you change them without editing Lua.

<img src="/images/keybinds-light.webp" alt="Keybinds page" class="screenshot light-only">
<img src="/images/keybinds-dark.webp" alt="Keybinds page" class="screenshot dark-only">

Each row shows the bind's description, its keys, the command or dispatcher it runs, and where it comes from: **Default** binds ship with Omarchy, **User** binds come from your own config files (usually `~/.config/hypr/bindings.lua`), and **Omarchist** binds are the ones you added here. A second tag says what happened to a bind:

| Tag | Meaning |
| --- | --- |
| **Modified** | You gave it new keys here. |
| **Custom** | You added it here. |
| **Disabled** | You turned it off here. **Reset to default** in the row menu brings it back. |
| **Unbound** | One of your own config files removes it with `hl.unbind`. Change that file to bring it back. |
| **Stale** | A change you made targets a bind Omarchy no longer ships (an update renamed it). **Reset to default** removes the change. |

A warning icon marks keys shared by more than one bind; Hyprland runs all of them.

## Searching

Type to match descriptions, commands, or keys (`super k`, `screenshot`). Press the keyboard button or <kbd>Ctrl</kbd> + <kbd>K</kbd> to search by pressing keys instead. The filters (<kbd>Alt</kbd> + <kbd>1</kbd> … <kbd>5</kbd>) narrow the list to modified binds, conflicts, Omarchy's defaults, or your own.

## Changing a keybind

Double-click a row or press <kbd>Enter</kbd> on it. <kbd>Delete</kbd> disables the selected bind. The row menu edits, resets, or disables it, and copies its command.

<img src="/images/keybind-dialog-light.webp" alt="Edit keybind dialog" class="screenshot light-only">
<img src="/images/keybind-dialog-dark.webp" alt="Edit keybind dialog" class="screenshot dark-only">

The **Keys** box records a combination: click it or press <kbd>Enter</kbd>, then press the keys. Recording stops at the first complete combination; a bare <kbd>Escape</kbd> stops it without recording (<kbd>Super</kbd> + <kbd>Escape</kbd> and other combinations record as usual). Keys the recorder cannot tell apart, such as mouse buttons, the numeric keypad, or Omarchy's `code:` workspace keys, go in the text field in Omarchy's syntax; Omarchist checks that the key name is one Hyprland knows:

```
SUPER + SHIFT + K
SUPER + mouse:272
XF86AudioMute
SUPER + code:10
```

If the keys are already in use, the dialog lists the other binds; press **Save** again to keep both. Binds that run a Lua function inside Omarchy's config cannot be re-bound, only disabled.

**Add keybind** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>, or **Add keybind** in the command palette from any page) makes a new bind the same way. The description fills itself in from the action until you type your own.

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

**Disable** turns a default bind off; **Reset to default** brings it back. A bind you added has no default, so its menu offers **Delete keybind** instead (the <kbd>Delete</kbd> key does the same).

## How it works

Omarchy declares binds in Lua, so Omarchist reads your `hyprland.lua` the same way Omarchy's keybinding menu does and records every bind. Your changes are saved to `~/.config/omarchist/hyprland/keybinds.json` and written as `hl.unbind` and `hl.bind` calls into `~/.config/hypr/omarchist.lua`, which loads after your `bindings.lua`. Hyprland reloads immediately, and your own files are never edited.

While recording, Omarchist switches Hyprland into an empty submap so your shortcuts do not fire, and switches back when recording ends. An input method such as fcitx5 keeps its own hotkeys, so type those combinations instead of pressing them. If Omarchist is killed mid-recording and your shortcuts stop responding, press <kbd>Super</kbd> + <kbd>Shift</kbd> + <kbd>Escape</kbd> (the one shortcut the recording submap keeps), start Omarchist again (every launch leaves the submap), or run:

```bash
hyprctl dispatch 'hl.dsp.submap("reset")'
```

Because that chord belongs to the recorder, you cannot record it; type it instead.
