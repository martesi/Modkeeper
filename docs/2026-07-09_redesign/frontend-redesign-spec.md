# Frontend Redesign Implementation Spec

Status: Implementation-ready.

Supersedes `docs/2026-05-10_redesign/frontend-redesign-spec.md`. That draft was labeled "ready for
implementation" while its own data/API contract still had 10 open review items. This revision is
ready for real: `frontend-redesign-data-api-contract.md` now resolves all of them, and this spec's
repository/type signatures have been updated to match (field names, bulk-only mod status calls,
async cache rebuild).

Applies to: Phase 3 frontend redesign in `docs/2026-07-09_redesign/outline-of-redesign.md`.

Inputs:
- `docs/2026-05-10_redesign/ui/PRD.md`
- `docs/2026-07-09_redesign/outline-of-redesign.md`
- `docs/2026-05-10_redesign/ui/MODIFICATIONS.md`
- `docs/2026-05-10_redesign/ui/fidelity_modern/DESIGN.md`
- `docs/2026-05-10_redesign/audits/1.8_frontend-redesign-audit.md`
- `docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md`
- `docs/2026-07-09_redesign/purpose-of-redesign.md` (Execution Strategy — this is now the canonical
  statement of the build-fresh-verify-continuously approach; §3 below restates it in implementation
  terms only)

## 1. Purpose

Create a new frontend implementation plan for the Fidelity Modern redesign while keeping the current frontend files available as reference material. The redesign must be built from new feature and shell files instead of continuing the existing `src/modules/*` layout.

The current app can be studied and its primitive UI components can be reused, but the new screen structure, state boundaries, and interaction flows must be created fresh.

## 2. Product Scope

### In Scope

- Fidelity Modern desktop shell with warm glass surfaces, fixed/sticky header behavior, and bottom-center navigation.
- Library empty state for no active library.
- Library empty state for active library with zero mods.
- Library grid with title-only mod cards, toolbar, local search, sort, filter, selection, and bulk actions.
- Unified Manage Library dialog.
- Configure Tool dialog UI.
- Simplified Settings view with row-based controls.
- Repositories that call the new backend contract directly as it's stubbed, with example-data fallback for what isn't implemented yet or isn't rich enough to check the UI against (§7) — not a prototype permanently isolated from real data.
- A runtime toggle to switch between the redesigned UI and the current UI for side-by-side functional/consistency checking during the transition (§5a).
- Consistent IPC error handling with user-visible toasts.
- i18n key discipline using Lingui.

### Out of Scope

- Active AI features.
- Load order management.
- Mod detail route and rich mod details panel.
- Live console or terminal drawer.
- Conflict/collision visualizer.
- Background process lifecycle management beyond existing launch command requirements — includes tool execution process IDs/live status, deferred per the data/API contract §8.
- New route content for `/library/$id`; mod cards must not navigate to a detail route.
- Direct dependence on the *current* `src/gen/bindings.ts` command functions (the old contract) anywhere in the new UI — repositories may call *newly generated* bindings matching the new contract once the backend stubs them (§7).

### Future Only

- Semantic library search can be considered later. It must not be implemented in this redesign pass.

## 3. Current File Preservation Rule

This is `purpose-of-redesign.md`'s "Frontend: Build Fresh, Migrate Later" execution strategy, restated at implementation granularity. If the two ever disagree, `purpose-of-redesign.md` wins.

Current frontend files are reference material. The redesign implementation must not depend on the old feature structure.

Do not import these old feature files from new redesign code:

- `src/modules/root/app-navigation.tsx`
- `src/modules/root/file-drop-handler.tsx`
- `src/modules/root/library-init.tsx`
- `src/modules/root/settings-init.tsx`
- `src/modules/mod-list/*`
- `src/modules/mod-details/*`
- `src/modules/settings/*`
- `src/components/header-portal.tsx`
- `src/utils/header-portal-context.ts`
- `src/utils/dependency-check.ts`
- `src/components/mod-version.tsx`

Allowed reuse:

- Accessible primitives in `src/components/ui/*`.
- The *current* generated command and DTO bindings in `src/gen/bindings.ts` are reference only — they belong to the old contract. Repositories instead call *newly generated* bindings matching `frontend-redesign-data-api-contract.md` as the backend stubs them, falling back to example data where they don't exist yet (§7).
- Low-level helpers such as `src/utils/result.ts`, `src/utils/error.ts`, `src/utils/i18n.ts`, and `src/utils/theme.ts` when they still fit the new boundaries.
- `src/lib/settings-storage.ts` may be reused or wrapped, but Settings UI must be rebuilt.

Route switch-over rule:

- Build the redesign under `src/redesign/` first.
- Existing route files may become thin adapters only after the new files exist.
- If an existing route file needs to be replaced as an adapter, first preserve its old content in `docs/2026-05-10_redesign/reference/current-frontend/`.
- Do not delete old feature files during the redesign switch-over. Cleanup is a later, explicit task.

## 4. New Source Layout

All new frontend screen and feature code should live under `src/redesign/`.

```text
src/redesign/
  app/
    redesign-root.tsx
    redesign-error-boundary.tsx
    redesign-initializers.tsx
  shell/
    desktop-shell.tsx
    app-background.tsx
    app-header.tsx
    bottom-navigation.tsx
    page-title.tsx
  styles/
    fidelity.css
  data/
    redesign-types.ts
    example-data.ts
    library-repository.ts
    settings-repository.ts
    tool-repository.ts
  i18n/
    common-text.ts
    library-text.ts
    settings-text.ts
    tool-text.ts
    error-text.ts
  state/
    library-state.ts
    settings-state.ts
  shared/
    components/
      fidelity-panel.tsx
      fidelity-button.tsx
      fidelity-icon-button.tsx
      fidelity-input.tsx
      fidelity-section.tsx
      confirm-dialog.tsx
    hooks/
      use-command-error.ts
      use-zip-file-picker.ts
      use-window-drop-zone.ts
    utils/
      app-opener.ts
      mod-display.ts
      file-filters.ts
      sorting.ts
  library/
    library-screen.tsx
    library-content.tsx
    library-title.tsx
    library-empty-card.tsx
    library-activate-empty-state.tsx
    library-drop-empty-state.tsx
    library-execution-bar.tsx
    mod-grid.tsx
    mod-grid-toolbar.tsx
    mod-title-card.tsx
    mod-category-icon.tsx
    bulk-actions-menu.tsx
    manage-library/
      manage-library-dialog.tsx
      library-tabs.tsx
      library-identity-section.tsx
      library-paths-section.tsx
      library-tools-section.tsx
      delete-library-confirm-dialog.tsx
    tools/
      configure-tool-dialog.tsx
  settings/
    settings-screen.tsx
    setting-row.tsx
    theme-mode-control.tsx
    accent-swatches.tsx
    language-select.tsx
    developer-settings-row.tsx
```

Route adapters after switch-over:

```text
src/routes/__root.tsx              -> renders RedesignRoot
src/routes/library.tsx             -> Outlet adapter only
src/routes/library.index.tsx       -> renders LibraryScreen
src/routes/settings.lazy.tsx       -> renders SettingsScreen
src/routes/library.$id.tsx         -> redirects to /library or is removed in cleanup
src/routes/library.$id.lazy.tsx    -> redirects to /library or is removed in cleanup
```

## 5. Shell Specification

### Desktop Shell

`DesktopShell` owns the full app frame below the native Tauri title/window controls. The Tauri window title remains `Modkeeper` and must not change when switching tabs.

Structure:

- Full-viewport app background.
- Sticky/fixed header inside the webview content area.
- Scrollable main content region.
- Centered content container with a maximum width so maximized windows do not stretch dense app content across the full display.
- Bottom-center navigation dock.
- Dialog and toast portals.

Rules:

- The old header portal system must not be used.
- The bottom navigation is always centered and includes Home and Settings.
- The header renders page title and subtitle only; it does not include native window controls.
- Empty library activation state must not render a toolbar.
- Scrollbars must be styled to match Fidelity Modern.
- Main content uses `width: min(100%, var(--mk-content-max))`, horizontal auto margins, and responsive padding.
- Recommended `--mk-content-max`: `1280px` for primary app content. Dialogs and overlays use their own fixed/responsive widths.

### Initializers

`RedesignInitializers` replaces old root initializer components and performs:

- Initial workspace load via `loadLibraryWorkspace` — real backend call with example-data fallback per §7's rules, not example-data hydration unconditionally.
- Window effect application only after a future backend bridge is available; until then the visual shell should work without it.
- Stored settings restore from the settings repository.
- Locale initialization.
- Global drag/drop listener for `.zip` files only.

## 5a. New/Old UI Switch

A developer-only runtime toggle mounts either the redesigned shell or the current app, so both can
be driven side by side and checked for functional parity while the redesign is in progress — this
is what `frontend-redesign-spec.md` §2's "runtime toggle" scope item refers to.

- The toggle is a `useLegacyUi` boolean in the settings repository (`settings-repository.ts`,
  persisted alongside theme/accent/language), surfaced as a row in the Settings developer section
  (§9.6) — not a build-time flag, so switching doesn't require a rebuild.
- The route adapter that would otherwise unconditionally render `RedesignRoot`
  (`src/routes/__root.tsx`, per §4's route adapter table) checks this setting first: if it's on,
  render the current app's existing root component tree instead. Both trees exist in the bundle
  simultaneously during the transition — this is the one deliberate exception to "old feature files
  are reference material only" (§3): they're still mounted, just not by default.
- This is a development/QA aid, not a user-facing feature — it doesn't appear in any acceptance
  criteria about the redesigned UI's own behavior, and it's expected to be removed in the same later
  cleanup pass that removes the old feature files (§3), not shipped indefinitely.

## 6. Design System Specification

`src/redesign/styles/fidelity.css` should define the redesign tokens and utilities. The values in `MODIFICATIONS.md` are authoritative where they differ from `DESIGN.md`.

Required tokens:

```css
:root {
  --mk-primary: #e91e63;
  --mk-on-primary: #ffffff;
  --mk-tertiary: #00828d;
  --mk-surface: #fff8f7;
  --mk-surface-container: rgba(255, 233, 232, 0.72);
  --mk-surface-strong: rgba(255, 248, 247, 0.84);
  --mk-outline: rgba(146, 110, 109, 0.38);
  --mk-text: #281717;
  --mk-text-muted: #5d3f3e;
  --mk-radius-control: 1rem;
  --mk-radius-panel: 2rem;
  --mk-radius-dialog: 2rem;
  --mk-content-max: 1280px;
  --mk-shadow-panel: 0 18px 60px rgba(40, 23, 23, 0.12);
}
```

Required utilities:

- `.mk-glass-standard`: `backdrop-filter: blur(24px) saturate(140%)`.
- `.mk-glass-strong`: `backdrop-filter: blur(40px) saturate(160%)`.
- `.mk-focus-ring`: visible focus ring using `#e91e63`.
- `.mk-scrollbar`: custom scrollbar with subtle warm track and primary hover thumb.

Typography:

- Use `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`.
- Do not scale font size with viewport width.
- Do not use negative letter spacing.
- Use small, compact headings inside toolbars, cards, dialogs, and settings rows.

Color and layout guardrails:

- Primary action color is `#e91e63`.
- Use teal sparingly for category/icon contrast and informational states.
- Avoid a one-note all-pink surface by balancing with warm neutrals and teal accents.
- Use stable dimensions for icon buttons, cards, toolbar controls, switches, and navigation items to prevent layout shifts.

## 7. Data and State Architecture

### Repository Layer

UI components never call `src/gen/bindings.ts` — the *old* generated command surface — directly, and never construct request/response shapes themselves. They call repository functions in `src/redesign/data/*-repository.ts`, typed against `redesign-types.ts` (which matches the contract).

What a repository function does internally is not fixed to "always mock" — it depends on whether the backend has that command yet, per `purpose-of-redesign.md`'s Execution Strategy:

1. **Real first.** If the backend has stubbed the matching command (`docs/2026-07-09_redesign/backend-redesign-spec.md` §7's new, renamed commands — not the old bindings), call it through the *new* generated bindings.
2. **Mock fallback, not mock-only.** If the command doesn't exist yet, or it exists but returns something too thin to check the UI against (an empty mod list when the point of the current work is to check grid layout, long-name truncation, multi-tool dialogs, etc.), fall back to `example-data.ts`.
3. **Mark the fallback, don't bury it.** Every fallback branch carries the same comment tag: `// MOCK-FALLBACK: <why>` (e.g. `// MOCK-FALLBACK: install_mod_archives not implemented yet` or `// MOCK-FALLBACK: real workspace has 0 mods, need examples to check grid layout`). This is the single grep target for finding and deleting every fallback once the real backend covers that case — `rg "MOCK-FALLBACK"` should return fewer results every week and zero once the backend migration (`frontend-redesign-spec.md` §12 step 13) is done. Don't scatter equivalent logic under other names or comments; one tag, everywhere.

This means the redesign is checked against real data continuously as the backend comes online, instead of as two isolated efforts (frontend against mocks, backend against its own tests) joined only at the end.

The expected backend data/API contract — what "real first" calls against — is documented separately in:

- `docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md`

`redesign-types.ts` (field names match the contract exactly — no rename needed at migration time):

```ts
type LibraryId = string
type ModId = string
type ToolId = string

type LibrarySummary = {
  id: LibraryId
  name: string
  gameRoot: string
  isActive: boolean // derived locally in the prototype; the future contract derives this from
                     // workspace.activeLibraryId instead of carrying it on the type — see the
                     // contract §2, item 2. Keep the derivation in one place (library-state.ts)
                     // so swapping it over later is a one-line change, not a prop rename.
  // No modCount: derive from modsByLibraryId[id].length, same reasoning as isActive above —
  // see contract §2 item 10. Don't add a field the prototype doesn't need to remove later.
}

type ModSummary = {
  id: ModId
  libraryId: LibraryId
  name: string
  type: 'client' | 'server' | 'both' | 'unknown'
  isEnabled: boolean
  iconDataUrl?: string
}

type ToolSummary = {
  id: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconDataUrl?: string
  launchArgs?: string
}

type OperationAccepted = {
  accepted: true
}

type WorkspaceEvent =
  | { type: 'cache_rebuild_completed'; libraryId: LibraryId; workspace: LibraryWorkspace }
  | { type: 'mod_install_completed'; libraryId: LibraryId; failures: unknown[]; workspace: LibraryWorkspace }
  | { type: 'bulk_update_completed'; libraryId: LibraryId; failures: unknown[]; workspace: LibraryWorkspace }
```

`example-data.ts`:

- Provides at least one active library with multiple mods.
- Provides one library with zero mods.
- Provides no-active-library scenario data for empty activation testing.
- Provides 2-3 example tools for Manage Library and Configure Tool states.
- Provides long names and paths to test truncation.
- `iconDataUrl` fields (read side) use small inline `data:` URLs (or omit the field) as a convenient stand-in — never a bare file path. The contract treats `iconDataUrl` as opaque/backend-produced (contract §9), so this is a practical prototype choice, not a format guarantee the UI may rely on.

**Maintenance workload for automated UI checks:** low-to-moderate, not open-ended, for two structural
reasons. First, the data model this redesign settled on is intentionally flat — no versions,
dependencies, or rich metadata (that's what `1.7_mod-manifest-removal.md` removed) — so covering
every §9 screen's state variants (empty/no-mods/populated grid, multi-library Manage Library, tool
dialog create/edit, settings rows) is on the order of a handful of fixtures, not a combinatorial
data-modeling exercise. Second, `example-data.ts` is typed against `redesign-types.ts`, which mirrors
the contract — a fixture that drifts from a contract change (a renamed/removed field, exactly the
kind of change this document went through several times while being written) fails to compile rather
than silently going stale, so most of the ongoing cost is "fix the type error," not "remember to
update a fixture nobody's watching." The real ongoing cost is process, not engineering: every new
screen/state added to §9 needs a matching fixture added at the same time, or automated checks built
on this data get a blind spot that a green test suite won't reveal. Treat "new UI state → new
fixture" as part of done for spec changes, not a follow-up task.

`library-repository.ts`:

```ts
loadLibraryWorkspace(): Promise<LibraryWorkspace>
createLibrary(input: { gameRoot: string; libraryRoot?: string; name?: string }): Promise<LibraryWorkspace>
activateLibrary(libraryId: LibraryId): Promise<LibraryWorkspace>
renameLibrary(input: { libraryId: LibraryId; name: string }): Promise<LibraryWorkspace>
deleteLibrary(input: { libraryId: LibraryId; deleteFiles: boolean }): Promise<LibraryWorkspace>

// Fire-and-track, matching the contract's "Non-Blocking Operations" (contract §6) exactly —
// resolve quickly with an acknowledgment, real completion arrives through listenWorkspaceEvent.
rebuildLibraryCache(libraryId: LibraryId): Promise<OperationAccepted>
installZipArchives(input: { libraryId: LibraryId; paths: string[] }): Promise<OperationAccepted>
bulkUpdateMods(input: { libraryId: LibraryId; modIds: ModId[]; action: 'enable' | 'disable' | 'delete' }): Promise<OperationAccepted>
toggleModStatus(input: { libraryId: LibraryId; modId: ModId; enabled: boolean }): Promise<OperationAccepted>

listenWorkspaceEvent(callback: (event: WorkspaceEvent) => void): () => void
```

`toggleModStatus` is a UI-ergonomics convenience only — the contract has no `set_mod_enabled`
(removed, see the contract §2 item 3). Implement it as a one-element call into
`bulkUpdateMods({ libraryId, modIds: [modId], action: enabled ? 'enable' : 'disable' })`
internally, so there is exactly one code path doing the mutation and the future backend swap only
touches this one function body, not every call site.

`listenWorkspaceEvent` mirrors the contract's `listen_workspace_event` shape exactly, even in the
prototype: the three fire-and-track mock functions above resolve `{ accepted: true }` quickly, then,
after a short simulated delay, mutate `example-data` and emit the matching `WorkspaceEvent` through
this same local mechanism (a small in-module event emitter is enough — no real IPC involved). The
reason to bother with this in a prototype that has no real backend: the state layer
(`library-state.ts`, `redesign-initializers.tsx`) subscribes to `listenWorkspaceEvent` once, and that
subscription code is *identical* whether it's driven by this mock emitter or the real Tauri event
listener later — only `listenWorkspaceEvent`'s own implementation changes at migration time, not its
callers. Skipping this and having mocks resolve the final `LibraryWorkspace` directly would mean
every component that starts one of these three operations has to be rewritten when the real backend
arrives, which is exactly the kind of call-site churn the mock-repository approach exists to avoid.

`tool-repository.ts`:

```ts
listTools(libraryId: LibraryId): Promise<ToolSummary[]>
saveTool(input: {
  id?: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconData?: string // base64 raw bytes, matches ToolUpsertInput — NOT iconDataUrl
  launchArgs?: string
}): Promise<ToolSummary[]>
deleteTool(toolId: ToolId): Promise<ToolSummary[]>
executeTool(toolId: ToolId): Promise<void>
```

`saveTool`'s input shape matches the contract's `ToolUpsertInput`, not `ToolSummary` — it
takes `iconData` (raw bytes the mock repository can process however it likes, e.g. store as-is or
wrap in a `data:` URL for display) and returns the resulting `ToolSummary[]` with `iconDataUrl`
populated. Keeping the input/output shapes distinct here, even in the prototype, is what makes the
future backend swap touch only this function's body (§7) instead of the Configure Tool dialog's
call site.

Tool repository methods should simulate success/failure states where useful, but they must not launch real processes in the prototype.

`settings-repository.ts`:

```ts
loadSettings(): Settings
saveSettings(settings: Settings): void | Promise<void>
applyTheme(theme: 'system' | 'light' | 'dark'): Promise<void>
applyAccent(color: string): void
applyLanguage(locale: string): Promise<void>
```

Local storage is a prototype-only stand-in here. The contract resolves settings storage to SQLite
(§8 of the contract) — `loadSettings`/`saveSettings` are the two functions that change internally
when the backend lands; nothing above them should need to change.

Path opening:

- Add `src/redesign/shared/utils/app-opener.ts`.
- Wrap `@tauri-apps/plugin-opener` in business-specific functions such as `openGameRoot(path)`, `openLibraryRoot(path)`, `openModSource(path)`, and `openToolExecutableLocation(path)`.
- UI components must not import the Tauri opener plugin directly.
- In prototype mode, the wrapper can toast/log the path instead of opening it if the Tauri runtime is unavailable.

### State Layer

Use Jotai for durable app state that should survive route/tab changes. Do not make every UI interaction global.

Global state is allowed for:

- Active library and library list.
- Current visible mod collection for the active library.
- Library search, sort, filter, and selected mod IDs, so a user does not lose grid context when switching between Home and Settings.
- Settings values that are shared across screens.

Parent-owned state should be used for:

- Which library tab is selected inside Manage Library.
- Which tool is currently being edited.
- Whether a section inside a specific dialog is expanded.
- In-progress form values owned by one screen or one dialog.

Local component state should be used for:

- Confirmation dialog open/closed state.
- Confirmation checkbox values.
- Text input drafts before save.
- Temporary copied indicators.
- Hover or transient visual state.

The current implementation already follows this direction in places: dialogs such as rename, close, and remove receive `open`, `onOpenChange`, and callbacks from the parent. The redesign should keep that direct parent-to-child control style for confirmations instead of centralizing it in a global UI atom.

`library-state.ts`:

- `libraryWorkspaceAtom`
- `activeLibraryAtom`
- `libraryListAtom`
- `modListAtom`
- `selectedModIdsAtom`
- `librarySearchAtom`
- `librarySortAtom`
- `libraryTypeFilterAtom`
- `visibleModsAtom`

`settings-state.ts`:

- `settingsAtom`
- `themeModeAtom`
- `accentColorAtom`
- `languageAtom`

Avoid adding a global `ui-state.ts` unless a state value demonstrably needs to survive route changes or coordinate independent top-level regions.

### Library Composition

The library screen has several states, but it should not split into unrelated route-level implementations. Use one central combining component with conditional display.

Required composition:

- `LibraryScreen`: obtains parent/global state and passes it down.
- `LibraryContent`: central coordinator for conditional states.
- `LibraryTitle`: handles title/subtitle/profile metadata display.
- `ModGridToolbar`: independently decides whether it should render based on active library and mod count.
- `LibraryExecutionBar`: reserved for non-console execution shortcuts/status that are in scope. It renders nothing when no active library or when no tools/status are available.
- `LibraryActivateEmptyState`: renders only for no active library.
- `LibraryDropEmptyState`: renders only for active library with zero mods.
- `ModGrid`: renders only for active library with one or more mods.

Major parts must receive the parent/global state they need and handle their own render/null decision. This keeps `LibraryScreen` from becoming a monolith while still expressing the library page as one coherent state machine.

### Error Handling

No redesign repository may use silent `.catch(() => {})` on either branch — mock fallback or real call.

Each `MOCK-FALLBACK` branch must:

- Return promises so async behavior, loading affordances, and error states can be exercised.
- Surface simulated failures through a toast.
- Keep dialog state stable after failed saves/deletes.
- Log technical details to console only as secondary context.

Each real backend call branch must:

- Convert backend `Result<T, SError>` with `ur` or the successor unwrap helper.
- Surface failures through a toast, using an i18n string looked up from `SError.code` — never `SError`'s raw form directly. See §10 for the lookup mechanism and its fallback.
- Keep dialog state stable after failed saves/deletes.
- Log `SError.code`/`SError.data` to console only as secondary, best-effort context — this is not the authoritative diagnostic trail. There is no message field to log: the backend logs the English detail to its own log file at the point of return (`backend-redesign-spec.md` §11), which is where a bug report should point, not the browser console.

## 8. Backend Command Mapping

Per §7's repository rule, the redesign calls the "New API name" column directly once the backend stubs it (new generated bindings, not the old `Current binding reference` column, which is shown only to document what behavior is being replaced). "Real once stubbed" below means exactly that — not "eventually, after a separate migration phase." The backend is built independently per `docs/2026-07-09_redesign/backend-redesign-spec.md` to satisfy `frontend-redesign-data-api-contract.md`.

| UI need | New API name | Current binding reference (old contract, reference only) | Status |
| --- | --- | --- | --- |
| Load profiles | `get_library_workspace` | `commands.init()` | Real once stubbed (`MOCK-FALLBACK` until then); API returns the full workspace. |
| Create library | `create_library` | `commands.createLibrary(requirement)` | Real once stubbed (`MOCK-FALLBACK` until then); API persists to SQLite. |
| Activate library | `activate_library` | `commands.openLibrary(repoRoot)` | Real once stubbed (`MOCK-FALLBACK` until then); API activates by library id. |
| Rename library | `rename_library` | `commands.renameLibrary(name, libraryId)` | Real once stubbed (`MOCK-FALLBACK` until then). |
| Rebuild cache | `rebuild_library_cache` | `commands.rebuildLibraryCache(libraryId)` | Real once stubbed (`MOCK-FALLBACK` until then, driven by `listenWorkspaceEvent`), matching the fire-and-track shape from day one. Future API is genuinely async — see contract §6. |
| Delete library entry only | `delete_library(deleteFiles=false)` | None | Future backend requirement. |
| Delete library with files | `delete_library(deleteFiles=true)` | `commands.removeLibrary(repoRoot)` | Future backend requirement with explicit confirmation. |
| Open library path in Explorer | `open_path` through opener utility | `@tauri-apps/plugin-opener` | Frontend utility requirement. |
| Install zip archive | `install_mod_archives` | `commands.addMods(paths, unknownName, backupName)` | Real once stubbed (`MOCK-FALLBACK` until then, driven by `listenWorkspaceEvent`); frontend filters to `.zip`. Future API reports per-archive failures on `mod_install_completed`, not the initial call's return value. |
| Get installed mods | Included in workspace | `LibraryDTO.mods` | Included in `get_library_workspace`'s response — see that row. |
| Toggle mod status | `bulk_update_mods` (one-element `modIds`) | repeated `commands.toggleMod` | Real once stubbed via `toggleModStatus` wrapper, fire-and-track like the other two rows below. No `set_mod_enabled` — removed from the contract. |
| Bulk enable/disable | `bulk_update_mods` | repeated `commands.toggleMod` | Real once stubbed (`MOCK-FALLBACK` until then, driven by `listenWorkspaceEvent`). |
| Bulk delete | `bulk_update_mods` | `commands.removeMods(ids)` | Real once stubbed (`MOCK-FALLBACK` until then), gated by local confirmation, same fire-and-track shape as the row above. |
| Save tool | `upsert_tool` | None | Backend requirement, no current equivalent — `MOCK-FALLBACK` until stubbed. |
| Delete tool | `delete_tool` | None | Backend requirement, no current equivalent — `MOCK-FALLBACK` until stubbed. |
| Execute tool | `execute_tool` | None | Backend requirement, no current equivalent — `MOCK-FALLBACK` until stubbed. |
| Save settings | `save_settings` | local storage | Real once stubbed; local storage is the `MOCK-FALLBACK` until the App Config-backed command exists (contract §8). |
| Get settings | `get_settings` | local storage | Real once stubbed; local storage is the `MOCK-FALLBACK` until the App Config-backed command exists (contract §8). |
| Cache/workspace change notifications | `listen_workspace_event` | None | Real Tauri event listener once the backend emits it; until then the mock `listenWorkspaceEvent` (§7) drives the same fire-and-track UI states with a simulated delay instead of a real async gap. |

If backend DTOs or commands change, run:

```powershell
cargo run --bin export_types
```

## 9. Screen Specifications

### 9.1 Library: Activate Empty State

Rendered when there is no active library.

Component path:

- `src/redesign/library/library-activate-empty-state.tsx`

Behavior:

- Header title: Library.
- Header subtitle: Click to create or activate a library.
- Toolbar is not rendered.
- Center content is `LibraryEmptyCard`.
- Clicking the card opens `ManageLibraryDialog`.
- Clicking the card button also opens `ManageLibraryDialog`.
- Bottom navigation marks Home active.

`LibraryEmptyCard` content:

- 16:9 dashed boundary.
- Cloud upload icon.
- Main text and short description.
- Primary `MANAGE LIBRARIES` action in activation mode.
- Hover: pointer, primary-tinted border, subtle white tint, scale transition.

### 9.2 Library: No Mods Drop State

Rendered when an active library has zero mods.

Component path:

- `src/redesign/library/library-drop-empty-state.tsx`

Behavior:

- Reuses `LibraryEmptyCard`.
- The entire card opens a native file picker filtered to `.zip`.
- Window drag/drop accepts only `.zip` archives.
- No special drop-in card effect is required.
- Dropping files filters out non-zip files before calling the mock archive install repository action.
- Unsupported dropped files produce a toast instead of invoking the backend.
- Bottom-right badge states `.zip`.

### 9.3 Library: Title-Only Grid

Rendered when an active library has one or more mods.

Component paths:

- `src/redesign/library/library-screen.tsx`
- `src/redesign/library/mod-grid-toolbar.tsx`
- `src/redesign/library/mod-grid.tsx`
- `src/redesign/library/mod-title-card.tsx`

Toolbar:

- Select All checkbox.
- Sort by name control.
- Filter control for `Client`, `Server`, `Both`, `Unknown`, and all.
- Bulk `ACTIONS [count]` menu.
- Search input.

Grid:

- 1 column on narrow windows.
- 2 columns on large windows.
- 3 columns on extra-wide windows.
- Cards have stable height, stable icon size, and stable toggle area.

Mod card:

- Left checkbox.
- Category icon container with a restrained gradient or color tint by mod type.
- Truncated title.
- Right enable/disable switch.
- No version, dependency, documentation, author, delete shortcut, explorer shortcut, or details navigation.
- Enabled card gets primary border/tint.
- Disabled card uses reduced opacity.

Interactions:

- Checkbox updates `selectedModIdsAtom`.
- Select All applies to currently visible mods only.
- Search filters locally in real time.
- Toggle calls `toggleModStatus` (which wraps the bulk mock repository action — see §7). This is fire-and-track (contract §6): the affected card shows a pending/disabled state from the moment the call resolves `{ accepted: true }` until `listenWorkspaceEvent` reports `bulk_update_completed`, not a synchronous style change.
- Bulk enable/disable uses the mock bulk update repository action, same pending-then-settle pattern — the toolbar's `ACTIONS` menu and the affected cards are disabled while pending, but this is a UX affordance, not the correctness mechanism (the backend independently guards against overlap, `backend-redesign-spec.md` §8a).
- Bulk delete uses the mock bulk update repository action after local confirmation, same pattern.

### 9.4 Manage Library Dialog

Component path:

- `src/redesign/library/manage-library/manage-library-dialog.tsx`

Purpose:

Consolidate old instance switcher, rename dialog, close dialog, remove dialog, path controls, and future tool controls into one dashboard overlay.

Structure:

- Strong glass dialog body with rounded 2rem corners.
- Top horizontal library tabs.
- Dashed plus tab.
- Identity section.
- Paths section.
- Tools section.
- Footer utilities and activation action.

Behavior:

- Profile tab switches the selected library panel.
- Plus tab opens native folder picker.
- If path is valid and not already registered, create the library and select it.
- Identity save calls the mock rename library repository action.
- Copy path writes to clipboard and shows temporary copied feedback or toast.
- Open Explorer calls the shared `app-opener.ts` utility, which wraps `@tauri-apps/plugin-opener` for app-specific path-opening use cases.
- Tool Settings opens `ConfigureToolDialog`.
- Rebuild Cache calls the mock rebuild library cache repository action — fire-and-track (contract §6): the button shows a busy state and is disabled from `{ accepted: true }` until `listenWorkspaceEvent` reports `cache_rebuild_completed`.
- Delete Library opens `DeleteLibraryConfirmDialog`.
- Activate calls the mock activate library repository action.
- If selected library is already active, primary button text is `Activated` and disabled.

Delete confirmation:

- Requires explicit confirmation.
- Includes checkbox for deleting files versus removing only the app entry.
- In prototype mode, both options mutate example data only.
- In backend mode, entry-only deletion and delete-with-files must call distinct future API behavior (`delete_library` with `deleteFiles: false` vs. `true`). Do not route entry-only deletion to the current destructive remove command.

### 9.5 Configure Tool Dialog

Component path:

- `src/redesign/library/tools/configure-tool-dialog.tsx`

Structure:

- Dark transparent overlay with subtle blur.
- Strong glass dialog body.
- Tool Identity section with name, preview, icon path/URL input, and Browse button to the right of icon input.
- Executable Path section with Browse button.
- Launch Arguments monospace text area.
- Footer with Delete Tool on the left and Cancel/Save on the right.

Behavior:

- Executable Browse filters to `.exe` on Windows.
- Icon Browse accepts any local file format that can be rendered by an HTML `img` element. Do not hard-code the picker to only `.png`, `.jpg`, `.jpeg`, and `.ico`.
- Whatever the user provides (browsed local file or a typed path/URL) is read into raw bytes and
  base64-encoded into `iconData` — the frontend does not construct a `data:` URL and does not send
  a path or URL to the backend. The backend decodes, validates, and processes `iconData` and is
  solely responsible for producing the `iconDataUrl` used to display the tool afterward (contract
  §9). The dialog's own preview, before saving, can render the locally-read bytes directly (e.g. via
  a local object URL) without waiting for a round trip — that's a local preview concern, not the
  `iconDataUrl` field.
- Focus state uses primary pink.
- Save updates example data in prototype mode (via `saveTool`, §7), then calls `upsert_tool` when the future backend exists.
- Delete opens confirmation before updating example data or calling `delete_tool`.
- Tool execution is simulated in prototype mode and should not launch real processes.

### 9.6 Settings View

Component path:

- `src/redesign/settings/settings-screen.tsx`

Structure:

- Single centered column.
- No tabs.
- Each setting is a `SettingRow` with icon box, label, description, and right-side control.

Rows:

- Appearance mode segmented control: System, Light, Dark.
- Accent swatches, defaulting to `#e91e63`.
- Language select.
- Import/export settings row if retained.
- Developer row includes the new/old UI switch (§5a) as a compact utility control, alongside anything else retained from the current developer row.

Behavior:

- Theme changes update `next-themes`, settings storage, and the Tauri window effect only after the future bridge is available.
- Accent changes update CSS variables immediately and persist to settings storage.
- Language changes call `changeLocale`.

## 10. i18n Specification

Use Lingui for all user-visible strings.

Centralization rule:

- User-visible translation functions belong in `src/redesign/i18n/*-text.ts`, one object per namespace (`libraryText`, `settingsText`, `toolText`, `commonText`, `errorText`).
- Components should import a namespace object and call its members instead of defining ad hoc translation functions beside UI code.
- No `t`-prefix convention. The namespace object already says "this is translated text" (`libraryText.`, `commonText.`) — prefixing every member with `t` on top of that is redundant. Name members plainly after what they say: `domain.text`, where `domain` is the namespace object and `text` is a plain descriptor.
- Examples: `libraryText.manageLibraries()`, `libraryText.noModsInstalled()`, `settingsText.appearance()`, `commonText.cancel()`, `commonText.home()`.
- Use `<Trans>` only when interpolation or rich React children make a helper function less clear.

Rules from `outline-of-redesign.md`:

- Each unique user-visible string has exactly one translation key.
- Key convention: `domain.context.descriptor`.
- Search existing translations before creating a new key.
- No inline duplicate strings when meanings are identical.
- Run and review extraction after implementation:

```powershell
bun run extract
```

Recommended key namespaces:

- `library.header.*`
- `library.empty.*`
- `library.toolbar.*`
- `library.card.*`
- `library.actions.*`
- `library.dialog.manage.*`
- `library.dialog.delete.*`
- `tool.dialog.configure.*`
- `settings.row.*`
- `settings.theme.*`
- `settings.language.*`
- `common.action.*`
- `common.status.*`
- `error.*` — see below.

### Error Code → i18n Mapping

Per `frontend-redesign-data-api-contract.md` §9/§11: the backend never sends English error prose,
only `SError.code` (the Rust variant name, e.g. `ModNotFound`) and optional `data`. Turning that
into what the user sees is a frontend responsibility, owned by a new `src/redesign/i18n/error-text.ts`
(added to the source layout in §4, alongside `common-text.ts` etc.):

- One plain-named member per known code on the `errorText` object, keyed under `error.*` — e.g.
  `errorText.modNotFound()`, `errorText.noActiveLibrary()` (the backend's `ModNotFound`/
  `NoActiveLibrary` code, decapitalized — no `t` prefix, same as every other namespace object).
  Interpolate `data` into the message where it's useful (e.g. a file collision's file list),
  following the same `<Trans>`-only-when-needed rule as everything else.
- A single fallback, `errorText.unknown()`, for any `code` without a mapped member yet — new backend
  variants (§9's `InvalidToolIcon`, §8's `StoreError`) must not be able to produce an un-translated
  raw code string in the UI; the fallback is the safety net until a real translation is added.
- `use-command-error.ts` (shared hook, §4) is where this lookup happens: it takes an `SError`,
  decapitalizes `code` and resolves `errorText[decapitalizedCode] ?? errorText.unknown`, and hands
  the resulting translated string to the toast — call sites never do this mapping themselves, and
  never reach for `SError.data` directly to build a message outside this hook.
- This table only ever grows — it is the concrete artifact that makes "the frontend must not show
  raw backend error text" (`backend-redesign-spec.md` §11) actually true in the UI, not just a rule
  stated in a doc.

## 11. Accessibility and Responsiveness

Required:

- Dialogs trap focus and restore focus on close.
- Icon-only buttons use accessible labels and tooltips.
- Focus indicators are visible on glass backgrounds.
- Text has sufficient contrast against transparent and opaque backgrounds.
- Toolbar controls do not wrap into overlapping states.
- Long mod names truncate cleanly.
- Buttons and cards use stable dimensions.
- Bottom nav remains centered at all supported window widths.
- Settings rows stack controls under labels on narrow widths.
- No in-app explanatory copy about how to use the UI beyond necessary labels and empty-state prompts.

## 12. Implementation Sequence

1. Add `src/redesign/styles/fidelity.css` and import it from the redesign root.
2. Add `redesign-types.ts`, `example-data.ts`, and mock repositories.
3. Add global library/settings state atoms and keep dialog/form confirmation state local to parents.
4. Add shared Fidelity components, centralized translation helpers, and `app-opener.ts`.
5. Add shell components and initializers.
6. Add Manage Library and Configure Tool dialogs.
7. Add the library central composition component, empty states, title, toolbar, execution bar, grid, and cards.
8. Add settings screen and setting rows.
9. Add route adapters to mount the redesign.
10. Redirect or retire `/library/$id` route behavior so mod cards do not navigate to details.
11. Run Lingui extraction and review duplicate keys.
12. Run frontend build and smoke checks.
13. When backend work starts, use `frontend-redesign-data-api-contract.md` and `backend-redesign-spec.md` to replace mock repositories with real API calls — field names and call shapes already match, so this should be a repository-internals-only change (§7).

## 13. Verification Plan

Commands:

```powershell
bun run build
bun run extract
```

When future backend command/DTO work is included:

```powershell
cargo run --bin export_types
cargo test
```

Manual checks:

- Launch with no active library and confirm Manage Library opens from the empty card.
- Launch with an active empty library and confirm clicking the card opens a `.zip` file picker.
- Drop a `.zip` onto the app and confirm install path is invoked.
- Drop a non-zip file and confirm the backend is not called.
- Toggle a mod and confirm enabled/disabled card styling updates.
- Select all visible mods, search/filter, and confirm selection count follows visible results.
- Bulk enable, disable, and delete selected mods.
- Create/switch/rename/rebuild/activate libraries from Manage Library.
- Confirm active library button shows `Activated` and is disabled.
- Confirm delete-library checkbox mutates example data only in prototype mode and does not call any destructive command.
- Confirm settings theme/accent/language persist and apply after reload.
- Resize the app through narrow, normal, large, and extra-wide widths.
- Maximize the app and confirm main content remains max-width constrained and centered.
- Verify contrast and focus rings on glass backgrounds.

## 14. Acceptance Criteria

- New screen and feature code lives under `src/redesign/`.
- Existing old feature files remain available for reference.
- Redesign code does not import the old `src/modules/*` feature structure.
- Redesign repositories call the new backend contract directly once stubbed; every remaining example-data fallback is tagged `// MOCK-FALLBACK` and traceable via a single grep. Nothing calls the *current* generated backend bindings (the old contract).
- Repository and type field names already match `frontend-redesign-data-api-contract.md` (`iconDataUrl` read-only on summaries, `iconData` write-only on tool save, bulk-only mod status calls) so backend migration touches repository internals only, not call sites.
- Only durable library/settings state is global; confirmation and form draft state is parent-owned or local.
- Library page is one central composition with independent title, toolbar, execution bar, empty state, and mod list parts.
- Empty activation state and empty drop state share one card component.
- Activation empty state does not render a toolbar.
- No active AI feature is implemented.
- No load order, mod detail, live console, or collision visualizer UI is implemented.
- Library cards are title-only and do not navigate to mod detail routes.
- Manage Library is the single library administration surface.
- Configure Tool dialog includes an icon Browse button next to the icon input.
- Configure Tool icon browsing accepts any file that can render in an HTML `img`.
- Path opening goes through a shared app opener utility wrapping the Tauri opener plugin.
- Main app content is max-width constrained and centered on very wide windows.
- Bottom navigation is centered and stable.
- Header stays visible during scroll.
- `.zip` is the only archive type accepted by the redesigned install surface.
- All new strings use centralized namespaced translation helpers (plain member names, no `t` prefix) where practical and follow the i18n key convention.
- `bun run build` passes after implementation.
