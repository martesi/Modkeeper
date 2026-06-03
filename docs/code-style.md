## Backend

- Use early exit and Result wrapper to avoid if nesting more than 1 level.

## Frontend
- Avoid using try-catch, always use Promise, Promise.try if not present.
- Avoid using setState in useEffect.
- Avoid loading/error state if not specified.
- Avoid ignore modules that can be auto-imported via unplugin-auto-import.
- Avoid explicitly using useMemo, useCallback if not passed in dependency array.
- Avoid import React namespace, instead import modules that are used.
- Prefer inline callback if function is shorter than three lines.
- Avoid using open status if there is a synced state present,
  e.g., open and id are always set at the same time, use id to indicate open.
- Prefer undefined over null if not provided from backend.
- Use "ur" to unwrap Result provided by backend, prefer to use like ".then(ur)".
- Use i18n solution when displaying text on ui, like t/msg macro or Trans component.
- Centralize translation helper functions in a single util/module per namespace, and import those helpers from UI components instead of redefining ad hoc text functions.
- Name translation helper functions with the `tText` pattern, such as `libraryText.tTextManageLibraries()` or `commonText.tTextCancel()`.
- Prefer to use shadcn/ui component to display content if availible.
- Avoid using raw text for type, use enum instead.

## General

- Avoid comments.
- Use inline function if more than 5 lines in a conditional block.
