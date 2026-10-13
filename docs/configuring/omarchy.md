---
outline: deep
---

# Omarchy

The **Omarchy** dialog shows the installed version, whether an update is waiting, and the latest release notes. It opens over whatever page you are on: click the Omarchy icon in the title bar, choose **Omarchy** in the command palette, or run `omarchist --view omarchy`.

<img src="/images/omarchy-light.webp" alt="Omarchy dialog" class="screenshot light-only">
<img src="/images/omarchy-dark.webp" alt="Omarchy dialog" class="screenshot dark-only">

The version and the update check come from Omarchy's own tools, `omarchy-version` and `omarchy-update-available`, so they match your package channel. The check runs when you open the dialog, and the <span class="icon-inline icon-inline-refresh"></span> button next to the status (**Check again**) runs it now. In the background it runs at startup and then on the interval you choose under **Omarchy Updates** on the Settings page (every six hours by default); switch **Check in the Background** off there to stop it. A pending update shows what it is, puts a red dot on the Omarchy icon, and **Update Omarchy** runs `omarchy-update` in a terminal. The dialog checks again when that finishes.

If the check cannot run, the dialog says so. An offline machine reads as up to date, as it does in Omarchy's own bar.

The release notes take focus when the dialog opens, so <kbd>↑</kbd>, <kbd>↓</kbd>, <kbd>Page Up</kbd> and <kbd>Page Down</kbd> scroll them. <kbd>Ctrl</kbd> + <kbd>R</kbd> checks for updates and loads the notes again.
