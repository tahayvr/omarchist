---
outline: deep
---

# Automations

An automation starts a flow by itself: at nine every weekday, when you unplug the charger, when your headphones connect. You do nothing; the flow runs.

Automations live in the flow editor's **Run it from** card. **Add automation** asks what to wait for, and the list under **Automations** shows each one with a switch to turn it off without removing it.

## Turn automations on

Automations are run by a small background service, so they work with the Omarchist window closed. It is off until you turn it on:

- in the flow editor, **Turn on** appears next to your first automation, or
- in **Settings**, under **Flows**: **Run Automations in the Background**.

The service starts with your session and stops when you log out. Turning it off in **Settings** removes it again.

## What an automation can wait for

| Group | Automation | The flow's input |
| --- | --- | --- |
| Time | **At a time of day**, every day or on the days you pick | |
| | **Every so often**, from every minute to once a day | |
| Apps and windows | **An app opens** (its first window) | The app |
| | **An app closes** (its last window) | The app |
| | **I switch to a workspace** | The workspace |
| | **A window goes full screen** | |
| Devices and network | **A display is connected** or **disconnected** | The display |
| | **A Bluetooth device connects**: one you name, or any | The device |
| | **A USB device is plugged in**: one you name, or any | The device |
| | **I join a Wi-Fi network**: one you name, or any | The network |
| Power | **The charger is plugged in** or **unplugged** | |
| | **The battery runs low**: falls below a level you set | The level |
| Session | **The computer wakes up** | |
| | **The screen is locked** or **unlocked** | |

For networks and devices, the dialog offers the ones it can see now. Leave the name empty for any.

What an automation is about reaches the flow as its [input](/flows/input): `{{input}}` holds the name of the headphones that connected, or the network you joined.

## Ask before running

Turn on **Ask before running** and the automation shows a notification, **Run** *flow name*, in place of running at once. Click it to run the flow. Ignore it, and nothing happens.

## Good to know

- A time that passes while the computer is asleep or off is skipped. **Every so often** counts from when the service started.
- A burst of the same event starts a flow once: an automation waits five seconds before it can run again.
- A flow that is still running is not started a second time.
- An automation runs the flow as saved. Save your changes for them to take effect.
- Power automations need a laptop battery.

## Flows made for automations

Three built-in [templates](/flows/#templates) pair well with an automation:

| Template | Add this automation |
| --- | --- |
| **Power by charger** | The charger is plugged in, and another for unplugged |
| **Low battery** | The battery runs low |
| **Headphones on** | A Bluetooth device connects |

A template never comes with automations. You add them once the flow is yours.

## From the command line

```bash
omarchist automations on       # install and start the service
omarchist automations off      # stop and remove it
omarchist automations status   # is it running, and what does it wait for
journalctl --user -u omarchist-automations   # what it did
```

## In the flow file

Each automation is a `[[triggers.automation]]` table. `on` says what to wait for:

| `on` | Keys |
| --- | --- |
| `time` | `at` (`"09:00"`), `days` (`["mon", "tue"]`; leave out for every day) |
| `every` | `minutes` |
| `app_opened`, `app_closed` | `class` (the window class) |
| `workspace` | `name` |
| `monitor_connected`, `monitor_disconnected`, `fullscreen` | none |
| `bluetooth_connected`, `usb_connected` | `device` (leave out for any) |
| `wifi_joined` | `name` (leave out for any) |
| `charger_connected`, `charger_disconnected` | none |
| `battery_below` | `percent` |
| `woke`, `locked`, `unlocked` | none |

Every automation also takes `ask = true` and `enabled = false`.

```toml
[[triggers.automation]]
on = "time"
at = "09:00"
days = ["mon", "tue", "wed", "thu", "fri"]

[[triggers.automation]]
on = "bluetooth_connected"
device = "WH-1000XM5"
ask = true
```

Automations belong to your machine, like the keybind and the launcher entry: [exporting](/flows/#sharing-flows) a flow leaves them out.
