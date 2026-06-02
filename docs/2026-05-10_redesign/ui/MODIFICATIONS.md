## UI

### Design Language
- Use system/sans-serif as the default font for support of languages.
- use #e91e63 as primary color

### Global

The window control, where the app title sits, is not a part of this design. The title doesn't change when switching tabs, remains as Modkeeper.

Only sematic searching in library should be considered in the future, other AI Integration should be marked as not planned.

This app doesn't include AI function.

### Library Empty state (Library Activation)
The card is identical to the one displayed while there is no mods in a activated library. They should share the same componet. When click the card in the state, it also opens the manage library dialog, instead of choosing a game root, which is never indicated in the UI in anyway.

The toolbar will not be rendered in the library state.

### Library Empty state (No mod)

Only accept .zip.

Doesn't need a drop-in effect in the central card.

The card is clickable, result in the same action as the browse button/put the click listener on the card instead of the button.

### Dialog - Configure Tool

add a browse button to the right of the icon input.

### Dialog - Manage Library

Change the set primary button to activate.  If the library is activated then the text should show activated and the state should be set disabled.

the plus tab button will open a native folder select dialog. If the path is valid and not added before, add a new library with that path, switch the tab to the newly added.

Delete library will trigger another dialog, which request comfirmation. Additional, there is a checkbox in the dialog, indicates that the user can also choose to remove actually files or just remove the entry from the app.

## Insights

- load order is not planned
- Mod detail, live console, conflict/collision visualizer is not currently planned
- Use symlink by default, fallback to copying.
- Ignore the background Process management for now, as it leads more discussion for cross platform compatibility.
