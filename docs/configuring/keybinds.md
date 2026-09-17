---
outline: deep
---

# Keybinds

The **Keybinds** page lists every keyboard shortcut Hyprland runs on your Omarchy system and lets you change them without editing Lua. Open it from the left sidebar or with <kbd>Ctrl</kbd> + <kbd>3</kbd>.

## What you see

Each row is one keybind:

- **Action**: the description Omarchy gives the bind, or its command when it has none.
- **Keystrokes**: the key combination as keycaps.
- **Command**: the shell command or Hyprland dispatcher the bind runs. Hover to see the full text.
- **Source**: where the bind comes from. **Default** binds ship with Omarchy, **User** binds come from your own `~/.config/hypr/bindings.lua`, and **Omarchist** binds are the ones you changed on this page. A **Modified**, **Custom**, **Disabled**, or **Unbound** tag shows what happened to it.

Rows with a warning icon share their keys with another bind. Hyprland runs every bind on a chord, so both fire. Omarchy stacks a few of its own binds on purpose (for example <kbd>Alt</kbd> + <kbd>Tab</kbd>), and those are not flagged.

## Searching

Type in the search box to match descriptions, commands, or keys (`super k`, `workspace`, `screenshot`). Press the keyboard button next to it to search by pressing keys instead: the page records a key combination and shows every bind on it. While only modifiers are held it shows every bind that uses them.

The filters narrow the list to **Modified** binds, **Conflicts**, Omarchy's defaults, or your own binds.

## Changing a keybind

Double-click a row, press <kbd>Enter</kbd> on a selected row, or right-click and choose **Edit**.

The **Keys** box records a key combination: click it (or press <kbd>Enter</kbd> while it is focused), then press the keys you want. Recording stops as soon as a complete combination arrives, so <kbd>Escape</kbd> can be recorded too. Use the stop button to cancel.

Some keys never reach an application, such as mouse buttons, media keys, or the layout-independent `code:` keys Omarchy uses for workspaces. For those, type the combination in the text field in Omarchy's own syntax:

```
SUPER + SHIFT + K
SUPER + mouse:272
XF86AudioMute
SUPER + code:10
```

Modifiers are `SUPER`, `SHIFT`, `CTRL`, and `ALT`, joined with `+`, followed by one key name.

You can also change the description and, for binds that run a shell command, the command itself. Binds that run a Hyprland dispatcher (such as closing a window) keep their action; only the keys change. Binds that run a Lua function inside Omarchy's config cannot be re-bound, but they can be disabled.

If the keys you chose are already in use, the dialog lists the other binds. Press **Save** again to keep both.

## Disabling and resetting

**Disable** in the edit dialog or the row menu turns a default bind off. **Reset to default** removes your change and brings back Omarchy's bind. Deleting a bind you added yourself is the same as resetting it.

## Adding a keybind

Press **Add keybind**, record or type the keys, give the bind a description, and enter the command to run.

## How it works

Omarchy defines its keybinds in Lua, and Hyprland cannot report what a bind does once it is loaded. Omarchist therefore reads your `~/.config/hypr/hyprland.lua` the same way Omarchy's own keybinding menu does, by evaluating it with the `lua` interpreter and recording every bind in order. The `lua` package must be installed (it is on Omarchy).

Your changes are saved to `~/.config/omarchist/hyprland/keybinds.json` and written as `hl.unbind` and `hl.bind` calls into `~/.config/hypr/omarchist.lua`, which loads after your own `bindings.lua`. Omarchist never edits `bindings.lua`. Hyprland reloads immediately.

While recording, Omarchist switches Hyprland into an empty submap so the compositor does not act on the keys you press, and switches back when recording ends, when the window loses focus, and when the app quits. If Omarchist is killed mid-recording and your shortcuts stop responding, run:

```bash
hyprctl dispatch 'hl.dsp.submap("reset")'
```
