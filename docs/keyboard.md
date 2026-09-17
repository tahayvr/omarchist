---
outline: deep
---

# Keyboard Navigation

Everything in Omarchist can be reached from the keyboard. Press <kbd>Ctrl</kbd> + <kbd>/</kbd> at any time (or <kbd>?</kbd> while a list, grid, or table has focus) to open the same shortcut list inside the app.

## The basics

- <kbd>Tab</kbd> and <kbd>Shift</kbd> + <kbd>Tab</kbd> move between controls: the sidebar, the page's controls, then the title-bar menus, and around again.
- Lists, grids, and tab strips are a single Tab stop. Once one has focus, the arrow keys move inside it and <kbd>Home</kbd> / <kbd>End</kbd> jump to its ends.
- <kbd>Enter</kbd> or <kbd>Space</kbd> activates the focused control.
- <kbd>Escape</kbd> steps outward: out of a grid or table to the controls above it, out of a page to the sidebar, and back again from the sidebar to the page.
- The control with focus always shows a ring in the theme's accent color.

## Global

| Keys | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>1</kbd> / <kbd>2</kbd> / <kbd>3</kbd> | Themes, Configuration, Keybinds |
| <kbd>Ctrl</kbd> + <kbd>,</kbd> | Settings |
| <kbd>Ctrl</kbd> + <kbd>N</kbd> | Create a new theme |
| <kbd>Ctrl</kbd> + <kbd>R</kbd> | Reload the current page (rescan keybinds, reload themes or the saved configuration) |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>R</kbd> | Re-apply the current Omarchy theme |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Show or hide the sidebar |
| <kbd>Ctrl</kbd> + <kbd>/</kbd> | Keyboard shortcuts |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>L</kbd> / <kbd>D</kbd> | Light or dark appearance |
| <kbd>Ctrl</kbd> + <kbd>Q</kbd> | Quit |

## Sidebar

<kbd>↑</kbd> / <kbd>↓</kbd> move between pages, <kbd>Enter</kbd> or <kbd>→</kbd> opens the page and puts focus on its first control.

## Themes

The filter tabs (All / Omarchist) are a strip: <kbd>←</kbd> / <kbd>→</kbd> switch, <kbd>Enter</kbd> or <kbd>↓</kbd> goes to the grid.

In the grid:

| Keys | Action |
| --- | --- |
| Arrows, <kbd>Home</kbd>, <kbd>End</kbd>, <kbd>PgUp</kbd>, <kbd>PgDn</kbd> | Move between cards (the grid scrolls with you) |
| <kbd>Enter</kbd> | Apply the theme |
| <kbd>E</kbd> | Edit the theme in the Theme Designer |
| <kbd>O</kbd> | Open the theme folder |
| <kbd>Delete</kbd> | Delete the theme, after a confirmation |
| <kbd>Escape</kbd> | Back to the filter tabs |

## Theme Designer

| Keys | Action |
| --- | --- |
| <kbd>Escape</kbd> or <kbd>Alt</kbd> + <kbd>←</kbd> | Back to Themes |
| <kbd>Ctrl</kbd> + <kbd>PgUp</kbd> / <kbd>PgDn</kbd>, <kbd>Ctrl</kbd> + <kbd>Tab</kbd> / <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>Tab</kbd> | Previous or next tab |
| <kbd>Ctrl</kbd> + <kbd>S</kbd> | Apply the theme |

<kbd>Tab</kbd> walks Back, Apply Theme, the tab strip (<kbd>←</kbd> / <kbd>→</kbd> switch tabs, <kbd>Enter</kbd> jumps into the tab), then every field on the tab. Color pickers open with <kbd>Enter</kbd>, switches toggle with <kbd>Space</kbd>, and the page scrolls to keep the focused section visible.

## Configuration

The section list on the left is a strip: <kbd>↑</kbd> / <kbd>↓</kbd> choose a section, <kbd>Enter</kbd> or <kbd>→</kbd> moves into its settings. Inside, <kbd>Tab</kbd> walks the fields, number fields step with <kbd>↑</kbd> / <kbd>↓</kbd>, switches toggle with <kbd>Space</kbd>, and <kbd>Escape</kbd> returns to the section list. The search box above filters every section at once.

## Keybinds

| Keys | Action |
| --- | --- |
| <kbd>Ctrl</kbd> + <kbd>F</kbd>, or <kbd>/</kbd> from the table | Search |
| <kbd>Ctrl</kbd> + <kbd>K</kbd> | Search by pressing keys |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>N</kbd> | Add a keybind |
| <kbd>Alt</kbd> + <kbd>1</kbd> … <kbd>5</kbd> | All, Modified, Conflicts, Omarchy, Mine |
| <kbd>↓</kbd> in the search box | Go to the table |
| <kbd>Escape</kbd> in the search box | Clear the search, then go to the table |

In the table: <kbd>↑</kbd> / <kbd>↓</kbd>, <kbd>Home</kbd> / <kbd>End</kbd>, <kbd>PgUp</kbd> / <kbd>PgDn</kbd> move, <kbd>Enter</kbd> edits, <kbd>Delete</kbd> disables, <kbd>Ctrl</kbd> + <kbd>C</kbd> copies the command, and <kbd>Escape</kbd> returns to the search box. The filter strip cycles with <kbd>←</kbd> / <kbd>→</kbd>.

## Dialogs

Dialogs open with their first control focused and keep <kbd>Tab</kbd> inside. <kbd>Escape</kbd> cancels; <kbd>Ctrl</kbd> + <kbd>Enter</kbd> confirms, even from a text field. In the keybind editor, <kbd>Enter</kbd> on the recorder starts recording and <kbd>Backspace</kbd> clears it.
