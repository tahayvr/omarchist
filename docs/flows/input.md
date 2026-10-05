---
outline: deep
---

# Input

A flow can work on something you give it: a text, a link, a few files. Inside the flow, that something is the variable **Input** (`{{input}}`). Use it like any other [variable](/flows/variables).

## Give a flow its input

| Start the flow | Its input |
| --- | --- |
| From the command line, with words after `--` | The words, one per line. |
| With text piped into it | The text. |
| On files, from the **Files menu** | The files' paths, one per line. |
| From another flow, with **Input** filled in | What that step hands over. |
| By an [automation](/flows/automations) | What the automation is about: the app, the network, the device. |
| With a keybind, the launcher, or **Run** | Whatever **Started without input, use** says. |

```bash
omarchist flow run search-selection -- rust traits
git log --oneline | omarchist flow run pick-a-commit
omarchist flow run archive-files -- ~/Documents/a.pdf ~/Documents/b.pdf
```

## Started without input

Once a flow uses **Input**, the **Run it from** card shows **Started without input, use**:

| Choice | The input is |
| --- | --- |
| **Nothing** | Empty. An **If** step's *is empty* check tells you so. |
| **Selected text** | The text selected anywhere on screen. |
| **Clipboard** | What is on the clipboard. |
| **Ask** | What you type when the flow asks. Pressing <kbd>Esc</kbd> ends the flow. |

**Selected text** makes a flow that acts on whatever you highlight: select a word, press the flow's keybind. The **Search selection** template works this way, and still takes words from the command line.

## Run a flow on files

Turn on **Files menu** in **Run it from** and save. In the Files app, right-click one or more files, open **Scripts**, and pick the flow. Its input is the files, one path per line.

**Repeat with each** goes through them one at a time:

```toml
[[step]]
type = "each"
items = "{{input}}"

[[step.do]]
type = "exec"
command = "cp {{item}} ~/Backup/"
wait = true
```

The menu entry carries the flow's name. Renaming the flow renames the entry, and deleting the flow removes it.

## Hand input to another flow

A **Run a flow** step has an **Input** field. What you put there, variables included, becomes that flow's input. With the field empty, the other flow starts without input and falls back to its own setting.

The other flow's output comes back the same way a command's does: save it under a name. Together, the two make a flow you can use as a building block:

```toml
[[step]]
type = "flow"
id = "shorten-link"
input = "{{clipboard}}"
output = "short link"
```

## In the flow file

- `input = "selection"`, `"clipboard"`, or `"ask"` at the top of the file says what to use when the flow is started without input. Leave it out for nothing.
- `[triggers]` takes `files = true` for the Files menu.
- A `flow` step takes `input`.

Flows that use these are written as file format 2.
