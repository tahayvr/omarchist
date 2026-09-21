---
outline: deep
---

# Omarchy

The **Omarchy** page (the Omarchy icon in the title bar) shows the installed version, whether an update is waiting, and the latest release notes.

<img src="/images/omarchy-light.webp" alt="Omarchy page" class="screenshot light-only">
<img src="/images/omarchy-dark.webp" alt="Omarchy page" class="screenshot dark-only">

The version and the update check come from Omarchy's own tools, `omarchy-version` and `omarchy-update-available`, so they match your package channel. The check runs at startup, every six hours, and when you open the page; **Check again** runs it now. A pending update shows what it is, puts a red dot on the Omarchy icon, and **Update Omarchy** runs `omarchy-update` in a terminal. The page checks again when that finishes.

If the check cannot run, the page says so. An offline machine reads as up to date, as it does in Omarchy's own bar.
