---
outline: deep
---

# Flows

A **flow** strings actions together and runs them in order, like Shortcuts on a Mac: open your browser, a terminal, and your music, then say good morning. Build it once on the **Flows** page (<kbd>Ctrl</kbd> + <kbd>4</kbd>) and start it from a keybind, the app launcher, at startup, or from any script.

<img src="/images/flows-light.webp" alt="Flows page" class="screenshot light-only">
<img src="/images/flows-dark.webp" alt="Flows page" class="screenshot dark-only">

Each card shows a flow's steps and how it can be started. **Run** starts it, the pencil edits it, and the <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu duplicates, exports, or deletes it. Deleting a flow also removes its keybind, launcher entry, and startup hook.

**New flow** starts a blank flow. **Templates**, next to it, opens the starter flows and your own templates. Its arrow offers **From scratch**, **From template**, and **Import flow**. The **Flows** menu in the title bar has the same three from any page, plus **Run** for every saved flow.

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
| **Wait** | Pauses for a number of milliseconds (up to ten minutes), for example to let a window appear. |
| **Notify** | Shows a desktop notification, which can copy or open something when you click it. |
| **Ask for text**, **Choose from a list**, **Confirm**, **Pick a file**, **Pick a folder** | Stops and asks you. See [Steps that ask](/flows/asking). |
| **If**, **Repeat**, **Repeat with each**, **Choose from a menu**, **Stop this flow** | Decides what runs, and how often. See [If, Repeat, and menus](/flows/logic). |
| **Set the volume**, **Copy to the clipboard**, **Take a screenshot**, and about thirty more | Ready-made actions with a form instead of a command. See [Ready-made actions](/flows/actions). |

**Add step** opens a list of every kind of step, grouped and searchable: type a few letters and press <kbd>Enter</kbd>, or move with the arrow keys. The row under the search narrows the list to one group. **Change** in a step's form goes back to the list.

A command step starts its program and moves on, which is what opening an app needs; a program that cannot start still fails the step. Turn on **Wait until it finishes** when the next step depends on it having completed; the step list marks such steps with *waits*. The switch on a step turns it off without removing it; the arrows move it up and down. **Keep going when a step fails** lets the rest of the flow run after an error.

A step can pass what it produced to later steps: save its output under a name and use it as `{{name}}`. See [Variables](/flows/variables).

A step whose program is not installed says so under the command. Nothing stops you from saving; it tells you what to install.

**Run** (<kbd>Ctrl</kbd> + <kbd>Enter</kbd>) runs the flow as it is in the editor, showing each step's result and, under a failed step, why it failed. **Stop** ends the run after the current step and stops a command the flow is waiting for. **Save** (<kbd>Ctrl</kbd> + <kbd>S</kbd>) writes it; leaving with unsaved changes asks first, and so does closing Omarchist.

## Triggers

Every saved flow has a command that works from anywhere:

```bash
omarchist flow run morning-start
```

The id comes from the flow's name and never changes, so renaming a flow breaks nothing. The **Run it from** section wires that command up:

- **Keybind** opens the keybind editor with the flow chosen as the action. The same bind appears on the [Keybinds](/configuring/keybinds) page.
- **App launcher** adds the flow to your app menu with its own icon.
- **At startup** runs it after every login through Omarchy's `post-boot` hook.
- **Files menu** adds it to the Files app's right-click **Scripts** menu. The files you picked become the flow's [input](/flows/input).
- **Automations** start it by themselves: at a time, when an app opens, when the charger is unplugged. See [Automations](/flows/automations).

A flow can be given something to work on: words on the command line, piped text, files, or the text you have selected. See [Input](/flows/input).

## Templates

A template is a flow file without an id. Omarchist ships thirty built-in ones; your own go in `~/.config/omarchist/templates/` as `<name>.flow.toml` files. **From template** opens the Templates page, and picking one opens it in the editor as a new, unsaved flow.

<img src="/images/templates-light.webp" alt="Templates page" class="screenshot light-only">
<img src="/images/templates-dark.webp" alt="Templates page" class="screenshot dark-only">

| Template | What it does |
| --- | --- |
| **Morning start** | Opens your browser, a terminal, and music, then says hello. |
| **Focus mode** | Moves to workspace 2 and opens your editor. |
| **Wrap up** | Gives you five seconds, then locks the screen. |
| **Deep work** | Silences notifications for 25 minutes, then reminds you to take a 5-minute break. |
| **Meeting** | Unmutes your mic, keeps the screen awake, silences notifications, and opens Google Meet. |
| **Present** | Hides the bar, silences notifications, keeps the screen awake with no screensaver, and moves to an empty workspace. |
| **Done presenting** | Undoes **Present**: the bar, notifications, idle, and the screensaver come back. |
| **Wind down** | Turns on the night light, dims the screen to 30%, and lowers the volume to 20%. |
| **Battery saver** | Switches to the power-saver profile, dims the screen to 40%, and turns off the keyboard backlight. |
| **Fix my connection** | Restarts Wi-Fi, Bluetooth, and audio. |
| **Search selection** | Searches the web for the text you have selected, or for the words you start it with. Uses [input](/flows/input) in a [ready-made action](/flows/actions). |
| **Daily note** | Opens today's note (`~/Notes/<date>.md`) in your editor, creating it the first time. |
| **Clipboard log** | Adds what you copied to `~/clipboard-log.txt`, with the date and time, and shows it. |
| **Where am I** | Shows the app, window and workspace you are on. |
| **Quick note** | Asks for a line and adds it to `~/Notes/inbox.md`. Click the notification to open the file. Uses [steps that ask](/flows/asking). |
| **Close an app** | Lists the apps that are open and closes the one you pick, after you confirm. |
| **Wallpaper from a file** | Sets a picture as your wallpaper: the file you start it on, or one you pick. |
| **Empty the trash** | Asks first, then empties the trash. |
| **Open copied link** | Opens the link on your clipboard, or tells you there is none. Uses [If and Stop](/flows/logic). |
| **Power by charger** | On battery: the power-saver profile and a dimmer screen. Plugged in: balanced and bright. |
| **Leave** | A menu to lock, suspend, restart, or shut down. Restart and shut down ask first. |
| **Open my sites** | Opens each link of a list in your browser. |
| **Four pomodoros** | Four rounds of 25 minutes of focus and a 5-minute break. |
| **Look up text on screen** | Lets you drag over any text on screen, even in a picture, and searches the web for it. Uses [ready-made actions](/flows/actions). |
| **Weather now** | Shows the weather where you are, from wttr.in, in a notification. |
| **Tidy copied text** | Trims what you copied, turns it into Title Case, and copies it back. |
| **Pick a color** | Pick any color on screen; click the notification to copy its code. |
| **Archive files** | Packs the files you start it on into one archive in your home folder. Made for the **Files menu**. |
| **Low battery** | Switches to power saver, dims the screen, and tells you. Made for the battery [automation](/flows/automations). |
| **Headphones on** | Sets a comfortable volume when your headphones connect. Made for the Bluetooth automation. |

The templates that change settings set things to a state rather than flipping them, so running one twice changes nothing the second time. Edit a template's steps to fit you: a different meeting link, brightness, or volume.

## Sharing flows

- **Export** from a card's menu, the editor's menu (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>E</kbd>), or `omarchist flow export`. The file leaves out the id and everything under **Run it from**, automations included, which belong to your machine.
- **Import** with **Import flow**, by dropping a `.flow.toml` file onto the Flows page, or with `omarchist flow import <file or https URL>`.

An imported flow opens in the editor with a note showing where it came from. Nothing is saved or run until you press **Save**, so read the steps first: a flow is a list of commands, and it runs them as you. The command line prints the steps and asks before saving.

## Flow files

Each flow is one TOML file in `~/.config/omarchist/flows/`, named after its id, so it can be copied, shared, or edited by hand. Omarchist starts every file it writes with a comment that links back to this page. Omarchist reloads the folder whenever the Flows page opens or you press <kbd>Ctrl</kbd> + <kbd>R</kbd>. A file it cannot read is named above the cards with the reason, and a new flow never takes its name.

```toml
# This is an Omarchist flow: https://omarchist.com/flows/

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

- Step types are `exec` (with optional `wait = true`), `lua` (only `hl.dsp.*(...)` calls), `wait` (`ms`), `notify` (`title`, `body`), `flow` (`id`), the [steps that ask](/flows/asking#in-the-flow-file), the [steps that hold steps](/flows/logic#in-the-flow-file), and `action` for a [ready-made action](/flows/actions#in-the-flow-file). `enabled = false` skips a step. `output = "name"` saves a step's output for later steps; see [Variables](/flows/variables).
- `format` is the file layout version. Omarchist writes format 1 unless the flow uses something newer (variables, steps that ask, If and Repeat, ready-made actions), which needs format 2. A file for a newer format is refused with a message to update; a file without the line is read as format 1.
- `input` and `[triggers]` `files` are described under [Input](/flows/input#in-the-flow-file).
- `[meta]` is optional: `author`, `version`, `homepage`, `tags`, `requires` (programs the flow expects), and `source` (the URL it was imported from).
- A file without an `id` takes its file name as the id, so `night-shift.toml` or `night-shift.flow.toml` copied in just works.
