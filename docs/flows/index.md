---
outline: deep
---

# Flows

A **flow** strings actions together and runs them in order, like Shortcuts on a Mac: open your browser, a terminal, and your music, then say good morning. Build it once on the **Flows** page (<kbd>Ctrl</kbd> + <kbd>4</kbd>) and start it from a keybind, the app launcher, at startup, or from any script.

<img src="/images/flows-light.webp" alt="Flows page" class="screenshot light-only">
<img src="/images/flows-dark.webp" alt="Flows page" class="screenshot dark-only">

Each card shows a flow's steps and how it can be started. **Run** starts it, the pencil edits it, and the <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu duplicates, exports, or deletes it. Deleting a flow also removes its keybind, launcher entry, and startup hook.

**New flow** starts a blank flow. Its arrow offers **From scratch**, **From template**, and **Import flow**.

## Building a flow

<img src="/images/flow-editor-light.webp" alt="Flow editor" class="screenshot light-only">
<img src="/images/flow-editor-dark.webp" alt="Flow editor" class="screenshot dark-only">

Give the flow a name, a description, and an icon, then press **Add step** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>) for each step.

| Step | Does |
| --- | --- |
| **App**, **Web app**, **Terminal** | Opens something, or focuses it if it is already open. |
| **Omarchy** | One of Omarchy's own commands, such as taking a screenshot or locking the screen. |
| **Window** | A Hyprland action, such as switching to a workspace. |
| **Flow** | Another flow, run to completion first. A flow cannot run itself. |
| **Command** | Any shell command. |
| **Wait** | Pauses for a number of milliseconds, for example to let a window appear. |
| **Notify** | Shows a desktop notification. |

A command step starts its program and moves on, which is what opening an app needs. Turn on **Wait until it finishes** when the next step depends on it having completed. The switch on a step turns it off without removing it; the arrows reorder steps. **Keep going when a step fails** lets the rest of the flow run after an error.

A step whose program is not installed says so under the command. Nothing stops you from saving; it tells you what to install.

**Run** (<kbd>Ctrl</kbd> + <kbd>Enter</kbd>) runs the flow as it is in the editor, showing each step's result. **Save** (<kbd>Ctrl</kbd> + <kbd>S</kbd>) writes it; leaving with unsaved changes asks first.

## Triggers

Every saved flow has a command that works from anywhere:

```bash
omarchist flow run 'morning-start'
```

The id in quotes comes from the flow's name and never changes, so renaming a flow breaks nothing. The **Run it from** section wires that command up:

- **Keybind** opens the keybind editor with the flow chosen as the action. The same bind appears on the [Keybinds](/configuring/keybinds) page.
- **App launcher** adds the flow to your app menu with its own icon.
- **At startup** runs it after every login through Omarchy's `post-boot` hook.

## Templates

A template is a flow file without an id. The three built-in ones ship with Omarchist; your own go in `~/.config/omarchist/templates/` as `<name>.flow.toml` files. **From template** opens the Templates page, and picking one opens it in the editor as a new, unsaved flow.

<img src="/images/templates-light.webp" alt="Templates page" class="screenshot light-only">
<img src="/images/templates-dark.webp" alt="Templates page" class="screenshot dark-only">

## Sharing flows

- **Export** from a card's menu, the editor's menu, or `omarchist flow export`. The file leaves out the id and triggers, which belong to your machine.
- **Import** with **Import flow**, by dropping a `.flow.toml` file onto the Flows page, or with `omarchist flow import <file or https URL>`.

An imported flow opens in the editor with a note showing where it came from. Nothing is saved or run until you press **Save**, so read the steps first: a flow is a list of commands, and it runs them as you. The command line prints the steps and asks before saving.

## Flow files

Each flow is one TOML file in `~/.config/omarchist/flows/`, named after its id, so it can be copied, shared, or edited by hand. Omarchist reloads the folder whenever the Flows page opens or you press <kbd>Ctrl</kbd> + <kbd>R</kbd>.

```toml
format = 1
id = "focus-mode"
name = "Focus mode"
description = "Moves to workspace 2 and opens your editor."
icon = "target"
on_error = "stop"

[meta]
author = "Taha"
requires = ["spotify"]

[triggers]
launcher = true

[[step]]
type = "lua"
expr = 'hl.dsp.focus({ workspace = "2" })'

[[step]]
type = "exec"
command = "omarchy-launch-editor"
wait = true

[[step]]
type = "notify"
title = "Focus mode"
body = "Everything else can wait"
```

- Step types are `exec` (with optional `wait = true`), `lua` (only `hl.dsp.*(...)` calls), `wait` (`ms`), `notify` (`title`, `body`), and `flow` (`id`). `enabled = false` skips a step.
- `format` is the file layout version. A file for a newer format is refused with a message to update; a file without the line is read as format 1.
- `[meta]` is optional: `author`, `version`, `homepage`, `tags`, `requires` (programs the flow expects), and `source` (the URL it was imported from).
- A file without an `id` takes its file name as the id, so `night-shift.toml` or `night-shift.flow.toml` copied in just works.
