---
outline: deep
---

# Ready-made actions

A ready-made action is a step with a form instead of a command. You pick **Set the volume**, type `40`, and you are done; no script to look up. **Add step** lists them by group, and the search finds them by what they do: type `volume`, `screenshot`, or `copy`.

Text fields take [variables](/flows/variables). An action that produces something saves it under a name, filled in for you, so the next step can use it.

## Text and clipboard

| Action | Does | Output |
| --- | --- | --- |
| **Text** | Builds a text from words and variables. | The text. |
| **Copy to the clipboard** | Puts a text on the clipboard. | |
| **Get the clipboard** | Reads the clipboard at that moment. | What is on it. |
| **Change case** | Turns a text into UPPERCASE, lowercase, or Title Case. | The new text. |
| **Replace text** | Replaces every match of a text with another, or removes it. | The new text. |
| **Trim text** | Removes the spaces and line breaks around a text. | The new text. |
| **Split into lines** | Cuts a text at every separator, such as a comma, into one item per line. | The lines. |

**Split into lines** pairs with **Repeat with each**, which runs its steps once per line. Write `\n` or `\t` for a line break or a tab as the separator.

`{{clipboard}}` holds what was on the clipboard when the flow first used it. Use **Get the clipboard** when an earlier step of the same flow copies something new.

## Apps

| Action | Does |
| --- | --- |
| **Open a link** | Opens a link in your browser. |
| **Open a file or folder** | Opens it with its usual app. A path can start with `~/`. |
| **Focus an app** | Brings an open app's window to the front. |
| **Close an app** | Closes an open app's window. |

**Open an app**, **Open a web app**, and **Run in a terminal** start things; see [Building a flow](/flows/#building-a-flow).

## Desktop

| Action | Does |
| --- | --- |
| **Set the volume** | Sets the volume, from 0 to 100%. |
| **Mute the sound** | Mutes or unmutes the speakers. |
| **Mute the microphone** | Mutes or unmutes the microphone. |
| **Set the brightness** | Sets the screen brightness, from 1 to 100%. |
| **Night light** | Turns the night light on or off. |
| **Do not disturb** | Silences notifications, or lets them through again. |
| **Stay awake** | Keeps the screen from idling, or lets it idle again. |
| **Show or hide the bar** | Shows or hides the top bar. |
| **Power profile** | Switches to power saver, balanced, or performance. |
| **Next wallpaper** | Moves to the theme's next wallpaper. |
| **Switch theme** | Applies one of your themes. |

Each of these sets a state. **Night light** *On* does nothing when the night light is already on, so a flow you run twice ends up the same.

## Capture

| Action | Does | Output |
| --- | --- | --- |
| **Take a screenshot** | Captures a region, a window, or the whole screen, then copies, saves, or opens it for editing. | |
| **Screen recording** | Starts or stops a recording. | |
| **Text from the screen** | Lets you drag over a part of the screen and reads the text in it. | The text. |
| **Pick a color** | Lets you click any pixel. | Its color code, such as `#1a2b3c`. |

## Web

| Action | Does | Output |
| --- | --- | --- |
| **Search the web** | Opens a web search for a text. | |
| **Get a link's contents** | Downloads what a link returns. | The contents, up to 64 KB. |
| **Get a value from JSON** | Reads one value out of JSON, such as `.name` or `.items[0].title`. | The value. |

**Get a link's contents** and **Get a value from JSON** work together: fetch an API, then read the field you want.

```toml
[[step]]
type = "action"
action = "web.get"
url = "https://api.github.com/repos/tahayvr/omarchist/releases/latest"
output = "release"

[[step]]
type = "action"
action = "json.get"
json = "{{release}}"
path = ".tag_name"
output = "version"

[[step]]
type = "notify"
title = "Latest Omarchist"
body = "{{version}}"
```

## What an action needs

Some actions rely on a program: **Get a value from JSON** on `jq`, **Text from the screen** on `tesseract`. Omarchy ships all of them. On a machine where one is missing, the step says so in the step list.

## Values stay values

What you type in a field, and what a variable holds, reaches the action as text. An action never runs it as a command, whatever it contains.

## In the flow file

An action step has `type = "action"`, the action's name in `action`, and its fields next to it:

```toml
[[step]]
type = "action"
action = "volume.set"
level = 40
```

| `action` | Fields |
| --- | --- |
| `text` | `text` |
| `clipboard.set` | `text` |
| `clipboard.get` | none |
| `text.case` | `text`, `to` (`upper`, `lower`, `title`) |
| `text.replace` | `text`, `find`, `with` (optional) |
| `text.trim` | `text` |
| `text.split` | `text`, `by` (default `,`) |
| `open.link` | `url` |
| `open.path` | `path` |
| `app.focus`, `app.close` | `app` (the window class) |
| `volume.set` | `level` (0 to 100) |
| `volume.mute`, `mic.mute` | `state` (`on` mutes, `off` unmutes) |
| `brightness.set` | `level` (1 to 100) |
| `nightlight.set`, `dnd.set`, `awake.set` | `state` (`on`, `off`) |
| `bar.set` | `state` (`on` shows, `off` hides) |
| `power.profile` | `profile` (`power-saver`, `balanced`, `performance`) |
| `wallpaper.next` | none |
| `theme.set` | `theme` (a theme's name) |
| `capture.screenshot` | `of` (`region`, `windows`, `fullscreen`), `then` (`copy`, `save`, `slurp` to edit) |
| `capture.record` | `state` (`on` starts, `off` stops) |
| `capture.text`, `capture.color` | none |
| `web.search` | `query` |
| `web.get` | `url` |
| `json.get` | `json`, `path` |

A field you leave out takes its default. Flows that use actions are written as file format 2.
