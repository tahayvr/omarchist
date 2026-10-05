---
outline: deep
---

# If, Repeat, and menus

Some steps hold other steps. An **If** step runs its steps only when a check passes. A **Repeat** step runs them several times. **Choose from a menu** asks you and runs the steps of your choice. In the step list, the steps a block holds sit indented under it.

| Step | Does |
| --- | --- |
| **If** | Runs its steps when a check passes, and the steps under **Otherwise** when it does not. |
| **Repeat** | Runs its steps a number of times. |
| **Repeat with each** | Runs its steps once for every line of a text. |
| **Choose from a menu** | Shows a menu and runs the steps of the choice you pick. |
| **Stop this flow** | Ends the flow there, as finished. |

## Working with blocks

- Each branch ends with an **Add step** line. Click it, or select it and press <kbd>Enter</kbd>, to add a step to that branch.
- **Add step** from the keyboard (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd>) adds after the selected step, in the same branch. With a branch heading selected, it adds to the end of that branch.
- The button under the list always adds to the end of the flow.
- <kbd>Alt</kbd> + <kbd>↓</kbd> and <kbd>Alt</kbd> + <kbd>↑</kbd> move a step one place at a time: past the next step, into a block it meets, and out of the block again.
- <kbd>←</kbd> hides a block's steps and <kbd>→</kbd> shows them. On a step inside a block, <kbd>←</kbd> goes to the block.
- Editing a block changes its check or its count. The steps inside stay.
- Removing, duplicating, moving, or switching off a block does the same to everything inside it.

Blocks can sit inside blocks, up to six deep.

## If

Pick what to check, then how:

| Check | Passes when |
| --- | --- |
| **Text** *is* / *is not* | A text equals another. Capital letters and spaces around the text do not count. |
| **Text** *contains* / *does not contain* | A text has another text in it. |
| **Text** *is empty* / *is not empty* | A text has nothing in it. A variable no step has set yet counts as empty. |
| **Command** *succeeds* / *fails* | A shell command exits with status 0, or does not. |
| **App** *is open* / *is not open* | The app has a window open. |
| **Power** *on battery* / *plugged in* | The machine runs on its battery. A desktop is always plugged in. |
| **Time** *is between* / *is outside* | The time of day is in a range, written as `09:00`. A range such as `22:00` to `06:00` runs over midnight. |

The text is usually a [variable](/flows/variables): the clipboard, an answer, a command's output.

This flow opens the link you copied, and tells you when there is none:

```toml
[[step]]
type = "if"
check = "contains"
value = "{{clipboard}}"
text = "://"
not = true

[[step.then]]
type = "notify"
title = "No link on the clipboard"

[[step.then]]
type = "stop"

[[step]]
type = "exec"
command = "xdg-open {{clipboard}}"
```

## Repeat

**Repeat** runs its steps up to 1000 times. Inside, **Round** (`{{index}}`) is the round you are on, counting from 1.

**Repeat with each** takes a text and runs its steps once per line, skipping empty lines. Inside, **Item** (`{{item}}`) is the line and `{{index}}` its number. It pairs well with a command that prints one thing per line, or with a list of files.

```toml
[[step]]
type = "exec"
command = "printf '%s\\n' https://omarchy.org https://github.com"
wait = true
output = "sites"

[[step]]
type = "each"
items = "{{sites}}"

[[step.do]]
type = "exec"
command = "xdg-open {{item}}"
```

`{{item}}` and `{{index}}` only exist inside their loop. In a loop inside a loop, they belong to the inner one.

## Choose from a menu

Type the choices, one per line. Each choice becomes a branch with its own steps. The choice you pick is the step's output, so the steps can use it.

Pressing <kbd>Esc</kbd> in the menu ends the flow, as with any [step that asks](/flows/asking#cancelling).

When you edit the menu later, a choice keeps its steps as long as its name stays. Rename a choice in place and it keeps the steps of that line.

## Stop this flow

**Stop this flow** ends the flow where it stands. The flow counts as finished, not failed. Use it inside an **If** to leave early. In a flow run by another flow, it ends only the inner one.

## While a flow runs

**Run** in the editor marks each step as it goes. A loop shows the round it is on, and the steps inside show the result of their latest round.

When a step fails inside a block, **Keep going when a step fails** decides what happens next, as it does for any step: the flow stops there, or carries on with the next step.

## In the flow file

A block's steps are tables under it: `then` and `otherwise` for `if`, `do` for the loops, and `choice` with its own `do` for a menu.

| Step type | Keys |
| --- | --- |
| `if` | `check`, the check's keys, `not = true` to turn it around, `then`, `otherwise` |
| `repeat` | `times`, `do` |
| `each` | `items`, `do` |
| `menu` | `prompt`, `choice` (each with `label` and `do`) |
| `stop` | none |

| `check` | Keys |
| --- | --- |
| `equals` | `value`, `to` |
| `contains` | `value`, `text` |
| `empty` | `value` |
| `command` | `command` |
| `app_open` | `class` (the window class) |
| `on_battery` | none |
| `time_between` | `from`, `to` |

```toml
[[step]]
type = "menu"
prompt = "Leave"

[[step.choice]]
label = "Lock"

[[step.choice.do]]
type = "exec"
command = "omarchy-system-lock"

[[step.choice]]
label = "Restart"

[[step.choice.do]]
type = "confirm"
prompt = "Restart now?"

[[step.choice.do]]
type = "exec"
command = "omarchy-system-reboot"
```

Flows that use these steps are written as file format 2.
