---
outline: deep
---

# Gallery

The **Gallery** is where people share flows. Browse it from inside Omarchist, read what a flow does, and install it in two clicks. Every flow in it was read by a reviewer before it was listed.

Open it with **Gallery** on the Flows page, with **Flows → Gallery** in the title bar, or from the command palette. Browsing needs no account.

## Find a flow

- Type in the search box to search names, descriptions, authors, and tags.
- The row under it narrows the list to one category. **Installed** shows the flows you have from the gallery.
- **Popular** puts the most installed first. **Newest** puts the latest additions and updates first.

A card shows the flow's icon and name, who made it, the icons of its steps, and how often it was installed. A check mark next to an author's name means the reviewers know their work.

## Read it first

Click a card to see the flow before you take it:

| Part | What it tells you |
| --- | --- |
| **Needs** | The programs the flow runs. One that is not on your machine is marked. |
| **Worth knowing** | What the checks flagged, if anything. See the list below. |
| **Steps** | Every step, as the editor shows it. |

The checks read the commands in a flow and point at the ones that deserve a closer look:

| Flag | Meaning |
| --- | --- |
| Runs a command as administrator | A step uses `sudo` or similar. |
| Deletes files or folders | A step removes a folder and what is in it. |
| Can erase a disk | A step writes to a disk directly. |
| Downloads a script and runs it | A step fetches code from the web and runs it unread. |
| Runs a variable as a shell command | What a variable holds is run as a command. |
| Runs a program named by a variable | The program to start comes from a variable. |
| Reads private keys or passwords | A step touches files such as `~/.ssh`. |
| Changes what runs at startup | A step edits a startup file. |
| Uses the network | A step talks to the internet. |
| Can send your clipboard or selected text over the network | A network step uses what you copied or selected. |

A flag is not a verdict. A flow that updates your system needs `sudo`. It tells you which step to read.

## Install

**Install** opens the flow in the editor, as an unsaved flow. Nothing is saved and nothing runs yet. Read the steps, change what you like, and press **Save**. From then on the flow is yours: give it a keybind, edit it, rename it.

On the Flows page, a flow from the gallery carries a **Gallery** tag.

## Updates

When an author publishes a new version of a flow you installed, its card shows **Update**, and the editor says which version is waiting.

1. Press **See what changes** in the editor, or choose **Show in the gallery** from the card's menu.
2. The dialog lists the steps with what the new version adds and removes.
3. **Update** puts the new steps into the editor, unsaved. Your name for the flow, its icon, its keybind, and its automations stay.
4. Press **Save** to keep the update. <kbd>Ctrl</kbd> + <kbd>Z</kbd> brings your old steps back.

Omarchist looks for updates once a day, and only if you installed something from the gallery.

If the reviewers pull a flow, its card shows **Pulled** and the editor says why. Your copy keeps working; you decide what to do with it.

## Report a flow

**Report**, in a flow's dialog, opens an issue for the reviewers on GitHub with the flow's name filled in. Say what is wrong and send it.

## Publish a flow

You need a GitHub account. In the flow's editor, open the <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu and choose **Publish to the gallery…**.

1. Enter your GitHub user name, pick a category, and add up to five tags.
2. Switch on **Release it under CC0**. Every flow in the gallery is free for anyone to use and change.
3. Press **Continue on GitHub**. Your browser opens the gallery's repository with the flow filled in as a new file. The file's text is also on your clipboard, in case the page comes up empty.
4. On GitHub, press **Commit changes**, then **Create pull request**.

A check runs on your pull request and lists what the flow does. A reviewer reads it and merges it, and the flow appears in the Gallery within minutes.

The flow needs a name of 3 to 48 characters and a description of 10 to 160. Omarchist leaves out what belongs to your machine: the keybind, the launcher entry, and the automations. A flow that runs another flow cannot be shared, because the other flow would not come along.

To publish a new version, change the flow and choose **Publish to the gallery…** again. Omarchist raises the version and opens the file on GitHub. Replace its text with what is on your clipboard and propose the change. Only you can update your flows.

Once you have published, the Gallery offers **Mine**: the flows listed under your name. **Your pull requests on GitHub**, shown with it, opens the ones still in review.

## Check a flow file

Before you import a file someone sent you, or publish your own, the command line tells you what it does:

```bash
omarchist flow check ~/Downloads/focus.flow.toml
```

```
ok      focus.flow.toml  'Focus' (3 steps)
    1. Turn do not disturb on
    2. sudo systemctl stop bluetooth
    3. notify "Focus"
  Needs: omarchy-shell, sudo
  Step 2: Runs a command as administrator
```

`--catalog` also applies the gallery's rules, and `--json` prints the report for scripts.

## Privacy

- Browsing the Gallery downloads its list and the flows you open. No account, no identifier.
- When you save a flow you installed, Omarchist tells the gallery the flow's name and version, so its install count grows. To count each install once, the server keeps a scrambled, one-way mark of your address and the flow for a day, then deletes it. Switch this off in **Settings → Flows → Count My Installs in the Gallery**.
- Publishing and reporting happen on GitHub, in your browser, under your GitHub account.

## How the gallery is kept safe

- Every flow and every new version is a pull request that a person reviews.
- A published version never changes. A change is a new version, and you see what it changes before you take it.
- The gallery's list is signed. Omarchist checks the signature, and checks every flow file against the list, so a flow that was not reviewed cannot reach you through the gallery, even if the server is broken into.
- Installing never runs a flow. You read it, then you save it.

Omarchist keeps a copy of the list, so the Gallery opens without a connection, and says so.
