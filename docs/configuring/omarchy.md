---
outline: deep
---

# Omarchy

The **Omarchy** page (the Omarchy icon in the title bar) shows the installed Omarchy version, whether an update is waiting, and the notes for the latest release.

## Version and updates

The version is what `omarchy-version` reports: the installed package version, such as `4.0.4-1`, or `dev (<commit>)` when Omarchy runs from a git checkout through `OMARCHY_PATH`.

Whether an update is available comes from Omarchy's own `omarchy-update-available`, which compares the installed package with the package channel you are on (stable, rc, or edge) and, for a checkout, counts new commits on its upstream branch. The check runs when Omarchist starts, every six hours after that, and when the page opens if the last result is more than five minutes old. **Check again** runs it now.

When an update is pending the page lists it, for example `omarchy 4.0.4-1 -> 4.0.5-1`, and a red dot appears on the Omarchy icon in the title bar. **Update Omarchy** opens `omarchy-update` in a terminal, the same way Omarchy's menu does. The page waits for that update to finish, then checks again, so the badge clears on its own.

If the check cannot run, for example because `checkupdates` is missing, the page says so instead of claiming you are up to date. Note that `checkupdates` reports nothing when the package mirrors cannot be reached, so an offline machine reads as up to date, just as it does in Omarchy's own bar.

## Release notes

The notes come from the latest stable release on GitHub. They describe the newest release, which may differ from the version installed on the rc or edge channel.
