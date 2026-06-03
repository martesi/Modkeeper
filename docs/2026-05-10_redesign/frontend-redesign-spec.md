# Frontend Redesign Implementation Spec

Status: Draft ready for implementation

Applies to: Phase 3 frontend redesign in `outline-of-redesign.md`

Inputs:
- `docs/2026-05-10_redesign/ui/PRD.md`
- `docs/2026-05-10_redesign/outline-of-redesign.md`
- `docs/2026-05-10_redesign/ui/MODIFICATIONS.md`
- `docs/2026-05-10_redesign/ui/fidelity_modern/DESIGN.md`
- `docs/2026-05-10_redesign/audits/1.8_frontend-redesign-audit.md`
- `docs/2026-05-10_redesign/frontend-redesign-spec-mod-1.md`

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
- Example-data driven frontend prototype that does not depend on current generated backend bindings.
- Separate future API/data contract for backend migration planning.
- Consistent IPC error handling with user-visible toasts.
- i18n key discipline using Lingui.

### Out of Scope

- Active AI features.
- Load order management.
- Mod detail route and rich mod details panel.
- Live console or terminal drawer.
- Conflict/collision visualizer.
- Background process lifecycle management beyond existing launch command requirements.
- New route content for `/library/$id`; mod cards must not navigate to a detail route.
- Direct dependence on current `src/gen/bindings.ts` command functions in the new UI prototype.

### Future Only

- Semantic library search can be considered later. It must not be implemented in this redesign pass.

## 3. Current File Preservation Rule

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
- Existing generated command and DTO bindings in `src/gen/bindings.ts` as reference only. The first redesign implementation should use example data.
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

- Example data hydration for the frontend prototype.
- Window effect application only after a future backend bridge is available; until then the visual shell should work without it.
- Stored settings restore from the settings repository.
- Locale initialization.
- Global drag/drop listener for `.zip` files only.

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

The first redesign implementation is a frontend prototype. UI components must not call current generated backend commands directly. They call mock repository functions backed by `example-data.ts`, using the same shape the future backend should satisfy.

The expected future backend data/API contract is documented separately in:

- `docs/2026-05-10_redesign/frontend-redesign-data-api-contract.md`

This separation lets the UI be built immediately while the backend is refactored toward its own new architecture and a planned SQLite-backed data model.

`redesign-types.ts`:

```ts
type LibraryId = string
type ModId = string
type ToolId = string

type LibrarySummary = {
  id: LibraryId
  name: string
  gameRoot: string
  isActive: boolean
  modCount: number
}

type ModSummary = {
  id: ModId
  libraryId: LibraryId
  name: string
  type: 'client' | 'server' | 'both' | 'unknown'
  isEnabled: boolean
  iconData?: string
}

type ToolSummary = {
  id: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconSrc?: string
  launchArgs?: string
}
```

`example-data.ts`:

- Provides at least one active library with multiple mods.
- Provides one library with zero mods.
- Provides no-active-library scenario data for empty activation testing.
- Provides 2-3 example tools for Manage Library and Configure Tool states.
- Provides long names and paths to test truncation.

`library-repository.ts`:

```ts
loadLibraryWorkspace(): Promise<LibraryWorkspace>
createExampleLibrary(input: { gameRoot: string; name?: string }): Promise<LibraryWorkspace>
activateExampleLibrary(libraryId: LibraryId): Promise<LibraryWorkspace>
renameExampleLibrary(input: { libraryId: LibraryId; name: string }): Promise<LibraryWorkspace>
rebuildExampleLibraryCache(libraryId: LibraryId): Promise<LibraryWorkspace>
deleteExampleLibrary(input: { libraryId: LibraryId; deleteFiles: boolean }): Promise<LibraryWorkspace>
installExampleZipArchives(input: { libraryId: LibraryId; paths: string[] }): Promise<LibraryWorkspace>
toggleExampleModStatus(input: { modId: ModId; enabled: boolean }): Promise<LibraryWorkspace>
bulkUpdateExampleMods(input: { modIds: ModId[]; action: 'enable' | 'disable' | 'delete' }): Promise<LibraryWorkspace>
```

`tool-repository.ts`:

```ts
listExampleTools(libraryId: LibraryId): Promise<ToolSummary[]>
saveExampleTool(tool: ToolSummary): Promise<ToolSummary[]>
deleteExampleTool(toolId: ToolId): Promise<ToolSummary[]>
executeExampleTool(toolId: ToolId): Promise<void>
```

Tool repository methods should simulate success/failure states where useful, but they must not launch real processes in the prototype.

`settings-repository.ts`:

```ts
loadSettings(): Settings
saveSettings(settings: Settings): void | Promise<void>
applyTheme(theme: 'system' | 'light' | 'dark'): Promise<void>
applyAccent(color: string): void
applyLanguage(locale: string): Promise<void>
```

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

No redesign repository or future API call may use silent `.catch(() => {})`.

Each prototype repository path must:

- Return promises so async behavior, loading affordances, and error states can be exercised.
- Surface simulated failures through a toast.
- Keep dialog state stable after failed saves/deletes.
- Log technical details to console only as secondary context.

Each future backend call path must:

- Convert backend `Result<T, SError>` with `ur` or the successor unwrap helper.
- Surface failures through a toast.
- Keep dialog state stable after failed saves/deletes.
- Log technical details to console only as secondary context.

## 8. Backend Command Mapping

The redesign prototype does not call these bindings. This table exists only to document how current behavior relates to future backend expectations. The backend will be refactored independently and should satisfy the separate API/data contract when the mock repositories are replaced.

| UI need | Future API name | Current binding reference | Prototype/future status |
| --- | --- | --- | --- |
| Load profiles | `get_library_workspace` | `commands.init()` | Mocked first; future API should return full workspace. |
| Create library | `create_library` | `commands.createLibrary(requirement)` | Mocked first; future API should persist to SQLite-backed model. |
| Activate library | `activate_library` | `commands.openLibrary(repoRoot)` | Mocked first; future API should activate by library id. |
| Rename library | `rename_library` | `commands.renameLibrary(name, libraryId)` | Mocked first. |
| Rebuild cache | `rebuild_library_cache` | `commands.rebuildLibraryCache(libraryId)` | Mocked first. |
| Delete library entry only | `delete_library(deleteFiles=false)` | None | Future backend requirement. |
| Delete library with files | `delete_library(deleteFiles=true)` | `commands.removeLibrary(repoRoot)` | Future backend requirement with explicit confirmation. |
| Open library path in Explorer | `open_path` through opener utility | `@tauri-apps/plugin-opener` | Frontend utility requirement. |
| Install zip archive | `install_mod_archives` | `commands.addMods(paths, unknownName, backupName)` | Mocked first; frontend filters to `.zip`. |
| Get installed mods | Included in workspace | `LibraryDTO.mods` | Mocked first as derived state. |
| Toggle mod status | `set_mod_enabled` | `commands.toggleMod(id, isActive)` | Mocked first. |
| Bulk enable/disable | `bulk_update_mods` | repeated `commands.toggleMod` | Mocked first. |
| Bulk delete | `bulk_update_mods` | `commands.removeMods(ids)` | Mocked first. |
| Save tool | `upsert_tool` | None | Future backend requirement. |
| Delete tool | `delete_tool` | None | Future backend requirement. |
| Execute tool | `execute_tool` | None | Future backend requirement. |
| Save settings | `save_settings` | local storage | Mocked/local first; backend command optional. |
| Get settings | `get_settings` | local storage | Mocked/local first; backend command optional. |

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
- Toggle calls the mock mod enabled repository action.
- Bulk enable/disable uses the mock bulk update repository action.
- Bulk delete uses the mock bulk update repository action after local confirmation.

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
- Rebuild Cache calls the mock rebuild library cache repository action.
- Delete Library opens `DeleteLibraryConfirmDialog`.
- Activate calls the mock activate library repository action.
- If selected library is already active, primary button text is `Activated` and disabled.

Delete confirmation:

- Requires explicit confirmation.
- Includes checkbox for deleting files versus removing only the app entry.
- In prototype mode, both options mutate example data only.
- In backend mode, entry-only deletion and delete-with-files must call distinct future API behavior. Do not route entry-only deletion to the current destructive remove command.

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
- Focus state uses primary pink.
- Save updates example data in prototype mode, then calls `upsert_tool` when the future backend exists.
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
- Developer row may be retained as a compact utility row if still required.

Behavior:

- Theme changes update `next-themes`, settings storage, and the Tauri window effect only after the future bridge is available.
- Accent changes update CSS variables immediately and persist to settings storage.
- Language changes call `changeLocale`.

## 10. i18n Specification

Use Lingui for all user-visible strings.

Centralization rule:

- User-visible translation functions belong in `src/redesign/i18n/*-text.ts`.
- Components should import text helpers from those files instead of defining ad hoc translation functions beside UI code.
- Translation helper names must use the `tText` suffix/pattern and be grouped by namespace.
- Examples: `libraryText.tTextManageLibraries()`, `libraryText.tTextNoModsInstalled()`, `settingsText.tTextAppearance()`, `commonText.tTextCancel()`.
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
13. When backend work starts, use `frontend-redesign-data-api-contract.md` to replace mock repositories with real API calls.

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
- Redesign prototype uses example data and mock repositories instead of current generated backend bindings.
- Future API/data expectations live in a separate contract document.
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
- All new strings use centralized namespaced `tText` translation helpers where practical and follow the i18n key convention.
- `bun run build` passes after implementation.
