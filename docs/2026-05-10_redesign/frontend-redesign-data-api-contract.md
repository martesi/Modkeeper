# Frontend Redesign Data and API Contract

> **Superseded.** See
> [`docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md`](../2026-07-09_redesign/frontend-redesign-data-api-contract.md),
> which resolves all 10 open items from `frontend-redesign-data-api-contract-mod-1.md` (some of
> which were decided here but never actually applied to this file — e.g. `isActive` below is still
> present despite the recorded decision to remove it). Kept for history; `mod-1` review notes stay
> here since the new document's §2 traces each one to its resolution.

Status: Draft contract for future backend migration

Companion spec:

- `docs/2026-05-10_redesign/frontend-redesign-spec.md`

## 1. Purpose

This document defines the data structures and API shape expected by the redesigned frontend after the prototype moves from example data to real backend data.

The frontend prototype should use mock repositories and `example-data.ts` first. The backend can then refactor independently toward this contract. The contract intentionally does not rely on the current generated `src/gen/bindings.ts` command surface.

## 2. Backend Direction

The future backend should use a SQLite-backed model instead of scattered cache and manifest files as the main source of truth.

SQLite should store:

- Registered libraries.
- Active library selection.
- Installed mods and their enabled state.
- Tool registrations.
- Settings that should survive app restarts.
- Cache/index metadata derived from the game root and library folders.

Files on disk remain authoritative for actual game/mod content. SQLite is the app state and index layer.

## 3. Frontend Data Principles

- Frontend IDs are stable opaque strings.
- APIs should accept IDs instead of paths whenever possible.
- Paths are returned for display and opener actions, but path identity should not be the main UI key.
- API responses should return enough updated workspace state for the UI to re-render without guessing.
- Destructive operations must have explicit action names and flags.
- Entry-only library removal and delete-files removal must be distinct backend behaviors.
- Tool execution should be separate from tool configuration.

## 4. TypeScript Contract

```ts
export type LibraryId = string
export type ModId = string
export type ToolId = string

export type ModType = 'client' | 'server' | 'both' | 'unknown'

export type LibraryWorkspace = {
  activeLibraryId?: LibraryId
  libraries: LibrarySummary[]
  modsByLibraryId: Record<LibraryId, ModSummary[]>
  toolsByLibraryId: Record<LibraryId, ToolSummary[]>
  settings: AppSettings
}

export type LibrarySummary = {
  id: LibraryId
  name: string
  gameRoot: string
  libraryRoot: string
  sptVersion?: string
  isActive: boolean
  modCount: number
  enabledModCount: number
  cacheStatus: CacheStatus
  updatedAt: string
}

export type CacheStatus = {
  state: 'ready' | 'dirty' | 'rebuilding' | 'failed'
  message?: string
  lastRebuiltAt?: string
}

export type ModSummary = {
  id: ModId
  libraryId: LibraryId
  name: string
  type: ModType
  isEnabled: boolean
  iconData?: string
  sourcePath?: string
  installedPath?: string
  updatedAt: string
}

export type ToolSummary = {
  id: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconSrc?: string
  launchArgs?: string
  updatedAt: string
}

export type AppSettings = {
  theme: 'system' | 'light' | 'dark'
  accentColor: string
  language: string
}
```

## 5. API Contract

All future backend calls should return `Result<T, AppError>` or the current project equivalent after backend refactor.

Recommended API names:

```ts
get_library_workspace(): Promise<LibraryWorkspace>

create_library(input: {
  gameRoot: string
  name?: string
}): Promise<LibraryWorkspace>

activate_library(input: {
  libraryId: LibraryId
}): Promise<LibraryWorkspace>

rename_library(input: {
  libraryId: LibraryId
  name: string
}): Promise<LibraryWorkspace>

rebuild_library_cache(input: {
  libraryId: LibraryId
}): Promise<LibraryWorkspace>

delete_library(input: {
  libraryId: LibraryId
  deleteFiles: boolean
}): Promise<LibraryWorkspace>

install_mod_archives(input: {
  libraryId: LibraryId
  archivePaths: string[]
}): Promise<LibraryWorkspace>

set_mod_enabled(input: {
  modId: ModId
  enabled: boolean
}): Promise<LibraryWorkspace>

bulk_update_mods(input: {
  modIds: ModId[]
  action: 'enable' | 'disable' | 'delete'
}): Promise<LibraryWorkspace>

upsert_tool(input: ToolUpsertInput): Promise<LibraryWorkspace>

delete_tool(input: {
  toolId: ToolId
}): Promise<LibraryWorkspace>

execute_tool(input: {
  toolId: ToolId
}): Promise<ToolExecutionResult>

get_settings(): Promise<AppSettings>

save_settings(input: AppSettings): Promise<AppSettings>
```

Supporting types:

```ts
export type ToolUpsertInput = {
  id?: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconSrc?: string
  launchArgs?: string
}

export type ToolExecutionResult = {
  toolId: ToolId
  state: 'started' | 'failed'
  message?: string
}
```

## 6. SQLite Model Sketch

The backend can adjust exact table names, but the frontend expects the domain model above.

Recommended tables:

```sql
libraries (
  id text primary key,
  name text not null,
  game_root text not null,
  library_root text not null,
  spt_version text,
  is_active integer not null default 0,
  cache_state text not null,
  cache_message text,
  last_rebuilt_at text,
  created_at text not null,
  updated_at text not null
)

mods (
  id text primary key,
  library_id text not null references libraries(id) on delete cascade,
  name text not null,
  type text not null,
  is_enabled integer not null default 0,
  icon_data text,
  source_path text,
  installed_path text,
  created_at text not null,
  updated_at text not null
)

tools (
  id text primary key,
  library_id text not null references libraries(id) on delete cascade,
  name text not null,
  executable_path text not null,
  icon_src text,
  launch_args text,
  created_at text not null,
  updated_at text not null
)

settings (
  key text primary key,
  value text not null,
  updated_at text not null
)

cache_entries (
  id text primary key,
  library_id text not null references libraries(id) on delete cascade,
  kind text not null,
  path text not null,
  value text,
  updated_at text not null
)
```

## 7. Migration Notes

Mock repositories should be written against the TypeScript contract in this document.

When the backend is ready:

1. Add backend DTOs matching this contract.
2. Export frontend bindings with `cargo run --bin export_types`.
3. Replace mock repository internals with generated command calls.
4. Keep UI component props unchanged.
5. Keep path opening in `app-opener.ts`, not in screen components.

## 8. Open Decisions

- Whether settings remain in local storage or move fully to SQLite.
- Whether `iconSrc` stores a local file path, app asset URL, or normalized data URL.
- Whether mod source paths are kept for display and opener actions after import.
- Whether tool execution results should later include process IDs or live status.
