Prefer FP style if no significant performance difference;
Prefer to avoid if nesting more than one level with Option/Result wrapper;
Prefer early exit to avoid nesting.
Ignore TypeScript binding update since it happens when app runs.
Frontend uses bun as package manager.
Check for related tests after refactoring.
Add tests if adding main logic.