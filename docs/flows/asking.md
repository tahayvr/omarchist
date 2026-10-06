---
outline: deep
---

# Steps that ask

A flow can stop and ask you something. The question opens in Omarchy's own menu, so it looks and works like the rest of your desktop: type, pick with the arrow keys, press <kbd>Enter</kbd>.

Your answer becomes the step's output. Save it under a name and later steps use it as a [variable](/flows/variables). A new asking step already has a name filled in, such as **answer** or **choice**; change it to something that reads well in your flow.

## The asking steps

| Step | Asks | Its output |
| --- | --- | --- |
| **Ask for text** | For a line of text. | What you typed. |
| **Choose from a list** | To pick one option. | The option you picked. |
| **Confirm** | Continue or Cancel. | Nothing. |
| **Pick a file** | For a file, in the desktop's file chooser. | The file's path. |
| **Pick a folder** | For a folder. | The folder's path. |

You can use variables in a question, for example `Close {{choice}}?`.

## Choose from a list

The options come from one of two places:

- **A list**: one field per option. <kbd>Enter</kbd> in a field, or **Add option**, adds another.
- **From a variable**: the lines of a variable become the options. Use it with a command that prints one item per line.

This flow lists your open apps and closes the one you pick:

```toml
[[step]]
type = "exec"
command = "hyprctl clients -j | jq -r '.[].class' | sort -u"
wait = true
output = "open apps"

[[step]]
type = "choose"
prompt = "Close which app?"
from = "{{open apps}}"
output = "choice"

[[step]]
type = "confirm"
prompt = "Close {{choice}}?"

[[step]]
type = "lua"
expr = 'hl.dsp.window.close({ window = "class:{{choice}}" })'
```

A list holds up to 500 options.

## Cancelling

Press <kbd>Esc</kbd> in a question, or pick **Cancel** in a **Confirm** step, and the flow ends there. That is not a failure: the steps after it do not run, and you get no error notification. A question asked by a flow inside another flow ends both.

An empty answer to **Ask for text** counts as cancelling.

## Notifications you can click

A **Notify** step can do something when you click it. Set **When clicked** to:

- **Copy**: puts text on the clipboard.
- **Open**: opens a link, a file, or a folder.

**What to copy** or **What to open** is the text to act on. Leave it empty to use the notification's message. A path can start with `~/`.

```toml
[[step]]
type = "notify"
title = "Noted"
body = "{{note}}"
on_click = "open"
target = "~/Notes/inbox.md"
```

## In the flow file

| Step type | Keys |
| --- | --- |
| `ask` | `prompt` |
| `choose` | `prompt`, and `options` (a list) or `from` (text with one option per line) |
| `confirm` | `prompt` |
| `pick` | `prompt` (optional title), `folder = true` for a folder |
| `notify` | `title`, `body`, `on_click` (`"copy"` or `"open"`), `target` |

Flows that use these steps are written as file format 2.
