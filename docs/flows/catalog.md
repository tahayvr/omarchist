---
outline: deep
---

# Catalog

The **Catalog** is where people share flows. Browse it from inside Omarchist, read what a flow does, and install it in two clicks. Every flow in it was read by a reviewer before it was listed.

<img src="/images/catalog-light.webp" alt="The Catalog" class="screenshot light-only">
<img src="/images/catalog-dark.webp" alt="The Catalog" class="screenshot dark-only">

Open it with **Catalog** on the Flows page, with **Flows → Catalog** in the title bar, or from the command palette. Browsing needs no account.

## Find a flow

- Type in the search box to search names, descriptions, authors, and tags.
- The row under it narrows the list to one category. **Installed** shows the flows you have from the catalog.
- **Newest** puts the latest additions and updates first. **Popular** puts the most installed first.

A card shows the flow's icon and name, who made it, the icons of its steps, and how often it was installed. A check mark next to an author's name means the reviewers know their work.

## Read it first

Click a card to see the flow before you take it:

<img src="/images/catalog-detail-light.webp" alt="A flow in the Catalog" class="screenshot light-only">
<img src="/images/catalog-detail-dark.webp" alt="A flow in the Catalog" class="screenshot dark-only">

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

A flow's dialog is where you read it before it reaches your machine. It shows who made it, what it does, what it needs, what is worth knowing, and every step. A step that runs a command shows that command in full, so there is nothing to guess at.

**Install** saves the flow into your flows folder and opens it in the editor. From then on the flow is yours: run it, give it a keybind, edit it, rename it. Nothing runs until you run it.

A flow you import from a file or a link, or an update from the catalog, opens in the editor unsaved, for you to read first. It cannot run until you save it.

On the Flows page, a flow from the catalog carries a **Catalog** tag.

## Updates

When an author publishes a new version of a flow you installed, its card shows **Update**, and the editor says which version is waiting.

1. Press **See what changes** in the editor, or choose **Show in the catalog** from the card's menu.
2. The dialog lists the steps with what the new version adds and removes.
3. **Update** puts the new steps into the editor, unsaved. Your name for the flow, its icon, its keybind, and its automations stay.
4. Press **Save** to keep the update. <kbd>Ctrl</kbd> + <kbd>Z</kbd> brings your old steps back.

Omarchist looks for updates once a day, and only if you installed something from the catalog.

If the reviewers pull a flow, its card shows **Pulled** and the editor says why. Your copy keeps working; you decide what to do with it.

## Report a flow

**Report**, in a flow's dialog, opens an issue for the reviewers on GitHub with the flow's name filled in. Say what is wrong and send it.

## Publish a flow

You need a GitHub account. In the flow's editor, open the <span class="icon-inline icon-inline-more" aria-hidden="true"></span> menu and choose **Publish to the catalog…**.

1. Enter your GitHub user name, pick a category, and add up to five tags.
2. Switch on **Release it under CC0**. Every flow in the catalog is free for anyone to use and change.
3. Press **Continue on GitHub**. Your browser opens the catalog's repository with the flow filled in as a new file. The file's text is also on your clipboard, in case the page comes up empty.
4. On GitHub, press **Commit changes**, then **Create pull request**.

A check runs on your pull request and lists what the flow does. A reviewer reads it and merges it, and the flow appears in the Catalog within minutes.

The flow needs a name of 3 to 48 characters and a description of 10 to 160. Omarchist leaves out what belongs to your machine: the keybind, the launcher entry, and the automations. A flow that runs another flow cannot be shared, because the other flow would not come along.

To publish a new version, change the flow and choose **Publish to the catalog…** again. Omarchist raises the version and opens the file on GitHub. Replace its text with what is on your clipboard and propose the change. Only you can update your flows.

Once you have published, the Catalog offers **Mine**: the flows listed under your name. **Your pull requests on GitHub**, shown with it, opens the ones still in review.

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

`--catalog` also applies the catalog's rules, and `--json` prints the report for scripts.

## Privacy

- Browsing the Catalog downloads its list and the flows you open. No account, no identifier.
- When you install a flow, Omarchist tells the catalog the flow's name and version, so its install count grows. To count each install once, the server keeps a scrambled, one-way mark of your address and the flow for a day, then deletes it. Switch this off in **Settings → Flows → Count My Installs in the Catalog**.
- Publishing and reporting happen on GitHub, in your browser, under your GitHub account.

## How the catalog is kept safe

- Every flow and every new version is a pull request that a person reviews.
- A published version never changes. A change is a new version, and you see what it changes before you take it.
- The catalog's list is signed. Omarchist checks the signature, and checks every flow file against the list, so a flow that was not reviewed cannot reach you through the catalog, even if the server is broken into.
- Installing never runs a flow. You read it in its dialog, with every command in full, then you install it.

Omarchist keeps a copy of the list, so the Catalog opens without a connection, and says so.
