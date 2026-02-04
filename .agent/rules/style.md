---
trigger: always_on
---

Use early exit and Result wrapper to avoid if nesting more than 1 level.
Ignore TypeScript binding update since it happens when app runs.
Frontend uses bun as package manager.
Check for related tests after refactoring.
Add tests if adding main logic.
Prefer to extract logic that will be used in different places.
Avoid using try-catch, always use Promise, Promise.try if not present.
Avoid using setState in useEffect.
Avoid loading/error state if not specified.
Avoid ignore modules that can be auto-imported via unplugin-auto-import.
Avoid explicitly using useMemo, useCallback if not passed in dependency array,
we're using react-compiler.
Avoid import React namespace, instead import modules that are used.
Prefer inline callback if function is shorter than three lines.
Avoid using open status if there is a synced state present,
e.g., open and id are always set at the same time, use id to indicate open.
Prefer undefined over null if not provided from backend.
Use "ur" to unwrap Result provided by backend, prefer to use like ".then(ur)".
Use i18n solution when displaying text on ui, like t/msg macro or Trans component.
Prefer to use shadcn/ui component to display content if availible.
Avoid using raw text for type, use enum instead.