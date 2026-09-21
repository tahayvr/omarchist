---
outline: deep
---

# Flows

A **flow** strings actions together and runs them in order, the way Shortcuts does on a Mac or iPhone: open your browser, a terminal, and your music, then say good morning. Build it once on the **Flows** page (<kbd>Ctrl</kbd> + <kbd>4</kbd>), then start it from a keybind, the app launcher, at startup, or from any script with one command.

## The Flows page

Every flow is a card with its icon, description, the shape of its steps, and how it can be started. **Run** starts it right away, **Edit** opens the editor, and the menu on the right duplicates or deletes it. Deleting a flow also removes its keybind, launcher entry, and startup hook.

An empty page offers three starter flows: **Morning start**, **Focus mode**, and **Wrap up**. Pick one to open it in the editor, or start from scratch with **New flow** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>).

The cards are one keyboard stop: <kbd>↓</kbd> from the search box reaches them, arrows move between them, <kbd>Enter</kbd> edits, <kbd>Ctrl</kbd> + <kbd>Enter</kbd> runs, <kbd>Ctrl</kbd> + <kbd>D</kbd> duplicates, and <kbd>Delete</kbd> deletes.

## Building a flow

The editor has the flow's details and triggers on the left and its steps on the right.

Give it a name, a description, and an icon. The icon shows on the card and in the app launcher. **Keep going when a step fails** lets the rest of the flow run when one step reports an error; otherwise the flow stops there.

### Steps

Press **Add step** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>) and choose what the step does. The first seven kinds are the same ones a keybind can run, described under [Choosing an action](/configuring/keybinds#choosing-an-action):

- **App**, **Web app**, **Terminal**: open something, or focus it if it is already open.
- **Omarchy**: one of Omarchy's own commands, such as toggling night light, taking a screenshot, or locking the screen.
- **Window**: a Hyprland action such as switching to a workspace or moving the focused window.
- **Flow**: another flow, run to completion before the next step. A flow cannot run itself, and a loop between flows is refused at run time.
- **Command**: any shell command.

A step that runs a command starts it and moves straight on, which is what opening an app needs. Turn on **Wait until it finishes** when the next step depends on the command having completed (a theme switch, a file copy): the flow then waits for it to exit and counts a non-zero status as a failure.

Two kinds exist only in flows:

- **Wait** pauses for a number of milliseconds, for example to give a window time to appear before the next step moves it.
- **Notify** shows a desktop notification, handy as the last step so you know the flow ran.

Each step card shows what it does and, underneath, the exact command or dispatcher. The switch on the right turns a step off without removing it. The arrows reorder steps, and the pencil edits one.

The step list is one keyboard stop: <kbd>↑</kbd> / <kbd>↓</kbd> select, <kbd>Enter</kbd> edits, <kbd>Space</kbd> turns the step on or off, <kbd>Alt</kbd> + <kbd>↑</kbd> / <kbd>↓</kbd> move it, <kbd>Ctrl</kbd> + <kbd>D</kbd> duplicates it, and <kbd>Delete</kbd> removes it.

### Running it

**Run** (<kbd>Ctrl</kbd> + <kbd>Enter</kbd>) runs the flow as it is in the editor, saved or not. Each step shows a spinner while it runs and a check or a cross when it is done, and a notification reports the outcome. **Save** (<kbd>Ctrl</kbd> + <kbd>S</kbd>) writes the flow; leaving with unsaved changes asks first.

## Triggers

Every saved flow has a command that works from anywhere:

```bash
omarchist flow run 'morning-start'
```

The id in quotes comes from the flow's name and never changes, so renaming a flow does not break anything that runs it. The name works too: `omarchist flow run "Morning start"`. `omarchist flow list` prints every flow with its id. A flow that fails from the command line also raises a desktop notification, so a keybind never fails silently.

The **Run it from** section wires that command up for you:

- **Keybind** opens the keybind editor with the flow already chosen as the action; record the keys and save. The chord then shows on the flow's card. The same bind appears on the [Keybinds](/configuring/keybinds) page, where **Flow** is one of the action kinds, so any keybind can run a flow.
- **App launcher** adds the flow to your app menu with its own icon, by writing a `.desktop` entry to `~/.local/share/applications/`.
- **At startup** runs the flow after every login, through Omarchy's `post-boot` hook (`~/.config/omarchy/hooks/post-boot.d/`).

## Sharing flows

A flow is one file, so sharing it is moving that file.

- **Export**: open the menu on a flow's card and choose **Export…**, or run `omarchist flow export <name>`. The file is written as `<id>.flow.toml` without the parts that belong to your machine (the id and the triggers), so the other side gets a clean copy.
- **Import**: choose **Import a file…** under **New flow**, drop a `.flow.toml` file onto the Flows page, or run `omarchist flow import <file or https:// URL>`. The flow opens in the editor with a notice showing where it came from. Nothing is saved or run until you press **Save**, so read the steps first: a flow is a list of commands, and an imported one runs them as you. The command line prints the steps and asks before saving; pass `--yes` to skip the question in a script.

An imported flow gets a new id from its name, and a flow imported from a URL remembers that URL in its `[meta]` table.

## Where flows live

Each flow is one TOML file in `~/.config/omarchist/flows/`, named after its id, so a flow can be copied to another machine, shared, or edited by hand (comments welcome):

```toml
format = 1
id = "focus-mode"
name = "Focus mode"
description = "Moves to workspace 2 and opens your editor."
icon = "target"
on_error = "stop"

[triggers]
launcher = true

[[step]]
type = "lua"
expr = 'hl.dsp.focus({ workspace = "2" })'

[[step]]
type = "exec"
command = "omarchy-launch-editor"

[[step]]
type = "notify"
title = "Focus mode"
body = "Everything else can wait"
```

Step types are `exec` (with an optional `wait = true`), `lua` (only `hl.dsp.*(...)` calls are accepted), `wait` (`ms`), `notify` (`title`, `body`), and `flow` (`id`). A step with `enabled = false` is skipped. Omarchist reloads the directory whenever the Flows page opens or <kbd>Ctrl</kbd> + <kbd>R</kbd> is pressed.

A file you copy in only needs the right name: when `id` is missing, the file name (without `.toml`) becomes the id. The name must be lowercase letters, digits, and hyphens.

`format` is the version of the file layout, `1` today. Omarchist refuses a file written for a newer format and tells you to update; a file without the line is read as format 1.

An optional `[meta]` table describes the flow rather than what it does. All of it is optional, and the editor never needs it:

```toml
[meta]
author = "Taha"
version = "1.0"
homepage = "https://example.com/flows"
tags = ["morning", "work"]
requires = ["spotify"]
```

`requires` lists programs the flow expects to find on the machine. The built-in templates are flow files in this format, shipped inside Omarchist.
