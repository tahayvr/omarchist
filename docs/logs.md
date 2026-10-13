---
outline: deep
---

# Logs

Omarchist keeps a log of what it does. When something goes wrong, the log tells you what happened, and it is what to attach to a bug report.

## Where the logs are

You find them in `~/.local/state/omarchist/logs`, one file a day:

```
omarchist-2026-10-12.log
omarchist-2026-10-12.1.log   # the same day, once the first file passed 10 MB
```

Every part of Omarchist writes to the same file: the window, the [automations](/flows/automations) service, and every flow you run from a keybind, the launcher or the command line. So when an automation starts a flow and the flow fails, you read both in order, in one place.

To open the folder, press **Open folder** under **Logs** in [Settings](/#settings). **Copy debug info** on the About page includes the folder too.

## Reading a log

Each line is one record: when, how serious, which process, where in Omarchist, and what happened.

```
2026-10-12T16:20:01.123-07:00 INFO  [service:4121] flows::automations: Battery below 20%: running 'low-battery'
2026-10-12T16:20:01.410-07:00 WARN  [flow:4133] flows::history: flow 'low-battery' failed after 15 ms (Battery below 20%): step 2 'brightnessctl set 30%': The command exited with status 127
```

The tag in brackets names the process and its id:

| Tag | Process |
| --- | --- |
| `app` | The Omarchist window |
| `service` | The service that runs automations |
| `flow` | A flow run from a keybind, the launcher, the Files menu, a startup hook or the command line |
| `cli` | Any other `omarchist` command |

A record that takes several lines, such as a crash report, continues on indented lines under its first one.

The levels, from most to least serious, are `ERROR`, `WARN`, `INFO`, `DEBUG` and `TRACE`. Every error and warning Omarchist shows you as a notification is in the log as well.

### From the command line

```bash
omarchist logs                   # today's log
omarchist logs --level warn      # only warnings and errors
omarchist logs --since 2h        # the last two hours, across days
omarchist logs --follow          # keep printing new records, until Ctrl+C
omarchist logs --path            # print the folder
```

You can combine them: `omarchist logs -f -l warn` follows warnings and errors only. See [Command Line](/cli#logs).

## What is logged

- What Omarchist changes: every file it writes, every Hyprland reload, every Omarchy setting and theme it applies, the automations service and the bar widget.
- Flows starting and ending, with the step that failed and why, and every command a flow runs.
- What the automations service notices and what it starts.
- Every page you open, update checks and Catalog loads.
- Every warning and error.

Omarchist's own records start at `DEBUG`. Libraries Omarchist is built on only log their warnings and errors.

### Changing it for one run

The `OMARCHIST_LOG` environment variable sets the level for one run:

```bash
OMARCHIST_LOG=info omarchist flow run morning-start   # less: no commands or pages
OMARCHIST_LOG=trace omarchist                         # more from the libraries
```

It also takes a level per part of Omarchist, separated by commas: `OMARCHIST_LOG=flows=trace,gpui=info`. A name is the part shown before the colon in a record (`flows::runner`), or a library's name. `all=` sets the level for every library at once; libraries log only their warnings unless you say otherwise.

## What is never logged

A log may end up in a bug report, so Omarchist leaves out what you type, copy or give a flow:

- the values of a flow's variables, its input, and your answers to its questions
- the clipboard and the selected text
- what a step prints

A flow's commands are logged as you wrote them, with `{{variables}}` still in place of their values.

## Cleanup

Omarchist deletes old logs by itself, when the window or the automations service starts and once a day while the service runs:

- logs older than **Keep logs for** in Settings (3, 7 or 30 days; 7 by default)
- then the oldest logs, until the folder holds 50 MB at most

Today's log is never deleted. `omarchist uninstall` removes the logs along with the rest of `~/.local/state/omarchist`.

## Crashes

If Omarchist crashes, the log ends with an `ERROR` record from `panic`: what went wrong, where, and a backtrace. Attach that record to your [bug report](https://github.com/tahayvr/omarchist/issues/new).
