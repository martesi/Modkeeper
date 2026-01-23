Prefer FP style if no significant performance difference;
Prefer to avoid if nesting more than one level with Option/Result wrapper;
Prefer early exit to avoid nesting.
Ignore TypeScript binding update since it happens when app runs.
Frontend uses bun as package manager.
Check for related tests after refactoring.
Add tests if adding main logic.
Prefer to extract logic that will be used in different places.
Avoid using try-catch, always use Promise, Promise.try if not present.
Avoid using setState in useEffect.
Avoid loading/error state if not specified.