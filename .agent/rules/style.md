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