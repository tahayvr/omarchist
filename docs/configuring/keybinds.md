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

Type in the search box to match descriptions, commands, or keys (`super k`, `workspace`, `screenshot`). Press the keyboard button next to it, or <kbd>Ctrl</kbd> + <kbd>K</kbd>, to search by pressing keys instead: the page records a key combination and shows every bind on it. While only modifiers are held it shows every bind that uses them.

The filters narrow the list to **Modified** binds, **Conflicts**, Omarchy's defaults, or your own binds (<kbd>Alt</kbd> + <kbd>1</kbd> … <kbd>5</kbd>).

The page opens with the search box focused. <kbd>↓</kbd> moves to the table, where <kbd>↑</kbd> / <kbd>↓</kbd>, <kbd>Home</kbd> / <kbd>End</kbd>, and <kbd>PgUp</kbd> / <kbd>PgDn</kbd> move between rows and <kbd>Escape</kbd> returns to the search box. The full list is on the [Keyboard Navigation](/keyboard) page.

## Changing a keybind

Double-click a row, press <kbd>Enter</kbd> on a selected row, or right-click and choose **Edit**. <kbd>Delete</kbd> disables the selected bind and <kbd>Ctrl</kbd> + <kbd>C</kbd> copies its command.

The **Keys** box records a key combination: click it (or press <kbd>Enter</kbd> while it is focused), then press the keys you want. Recording stops as soon as a complete combination arrives, so <kbd>Escape</kbd> can be recorded too. Use the stop button to cancel.

Some keys never reach an application, such as mouse buttons, media keys, or the layout-independent `code:` keys Omarchy uses for workspaces. For those, type the combination in the text field in Omarchy's own syntax:

```
SUPER + SHIFT + K
SUPER + mouse:272
XF86AudioMute
SUPER + code:10
```

Modifiers are `SUPER`, `SHIFT`, `CTRL`, and `ALT`, joined with `+`, followed by one key name.

You can also change the description and what the bind does; see [Choosing an action](#choosing-an-action). Binds that run a Lua function inside Omarchy's config cannot be re-bound, but they can be disabled.

If the keys you chose are already in use, the dialog lists the other binds. Press **Save** again to keep both.

## Disabling and resetting

**Disable** in the edit dialog or the row menu turns a default bind off. **Reset to default** removes your change and brings back Omarchy's bind. Deleting a bind you added yourself is the same as resetting it.

## Adding a keybind

Press **Add keybind**, record or type the keys, and choose what the bind does. The description fills itself in from your choice until you type your own.

## Choosing an action

The **Action** section builds the command for you. Pick a kind, then fill in its options; the line underneath always shows exactly what Hyprland will run.

- **App**: an application from your menus, with a search box and icons. Turn on *Focus the window if it is already open* to switch to a running window instead of starting another copy, the way Omarchy's own Obsidian and WhatsApp binds behave.
- **Web app**: a URL opened in its own browser window, or one of the web apps you have installed. The same focus switch applies.
- **Terminal**: a command run inside a new terminal window, such as `btop` or `lazydocker`, with the focus switch.
- **Omarchy**: Omarchy's own commands, grouped into apps, menus, panels, capture, notifications, media, window tweaks, display, and system. Search by name or group.
- **Window**: Hyprland actions such as closing, floating, focusing or swapping in a direction, switching or moving to a workspace, the scratchpad, monitors, groups, and resizing. Actions that need a direction, a workspace, or an amount show those controls.
- **Flow**: a [flow](/flows/) from the Flows page, so one key runs a whole sequence of actions.
- **Command**: any shell command, for everything else. It runs through Hyprland's `exec` dispatcher, so shell syntax such as `||` works.

Editing an existing bind opens the kind that matches its command, so an Omarchy default that launches a web app opens on **Web app** with its address filled in. A dispatcher the builder cannot express is kept as is until you choose a different action.

## How it works

Omarchy defines its keybinds in Lua, and Hyprland cannot report what a bind does once it is loaded. Omarchist therefore reads your `~/.config/hypr/hyprland.lua` the same way Omarchy's own keybinding menu does, by evaluating it with the `lua` interpreter and recording every bind in order. The `lua` package must be installed (it is on Omarchy).

Your changes are saved to `~/.config/omarchist/hyprland/keybinds.json` and written as `hl.unbind` and `hl.bind` calls into `~/.config/hypr/omarchist.lua`, which loads after your own `bindings.lua`. Omarchist never edits `bindings.lua`. Hyprland reloads immediately.

While recording, Omarchist switches Hyprland into an empty submap so the compositor does not act on the keys you press, and switches back when recording ends, when the window loses focus, and when the app quits. An input method such as fcitx5 sits below the compositor and keeps its own hotkeys (fcitx5 takes <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>P</kbd> to toggle preedit by default), so a combination it owns cannot be recorded by pressing it; type it in the keys field instead, or change the hotkey in the input method's settings. If Omarchist is killed mid-recording and your shortcuts stop responding, run:

```bash
hyprctl dispatch 'hl.dsp.submap("reset")'
```
