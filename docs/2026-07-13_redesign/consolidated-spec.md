# Modkeeper Redesign — Consolidated Implementation Spec

Status: Implementation-ready. **Standalone.** This document supersedes the `2026-07-09_redesign/*`
spec set and the `2026-07-13_redesign/outline-of-redesign.md` for execution purposes — you build
from this one file. The earlier docs become historical: their reasoning is preserved here in
condensed form (see §14 Decision Ledger for the C1–C15/T1 history), and only consult them if you
want the extended debate behind a decision, never to know *what* to build.

Sources folded in: `2026-07-09_redesign/backend-redesign-spec.md`,
`2026-07-09_redesign/frontend-redesign-spec.md`,
`2026-07-09_redesign/frontend-redesign-data-api-contract.md`,
`2026-07-09_redesign/design-review.md` (C1–C15/T1),
`2026-07-13_redesign/outline-of-redesign.md` (scope + sequencing deltas),
`2026-07-13_redesign/current-implementation-audit.md` (baseline).

---

## 1. Purpose and Framing

Two tracks, one goal: a finished, coherent, demonstrable codebase.

- **This app is a portfolio/resume piece, not a shipping product.** SPT's Forge doesn't allow
  AI-written apps (closing the public-distribution path) and the author no longer plays the game.
  The bar is *minimal closure* — a smaller coherent codebase beats a larger one with more features.
  Scope cuts here (manifest system, backup-browsing UI, tool registry) remove working code that this
  version of the app doesn't need to carry, not broken code.
- **Backend is a refactor, not a redesign.** Goal: FP-by-default so a function is checkable in
  isolation instead of requiring the reader to hold a mutable object's whole state in their head.
  A legibility goal for a solo maintainer. Anything that isn't restructuring existing logic (new
  subsystem, new dependency, new command surface) is out of scope for the backend track by
  definition. **Behavior must not change** as a side effect of restructuring; the existing test
  suite is the correctness oracle at every step.
- **Frontend is the primary effort** and is staged by architectural layer with a walking-skeleton
  gate (§8), so a wrong cross-layer contract surfaces once — cheaply, against one screen — instead
  of at the end when every screen already depends on it.

---

## 2. Current Implementation Baseline

Condensed from the audit; the factual starting point every change is measured against.

- **Stack:** Tauri v2, Rust backend (`src-tauri/`), React 19 + TS frontend (`src/`), IPC via
  `tauri-specta`-generated `src/gen/bindings.ts`.
- **Domain:** A *library* pairs one **game root** (SPT install) with one **library/repo root**
  (`game_root/.mod_keeper` by default). Mods are imported from `.zip`/folders/loose files, classified
  `Client`/`Server`/`Both`/`Unknown`, enabled/disabled cheaply (recorded state), then **synced** —
  deployed into the game root via symlinks.
- **`core/library.rs::Library`** is the central stateful object, one loaded at a time in
  `AppRegistry.active_instance: Arc<Mutex<Option<Library>>>`. Owns `id` (uuid, minted at create,
  persisted in `manifest.toml`), `mods: BTreeMap` (recorded), `cache: LibraryCache` (derived from
  disk), `is_dirty: bool` (set on mutation, cleared by `sync()`), and path-rule structs.
- **Persistence:** two plain TOML files per library — `manifest.toml` (`LibraryDTO`) and
  `cache.toml`. No database. App-level config is `confy`-persisted `GlobalConfig`
  (`library_last`/`library_recent` — plain paths, no ids).
- **Two sources of truth:** `Library.mods` (recorded) vs `Library.cache.mods` (on-disk) must be kept
  in step by hand; `rebuild_library_cache` reconciles drift (and today also renames folders on disk
  and drops backups for renamed mods).
- **Deployment:** ownership-map symlinking (`core/deployment.rs`) — collision check, then link at the
  highest uniquely-owned ancestor path. `Library::sync()` is `purge → deploy → mark_clean → persist`
  (full teardown+rebuild, not incremental).
- **Concurrency:** every mutating command clones `Arc` handles, runs in `spawn_blocking`, locks the
  mutex for the operation's duration (`utils/thread.rs::with_lib_arc(_mut)`). No queue, no overlap
  guard beyond the mutex; `parking_lot::Mutex` is not FIFO.
- **Startup:** `lib.rs::run()` staged 1–7; background `load_initial_library`, a 10s watchdog that
  `exit(1)`s if the frontend's `init` command hasn't fired, and `init` is what shows/focuses the
  window (prevents unstyled flash).
- **Errors:** one flat `SError` enum, default serde tagging, `Display` for messages, raw enum crosses
  IPC; frontend `translateError` pattern-matches the serialized shape.
- **Known cruft:** `help` crate unused; `models/config.rs::GlobalConfig` is a dead second struct;
  `commands/test.rs::create_simulation_game_root` ships in release (only its bindings export is
  debug-gated); backups are a full user-facing history feature tied to overwrite-upgrades.
- **Tests:** three Rust integration suites (`tests/library.rs`, `tests/mod_fs.rs`, `tests/linker.rs`
  + `tests/common/`). No frontend tests.

---

## 3. Scope

### In scope
- FP-by-default backend refactor (§4–§7) with behavior preserved.
- Plain-file App Config replacing `confy` (stable library ids, app state, backend-owned settings).
- `verb_noun` endpoint contract matching the frontend (§7).
- Fidelity Modern frontend redesign built fresh under `src/redesign/` (§8–§12).
- Structured logging + error boundary (§13).
- Backup collapsed into an internal upgrade-safety mechanism (§7f).
- Manifest removal (hash-only mod identity) landed as a behavior change before structural work.

### Out of scope (cut against minimal-closure)
- Manifest metadata system (version/author/description/deps/effects/links/documentation) — removed;
  identity becomes hash-only.
- User-facing backup browse/restore UI — removed; backup becomes internal plumbing only.
- Mod detail route, load-order management, live console, conflict visualizer, AI features.
- SQLite Library DB — cancelled (C4). `manifest.toml`/`cache.toml` stay the live per-library format.
- Changing the deployment/linking algorithm or on-disk layout — only call boundaries move.

### Deferred (not cancelled — revisit as a separate initiative)
- **Executable tool registry** in full: backend `commands/tool.rs`, `core/tool_service.rs`,
  `tools.toml`, icon validation pipeline (`image` crate / `utils/icon.rs` rework); frontend Configure
  Tool dialog and Manage Library's tools-section content; `tool-text.ts` copy. Nothing in this spec
  assumes the tool registry exists. The `ToolSummary` type and `toolsByLibraryId` stay in the
  workspace shape (present-but-empty this pass) for forward compatibility — see §7d note.

### Deltas from the 2026-07-09 set (applied throughout this document)
1. Tool registry **deferred** (above).
2. `GameOrServerRunning` guard **kept** (reverses C2) — single-platform personal use; it protects the
   author's own running game/server from mid-sync corruption on Windows today.
3. `bulk_update_mods` (enable/disable/delete) is a **plain blocking** `Result<LibraryWorkspace,
   SError>` call, **not** fire-and-track (reverses part of C1/§8a). Enable/disable is metadata-only;
   delete touches only selected mods — neither walks the whole tree. **Fire-and-track is exactly
   three commands:** `install_mod_archives`, `rebuild_library_cache`, `sync_mods`.
4. Comment removal is its own later phase (§15), not a rule enforced mid-refactor.
5. Frontend staged by architectural layer + walking skeleton (§8), not a flat file sequence.

---

## 4. Backend: FP / OOP Decision Table

Keep stateful objects OOP where forcing FP means threading every field through every call; convert
the "struct-with-static-methods" utilities and the data-plus-logic `ModFS` to free functions.

| File | Decision | Notes |
|------|----------|-------|
| `core/library.rs` | **OOP (keep)** | Encapsulates dirty-flag, cache, paths. Dirty flag survives (deploy stays explicit, C3); cache/mods stay in-memory (SQLite cancelled, C4). |
| `core/registry.rs` | **OOP (keep)** | `AppRegistry`, thread-safe mutable app state via `Mutex`/`Arc`. Also gains the in-flight `taskId` map (§7e). |
| `config/global.rs` | **OOP → replaced** | Its confy-backed role moves to the new `store/` App Config (§7c). |
| `core/mod_fs.rs` | **Refactor to FP** | `ModFS` becomes a plain DTO; `resolve_id`/`infer_mod_type` become free functions taking `&Utf8Path`/`&[Utf8PathBuf]` + config, not `&self`. `read_manifest`/`read_manifest_guid` **deleted** (manifest removal), not refactored. |
| `utils/file.rs` | **Refactor to FP** | Drop `FileUtils` struct; top-level functions (`copy_recursive`, `atomic_write`, …). |
| `utils/toml.rs` | **Refactor to FP** | Drop `Toml` wrapper. |
| `utils/process.rs` | **Refactor to FP** | Drop `ProcessChecker`; free functions. |
| `core/mod_manager.rs` | **Already FP** | `add_mod`/`remove_mod`/`toggle_mod` over `&mut Library`. No action. |
| `core/decompression.rs` | **FP, relocate** | Single `extract` fn — move into `utils/file.rs` (or new `utils/archive.rs`); delete `decompression.rs`, no re-export shim. |
| `core/cleanup.rs`, `deployment.rs`, `cache.rs`, `linker.rs`, `mod_stager.rs`, `version.rs` | **No change** | Already FP; logic untouched. |
| `core/dto_builder.rs` | **Delete** | Manifest/icon enrichment dies with the manifest removal (C6). |
| `core/mod_documentation.rs` | **Delete** | Reads `manifest.documentation` — nothing left to read. |
| `core/mod_backup.rs` | **Shrink → internal** | Multi-backup/named/timestamped model goes; becomes one transient snapshot (§7f). |
| `utils/icon.rs` | **Deferred** | Its only caller (mod icons) dies with C6. The tool-icon rework that would repurpose it is deferred with the tool registry. Leave `utils/icon.rs` unused this pass, or delete it — do not build the `image`-crate validation path now. |
| `utils/id.rs` | **Keep** | `hash_id` (blake3 + base64url) is the hash-only identity mechanism. |
| `utils/time.rs`, `utils/thread.rs` | **Keep** | `thread.rs`'s `with_lib_arc(_mut)` is the kept lock model (§7e). |
| `commands/test.rs`, `models/test.rs` | **Delete outright** | `create_simulation_game_root` + `pub mod test;` + `collect_commands!` entry all go (C9). |
| `models/*` | Individually | `mod_dto.rs` shrinks (manifest fields + `icon_data` drop); `error.rs` gains `ConfigSaveFailed`, `TaskIdInUse`; `global.rs`/`config.rs` change with confy→toml; `mod_backup.rs` shrinks; `test.rs` deleted; `library.rs`/`paths.rs` unchanged. |

**Execution method (mechanical-first):** for each conversion do two commits — (a) move code verbatim
into the new shape, compile + green; (b) clean up naming/structure. A red test after a combined
commit can't tell you whether the move or the redesign broke it. Land the manifest-removal behavior
change (hash-only id) **first**, as its own commit, so the oracle reflects intended behavior before
structural work.

---

## 5. Core / Service / Interface Boundary

| Layer | Location | Responsibility | Rules |
|-------|----------|----------------|-------|
| Interface | `commands/` | Receive IPC params, validate input shape, call one service, return result. | No business logic, no direct FS, no direct `store/` access. |
| Service | `core/*_service.rs` | Orchestrate core functions + `store/` into a pipeline. | No Tauri types; plain params. |
| Core/Util/Store | `core/*.rs`, `utils/`, `store/` | Single-purpose functions, pure where possible. | No orchestration. |

**Known violation to fix first:** `commands/library.rs::add_mods` (and `remove_mods`) inline the
whole pipeline inside a `spawn_blocking` closure — service work in the interface layer. Extract into
`library_service::install_mods(handle, inputs, unknown_mod_name) -> Result<…, SError>` and
`library_service::remove_mods(handle, ids)`; the commands become input-marshaling + one call +
return. This is also the template for the endpoint rework (§7).

**Audit pass:** `commands/global.rs` (`open_library`/`create_library`/`init`/`close_library`/
`remove_library`) against "one command → one service call" during the §7 rename;
`core/library_service.rs` and `core/global_service.rs` re-checked after §6/§7c land new call shapes.

**Low-level FS exit criterion:** after §4 + the `extract` relocation land,
`rg "std::fs::|tokio::fs::" src-tauri/src/core` returns nothing — every FS primitive in `core/` goes
through `utils/file.rs`.

---

## 6. Manifest Removal (land first)

Identity becomes **hash-only** (drop the manifest-first branch). `ModFS::resolve_id` keeps only the
hash path: collect every file under `SPT/user/mods/<name>` and every `.dll` under `BepInEx/plugins/**`,
sort relative paths, concatenate, `blake3` + base64url (`utils/id.rs::hash_id`). `infer_mod_type`
(structural, no manifest) is unchanged. Delete `read_manifest*`, `dto_builder.rs`,
`mod_documentation.rs`, the manifest fields on `models/mod_dto.rs`, and `icon_data`. Update
`tests/library.rs`/`tests/mod_fs.rs` manifest-dependent cases to hash-only assertions and remove
`test_collect_files_excludes_manifest`. This lands as its own commit with the oracle green before any
FP restructuring.

---

## 7. Endpoint Contract, App Config, and Task Machinery

### 7a. Naming and shape rules
- `verb_noun`, matching the TS contract (§9) exactly — the contract owns names, this section owns
  which Rust file implements each.
- Commands receive serializable DTOs/primitives only (plus the `State<'_, AppRegistry>` extractor).
- Output `Result<T, SError>` returned directly at the boundary — no new boundary error type.
- One command → one service call. Domain-grouped files: `commands/library.rs`, `commands/global.rs`.
  (No `commands/tool.rs` this pass — deferred.)

### 7b. Command mapping (live this pass)

| Current | New command | File | Notes |
|---|---|---|---|
| `init` | `get_library_workspace` | `global.rs` | Returns full `LibraryWorkspace`. **Must carry over `state.init_called.store(true, …)`** (the startup watchdog handoff, C11) into this body. Assembly is read-only (§7c): a registered library whose manifest can't be read is a path-only stub, never dropped, never eagerly opened. |
| `open_library` | `activate_library` | `global.rs` | Takes `libraryId`. Looks up `library_root` from App Config `known_libraries`, opens (`Library::load`), writes `active_library_id` to `app_state`. Open-time errors surface here: `UnsupportedSPTVersion` hard-fails; unreadable manifest/cache → `InvalidLibrary` (C5). |
| `create_library` | `create_library` | `global.rs` | Optional `libraryRoot` (derive via `derive_library_root` when omitted). Writes a `known_libraries` entry **and** creates the library's own files. Adding a dir with a readable manifest **adopts** its `id`, never mints (C7). |
| `close_library` | *(removed)* | — | "Closing" is just clearing `active_library_id`. Confirm with product before deleting. |
| `remove_library` | `delete_library` | `library.rs` | Two **distinct** service code paths (destructive stakes): `deleteFiles: true` deletes the `known_libraries` row **and** the `library_root` dir; `deleteFiles: false` deletes only the row, leaving files so re-adding the same `gameRoot` re-adopts its id (C7). **Keeps the `GameOrServerRunning` guard** for `deleteFiles: true` (delta 2). |
| `rename_library` | `rename_library` | `library.rs` | Writes `manifest.toml` `name` only (App Config stores no name). Drop the optional-library-id dual path. |
| `rebuild_library_cache` | `rebuild_library_cache` | `library.rs` | **Fire-and-track** (§7e). Folder normalization renames only — the `remove_all_backups` coupling is **removed**, and rebuild does **not** re-link/deploy; a rename dangles that mod's deployed links until `sync_mods` (C8). Runs under the active-library mutex for its full duration. |
| `get_library` | *(folded into `get_library_workspace`)* | — | Per-mod data is part of the workspace. |
| `add_mods` | `install_mod_archives` | `library.rs` | **Fire-and-track** (§7e). Per-archive partial failure: collect per-archive errors instead of failing the whole call; failures ride the `mod_install_completed` event. Upgrading a mod takes a transient pre-overwrite snapshot (§7f). |
| `remove_mods` | `bulk_update_mods` (`action:'delete'`) | `library.rs` | **Plain blocking** `Result<LibraryWorkspace, SError>` (delta 3). |
| `toggle_mod` | `bulk_update_mods` (`action:'enable'\|'disable'`, one-element `modIds`) | `library.rs` | **Plain blocking** (delta 3). No `set_mod_enabled`. Commits mod state (`is_active`, `mark_dirty()`, `persist()` — metadata-only, cheap); it does **not** deploy — no symlinks touched, no collision check (C3). Sets `deployStale`. |
| `sync_mods` | `sync_mods` | `library.rs` | **Kept** explicit deploy (C3/M5). **Fire-and-track** (§7e). Wraps `Library::sync` (purge → redeploy → collision check → mark clean). **Keeps the `GameOrServerRunning` guard** (delta 2, reverses C2). Surface: execution-bar Sync button, highlighted when `deployStale`. |
| `reveal_mod` | *(removed — frontend-side)* | — | Frontend `app-opener.ts` calls `@tauri-apps/plugin-opener` directly. |
| `get_backups`/`create_backup`/`restore_backup`/`remove_backup` | *(removed)* | — | No user-facing backup feature; internal only (§7f). |
| `get_mod_documentation` | *(removed)* | — | Manifest removal. |
| `create_simulation_game_root` | *(deleted outright)* | — | C9. |
| `apply_window_effect` | `apply_window_effect` | `global.rs` | Unchanged. |
| *(new)* | `get_settings`, `save_settings` | `global.rs` | Backed by App Config `settings` (§7c). Mutating commands return the **full** settings object; frontend replaces its atom wholesale (T1). |
| *(new — not a command)* | `listen_workspace_event` | n/a | Frontend wrapper over Tauri events. Backend side: `app_handle.emit(...)` from the **three** fire-and-track ops on completion, each carrying its `taskId`. |

### 7c. App Config (plain file) + read-only workspace assembly

New top-level module `src-tauri/src/store/` (peer to `core/`/`commands/`/`utils/`/`models/`),
App-Config-only: `store/mod.rs` (load/save) + `store/app_config.rs` (known_libraries, app_state,
settings). Plain functions, no orchestration. Services call into `store/*`; `commands/` never touch
`store/` directly.

**Crate choice — drop `confy`, use `toml` directly** (already a dependency). `confy::load` silently
returns `T::default()` on any read/parse failure and `save` is `let _ = confy::store(...)` (write
failures discarded) — for a file about to hold stable library ids + settings, a silent reset means
losing a user's registered libraries with no error. `toml` gives explicit read/parse/error control.
`directories` still locates the file (`app_data_dir/config.toml`).

**Write path — atomic + surfaced (C12/M7):**
1. New `SError::ConfigSaveFailed(String)`.
2. `save(&self) -> Result<(), SError>`; update every call site to propagate — **including**
   `lib.rs::load_initial_library`'s startup-thread `config.save()`: log at the call site, surface as
   a toast on the next successful init via a pending-warning flag the frontend reads once from the
   first `get_library_workspace`. Don't block/fail startup over it.
3. Atomic write: serialize to a temp file in the same dir, `std::fs::rename` over the target.
   Implement as free fn `utils/file.rs::atomic_write(...)` so manifest/cache writes can reuse it.

**Migration (once, at startup):** if `known_libraries` is empty and a `confy` `GlobalConfig` exists,
read `library_last`/`library_recent`; for each path, **adopt the `id` from that library's
`manifest.toml`** (mint a new `uuid` only where no readable manifest exists — C7). `library_last` →
`app_state.active_library_id`. Do not open any library (`Library::load`) — this is a plain TOML read.
Leave the old confy file in place until §12's removal step.

**Assembly of `get_library_workspace` is read-only (C13):** read `known_libraries`, then a read-only
read of each library's own files (same shape as today's `get_known_library_summary`) to fill
`modsByLibraryId`/`toolsByLibraryId` and derive each `LibrarySummary`'s `cacheStatus` and
`deployStale` (`deployStale` = that library's persisted dirty flag; `modCount`/`enabledModCount` are
**not** DTO fields — the frontend computes them). Assembly **never** calls `Library::load`, never runs
`version::fetch_and_validate`, never migrates. A library whose manifest can't be read (`IOError`
**or** `ParseError`) is a **path-only stub** (`{ path }`), never dropped, never a workspace-level
failure.

### 7d. `tools.toml` / tool registry — deferred
No `tools.toml`, `commands/tool.rs`, `core/tool_service.rs`, or icon pipeline this pass. Workspace
assembly still populates `toolsByLibraryId` as an **empty** map per library so the DTO shape is
stable for when the registry is picked back up. `ToolSummary`/`ToolUpsertInput` stay defined in the
contract (§9) but unused by live commands.

### 7e. Non-blocking operations, kept lock model, task tracking

**Three** operations are fire-and-track: `install_mod_archives`, `rebuild_library_cache`,
`sync_mods`. (`bulk_update_mods` is **not** — delta 3.)

- **Lock model kept as-is:** `Arc<Mutex<Option<Library>>>` + `with_lib_arc(_mut)`. Overlapping
  mutations serialize on the mutex; **no** reject-on-overlap, **no** `LibraryOperationInProgress`
  (C1). The mutex only ever holds the *active* library.
- **Active-library scope:** each fire-and-track command validates `libraryId == activeLibraryId` and
  rejects otherwise with a plain validation error before touching anything. `libraryId` is an
  assertion + the id completion events report against, not a guard key.
- **Ordering:** `parking_lot::Mutex` barges (not FIFO). Acceptable because mod-state writes are
  absolute `is_active: bool` sets — a race resolves to one of the requested states, never a corrupt
  third. (Applies to `bulk_update_mods` too, even though it's now blocking.)
- **Reads block behind in-flight active-library mutations** — a stated, accepted cost. Heavy ops
  hold the mutex for their FS duration, so `get_library_workspace` blocks that long against the
  active library.
- **Client-minted `taskId` (C15/M6):** every fire-and-track input carries a frontend-generated uuid.
  Client-minted because waiting for an accept round-trip reopens a race (a fast task could finish
  first). Frontend registers its handler in a single persistent event bus **before** invoking.
- **In-flight registry:** a `taskId → status` map on `AppRegistry`, inserted on accept, removed on
  completion. A `taskId` reused while present is rejected with new `SError::TaskIdInUse(String)`.
- **Two-phase signal:** the invoke `Result` is *accept* (sync validation failures reject with no
  event); the completion event, matched by `taskId`, is the *outcome*. Events on a stale bus (after
  reload) are silently dropped; the view self-corrects on the next read.

**Rebuild's real shape (C8):** `normalize_mod_folders` renames folders on disk during the slow phase
— safe under the whole-rebuild mutex hold. Remove the `remove_all_backups` call from normalization
(`library_service::rebuild_library_cache`); orphaned backups are left on disk. Rebuild does not
re-link, so a rename dangles all of that mod's links until an explicit `sync_mods`.

### 7f. Backup → internal upgrade-safety only

No user-facing backup feature. Reshape `core/mod_backup.rs` (or rename `mod_snapshot.rs`) — the
multi-backup/named/timestamped/`BackupManifest` model goes:
1. Before overwriting an existing mod's folder, take **one** transient snapshot (keep the config-file
   copy `create_backup` already does; drop naming/manifests — at most one snapshot in flight per mod).
2. Overwrite succeeds → discard the snapshot immediately.
3. Overwrite fails partway → restore from snapshot before returning the error (mod left exactly as it
   was).
4. `remove_mod` keeps a defensive cleanup of any leftover snapshot dir (crash backstop).

Unreachable from `commands/` — no rows, no new `SError` variants beyond what an upgrade failure needs.
It's plumbing inside `library_service::install_mods`. `models/mod_backup.rs`'s
`ModBackup`/`BackupManifest` shrink or disappear.

### 7g. Error contract — `SError`'s wire shape

`SError` stays as-is (same variants/payloads/`Display`/`impl_from!`); commands return
`Result<T, SError>` directly. **One change:** add `#[serde(tag = "code", content = "data")]` so every
variant serializes to a stable `{ code, data }` (`code` = variant name verbatim, generated by the
attribute — no mapping table to drift). New variants (`ConfigSaveFailed`, `TaskIdInUse`) get a `code`
for free. No `CorruptLibrary` variant — unreadable/unparseable maps to existing
`InvalidLibrary(String, String)`, detected at `read_library_manifest` catching **both** `IOError` and
`ParseError` (C5/M8).

**`message` never crosses the wire.** `Display` is used exactly once, at the point a command returns
`Err`, to log an English line for developers (`tracing::error!("{e}")` — ties into §13). The frontend
receives only `code`/`data` and maps `code` to a translated string.

### 7h. Dependency changes
- Remove `help` (unused).
- Remove `radix-ui` (frontend, unused — recorded here, executed frontend-side).
- No database dependency added (SQLite cancelled). `toml` already covers App Config + per-library.
- **Do not** add `image` this pass (tool-icon validation is deferred with the registry).
- Drop `confy` once §12 confirms nothing reads it.

---

## 8. Frontend: Build Order (staged by layer + walking skeleton)

Build fresh under `src/redesign/`. The current `src/modules/*` files are **reference only** — do not
import them from redesign code (see §10 preservation rule). Build order: a design-system stage, then
one vertical slice that proves the cross-layer contracts end-to-end, then three horizontal stages
built against contracts the slice already validated.

### 8.1 Structure Stage — design system, shared primitives, Storybook
- `src/redesign/styles/fidelity.css` — tokens + utilities (§11).
- `src/redesign/shared/components/*` — `fidelity-panel`, `fidelity-button`, `fidelity-icon-button`,
  `fidelity-input`, `fidelity-section`, `confirm-dialog`.
- **Storybook, scoped to `shared/components/*` only** (new this revision): `@storybook/react-vite`
  (matches Vite + React 19 — no new bundler), config at `.storybook/main.ts`/`preview.ts`, stories
  colocated (`fidelity-button.stories.tsx` beside the component). Each primitive gets a default story
  plus one per visually distinct state (disabled, focus-visible, error/destructive). Add
  `bun run storybook` (devDependency + script) now.
- **Exit:** every shared primitive renders standalone in Storybook with no dependency on
  `redesign-types.ts`, repositories, or Jotai. If a component needs business data, it isn't a shared
  primitive — it belongs in Composition.

### 8.2 Walking Skeleton — one screen, all layers
Cut one thin vertical slice through every layer before the horizontal build-out, to validate the
contracts the horizontal stages would otherwise only discover at the end. Scope: the Library screen
reduced to a single mod card + the execution bar.

Deliverables (thin, one instance each):
- A minimal route + shell mount (thin slice of 8.3) so the slice renders in the real app, not just
  Storybook.
- One repository + one atom pair for the mod list (thin slice of 8.4), backed by `example-data.ts`
  via the real-first/mock-fallback path — so `MOCK-FALLBACK` is exercised, not just declared.
- One **plain** call (`toggleModStatus`) with local per-click pending state, and one
  **fire-and-track** call (`syncMods`) with client-minted `taskId` + event-bus registration — so
  both repository shapes are proven against a real consumer.
- The derived **"library busy"** state wired producer→consumer: produced by the fire-and-track call,
  read by both the shell affordance and the card's disabled/pending treatment. (This is the one
  contract split across three horizontal stages; the slice collapses it into one checkable path.)
- One mod card + toggle + execution bar (thin slice of 8.5) consuming the above.

**Exit:** clicking the toggle updates through the real atom/repository path; triggering `syncMods`
flips "library busy" and both shell + card reflect it; killing the real backend call falls back to
mock without a crash. Once green, the atom/repository/library-busy shapes are **frozen** — 8.3–8.5
build out against them, not toward them. A forced shape change here is the plan working as intended.

### 8.3 Global Stage — routes, store, i18n scaffolding
- `src/redesign/app/*` (`redesign-root.tsx`, `redesign-error-boundary.tsx`, `redesign-initializers.tsx`).
- `src/redesign/shell/*` (`desktop-shell.tsx`, `app-background.tsx`, `app-header.tsx`,
  `bottom-navigation.tsx`, `page-title.tsx`).
- Route adapters (§10 table) — thin mounts.
- `src/redesign/state/*` (atom shapes; full data wiring in 8.4, minus the atom already proven in 8.2).
- `src/redesign/i18n/*` (`common-text.ts`, `library-text.ts`, `settings-text.ts`, `error-text.ts`).
  Skip `tool-text.ts` (tool registry deferred) — omit the file or leave it empty.
- The new/old UI runtime toggle (§10a) — shell-level.
- **Exit:** app boots to a working shell (header, nav, routing) before real/mock data flows through it.

### 8.4 Business Layer Stage — data, repositories, state wiring
- `src/redesign/data/*` (`redesign-types.ts`, `example-data.ts`, `library-repository.ts`,
  `settings-repository.ts`; `tool-repository.ts` stubbed only if trivial — real content deferred).
- Real-first/mock-fallback (§12), `MOCK-FALLBACK` tag discipline.
- `bulkUpdateMods`/`toggleModStatus` wired as plain awaited calls (delta 3) — no `taskId`, no
  event-bus registration. Local per-click pending only: set on click, cleared when the promise
  settles — instant feedback even when queued behind a heavy op's lock hold.
- `rebuildLibraryCache`/`installZipArchives`/`syncMods` keep the fire-and-track repository shape
  (client-minted `taskId`, register-before-invoke, `listenWorkspaceEvent`).
- Derived **"library busy"** — true whenever a fire-and-track task is pending for the active library —
  surfaced for the shell/execution bar. Turns an unexplained delay (a toggle queued behind an install)
  into a visible affordance instead of a silent hang.
- Wire `library-state.ts`/`settings-state.ts` atoms to the repository layer.
- **Exit:** every atom from 8.3 is backed by a real repository call or its mock fallback; no
  screen-level component beyond the 8.2 slice's card exists yet, but the data/state layer is fully
  functional against `example-data.ts`.

### 8.5 Composition Stage — screens, dialogs, cards
- Library composition (`library-screen.tsx`, `library-content.tsx`, empty states, toolbar, execution
  bar, grid, cards).
- Manage Library dialog, with its **tools section rendered empty/hidden** (tool registry deferred).
- Configure Tool dialog — **not built** this pass.
- Settings screen, minus any tool-related rows.
- **Exit:** §12's acceptance criteria, scoped down by the §3 deltas (no tool-registry UI, no
  fire-and-track on `bulk_update_mods`).

---

## 9. TypeScript Contract (source of truth for names + shapes)

All camelCase (matches `tauri-specta` from `#[serde(rename_all = "camelCase")]`). Timestamps are
ISO 8601 UTC. IDs are opaque strings; APIs accept ids, not paths.

```ts
export type LibraryId = string
export type ModId = string
export type ToolId = string
export type ModType = 'client' | 'server' | 'both' | 'unknown'

// SError as it serializes with #[serde(tag="code", content="data")] — no message field.
export type SError = { code: string; data?: unknown }

export type LibraryWorkspace = {
  activeLibraryId?: LibraryId
  libraries: (LibrarySummary | LibraryStub)[]
  modsByLibraryId: Record<LibraryId, ModSummary[]>
  toolsByLibraryId: Record<LibraryId, ToolSummary[]> // empty this pass (tool registry deferred)
  settings: AppSettings
}

export type LibraryStub = { path: string } // registered-but-unreadable; render bare path, remove-only

export type LibrarySummary = {
  id: LibraryId
  name: string
  gameRoot: string
  libraryRoot: string
  sptVersion?: string
  cacheStatus: CacheStatus
  deployStale: boolean // deployed symlinks no longer match recorded state; drives Sync highlight
  updatedAt: string
  // No modCount/enabledModCount: derive from modsByLibraryId[id]
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
  sourcePath?: string
  installedPath?: string
  updatedAt: string
  // No iconDataUrl: card shows a ModType-tinted category icon (C6)
}

// Defined for forward-compat; unused by live commands this pass (tool registry deferred).
export type ToolSummary = {
  id: ToolId; libraryId: LibraryId; name: string; executablePath: string
  iconDataUrl?: string; launchArgs?: string; updatedAt: string
}
export type ToolUpsertInput = {
  id?: ToolId; libraryId: LibraryId; name: string; executablePath: string
  iconData?: string; launchArgs?: string
}

export type AppSettings = { theme: 'system' | 'light' | 'dark'; accentColor: string; language: string }

export type OperationAccepted = { accepted: true }

// Three fire-and-track completion events (bulk_update dropped — delta 3).
export type WorkspaceEvent =
  | { type: 'cache_rebuild_completed'; taskId: string; libraryId: LibraryId; cacheStatus: CacheStatus; workspace: LibraryWorkspace }
  | { type: 'mod_install_completed'; taskId: string; libraryId: LibraryId; failures: { archivePath: string; error: SError }[]; workspace: LibraryWorkspace }
  | { type: 'sync_completed'; taskId: string; libraryId: LibraryId; failures: { modId: ModId; error: SError }[]; workspace: LibraryWorkspace }
```

### API surface (live this pass)

```ts
get_library_workspace(): Promise<LibraryWorkspace>
create_library(input: { gameRoot: string; libraryRoot?: string; name?: string }): Promise<LibraryWorkspace>
activate_library(input: { libraryId: LibraryId }): Promise<LibraryWorkspace>
rename_library(input: { libraryId: LibraryId; name: string }): Promise<LibraryWorkspace>
delete_library(input: { libraryId: LibraryId; deleteFiles: boolean }): Promise<LibraryWorkspace>

// PLAIN blocking (delta 3) — returns the updated workspace directly; never deploys.
bulk_update_mods(input: { libraryId: LibraryId; modIds: ModId[]; action: 'enable' | 'disable' | 'delete' }): Promise<LibraryWorkspace>

// Fire-and-track (client-minted taskId; outcome via listen_workspace_event).
rebuild_library_cache(input: { taskId: string; libraryId: LibraryId }): Promise<OperationAccepted>
install_mod_archives(input: { taskId: string; libraryId: LibraryId; archivePaths: string[] }): Promise<OperationAccepted>
sync_mods(input: { taskId: string; libraryId: LibraryId }): Promise<OperationAccepted>

get_settings(): Promise<AppSettings>
save_settings(input: AppSettings): Promise<AppSettings> // returns FULL settings; frontend replaces wholesale (T1)

listen_workspace_event(callback: (event: WorkspaceEvent) => void): () => void

// Deferred (not built this pass): upsert_tool, delete_tool, execute_tool.
```

**`SError.data`** is a positional **array** for multi-field tuple variants (e.g.
`InvalidLibrary(String,String)` → `[path, reason]`) — interpolation must be written against that,
not an object.

---

## 10. Frontend Source Layout, Preservation, Repositories, State

### Source layout (`src/redesign/`)
`app/` (redesign-root, error-boundary, initializers) · `shell/` (desktop-shell, app-background,
app-header, bottom-navigation, page-title) · `styles/fidelity.css` · `data/` (redesign-types,
example-data, library-repository, settings-repository; tool-repository stub only) · `i18n/`
(common-text, library-text, settings-text, error-text — no tool-text) · `state/` (library-state,
settings-state) · `shared/{components,hooks,utils}` (hooks: use-command-error, use-zip-file-picker,
use-window-drop-zone; utils: app-opener, mod-display, file-filters, sorting) · `library/`
(library-screen, library-content, library-title, library-empty-card, library-activate-empty-state,
library-drop-empty-state, library-execution-bar, mod-grid, mod-grid-toolbar, mod-title-card,
mod-category-icon, bulk-actions-menu; `manage-library/*` dialog + sections — tools section rendered
empty; **no `tools/configure-tool-dialog.tsx` this pass**) · `settings/` (settings-screen,
setting-row, theme-mode-control, accent-swatches, language-select).

Route adapters after switch-over: `__root.tsx` → RedesignRoot; `library.tsx` → Outlet adapter;
`library.index.tsx` → LibraryScreen; `settings.lazy.tsx` → SettingsScreen; `library.$id.*` → redirect
to `/library` or remove in cleanup.

### Preservation rule
Current frontend files are reference material; redesign code must not depend on the old feature
structure. Do not import from redesign code: `modules/root/{app-navigation,file-drop-handler,
library-init,settings-init}.tsx`, `modules/mod-list/*`, `modules/mod-details/*`, `modules/settings/*`,
`components/header-portal.tsx`, `utils/header-portal-context.ts`, `utils/dependency-check.ts`,
`components/mod-version.tsx`. **Allowed reuse:** `components/ui/*` primitives; low-level helpers
(`utils/result.ts`, `utils/error.ts`, `utils/i18n.ts`, `utils/theme.ts`); `lib/settings-storage.ts`
(may wrap, but Settings UI is rebuilt). Build under `src/redesign/` first; existing route files
become thin adapters only after new files exist; preserve replaced route content under
`docs/.../reference/current-frontend/`; don't delete old files during switch-over (later explicit
cleanup). **Recorded exception:** `modules/settings/developer-settings.tsx` is deleted outright with
its backend command (C9).

### 10a. New/old UI runtime toggle
`useLegacyUi` boolean in the settings repository (persisted with theme/accent/language), surfaced as
a single transition-only utility row on the Settings screen (§12.6), explicitly temporary. Not a
build-time flag. `__root.tsx` checks it and renders the current app's root tree when on — both trees
in the bundle during transition (the one exception to "old files are reference only"). Removed in the
same later cleanup that removes the legacy UI.

### Repositories (`data/*-repository.ts`)
UI never calls `src/gen/bindings.ts` (old contract) or constructs request/response shapes; it calls
repository functions typed against `redesign-types.ts`. Repository internals follow **real-first /
mock-fallback**: call the new generated binding if stubbed; else fall back to `example-data.ts`, every
fallback tagged `// MOCK-FALLBACK: <why>` (single grep target, → zero once backend covers it).

```ts
// library-repository.ts
loadLibraryWorkspace(): Promise<LibraryWorkspace>
createLibrary(input: { gameRoot: string; libraryRoot?: string; name?: string }): Promise<LibraryWorkspace>
activateLibrary(libraryId: LibraryId): Promise<LibraryWorkspace>
renameLibrary(input: { libraryId: LibraryId; name: string }): Promise<LibraryWorkspace>
deleteLibrary(input: { libraryId: LibraryId; deleteFiles: boolean }): Promise<LibraryWorkspace>

// PLAIN awaited (delta 3): resolve with the updated workspace; local per-click pending only.
bulkUpdateMods(input: { libraryId: LibraryId; modIds: ModId[]; action: 'enable'|'disable'|'delete' }): Promise<LibraryWorkspace>
toggleModStatus(input: { libraryId: LibraryId; modId: ModId; enabled: boolean }): Promise<LibraryWorkspace>
// one code path: internally calls bulkUpdateMods({ modIds:[modId], action: enabled?'enable':'disable' })

// Fire-and-track: repository mints taskId (uuid) + registers completion in the bus BEFORE invoking.
rebuildLibraryCache(libraryId: LibraryId): Promise<OperationAccepted>
installZipArchives(input: { libraryId: LibraryId; paths: string[] }): Promise<OperationAccepted>
syncMods(libraryId: LibraryId): Promise<OperationAccepted>

listenWorkspaceEvent(callback: (event: WorkspaceEvent) => void): () => void
```

`listenWorkspaceEvent` is a **single persistent bus**, initialized once by `redesign-initializers.tsx`,
dispatching by `taskId`; identical caller code whether driven by the mock emitter (prototype) or the
real Tauri listener. In the prototype the three fire-and-track mocks resolve `{ accepted: true }`
quickly, then after a short delay mutate `example-data` and emit the matching event.

`settings-repository.ts`: `loadSettings`/`saveSettings` (both async from day one; `saveSettings`
returns the full object, replaces the atom wholesale — T1), `applyTheme`/`applyAccent`/`applyLanguage`.
Local storage is the prototype-only `MOCK-FALLBACK` until App Config lands (contract §8).

`app-opener.ts` wraps `@tauri-apps/plugin-opener` in `openGameRoot`/`openLibraryRoot`/`openModSource`;
UI never imports the opener plugin directly; in prototype it toasts/logs when the runtime is absent.

### State (Jotai — durable only)
`library-state.ts`: `libraryWorkspaceAtom`, `activeLibraryAtom`, `libraryListAtom`, `modListAtom`,
`selectedModIdsAtom`, `librarySearchAtom`, `librarySortAtom`, `libraryTypeFilterAtom`,
`visibleModsAtom`, plus a derived **library-busy** atom (true when a fire-and-track task is pending
for the active library). `settings-state.ts`: `settingsAtom`, `themeModeAtom`, `accentColorAtom`,
`languageAtom`. Confirmation/form-draft/transient state stays parent-owned or local. `isActive` is
derived (`library.id === workspace.activeLibraryId`) in one place.

### Error handling
No silent `.catch(() => {})` on either branch. Mock-fallback branches return promises, surface
simulated failures via toast, keep dialog state stable, log to console as secondary context. Real
branches convert `Result<T, SError>` with `ur`, surface failures via a toast using an i18n string
looked up from `SError.code` (never raw), keep dialog state stable, log `code`/`data` as secondary
(the English detail lives in the backend log, §13).

---

## 11. Design System

`fidelity.css` tokens (MODIFICATIONS.md authoritative over DESIGN.md):

```css
:root{
  --mk-primary:#e91e63; --mk-on-primary:#fff; --mk-tertiary:#00828d;
  --mk-surface:#fff8f7; --mk-surface-container:rgba(255,233,232,.72);
  --mk-surface-strong:rgba(255,248,247,.84); --mk-outline:rgba(146,110,109,.38);
  --mk-text:#281717; --mk-text-muted:#5d3f3e;
  --mk-radius-control:1rem; --mk-radius-panel:2rem; --mk-radius-dialog:2rem;
  --mk-content-max:1280px; --mk-shadow-panel:0 18px 60px rgba(40,23,23,.12);
}
```

Utilities: `.mk-glass-standard` (`blur(24px) saturate(140%)`), `.mk-glass-strong`
(`blur(40px) saturate(160%)`), `.mk-focus-ring` (`#e91e63`), `.mk-scrollbar` (warm track, primary
hover thumb). Typography: `system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`; no
viewport-scaled font size; no negative letter spacing; compact headings. Guardrails: primary action
`#e91e63`, teal sparingly, balance pink with warm neutrals, stable dimensions for icon
buttons/cards/toolbar controls/switches/nav to prevent layout shift. Main content:
`width: min(100%, var(--mk-content-max))`, auto horizontal margins, responsive padding.

---

## 12. Screens (Composition stage detail)

**Shell:** `DesktopShell` owns the frame below the native title bar; window title stays `Modkeeper`.
Full-viewport background, sticky header (title + subtitle only, no window controls), scrollable main,
centered max-width container, bottom-center nav (Home + Settings), dialog/toast portals. No old
header-portal system. Activation empty state renders no toolbar. `RedesignInitializers`: workspace
load via `loadLibraryWorkspace` (real + fallback), settings restore, locale init, global `.zip`-only
drag/drop; window effect only after a future backend bridge.

**12.1 Activate empty state** (no active library): title "Library", subtitle "Click to create or
activate a library", no toolbar, centered `LibraryEmptyCard` (16:9 dashed boundary, cloud-upload icon,
`MANAGE LIBRARIES` primary) opening `ManageLibraryDialog`. Home active.

**12.2 No-mods drop state** (active library, zero mods): reuse `LibraryEmptyCard`; whole card opens a
`.zip`-filtered picker; window drop accepts `.zip` only; non-zip → toast, no backend call; `.zip`
badge bottom-right.

**12.3 Title-only grid** (active library, ≥1 mod): Toolbar (Select All, sort-by-name, filter
Client/Server/Both/Unknown/all, bulk `ACTIONS [count]`, search). Grid 1/2/3 columns by width; stable
card height/icon/toggle. Card: left checkbox, `ModType`-tinted category icon, truncated title, right
enable/disable switch — no version/deps/docs/author/delete/explorer/detail-nav; enabled → primary
border/tint, disabled → reduced opacity. Interactions: checkbox → `selectedModIdsAtom`; Select All →
visible only; search filters locally live; **toggle calls `toggleModStatus` — plain awaited, local
per-click pending** (set on click, cleared on settle); bulk enable/disable/delete via `bulkUpdateMods`
(delete after local confirmation), same local-pending pattern; toggling never deploys — it sets
`deployStale`, reflected by the Sync button. The **library-busy** state (from a pending fire-and-track
op) disables the toolbar actions and shows a busy affordance so a queued toggle reads as "library
busy," not a hang.

**12.4 Manage Library dialog:** strong-glass body (2rem), top library tabs + dashed plus tab,
identity section, paths section, **tools section rendered empty/hidden (deferred)**, footer utilities +
activate. Plus tab → folder picker; valid+unregistered → create+select; identity save → `renameLibrary`;
copy path → clipboard + feedback; open explorer → `app-opener`; Rebuild Cache → `rebuildLibraryCache`
(fire-and-track, busy until `cache_rebuild_completed`); Delete → `DeleteLibraryConfirmDialog`
(checkbox deleteFiles vs entry-only → distinct `delete_library` calls; never route entry-only to a
destructive remove); Activate → `activateLibrary`; already-active → `Activated`, disabled. **No Tool
Settings entry this pass.**

**12.5 Configure Tool dialog — not built this pass (deferred).**

**12.6 Settings:** single centered column, no tabs; each row is a `SettingRow` (icon, label,
description, right control). Rows: appearance segmented (System/Light/Dark), accent swatches (default
`#e91e63`), language select, import/export (if retained), and the single transition-only new/old UI
switch row (§10a). No developer section. Theme updates `next-themes` + storage (+ window effect once
bridged); accent updates CSS vars immediately + persists; language calls `changeLocale`.

**Accessibility:** dialogs trap+restore focus; icon-only buttons labeled; visible focus on glass;
sufficient contrast; toolbar controls don't wrap into overlap; long names truncate; stable
dimensions; centered bottom nav at all widths; settings rows stack on narrow; no explanatory copy
beyond labels/empty-state prompts.

---

## 13. Logging Fix (Phase 4)

- Backend: structured logging + file sink for `tauri-plugin-log` (currently info-level, no sink);
  `tracing::error!("{e}")` at the point each command returns `Err` (the English detail trail — §7g).
- Frontend: global error boundary at `__root.tsx` (redesign: `redesign-error-boundary.tsx`); no
  silent `.catch(() => {})` on IPC calls.

---

## 14. Decision Ledger (C1–C15 / T1, as applied here)

Condensed outcomes so this spec is self-contained; consult `2026-07-09_redesign/design-review.md` only
for extended rationale. **Bold** entries are reversed/re-scoped by the 2026-07-13 deltas.

- **C1** — No per-library overlap guard / `LibraryOperationInProgress`; keep the mutex, serialize on
  lock. **Re-scoped:** fire-and-track is now three ops, not four (`bulk_update_mods` is plain
  blocking — delta 3).
- **C2** — Originally dropped the `GameOrServerRunning` guard. **Reversed (delta 2):** guard kept on
  `sync_mods` and `delete_library(deleteFiles:true)`.
- **C3 / M5** — Toggle commits mod state only; deploy is the explicit `sync_mods` step; `deployStale`
  drives the Sync highlight. Kept.
- **C4** — SQLite Library DB cancelled; per-library plain TOML stays. Kept.
- **C5 / M8** — Unreadable/unparseable manifest → `InvalidLibrary` (catches both `IOError` and
  `ParseError`) at open time. Kept.
- **C6** — Mods carry no icon; card uses a `ModType`-tinted category icon. Kept.
- **C7** — Library identity single-sourced in `manifest.toml`; migration/re-add **adopt** the id,
  never re-mint where a readable manifest exists. Kept.
- **C8** — Rebuild normalization renames only; `remove_all_backups` coupling removed; no re-link. Kept.
- **C9 / M3** — Delete `create_simulation_game_root` + `commands/test.rs` + `models/test.rs` +
  `developer-settings.tsx`. Kept.
- **C10 / M10** — Tool-icon validation via refactored `utils/icon.rs` (content-sniffed MIME, `image`
  decode/resize). **Deferred** with the tool registry — not built this pass.
- **C11** — `get_library_workspace` must carry over `init`'s `init_called.store(true, …)` watchdog
  handoff. Kept.
- **C12 / M7** — App Config writes atomic (`atomic_write`) + surfaced (`save() -> Result`,
  `ConfigSaveFailed`), including the startup-thread log-and-toast-later path. Kept.
- **C13** — Workspace assembly read-only; unreadable library → path-only stub, never dropped. Kept.
- **C15 / M6** — Client-minted `taskId`, register-before-invoke, single persistent bus, drop stale
  events. Kept for the **three** fire-and-track ops.
- **T1** — Settings backend-owned in App Config; mutating commands return the full object; frontend
  replaces wholesale. Kept.

---

## 15. Comment Removal (Phase 6 — last)

Run once, after §4–§13 land and the codebase stops moving structurally. Batch, file by file. Strip
comments that restate what the code says (e.g. "debounce function for debouncing"); **keep** comments
that encode non-obvious platform/runtime constraints — the `linker.rs` Windows directory-symlink
`remove_dir`-then-`remove_file` fallback is the concrete example on record.

---

## 16. Implementation Sequence + Verification

### Backend order
1. Lock the oracle green; land manifest-removal (hash-only id) as its own commit (§6).
2. FP refactor (`mod_fs`, `utils/file|toml|process`), mechanical-then-refined (§4).
3. Relocate `decompression::extract` into `utils/file.rs`; delete `decompression.rs` (§5).
4. Fix the `add_mods`/`remove_mods` boundary violation (§5) — template for the endpoint rework.
5. Add `store/` App Config additively, with `atomic_write` + `save() -> Result` (§7c).
6. App Config migration: confy → `known_libraries`/`app_state`, ids adopted from manifests (§7c, C7).
7. Cut settings to App Config (`get_settings`/`save_settings`, T1).
8. Endpoint rework (§7): rename/regroup into `commands/library.rs`/`global.rs`; add
   `#[serde(tag="code",content="data")]`; move `init_called.store` into the startup command (C11);
   delete `commands/test.rs`/`models/test.rs` (C9). **`bulk_update_mods` is a plain blocking call.**
9. Fire-and-track machinery (§7e) for the **three** ops: active-library validation, `taskId` registry
   on `AppRegistry`, `TaskIdInUse`; emit `cache_rebuild_completed`/`mod_install_completed`/
   `sync_completed` carrying the `taskId`. Remove `remove_all_backups` from rebuild (C8). Add
   `deployStale` to `LibrarySummary` (C3). **Keep the `GameOrServerRunning` guard** on `sync_mods`
   and `delete_library(deleteFiles:true)` (delta 2).
10. Backup → internal upgrade-safety (§7f).
11. Dependency cleanup + drop `confy` once nothing reads it (§7h).
12. `cargo run --bin export_types` → hand off to the frontend.

### Frontend order
Follow §8 stages: 8.1 Structure (+ Storybook) → 8.2 Walking Skeleton → 8.3 Global → 8.4 Business →
8.5 Composition → route adapters → retire `/library/$id` detail nav → `bun run extract` → build/smoke.
When backend commands land, replace mock repository internals with generated calls (field names/shapes
already match — repository-internals-only change).

### Then §13 logging, then §15 comment removal.

### Verification
- `cargo build`/`cargo clippy` clean throughout; `tests/library.rs`/`mod_fs.rs`/`linker.rs` green
  after every backend step.
- `bun run build`/`bun run lint` clean; **`bun run storybook` builds clean from the Structure stage
  onward**, not just at the end.
- New backend test coverage: App Config migration (ids adopted from manifests, minted only where none);
  read-only workspace assembly (unreadable manifest → path-only stub, never `Library::load` at list
  time; unsupported-SPT-version fixture lists fully, only fails on `activate_library`); App Config
  write path (`ConfigSaveFailed` surfaced; no half-written `config.toml`); `TaskIdInUse` on reused
  in-flight `taskId`; non-active `libraryId` rejected before touching anything; two overlapping
  same-mod writes both complete to one of the requested states; `bulk_update_mods` deploys nothing and
  sets `deployStale` until `sync_mods` clears it; rebuild normalization preserves backup dirs; §7f
  upgrade-safety (failed overwrite restores prior contents, success leaves no snapshot);
  **`sync_mods`/`delete_library` refuse with `GameOrServerRunning` when the game/server runs** (delta
  2).
- `cargo run --bin export_types` succeeds and bindings match §9 — a mismatch is a spec bug in whichever
  document is wrong, not something to paper over in generated code.

---

## 17. Deferred / Cut, Collected

- **Executable tool registry** — backend `commands/tool.rs`/`core/tool_service.rs`/`tools.toml`/icon
  pipeline (C10), frontend Configure Tool dialog + Manage Library tools content, `tool-text.ts`.
  Deferred, not cancelled. `ToolSummary`/`toolsByLibraryId` kept in the DTO (empty) for forward-compat.
- **SQLite Library DB** — cancelled (C4).
- **Fire-and-track for `bulk_update_mods`** — replaced by a plain blocking call + local per-click
  pending + derived "library busy" (delta 3).
- **Manifest metadata system, user-facing backup UI, mod detail route, load order, live console,
  collision visualizer, AI features** — cut against minimal-closure.
```
