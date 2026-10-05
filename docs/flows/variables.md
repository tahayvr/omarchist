---
outline: deep
---

# Variables

A step can hand what it produced to the steps after it. You save a step's output under a name, then use it in a later step as `{{name}}`. Omarchist also has built-in variables, such as the clipboard and today's date, that any step can use.

## Save a step's output

Two kinds of step produce output:

| Step | Its output |
| --- | --- |
| **Command** with **Wait until it finishes** on | What the command prints (its standard output). |
| **Flow** | The output of the last step in that flow that produced one. |

Edit the step and type a name in **Save output as**, for example `url`. The step list shows the name next to the step as a small token.

A name starts with a letter and can contain letters, digits, spaces, `-` and `_`, up to 32 characters. Case does not matter: `{{URL}}` and `{{url}}` are the same variable. Output is cut at 64 KB, and the trailing line break a command prints is dropped.

## Use a variable

Type `{{name}}` in a step's text field, or pick a variable from the **Insert** row under the fields: with the keyboard, <kbd>Tab</kbd> to the row, choose with <kbd>←</kbd> and <kbd>→</kbd>, and press <kbd>Enter</kbd>. Outside the text field, Omarchist shows each variable as a token with its name, such as **Clipboard** or **url**. You can use variables in:

- a **Command**, **Terminal**, or **Web app** step,
- a **Notify** step's title and message,
- a **Window** step, inside a quoted value.

A step can use the built-in variables and any name a step **before** it saves. If you move or delete steps so that a step uses a name nothing before it saves, the step list warns you, and the flow cannot be saved until you fix it.

## Built-in variables

| Variable | Holds |
| --- | --- |
| `{{clipboard}}` | What is on the clipboard. |
| `{{selection}}` | The text selected anywhere on screen. |
| `{{date}}` | Today's date, as `2026-10-04`. |
| `{{time}}` | The time, as `14:05`. |
| `{{window}}` | The focused window's title. |
| `{{app}}` | The focused window's app. |
| `{{workspace}}` | The current workspace. |

A built-in is read the first time a step in the run uses it. An empty clipboard reads as empty text.

## Values are never run as commands

A variable always stands for its text, wherever you put it in a command. If your clipboard holds `$(rm -rf ~)`, a command with `{{clipboard}}` receives that text as an argument; it is never run. This holds whether you write `{{clipboard}}`, `"{{clipboard}}"`, or `'{{clipboard}}'`, so you do not need to add quotes yourself.

In a **Window** step, put the variable inside the quotes of a value, as in `hl.dsp.focus({ workspace = "{{name}}" })`. Omarchist escapes the value for you, and refuses a variable outside quotes.

## Example

This flow searches the web for whatever you selected:

```toml
[[step]]
type = "exec"
command = "printf '%s' {{selection}} | jq -sRr @uri"
wait = true
output = "query"

[[step]]
type = "exec"
command = "omarchy-launch-webapp https://duckduckgo.com/?q={{query}}"
```

Bind it to a key and select some text: the first step turns the selection into a URL-safe search term, and the second opens the search.

## When you run the flow

**Run** in the editor shows each step's output under the step, marked **Result**, so you can see what a variable will hold. From the command line, `omarchist flow run` prints each command's output.

Flows that save outputs or use variables are written as file format 2. Omarchist 2.0.0 cannot open them; update Omarchist on every machine that runs your shared flows.
