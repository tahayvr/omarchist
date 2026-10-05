---
outline: deep
---

# Run history

Omarchist keeps the last 50 runs of every flow, whatever started them. When a keybind did nothing or an automation ran while you were away, the history says what happened.

## See a flow's runs

- In the editor, press the **Run history** button next to **Run** (<kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>H</kbd>).
- On the **Flows** page, open a card's <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu and choose **Run history**.

A card also shows when its flow last ran and how that run ended.

Each run says what started it, when, and how long it took. Open a run to see every step: whether it worked, the first line of what it produced or the reason it failed, and its time. Steps inside a block sit under it, once per round.

| A run ends as | Meaning |
| --- | --- |
| **Finished** | Every step ran, or the flow reached **Stop this flow**. |
| **Failed** | A step failed. With **Keep going when a step fails**, the rest still ran. |
| **Cancelled** | You dismissed a question the flow asked. |
| **Stopped** | The run was ended from outside: **Stop** in the editor, `omarchist flow stop`, or logging out. |

What started a run reads as **Keybind**, **Launcher**, **Startup**, **Files menu**, **Terminal**, **Script**, **Editor**, **Omarchist** (a card or the title bar menu), or the automation's own words, such as *At 09:00 on weekdays*.

| Key | Does |
| --- | --- |
| <kbd>↑</kbd> <kbd>↓</kbd> | Moves through the runs. |
| <kbd>Enter</kbd> or <kbd>Space</kbd> | Opens or closes the run. |
| <kbd>Home</kbd> <kbd>End</kbd> | Jumps to the newest or oldest run. |

**Clear history** forgets the flow's runs. Deleting a flow does the same.

## Stop a running flow

**Stop** in the editor ends the run you started there. To end a flow that something else started, use its name or id:

```bash
omarchist flow stop morning-start
```

The flow ends after its current step. A command it is waiting for is stopped with it.

## From the command line

```bash
omarchist flow history morning-start
```

```
Finished   5 min ago      1.2 s     Keybind
Failed     yesterday      0.4 s     At 09:00 on weekdays
           step 2: sh: line 1: notes: command not found
```

`--json` prints every kept run with its steps, for scripts.

## Where it is kept

Runs are lines in `~/.local/state/omarchist/runs/<id>.jsonl`. A step's output is kept as its first line, cut at 240 characters, so a history never holds a whole clipboard or file.
