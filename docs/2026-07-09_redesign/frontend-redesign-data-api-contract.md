# Frontend Redesign Data and API Contract

Status: Implementation-ready contract for future backend migration.

Supersedes `docs/2026-05-10_redesign/frontend-redesign-data-api-contract.md`. That draft had 10 open
review comments recorded in `docs/2026-05-10_redesign/frontend-redesign-data-api-contract-mod-1.md`;
this revision resolves all 10 (table below) instead of leaving them as recorded-but-unapplied decisions.

Companion docs:

- `docs/2026-07-09_redesign/frontend-redesign-spec.md`
- `docs/2026-07-09_redesign/backend-redesign-spec.md`

## 1. Purpose

This document defines the data structures and API shape expected by the redesigned frontend after the prototype moves from example data to real backend data.

The frontend prototype uses mock repositories and `example-data.ts` first. The backend refactors independently toward this contract. The contract intentionally does not rely on the current generated `src/gen/bindings.ts` command surface.

## 2. Resolved Review Items

| # | Item | Resolution |
|---|------|------------|
| 1 | `iconData` vs `iconSrc` naming | Split by direction, not unified: reads use `iconDataUrl` (opaque, ready-to-render `<img>` src, backend-produced) on `ToolSummary`; writes use `iconData` (base64 raw bytes) on `ToolUpsertInput`, processed by the backend. See §9 Format Notes. Mods no longer carry an icon field at all — its only source dies with the manifest removal, and the redesigned mod card uses a `ModType`-tinted category icon (`design-review.md` C6). |
| 2 | `isActive` redundant with `activeLibraryId` | Removed from `LibrarySummary`. UI derives it: `library.id === workspace.activeLibraryId`. |
| 3 | `bulk_update_mods` bundles enable/disable/delete | Kept as a single call per product decision (see §5 rationale note), `set_mod_enabled` removed. |
| 4 | `rebuild_library_cache` async gap | Resolved more broadly than originally scoped: `rebuild_library_cache`, `install_mod_archives`, `bulk_update_mods`, and `sync_mods` (kept per `design-review.md` C3/M5) all became fire-and-track (`OperationAccepted` + a matching `listen_workspace_event` completion event, correlated by a client-minted `taskId`) — see §6 "Non-Blocking Operations". |
| 5 | `create_library` missing `libraryRoot` | Added optional `libraryRoot` input; backend derives it from `gameRoot` when omitted, matching the existing `derive_library_root` function in `core/library_service.rs`. |
| 6 | `AppError` undefined | Reversed rather than resolved as originally planned: there is no `AppError`. The backend's `SError` (unchanged) is the wire type directly — defined in §5, with its stable shape explained in §9. |
| 7 | `install_mod_archives` no partial-failure model | Moved further than originally planned: `install_mod_archives` is now fire-and-track (§6), and the `failures` array is carried on the `mod_install_completed` event rather than the call's own return value. |
| 8 | Open decisions blocking mock repositories | All four resolved — see §8. |
| 9 | Timestamp format unspecified | ISO 8601 UTC, noted once in §3 instead of per-field. |
| 10 | `modCount`/`enabledModCount` derivation unstated | Resolved further than originally planned: removed from `LibrarySummary` entirely rather than kept as backend-derived fields — trivially computable from `modsByLibraryId[id]`, no DTO field needed (§3). |

## 3. Backend Direction

Two storage tiers, neither of them SQLite — the SQLite Library DB from earlier revisions is cancelled (`design-review.md` C4). The split is invisible to this contract (the frontend still calls `get_library_workspace()` and gets one `LibraryWorkspace` back), but it matters for where things live on disk and how migration works, so it's documented here rather than only in `backend-redesign-spec.md`.

**App Config** (one plain config file per install, lives in the app's own data directory, not inside any library — the `toml` crate directly, decided in `backend-redesign-spec.md`):

- Registered libraries — just enough to locate each one (stable ID + its `libraryRoot` path), not their content. The ID is a cached copy of the id in that library's own `manifest.toml`, never the origin (`design-review.md` C7).
- Active library selection.
- Settings that should survive app restarts (`AppSettings` — see §8, resolved), including ones needed before the frontend loads (e.g. theme, for window vibrancy at startup) — this is why settings live here and not in frontend `localStorage`.

**Library files** (plain TOML, per library, living inside that library's own mod-manager directory under its game root — i.e. traveling with the library, not with the app installation):

- `manifest.toml`: that library's own identity fields (`id`, `name`, `gameRoot`, `sptVersion`, etc.) and installed mods with their enabled state — the existing file, unchanged as the format.
- `cache.toml`: cache/index metadata derived from that library's game root and mod folder — the existing file.
- `tools.toml` (new): tool registrations for that library (`design-review.md` C4/M4).

Files on disk remain authoritative for actual game/mod content. Both tiers' shapes are sketched in §7.

All timestamp fields (`updatedAt`, `lastRebuiltAt`, `createdAt`) are ISO 8601 UTC strings, e.g. `"2026-07-09T12:00:00Z"`.

There is no `modCount`/`enabledModCount` field on `LibrarySummary` — a count that's trivially derivable from data already in the same response (`modsByLibraryId[id].length` and its filtered-enabled count) doesn't get a redundant DTO field; the frontend computes it. Same reasoning as dropping `isActive` (§2 item 2): one source of truth, not a mirrored value that can drift out of sync.

## 4. Frontend Data Principles

- Every field in this contract is camelCase, with no exceptions — this matches what `tauri-specta` actually generates from `#[serde(rename_all = "camelCase")]` Rust DTOs (already the convention in, e.g., `models/global.rs::GlobalConfig`), so the generated bindings need no field-name translation layer against this document.
- Frontend IDs are stable opaque strings.
- APIs accept IDs instead of paths whenever possible.
- Paths are returned for display and opener actions, but path identity is not the main UI key.
- API responses return enough updated workspace state for the UI to re-render without guessing.
- Every type in §5 is either a response shape (backend-populated, e.g. `LibrarySummary`, `ModSummary`, `ToolSummary` — the frontend never constructs these, only reads them) or a request input shape (frontend-supplied, e.g. `ToolUpsertInput`) — never both. Where a field is populated one direction and not the other (the `iconDataUrl`/`iconData` split is the clearest example), that's stated explicitly rather than left to be inferred from context.
- Destructive operations have explicit action names and flags, enforced at the UI-confirmation layer where the wire-level call is unified (see §5 rationale note on `bulk_update_mods`).
- Entry-only library removal and delete-files removal are distinct backend behaviors.
- Tool execution is separate from tool configuration.

## 5. TypeScript Contract

```ts
export type LibraryId = string
export type ModId = string
export type ToolId = string

export type ModType = 'client' | 'server' | 'both' | 'unknown'
// Still used, checked against the actual redesign scope, not carried over by default: the mod-grid
// toolbar's filter control and each mod card's category icon tint both key off this
// (frontend-redesign-spec.md §9.3). Backend-side it's still real logic, not vestigial manifest
// metadata — `core/mod_fs.rs::infer_mod_type` derives it from where a mod's files land
// (BepInEx/plugins vs. SPT/user/mods), independent of the manifest system §1.7 removes.

// This is what the backend's SError enum (models/error.rs) actually serializes to once it has
// #[serde(tag = "code", content = "data")] — not a separate boundary type. See §9. There is no
// message field: SError's Display text is logged backend-side (English, for developers) and never
// crosses the IPC boundary — see §9 for what the frontend does instead.
export type SError = {
  code: string // the Rust variant name verbatim, e.g. 'ModNotFound', 'NoActiveLibrary'
  data?: unknown // present for variants with a payload; shape depends on the variant
}

export type LibraryWorkspace = {
  activeLibraryId?: LibraryId
  libraries: (LibrarySummary | LibraryStub)[]
  modsByLibraryId: Record<LibraryId, ModSummary[]>
  toolsByLibraryId: Record<LibraryId, ToolSummary[]>
  settings: AppSettings
}

// A registered library whose files can't currently be read (missing folder, corrupt manifest,
// unplugged drive) collapses to this stub — only its registered path, no other fields, rather
// than a full summary with a status enum bolted on (design-review.md C13). The UI renders it as
// a bare path whose only action is remove. Recovery is manual and coarse on purpose: no in-app
// retry/repair — restart the app after fixing the underlying problem, or remove and re-add.
export type LibraryStub = {
  path: string
}

export type LibrarySummary = {
  id: LibraryId
  name: string
  gameRoot: string
  libraryRoot: string
  sptVersion?: string
  cacheStatus: CacheStatus
  deployStale: boolean // deployed symlinks no longer match recorded mod state (the backend's
                       // dirty flag). Deliberately NOT part of cacheStatus — the cache/rebuild
                       // machinery is a different concern (design-review.md C3/M5). The Sync
                       // button highlights when this is true.
  updatedAt: string
  // No modCount/enabledModCount: derive from modsByLibraryId[id] (§3).
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
  // No iconDataUrl: mods have no per-mod image in the redesign — the card shows a category icon
  // tinted by `type` (design-review.md C6). Its only producer (manifest.icon) dies with the
  // manifest removal; mods that show a custom icon today intentionally lose it.
  sourcePath?: string
  installedPath?: string
  updatedAt: string
}

export type ToolSummary = {
  id: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconDataUrl?: string // read-only, opaque, backend-produced — see §9. Never sent by the frontend.
  launchArgs?: string
  updatedAt: string
}

export type AppSettings = {
  theme: 'system' | 'light' | 'dark'
  accentColor: string
  language: string
}
```

## 6. API Contract

All backend calls return `Result<T, SError>` (Tauri command errors resolve to a rejected promise carrying `SError` — the existing backend error type, unchanged, not a new boundary type; see §9).

```ts
get_library_workspace(): Promise<LibraryWorkspace>

create_library(input: {
  gameRoot: string
  libraryRoot?: string // if omitted, backend derives it from gameRoot (derive_library_root)
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
  taskId: string // client-minted, see "Non-Blocking Operations" below
  libraryId: LibraryId
}): Promise<OperationAccepted>
// Fire-and-track — see "Non-Blocking Operations" below. Resolving this promise means the request
// was accepted, NOT that the rebuild finished. Completion arrives via listen_workspace_event's
// 'cache_rebuild_completed'.

delete_library(input: {
  libraryId: LibraryId
  deleteFiles: boolean
}): Promise<LibraryWorkspace>

install_mod_archives(input: {
  taskId: string
  libraryId: LibraryId
  archivePaths: string[]
}): Promise<OperationAccepted>
// Fire-and-track. Completion (including any per-archive failures) arrives via
// listen_workspace_event's 'mod_install_completed'.

bulk_update_mods(input: {
  taskId: string
  libraryId: LibraryId
  modIds: ModId[]
  action: 'enable' | 'disable' | 'delete'
}): Promise<OperationAccepted>
// Fire-and-track. Completion (including any per-mod failures) arrives via
// listen_workspace_event's 'bulk_update_completed'. Single-mod toggle is bulk_update_mods with a
// one-element modIds array. set_mod_enabled does not exist — see rationale note below.
// libraryId is required and must be the ACTIVE library: the backend validates
// libraryId === activeLibraryId and rejects otherwise, and completion events report against it.
// It is an assertion input, not a key into a per-library guard — that guard no longer exists
// (design-review.md C1/M2). This never deploys: enable/disable/delete commit mod state only;
// deployment is the explicit sync_mods step (design-review.md C3).

sync_mods(input: {
  taskId: string
  libraryId: LibraryId
}): Promise<OperationAccepted>
// Fire-and-track (the fourth operation) — the explicit, user-triggered deploy step
// (design-review.md C3/M5). Completion arrives via listen_workspace_event's 'sync_completed'.
// Walks and relinks the whole tree, same cost profile as rebuild_library_cache, which is why it
// qualifies under the "touches potentially many files" criterion alongside the other three. The
// Sync button (execution bar) calls this, highlighted while LibrarySummary.deployStale is true.

upsert_tool(input: ToolUpsertInput): Promise<LibraryWorkspace>

delete_tool(input: {
  toolId: ToolId
}): Promise<LibraryWorkspace>

execute_tool(input: {
  toolId: ToolId
}): Promise<ToolExecutionResult>

get_settings(): Promise<AppSettings>

save_settings(input: AppSettings): Promise<AppSettings>

listen_workspace_event(
  callback: (event: WorkspaceEvent) => void
): () => void // returns an unsubscribe function
```

Supporting types:

```ts
export type ToolUpsertInput = {
  id?: ToolId
  libraryId: LibraryId
  name: string
  executablePath: string
  iconData?: string // raw image bytes, base64-encoded (or similar) — NOT a data: URL, see §9.
                     // The backend decodes, validates, and processes this; the resulting
                     // ToolSummary.iconDataUrl is computed by the backend, not echoed from this input.
  launchArgs?: string
}

export type ToolExecutionResult = {
  toolId: ToolId
  state: 'started' | 'failed'
  message?: string
  // No process ID or live status field: out of scope for this round, see §8.
}

export type OperationAccepted = {
  accepted: true
  // No other fields: this is a lightweight acknowledgment, not a status snapshot. Don't grow this
  // into a second, competing source of workspace state — completion events carry the real update.
}

export type WorkspaceEvent =
  | {
      type: 'cache_rebuild_completed'
      taskId: string
      libraryId: LibraryId
      cacheStatus: CacheStatus
      workspace: LibraryWorkspace
    }
  | {
      type: 'mod_install_completed'
      taskId: string
      libraryId: LibraryId
      failures: { archivePath: string; error: SError }[]
      workspace: LibraryWorkspace
    }
  | {
      type: 'bulk_update_completed'
      taskId: string
      libraryId: LibraryId
      action: 'enable' | 'disable' | 'delete'
      failures: { modId: ModId; error: SError }[]
      workspace: LibraryWorkspace
    }
  | {
      type: 'sync_completed'
      taskId: string
      libraryId: LibraryId
      failures: { modId: ModId; error: SError }[] // e.g. FileCollision surfaces here, at deploy time
      workspace: LibraryWorkspace
    }
```

Each completion event carries the fresh `workspace: LibraryWorkspace` directly, not just a status
flag — the frontend re-renders from the event itself and never needs a second round trip. Events
are matched back to the submitting call by `taskId` — client-minted, registered in the frontend's
single persistent event bus *before* the command invoke goes out, which is what makes the match
race-free (`design-review.md` C15/M6). There's no generic `workspace_changed` catch-all: only
these four operations are fire-and-track (see "Non-Blocking Operations" below), so there's no case
that needs an event without a specific shape yet.

**Rationale note on `bulk_update_mods`:** `purpose-of-redesign.md` requires destructive operations to have explicit action names and flags. A single call bundling `'enable' | 'disable' | 'delete'` looks like it violates that at the wire level. This was a deliberate product decision (recorded in the superseded `frontend-redesign-spec-mod-1.md` review) to avoid a redundant `set_mod_enabled` alongside a one-element bulk call. The principle is still satisfied at the call-site layer: `frontend-redesign-spec.md` requires local confirmation before any `action: 'delete'` bulk call, so the explicit-flag requirement is enforced in the UI rather than the wire shape. If this becomes a real footgun (e.g. a future caller passes `'delete'` without going through the confirmation dialog), split the call — the two-call shape from the original review is the fallback.

### Non-Blocking Operations

The general idea of operation in this contract is non-blocking — but that's scoped, not universal.
Exactly four operations are fire-and-track: `install_mod_archives` ("add mod"), `bulk_update_mods`
("remove mod" and enable/disable), `rebuild_library_cache`, and `sync_mods` (deploy). These four
touch potentially many files or scan a whole tree, which is why they matter (`bulk_update_mods`'s
enable/disable is metadata-only after `design-review.md` C3, but the command stays uniformly
fire-and-track across all three actions for one predictable client-side handling path — `delete`
still removes files); everything else in this contract (`create_library`, `activate_library`,
`rename_library`, `delete_library`, `upsert_tool`, `delete_tool`, settings) is a small, fast
operation and stays a direct blocking `Promise<LibraryWorkspace>` call — don't apply the
fire-and-track pattern to those without a reason.

One honesty note on "non-blocking" (`design-review.md` C1/M2): the backend keeps its single
active-library lock, so a *read* (`get_library_workspace`) issued while a heavy operation is in
flight blocks until that operation finishes. "Non-blocking" here means the UI isn't awaiting the
operation's own promise — not that concurrent reads are free. Toggles hold the lock only for a
negligible metadata write.

Rules for the four fire-and-track operations:

- Each call carries a **client-minted `taskId`** (uuid, generated by the frontend). Register the
  completion handler in the event bus *before* invoking; the matching completion event carries the
  same `taskId`. Reusing a `taskId` while its task is still in flight is rejected with
  `TaskIdInUse` — a client bug surfaced, not papered over.
- These operations target the **active library only**: the backend validates
  `libraryId === activeLibraryId` and rejects otherwise as a synchronous validation failure.
- The promise resolving with `OperationAccepted` means the request was validated and accepted, not
  that the operation finished. A rejected promise (`SError`) at this point means the request itself
  was invalid (non-active `libraryId`, empty `archivePaths`, `TaskIdInUse`, etc.) — a synchronous,
  immediate failure, not a background one.
- The actual outcome — success, partial failure, or full failure — arrives later via
  `listen_workspace_event`, in the matching event shape above.
- The UI should prevent the user from starting a redundant overlapping action (e.g. disable the
  Rebuild Cache button while one is pending) as a UX affordance, but overlap is not an error:
  overlapping mutations on the active library serialize by blocking on the backend's lock
  (unordered — acceptable because enable/disable are absolute sets, so a race's outcome is always
  one of the requested states). There is no reject-on-overlap guard
  (`backend-redesign-spec.md` §8a).
- If the frontend reloads mid-operation, its `taskId` registrations are gone and the completion
  event is dropped — accepted: the view self-corrects on the next workspace read
  (`design-review.md` C15).

## 7. Storage Sketch

The backend can adjust exact field/key names, but the frontend expects the domain model above. The
`tools.toml` sketch below supersedes the `Tool` struct sketch in
`docs/2026-05-10_redesign/audits/1.9_backend-redesign-audit.md`, which used `icon`/`arguments`
field names — align to `icon_data_url`/`launch_args` here instead so the storage layer and the TS
contract don't drift. (Earlier revisions of this section sketched a per-library SQLite Library DB;
that design is cancelled, `design-review.md` C4 — library state stays in that library's own plain
TOML files.)

Two separate storage locations, per §3. There is no cross-reference between them beyond the App
Config's `library_root` pointer (whose `id` is a cached copy of the manifest's own `id`, C7) — a
library's files don't know about other libraries, and the App Config doesn't store any library's
mods/tools. The backend assembles one `LibraryWorkspace` by reading the App Config's registry,
then performing a read-only read of each known library's own files to fill in
`modsByLibraryId`/`toolsByLibraryId` and derive that library's `LibrarySummary` fields
(`cacheStatus`, `deployStale` — note there is no `modCount`/`enabledModCount` to derive here
either; per §4, the frontend computes those from `modsByLibraryId[id]` itself). A library whose
files can't be read surfaces as the path-only `LibraryStub` (§5), never dropped. Full rationale
and file locations are in `backend-redesign-spec.md` §8.

### App Config — one plain config file per install

```toml
# known libraries: identity only (name, game_root, ... live in the library's own DB, not here) —
# opening a library's DB is how you find out anything about it beyond "it exists and where it is".
[[known_libraries]]
id = "..."
library_root = "..."   # where to find this library's own DB; see Library DB below
last_opened_at = "..."

[app_state]
active_library_id = "..."

[settings]
theme = "system"
accent_color = "#e91e63"
language = "en"
```

Shown as TOML for readability; the exact Rust struct shape and the `toml`-crate decision are in `backend-redesign-spec.md` §8.

### Library files — per library, inside that library's own directory

`manifest.toml` (existing file — identity, mods, enabled state, dirty flag; the source of
`LibrarySummary` and `ModSummary` fields; no `icon_data_url` on mods, per §5/C6) and `cache.toml`
(existing file — derived index/cache data; the source of `cacheStatus`) keep their current shapes,
evolving only as `1.7`'s manifest-field removals dictate — this contract doesn't re-sketch them.

`tools.toml` (new sibling file):

```toml
[[tools]]
id = "..."
name = "..."
executable_path = "..."
icon_data_url = "..."  # stores the backend-processed, ready-to-render src (§9); upsert_tool's
                       # raw iconData input is decoded/validated/processed before it lands here,
                       # never stored as-received.
launch_args = "..."
created_at = "..."
updated_at = "..."
```

## 8. Resolved Decisions (formerly "Open Decisions")

- **Settings storage:** Moves fully out of frontend `localStorage` and into the App Config file (§7) — a plain config file, not SQLite, since settings are app-level and small — per `purpose-of-redesign.md`'s "settings that should survive app restarts" scope and its note that some settings (theme) are needed before the frontend loads. Local storage (`settings-repository.ts` in the frontend prototype) is a prototype-only stand-in until the backend exists, not a permanent alternative.
- **Old config migration:** Only the App Config migrates — its `known_libraries` migrates from the current `confy`-based `GlobalConfig` (`library_last`/`library_recent` paths) on first run of the new version, adopting each library's id from its own `manifest.toml` (minting only where no readable manifest exists — `design-review.md` C7). There is no per-library data migration: `manifest.toml`/`cache.toml` remain the live format (C4). See `backend-redesign-spec.md` §8 for the mechanics.
- **Icon fields are split by direction, and processing is backend-only:** `iconDataUrl` (`ToolSummary` — tools only; mods have no icon field, C6) is read-only and opaque — a ready-to-use `<img>` src that the frontend passes straight through without interpreting it. It is not guaranteed to be a `data:` URL specifically; the backend owns the representation (could be a `data:` URL, could be a Tauri asset-protocol URL — an implementation detail of `backend-redesign-spec.md` §9, not a frontend concern). `iconData` (`ToolUpsertInput`, write-only) is base64-encoded raw image bytes — whatever the frontend read off disk when the user browsed an icon file, unprocessed. The backend decodes, validates, and processes `iconData` (format/size normalization) and computes the `iconDataUrl` that later reads return; the frontend never constructs a `data:` URL itself and never echoes `iconData` back as `iconDataUrl`.
- **Mod source paths after import:** Kept. `sourcePath` on `ModSummary` remains for display and opener actions after import; it is not cleared once the mod is installed.
- **Tool execution result detail (process ID / live status):** Explicitly deferred, not open. `frontend-redesign-spec.md` scopes out live console/process lifecycle management for this round; `ToolExecutionResult` stays `started | failed` only. Revisit if/when a console or process-monitoring feature is scoped.

## 9. Format Notes

- All timestamps: ISO 8601 UTC strings (§3).
- Icon fields (tools only): `iconDataUrl` (read, opaque ready-to-render src, backend-produced) vs. `iconData` (write, base64 raw bytes, backend-processed) — never conflate the two or send a bare file path through either (§8). Mods carry no icon field (C6).
- `SError.code` is the backend's `SError` enum variant name verbatim (e.g. `ModNotFound`, `NoActiveLibrary`) — not a hand-assigned string, so there's no separate mapping table to drift out of sync with the enum. Full explanation in `backend-redesign-spec.md` §11.
- `SError.data` is a positional **array** for multi-field tuple variants (e.g. `InvalidLibrary(String, String)` → `data: [path, reason]`) — interpolation code must be written against that shape, not assume an object.
- There is no message field. The frontend never receives English error prose — it maps `code` to an already-translated string (`frontend-redesign-spec.md` §10), with a generic fallback for any `code` without a mapping yet. The backend logs `SError`'s `Display` text (English) at the point of return, for developers, regardless of what's returned to the frontend (`backend-redesign-spec.md` §11).

## 10. Migration Notes

Mock repositories are written against the TypeScript contract in this document.

When the backend is ready:

1. Add backend DTOs matching this contract, including `SError`'s `#[serde(tag = "code", content = "data")]` attribute (`backend-redesign-spec.md` §11).
2. Export frontend bindings with `cargo run --bin export_types`.
3. Replace mock repository internals with generated command calls, including wiring `listen_workspace_event` to the Tauri event emitted by `rebuild_library_cache`.
4. Keep UI component props unchanged.
5. Keep path opening in `app-opener.ts`, not in screen components.
