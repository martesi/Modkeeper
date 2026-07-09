# Backend Redesign Implementation Spec

Status: Implementation-ready.

New document. There was no backend equivalent of `frontend-redesign-spec.md` in the 2026-05-10
round — backend only had `purpose-of-redesign.md`, `outline-of-redesign.md`, and audits
`1.1`–`1.9`, none of which pin down concrete file targets the way the frontend spec does. This
closes that gap so Phase 2 starts with the same rigor Phase 3 has.

Applies to: Phase 2 backend restructure in `docs/2026-07-09_redesign/outline-of-redesign.md`.

Inputs:
- `docs/2026-07-09_redesign/purpose-of-redesign.md` (Execution Strategy — the "mechanical refactor
  in place" rule is canonical there; §3 below restates it in implementation terms only)
- `docs/2026-07-09_redesign/outline-of-redesign.md`
- `docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md` (this spec must satisfy it)
- `docs/2026-05-10_redesign/audits/1.1_codebase-structure.md`
- `docs/2026-05-10_redesign/audits/1.2_dependencies.md`
- `docs/2026-05-10_redesign/audits/1.4_tests.md`
- `docs/2026-05-10_redesign/audits/1.5_justification.md`
- `docs/2026-05-10_redesign/audits/1.6_api-design.md`
- `docs/2026-05-10_redesign/audits/1.7_mod-manifest-removal.md`
- `docs/2026-05-10_redesign/audits/1.9_backend-redesign-audit.md`

All file targets below were checked against `src-tauri/src/` as it exists today, not assumed from
the audits — two files (`core/mod_manager.rs`, `core/decompression.rs`) exist in the codebase but
predate `1.1_codebase-structure.md` and are classified fresh in §4.

## 1. Purpose

Give backend Phase 2 the same thing frontend Phase 3 got from `frontend-redesign-spec.md`: a
concrete, file-by-file plan instead of a set of principles. Three things land in this phase that
didn't exist in the 2026-05-10 scope: the SQLite state/index layer, the executable tool registry,
and an endpoint contract that actually matches what `frontend-redesign-data-api-contract.md`
expects to call.

## 2. Refactor Scope

### In Scope

- FP-by-default refactor of `core/mod_fs.rs` and the `utils/*` "struct with static methods" files.
- Low-level FS extraction from `mod_fs.rs` and `core/decompression.rs` into `utils/file.rs`.
- Core/service/interface boundary enforcement across `commands/` and `core/*_service.rs`.
- `verb_noun` endpoint contract, matching `frontend-redesign-data-api-contract.md` exactly.
- SQLite-backed `store/` module for app state and index data.
- Executable tool registry: data model, store, service, commands, process spawning.
- Collapse the user-facing backup/restore feature into an internal upgrade-safety mechanism — see §9a.
- Comment removal, as a batch pass after structural moves settle.
- Dependency cleanup: remove `help`; evaluate `confy` removal once SQLite lands.

### Out of Scope

- Changing on-disk mod/game content layout or the deployment/linking algorithm itself (`core/deployment.rs`, `core/linker.rs`) — only their call boundaries move, not their logic.
- Any change to `core/deployment.rs`'s "immediate sync on toggle" logic beyond wiring it to the new `bulk_update_mods` endpoint shape — the sync-pipeline redefinition in `1.9_backend-redesign-audit.md` §2.B (remove the manual Sync button, commit on toggle) is accepted as-is and implemented via §7, not redesigned further here.

## 3. Execution Method

This restates `purpose-of-redesign.md`'s "Backend: Mechanical Refactor In Place" at
implementation granularity. If the two disagree, `purpose-of-redesign.md` wins.

1. **Lock the oracle.** `tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs` must be green before any file in §5–§9 is touched. Where `1.7_mod-manifest-removal.md`'s behavior change (hash-only mod ID) hasn't landed yet, land it first, as its own commit, so the oracle reflects the actually-intended behavior before structural work starts.
2. **Port mechanically, then refine.** Moving `ModFS`'s methods to free functions (§4) is two separate commits: (a) move the code verbatim into its new shape, get it compiling and green; (b) clean up naming/structure. Don't do both in one commit — a red test after a combined commit doesn't tell you whether the move or the redesign broke it.
3. **Batch mechanical violations.** For the endpoint audit (§7) and comment removal (§10), produce the full violation list first, then fix in grouped passes by category, not one-by-one as noticed.
4. **Review adversarially.** Each of §5 (boundary enforcement), §7 (endpoint contract), and §8 (SQLite migration) gets an independent pass — someone other than whoever wrote the change — specifically checking for a boundary violation, a silently changed behavior, or a test that was weakened to pass rather than fixed.

## 4. FP/OOP Decision Table

Finalized from `1.1_codebase-structure.md` and `1.5_justification.md`, plus classification for
the two files that postdate that audit.

| File | Decision | Notes |
|------|----------|-------|
| `core/library.rs` | OOP (justified) | Encapsulates dirty-flag, cache, paths. Forcing FP here means threading every field through every function call — rejected per `1.5_justification.md`. |
| `core/registry.rs` | OOP (justified) | `AppRegistry`, thread-safe mutable app state via `Mutex`/`Arc`. |
| `config/global.rs` | OOP (justified) | Owns the global config file's in-memory representation. |
| `core/mod_fs.rs` | **Refactor to FP** | `ModFS` struct currently holds both data and static-ish logic (`resolve_id`, `infer_mod_type`, `read_manifest`). Becomes a plain DTO; the three methods become free functions taking `&Utf8Path`/`&[Utf8PathBuf]` + config, not `&self`. `read_manifest` is deleted entirely per `1.7_mod-manifest-removal.md`, not refactored. |
| `utils/file.rs` | **Refactor to FP** | Drop the `FileUtils` struct; expose top-level functions (`copy_recursive`, etc.). |
| `utils/toml.rs` | **Refactor to FP** | Drop the `Toml` struct wrapper. |
| `utils/process.rs` | **Refactor to FP** | Drop `ProcessChecker`; expose free functions. |
| `core/mod_manager.rs` | **Already compliant** | Not in `1.1`'s table — postdates it. `add_mod`, `remove_mod`, `toggle_mod` are already free functions taking `&mut Library`. No action. |
| `core/decompression.rs` | **Already compliant, relocate** | Not in `1.1`'s table. Single free function `extract`. Already FP, but it's a low-level FS-adjacent primitive living in `core/` — move it per §6. |
| `core/cleanup.rs`, `core/deployment.rs`, `core/cache.rs`, `core/linker.rs`, `core/mod_stager.rs`, `core/mod_backup.rs`, `core/version.rs`, `core/dto_builder.rs` | No change | Already FP per `1.1`. `dto_builder.rs` is deleted, not refactored, per `1.7`. |
| `commands/*.rs` | No change (paradigm) | Already FP (interface layer, orchestration only) — boundary *content* is fixed in §5, not paradigm. |

## 5. Core / Service / Interface Boundary Map

The three-layer rule from `outline-of-redesign.md`:

| Layer | Location | Responsibility | Rules |
|-------|----------|---------------|-------|
| Interface | `commands/` | Receive IPC params, validate input shape, call one service, return result. | No business logic. No direct FS. No direct `store/` access. |
| Service | `core/*_service.rs` | Orchestrate core functions and `store/` calls into a pipeline. | No Tauri-specific types. Takes plain params. |
| Core / Util / Store | `core/*.rs`, `utils/`, `store/` | Single-purpose functions. Pure where possible. | No orchestration. |

### Known violation to fix first

`commands/library.rs::add_mods` (verified against the current file, lines 18–62) inlines the
entire pipeline inside the command body, inside a `spawn_blocking` closure: it resolves staging
material, calls `mod_stager::resolve`, then `mod_manager::add_mod` + `mod_stager::clean_up` in a
`try_for_each`, then builds the frontend DTO — all before returning. This is service-layer work
sitting in the interface layer. Extract it into `library_service::install_mods(handle, inputs,
unknown_mod_name, backup_name) -> Result<LibraryDTO, SError>`; the command becomes
input-marshaling + one call + return.

`remove_mods` (same file) follows the same shape and gets the same treatment:
`library_service::remove_mods(handle, ids)`.

### Audit pass required for

- `commands/global.rs`: `open_library`, `create_library`, `init`, `close_library`, `remove_library` — check each against the "one command → one service call" rule during §7's endpoint rework, since their names and shapes are changing anyway.
- `core/library_service.rs`: already mostly clean (verified — `open_library`, `create_library`, `rename_library`, `rebuild_library_cache`, `reveal_mod` etc. take plain `&Library`/`&mut Library`/`&GlobalConfig` params, no direct `std::fs`/`tokio::fs` calls found). Re-check after §6 and §8 land, since both add new call shapes into this file.
- `core/global_service.rs`: audit alongside `commands/global.rs` rework in §7.

## 6. Low-Level FS Separation

- `core/mod_fs.rs` keeps only mod-aware business logic (`resolve_id`, `infer_mod_type` as free functions, post-§4) that composes low-level utils. `read_manifest`/`read_manifest_guid` are deleted per `1.7_mod-manifest-removal.md`, not moved.
- `core/decompression.rs`'s `extract` moves into `utils/file.rs` (or a new `utils/archive.rs` if keeping archive extraction separate from generic file ops reads more clearly — either is acceptable, pick one and keep `core/decompression.rs` deleted, not left as a re-export shim).
- Exit criterion, not just a goal: after §4 and this section land, `rg "std::fs::|tokio::fs::" src-tauri/src/core` returns nothing outside files explicitly exempted here (none are exempted) — every FS primitive call in `core/` goes through `utils/file.rs`.

## 7. Endpoint Contract

### Naming and Shape Rules

1. **Naming:** `verb_noun`, matching `frontend-redesign-data-api-contract.md` exactly — the contract is the source of truth for names, this section is the source of truth for which Rust file implements each one.
2. **Input:** Commands receive only serializable DTOs or primitives, never raw state handles beyond the `State<'_, AppRegistry>` extractor.
3. **Output:** `Result<T, SError>` — the existing type, unchanged, returned directly at the command boundary exactly as commands do today. No new boundary error type. See §11 for the one attribute added to `SError` so its wire shape is stable.
4. **Scope:** One command → one service call.
5. **Grouping:** Domain-grouped files — `commands/library.rs`, `commands/global.rs`, new `commands/tool.rs`. No catch-all files.

### Command Mapping

| Current command | File | New command | File | Notes |
|---|---|---|---|---|
| `init` | `commands/global.rs` | `get_library_workspace` | `commands/global.rs` | Returns full `LibraryWorkspace`, not `LibrarySwitch`. |
| `open_library` | `commands/global.rs` | `activate_library` | `commands/global.rs` | Takes `libraryId`, not a path. Looks up `library_root` from the App Config's `known_libraries`, opens (or lazily migrates, §8) that library's `library.db`, then writes `active_library_id` to the App Config's `app_state` section. One file write, one DB open, in that order. |
| `create_library` | `commands/global.rs` | `create_library` | `commands/global.rs` | Add optional `libraryRoot` input per contract §5, item 5; derive via existing `derive_library_root` (`core/library_service.rs`) when omitted. Writes a new `known_libraries` entry (App Config) *and* creates that library's `library.db` (§8) — both, since a registered library with no DB or a DB with no registry entry are both invalid states. |
| `close_library` | `commands/global.rs` | *(removed)* | — | No longer a distinct concept once `activeLibraryId` lives in the App Config's `app_state` section — "closing" is just clearing that key. Confirm with product before deleting; if kept, it isn't part of the new contract, so it would be a non-contract escape hatch. |
| `remove_library` | `commands/global.rs` | `delete_library` | `commands/library.rs` | Splits into `deleteFiles: true` (this command's old behavior — deletes the `known_libraries` row *and* the `library_root` directory, `library.db` included) vs. `deleteFiles: false` (new — entry-only, deletes only the `known_libraries` row, leaves `library_root`/`library.db` on disk so re-adding the same `gameRoot` later picks it back up). These must be two distinct code paths in the service, not a shared function with a flag that's easy to get backwards given the destructive stakes. |
| `rename_library` | `commands/library.rs` | `rename_library` | `commands/library.rs` | Writes to that library's own `library_meta.name` (Library DB) — the App Config's `known_libraries` never stored a name to begin with (§8), so there's no second place to update. Drop the optional-library-id dual-path logic noted in `1.6_api-design.md` §5 now that IDs are non-optional. |
| `rebuild_library_cache` | `commands/library.rs` | `rebuild_library_cache` | `commands/library.rs` | Fire-and-track: returns `OperationAccepted` immediately, acquiring the per-library operation guard (§8a) before returning; emits `cache_rebuild_completed` on completion — see §8a for the scan/commit split and why this doesn't hold the DB lock for the scan. This is the concrete fix for the async gap flagged in the contract's resolved item 4. |
| `get_library` | `commands/library.rs` | *(folded into `get_library_workspace`)* | — | Per-mod-list data becomes part of the workspace response. |
| `add_mods` | `commands/library.rs` | `install_mod_archives` | `commands/library.rs` | Fire-and-track (contract §6): returns `OperationAccepted` immediately, acquiring the per-library operation guard (§8a); emits `mod_install_completed` with per-archive failures on completion — service must collect per-archive errors instead of failing the whole call on the first `SError`. Upgrading an existing mod (destination folder already exists) takes a transient pre-overwrite snapshot and discards or restores it per §9a — invisible to the frontend either way. |
| `remove_mods` | `commands/library.rs` | `bulk_update_mods` (`action: 'delete'`) | `commands/library.rs` | Fire-and-track, see `toggle_mod` row below — same command, same guard. |
| `toggle_mod` | `commands/library.rs` | `bulk_update_mods` (`action: 'enable'\|'disable'`, one-element `modIds`) | `commands/library.rs` | No separate `set_mod_enabled` — matches the contract's resolved item 3. Fire-and-track (contract §6): returns `OperationAccepted`, acquires the per-library operation guard (§8a), emits `bulk_update_completed` with per-mod failures on completion. Per `1.9_backend-redesign-audit.md` §2.B, this must commit to disk immediately (symlink/copy) as part of that background work, not just flip an in-memory flag. |
| `sync_mods` | `commands/library.rs` | *(removed)* | — | The manual Sync button goes away; `bulk_update_mods` applies filesystem changes as part of the call per the immediate-sync redefinition in `1.9`. |
| `reveal_mod` | `commands/library.rs` | *(removed — frontend-side)* | — | Frontend's `app-opener.ts` calls `@tauri-apps/plugin-opener` directly (`frontend-redesign-spec.md` §7); no backend command needed for this one. |
| `get_backups`, `create_backup`, `restore_backup`, `remove_backup` | `commands/library.rs` | *(removed — no longer commands)* | — | Resolved: there is no user-facing backup feature in the redesign — it existed to support the old mod detail page's backup history/restore UI, which is gone (no detail route, per `frontend-redesign-spec.md` §2 Out of Scope). Backup becomes a purely internal upgrade-safety mechanism with no IPC surface at all. See §9a. |
| `get_mod_documentation` | `commands/library.rs` | *(removed)* | — | Per `1.7_mod-manifest-removal.md`. |
| `apply_window_effect` | `commands/global.rs` | `apply_window_effect` | `commands/global.rs` | Unchanged — window chrome, not workspace data, not part of the contract's domain model. |
| *(new)* | — | `upsert_tool`, `delete_tool`, `execute_tool` | `commands/tool.rs` | See §9. |
| *(new)* | — | `get_settings`, `save_settings` | `commands/global.rs` | Backed by the App Config's `settings` section (§8), not `confy`, once migrated. |
| *(new — not a command)* | — | `listen_workspace_event` | n/a | This is a frontend-side wrapper (`frontend-redesign-spec.md`/contract §6) around the standard Tauri event API, not a `#[tauri::command]`. Backend side is `app_handle.emit(...)` calls from `rebuild_library_cache`, `install_mod_archives`, and `bulk_update_mods` when each finishes (§8a) — exactly the three fire-and-track operations, no others. No new specta command needed for this row. |

## 8. Library State/Index Layer (SQLite) and App Config (Plain File)

Two storage tiers, not one, and only one of them is SQLite. App-level state (which libraries are
known, which is active, app settings) is small, flat, and doesn't grow the way a library's own
mod/tool/cache data does — it doesn't need a database, and giving it one was the first draft's
mistake. The schema/shape for both tiers is in `frontend-redesign-data-api-contract.md` §7; this
section covers the Rust-side module layout, the confy-vs-toml decision, and the migration mechanics.

### Module Layout

New top-level module, peer to `core/`, `commands/`, `utils/`, `models/`:

```text
src-tauri/src/store/
  mod.rs                -- loads the App Config, opens each Library DB on demand
  schema.rs              -- versioned migrations for the Library DB schema (contract §7)
  app_config.rs            -- App Config: known_libraries, app_state, settings (plain file, not SQLite)
  library_store/
    mod.rs                  -- opens/creates one library's DB given its library_root
    meta.rs                  -- library_meta (singleton row)
    mods.rs                   -- mods table
    tools.rs                   -- tools table
    cache.rs                    -- cache_entries table
```

`store/*` is the low-level tier for persistence, the same role `utils/file.rs` plays for the
filesystem: plain functions, no orchestration. `app_config.rs` functions take a handle to the single,
in-memory `AppConfig` struct (loaded once at startup, saved on write — see Crate Choice below; no
"connection" concept, it's a file, not a database). Every `library_store::*` function takes a
`library_root: &Utf8Path` (or an already-open `rusqlite::Connection` — implementation detail)
identifying *which* library's DB to touch, since there is one such file per library, not one shared
table. Services (`library_service.rs`, new `tool_service.rs`) call into `store/*`, never the other
way around, and `commands/` never touches `store/` directly (§5).

### File Locations

- **App Config:** the app's own data directory (already reachable via the existing `directories`
  crate dependency — same place `confy`'s `GlobalConfig` lives today), e.g.
  `app_data_dir/config.toml`. Loaded once at startup, held in memory, written back on change.
- **Library DB:** inside that library's own mod-manager directory, i.e. `library_root/library.db`
  — a sibling of the existing `mods/`, `backups/`, `staging/` directories (`LibPathRules`,
  `models/paths.rs`), replacing `manifest.toml` and `cache.toml` in that same directory.
  `library_root` defaults to `game_root/.mod_keeper` (`derive_library_root`,
  `core/library_service.rs`) but can be overridden — the DB always lives wherever that library's
  `library_root` actually resolves to, not at a fixed path.

### Crate Choice

**Library DB (SQLite):** add `rusqlite` (with the `bundled` feature, so SQLite ships inside the
binary — no separate native dependency) and `rusqlite_migration` for versioned schema migrations.
Both are synchronous, matching the existing `tauri::async_runtime::spawn_blocking` pattern already
used for FS-bound work in `commands/`. Do not add `tauri-plugin-sql`: it's designed for direct
frontend-to-SQL access via JS, which this app doesn't want — all DB access stays behind Rust
commands per the contract's IPC boundary.

**App Config (plain file):** drop `confy`, use the `toml` crate directly (already a dependency).
`confy`'s `load()` silently returns `T::default()` on any read/parse failure
(`confy::load(...).unwrap_or_default()` — that's the current code, `config/global.rs`) — for a
small file that's merely inconvenient, but this file is about to gain new sections (settings,
stable library IDs) that didn't exist when it was last shaped, and a silent reset back to defaults
on a shape mismatch would mean losing a user's registered libraries with no error surfaced at all.
`toml` directly gives explicit control: read, and on a parse error, decide (log + start fresh, or a
hand-written migration from the previous shape) instead of confy making that decision invisibly.
This costs a small amount of boilerplate (`std::fs::read_to_string` + `toml::from_str`,
`toml::to_string_pretty` + `std::fs::write` — using `utils/file.rs` post-§4/§6, not raw `std::fs`)
in exchange for that control, which is the right trade for a file this central. `directories`
(already a dependency) still locates the file; only the read/write/error-handling layer changes.

### Migration: Old Config → App Config

Runs once, at startup, before anything else touches `store/`:

1. On first launch of the new version, if `known_libraries` is empty and a `confy` `GlobalConfig`
   file exists, read it (`library_last`, `library_recent` — both plain paths, no IDs today).
2. For each path, mint a new stable `LibraryId` (`uuid` — already a dependency) and insert a
   `known_libraries` row `{ id, library_root: path, last_opened_at }`. `library_last` becomes the
   `app_state` row `active_library_id`; `library_recent` entries get no special ordering column
   beyond `last_opened_at`, which the frontend can sort by if it wants an MRU list.
3. This step does **not** open or migrate any individual library's DB — it only transcribes the
   registry. Each library's own data migrates lazily, per below, the first time that library is
   actually opened.
4. Leave the old `confy` file in place until §12's `confy` removal step; don't delete it as part of
   this migration in case the new version needs to be rolled back.

### Migration: Old Manifest/Cache → Library DB (Lazy, On Open)

This is the "migrate when open" requirement — not a batch job over every registered library, since
a user may have libraries registered that they haven't touched in months and there's no reason to
pay that migration cost for libraries nobody is using.

Triggered inside `activate_library` (and `create_library`'s open-existing-and-valid branch, §7):

1. If `library_root/library.db` already exists, open it — no migration needed, already on the new format.
2. If it doesn't exist but `library_root/manifest.toml` does, this is a pre-migration library:
   read the manifest (`Library::read_library_manifest`, today's `Toml::read::<LibraryDTO>`) and
   `cache.toml`, create a fresh `library.db`, and populate `library_meta`/`mods`/`cache_entries`
   from what was read. `tools` starts empty — there's no legacy tool data, the registry is new.
3. Once populated and verified readable, leave `manifest.toml`/`cache.toml` in place for this
   release (don't delete on migrate — same rollback-safety reasoning as step 4 above) but stop
   writing to them; `library.db` is authoritative from this point forward for that library.
4. If neither `library.db` nor `manifest.toml` exists, this is a genuinely new library —
   `create_library`'s normal path creates `library.db` directly, no migration involved.

### Assembling `get_library_workspace`

Per the contract §7 rationale note: building `LibraryWorkspace` means reading the App Config's
`known_libraries`, then opening **every** known library's DB (running the lazy migration above for
any that haven't been touched yet) to fill in `modsByLibraryId`/`toolsByLibraryId` and derive each
`LibrarySummary`'s `cacheStatus` (`modCount`/`enabledModCount` aren't part of the DTO at all — the
contract has the frontend compute those from `modsByLibraryId`, so there's nothing to derive or
mirror for them here). This is a deliberate tradeoff, not an oversight: it keeps `cacheStatus` in
exactly one place (no mirrored copy in the App Config to go stale), at the cost of opening N small
SQLite files on workspace load. Acceptable for the realistic library counts this app deals with
(single digits to low tens); if that stops being true, the fallback is a denormalized summary cache
in the App Config, refreshed on write — don't build that preemptively.

### Rollout Sequence

1. Add `store/` (both the App Config path and the Library DB path) alongside the existing `confy`/manifest-based persistence — additive, not a cutover. `1.4_tests.md`'s oracle must stay green through this step.
2. Land the App Config migration (above) first — it's the smaller, lower-risk piece and unblocks assigning every library a stable ID, which `activate_library`/`rename_library`/etc. need regardless of when each library's own DB migrates.
3. Land the tool registry against Library DB next (§9) — net-new, no legacy data, good proving ground for the per-library-DB pattern before the higher-stakes mods/cache migration.
4. Land the lazy manifest/cache migration (above). Each library gets its own test coverage (a fixture with a pre-migration `manifest.toml`/`cache.toml`, asserting the post-migration `library.db` reads back the same `LibraryDTO`-equivalent state) before cutover.
5. Once a domain's `store` path is verified equivalent to the legacy path for a given library (same test oracle, green), that library's commands read/write through `library.db` only — don't leave both paths live for the same library past this step, that's a dual-source-of-truth bug waiting to happen.
6. After rollout, revisit `confy` and the old manifest/cache read paths (flagged in `1.2_dependencies.md` as droppable) for removal — once nothing reads them for any library, not before.

## 8a. Non-Blocking Operations and the Per-Library Operation Guard

Three operations are fire-and-track per the contract §6: `install_mod_archives`, `bulk_update_mods`,
`rebuild_library_cache`. This section is the backend half of that — how they avoid blocking each
other, and the answer to the specific question of whether `rebuild_library_cache` should hold the
Library DB's write lock for the whole operation.

### Short answer: no, don't hold the DB lock for the slow part

Holding a lock (DB-level or an in-process mutex) for the full duration of a rebuild — which scans
the entire game root and mod folder, potentially slow — would make "non-blocking" a lie: a user
toggling one mod while a rebuild is running would hang until the scan finishes. The fix is to split
the operation instead of locking around it:

1. **Scan phase touches no database state at all.** Walking the game root/mod folders and hashing
   files (the slow part) is pure filesystem read + in-memory compute. It can't block or conflict
   with anything else because it doesn't touch `library.db`.
2. **Commit phase is one short transaction.** Once the scan produces a new index, a single fast
   write transaction replaces `cache_entries` and reconciles `mods` (new mods found, mods no longer
   present). This is the only part that needs the DB write lock, and it's brief by construction —
   SQLite serializes writers regardless, so this is also the only point where a concurrent
   `bulk_update_mods` write might have to wait a moment, not the whole scan duration.
3. **Reconcile by upsert, not blind replace.** The commit step upserts by mod ID rather than
   `DELETE FROM mods` + reinsert — this is defense-in-depth (see the guard below for why the race
   this protects against shouldn't happen anyway), and it also means a rebuild that's interrupted
   mid-commit (app crash) leaves a partially-updated table instead of an emptied one on restart.

`install_mod_archives` follows the same shape: extracting/staging each archive (slow I/O) doesn't
touch the DB; each mod's own row is written in its own short transaction as soon as that mod is
ready, which is also what makes the partial-failure model (contract §6) natural — a failure on
archive 3 of 5 doesn't roll back or block archives 1, 2, 4, 5.

### The actual answer to "what stops two operations from racing on the same library"

Not the DB lock — a lightweight **per-library operation guard**, held in memory (e.g. on
`AppRegistry`/the `Library` instance, an `AtomicBool` or a `TryLock`, not a database row):

- `install_mod_archives`, `bulk_update_mods`, and `rebuild_library_cache` each try to acquire the
  guard for their `libraryId` before doing anything else. If it's already held (another one of
  these three is in flight for that library), the call fails immediately with a new `SError`
  variant — `LibraryOperationInProgress(LibraryId)` — rather than queuing, silently no-opping, or
  racing. This is the backend-side half of "the UI should prevent the user from starting a
  redundant action, but the backend still needs to handle it": the guard is what makes that true
  even if the UI's disabled-button state is stale, double-clicked through, or bypassed by a second
  window.
- The guard is released when the operation's background work finishes (success or failure) — right
  before emitting the completion event.
- The guard does **not** block `get_library_workspace` or any other read of that library, and does
  **not** affect any other library — it only serializes these three background-job types against
  each other, per library.
- Because of this guard, a rebuild's scan phase can never actually overlap a concurrent mod
  install/toggle/delete for the *same* library in a correctly-running system — the upsert-not-
  replace reconciliation above is the backstop for a crash or bug, not the primary correctness
  mechanism. Don't skip the guard on the theory that upsert semantics alone are enough.

New `SError` variant for this (add to §11's growing list): `LibraryOperationInProgress(LibraryId)`.

## 9. Executable Tool Registry

- `commands/tool.rs`: `upsert_tool`, `delete_tool`, `execute_tool` — interface layer only.
- `core/tool_service.rs`: orchestrates `store/library_store/tools.rs` (that library's own DB — a tool belongs to one library, per the contract's `toolsByLibraryId`) for config CRUD, and process spawning for `execute_tool`.
- Configuration and execution are separate service functions even though they're both reached via `commands/tool.rs` — `upsert_tool`/`delete_tool` never spawn a process; `execute_tool` never writes to `store`.
- Process spawning (`execute_tool`): use `std::process::Command`, spawned detached on Windows (per `1.9_backend-redesign-audit.md` §2.A) so closing Modkeeper doesn't kill a running game server and cause a port-binding conflict on relaunch. Returns `ToolExecutionResult { toolId, state: 'started' | 'failed', message }` — no PID/live-status tracking, deferred per contract §8.
- New `SError` variants needed for this domain (see §11): a launch failure and an executable-not-found case distinct from the generic `FileOrDirectoryNotFound`, so the frontend can show a tool-specific message.

### Icon Processing

Per the contract's read/write icon split (contract §9): `upsert_tool`'s `iconData` input is raw
image bytes, base64-encoded, as read by the frontend from whatever local file the user browsed or
typed a path to. The backend, not the frontend, is responsible for turning that into the
`iconDataUrl` that reads return. `core/tool_service.rs`'s `upsert_tool` path:

1. Base64-decode `iconData`.
2. Validate it's a real, supported image (reject anything else with a dedicated error — see below — rather than silently storing garbage that fails to render later).
3. Normalize it (at minimum, cap dimensions/file size so a user-selected 4K icon doesn't bloat `library.db` — exact limits are an implementation detail, not specified here).
4. Produce the stored representation and write it to `tools.icon_data_url` (contract §7) — a `data:` URL is the simplest choice (self-contained, no separate asset-serving path needed), but this is a backend implementation detail per the contract; the frontend must keep treating `iconDataUrl` as opaque either way.

Step 2 needs an image-decoding dependency — the codebase has none today (`1.2_dependencies.md`
audited `Cargo.toml` before this requirement existed). Add the `image` crate for decode/validate/
resize; flag this as a recommendation to confirm before implementation, not a hard requirement — a
narrower hand-rolled magic-byte check would also satisfy step 2 if pulling in `image` is undesirable.

New `SError` variant for this: `InvalidToolIcon(String)` → `tool.invalid_icon` (add to the §11 table).

## 9a. Backup Becomes an Internal Upgrade-Safety Mechanism

Resolves the open question from the first draft of this document. Mod identity is still
filename/folder-hash based (`1.7_mod-manifest-removal.md`), so "upgrading" a mod is still "the
folder for this mod ID already exists and we're about to overwrite it" — the same case
`core/mod_manager.rs::add_mod` already detects today (`if dst.exists() { mod_backup::create_backup(...) }`).
What changes is what happens to that backup afterward, and who can see it.

**Decision:** there is no user-facing backup feature in the redesign. It existed to back the old
mod detail page's backup history list and manual restore (`get_backups`/`restore_backup`). That
page doesn't exist in the redesign (`frontend-redesign-spec.md` §2, mod detail route is out of
scope), so there's nothing for it to serve. What's still needed — and still worth keeping — is the
safety property: an upgrade that fails partway shouldn't leave a mod half-overwritten or gone.

New shape for `core/mod_backup.rs` (name can stay or become `core/mod_snapshot.rs` — pick one, but
the current multi-backup, named, timestamped, manifest-per-backup model in `list_backups` /
`ModBackup` / `BackupManifest` goes away, since there's no browsing UI left to justify it):

1. Before overwriting an existing mod's folder, take one transient snapshot (still needs the config-file copy `create_backup` already does — that part of the current logic is fine, just drop the manifest/naming since there's only ever at most one snapshot in flight per mod).
2. If the overwrite (extract + copy) succeeds, discard the snapshot immediately — this is the "removed after upgrade" behavior. Don't wait for `remove_mod` to clean it up; that was the old accumulate-until-full-removal behavior and it's what "dropped the full backup system for simplicity" is replacing.
3. If the overwrite fails partway, restore from the snapshot (reusing the existing `restore_backup` copy-back logic) before returning the error, so a failed upgrade leaves the mod exactly as it was.
4. `remove_mod` keeps a defensive cleanup of any leftover snapshot directory for that mod ID (covers the crash-mid-upgrade case where step 2/3 never ran), but this is a backstop, not the primary cleanup path anymore.

None of this is reachable from `commands/` — no `commands/backup.rs`, no rows in the endpoint
table, no new `SError` variants for it beyond what an upgrade failure already needs (`IOError`,
etc. from §11). It's plumbing inside `library_service::install_mods` (the extraction target for
`add_mods`'s current inline logic, §5), not a feature.

`models/mod_backup.rs`'s `ModBackup`/`BackupManifest` types shrink or disappear along with this —
they only exist today to serialize `get_backups`' response, which no longer exists.

## 10. Comment Removal

Batch pass, after §4–§9 land, not before (per Execution Method item 3 / `outline-of-redesign.md`
Phase 2.7) — comments in code that's about to move or be deleted are wasted triage effort.

## 11. Error Contract: `SError`'s Wire Shape

Earlier drafts of this document introduced a separate `AppError` struct as the IPC-boundary error
type, converted from `SError` at the command layer. That's reversed here: **`SError` stays exactly
as it is — same variants, same payloads, same `Display` impls, same `impl_from!` conversions — and
commands return `Result<T, SError>` directly, exactly as they do today.** No new Rust type, no
conversion step, no second name for the same concept. If `SError` isn't changing shape, it
shouldn't get a second name at the contract level.

### The One Change: Stable Serialization

Today `SError` derives `Serialize` with no tagging configuration, so serde's default external
tagging applies — unit variants serialize as a bare string (`"Unexpected"`), variants with data as a
single-key object (`{"ModNotFound": "abc123"}`, or an array for multi-field tuple variants like
`InvalidLibrary`). That's usable but inconsistent shape-to-shape. Add one attribute:

```rust
#[derive(Type, Serialize, Deserialize, Debug, Display)]
#[serde(tag = "code", content = "data")]
pub enum SError {
    // ...unchanged...
}
```

This normalizes every variant to the same two-field shape: `{ code: "ModNotFound", data: "abc123" }`
for variants with a payload, `{ code: "Unexpected" }` (no `data`) for unit variants. `code` is the
variant name verbatim — there is no separate mapping table to maintain, and no risk of it drifting
from the enum, because it's generated from the enum by the attribute, not hand-assigned. New
variants needed elsewhere in this document (`InvalidToolIcon` for §9, `StoreError` for §8,
`LibraryOperationInProgress` for §8a) get a
`code` for free the moment they're added — nothing to remember to update here.

### `message` Never Crosses the Wire

This is the actual point of the redesign, not just the naming fix: **the frontend never receives an
English message string, only `code` and `data`.** `SError`'s `Display` impl (already implemented,
already human-readable) is not part of the serialized shape — it's used exactly once, at the point
a command is about to return `Err`, to write an English log line for developers:

```rust
if let Err(ref e) = result {
    tracing::error!("{e}"); // ties into the Phase 4 logging fix (L-004) — this is
                            // the "sparse structured logging in core/" gap, closed here.
}
result
```

Whether the log file exists yet or this falls back to console-only (Phase 4, `1.3_logging.md`
L-003) doesn't change this rule: the message is logged, in English, for developers, regardless of
whether it was also returned to the frontend. It is never returned to the frontend.

### What the Frontend Does With `code`/`data`

Not this document's concern beyond stating the boundary — `frontend-redesign-spec.md` §10 owns the
mechanism — but the shape of the deal is: `code` is a lookup key into an i18n string table (falling
back to a generic translated "something went wrong" for any `code` without a mapped string yet),
and `data` may feed template interpolation (e.g. `FileCollision`'s file list rendered inside an
already-translated sentence) without ever surfacing the raw English `Display` text.

## 12. Dependency Changes

- Remove `help` (`Cargo.toml`) — confirmed unused, `1.2_dependencies.md`.
- Remove `radix-ui` (frontend `package.json`) — confirmed unused, `1.2_dependencies.md`. Not this file's concern to execute, but recorded here since it's part of the same audit-driven cleanup.
- Add `rusqlite` (`bundled` feature) and `rusqlite_migration` for the Library DB (§8). `toml` is already a dependency and needs no addition for the App Config.
- Add `image` (or a narrower hand-rolled alternative — see §9) for tool icon validation/normalization.
- Drop `confy` (§8 Crate Choice) once §8's Rollout Sequence step 6 confirms nothing reads it anymore — the decision is made, this is just sequencing the removal, not re-litigating whether to.

## 13. Implementation Sequence

1. Lock the test oracle (Execution Method item 1); land the `1.7` manifest-removal behavior change first if not already done.
2. FP refactor: `core/mod_fs.rs`, `utils/file.rs`, `utils/toml.rs`, `utils/process.rs` (§4), mechanically then refined.
3. Low-level FS separation: relocate `core/decompression.rs`'s `extract` (§6).
4. Fix the known `add_mods`/`remove_mods` boundary violation (§5) — this both closes the flagged violation and becomes the template for the endpoint rework in step 5.
5. Add `store/` additively — both the App Config path and the Library DB path (§8 Rollout Sequence step 1).
6. Land the App Config migration (§8 Rollout Sequence step 2): `confy` → `known_libraries`/`app_state`, stable IDs assigned.
7. Add the tool registry against Library DB (§9, §8 Rollout Sequence step 3) — no legacy data, proves the per-library-DB pattern.
8. Land the lazy manifest/cache → Library DB migration and cut library/mod state over (§8 Rollout Sequence steps 4–5).
9. Endpoint rework (§7): rename, regroup into `commands/library.rs` / `commands/global.rs` / `commands/tool.rs`, add the `#[serde(tag = "code", content = "data")]` attribute to `SError` (§11).
10. Implement the per-library operation guard and the scan/commit split for `rebuild_library_cache`, `install_mod_archives`, `bulk_update_mods` (§8a); wire each to emit its completion event (`cache_rebuild_completed`, `mod_install_completed`, `bulk_update_completed`) when the guard releases.
11. Comment removal batch pass (§10).
12. Dependency cleanup (§12), including §8 Rollout Sequence step 6 (`confy`/legacy manifest-cache read-path removal).
13. Run `cargo run --bin export_types` and hand off to `frontend-redesign-spec.md` §12 step 13.

## 14. Verification Plan

- `cargo build`, `cargo clippy` clean throughout, not just at the end.
- `tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs` green after every step in §13, per Execution Method item 1.
- Per `1.4_tests.md`: refactor `tests/library.rs`/`tests/mod_fs.rs` manifest-dependent cases to hash-only assertions (already scoped by `1.7`); remove `test_collect_files_excludes_manifest`.
- Add test coverage for the App Config migration: a fixture `confy` `GlobalConfig` with `library_last`/`library_recent` migrates to the expected `known_libraries`/`app_state` entries, with stable IDs assigned (§8).
- Add test coverage for the lazy Library DB migration: a fixture library with `manifest.toml`/`cache.toml` and no `library.db` produces a `library.db` on `activate_library` that reads back equivalent `LibraryDTO` state (§8) — and a second fixture library that's never activated must NOT get a `library.db` (proves the migration is actually lazy, not eager-at-startup).
- Add test coverage for `execute_tool`'s detached-spawn behavior on Windows (`1.9_backend-redesign-audit.md` §4, Windows Verification).
- Add test coverage for §9a's upgrade-safety mechanism: a failed overwrite restores the prior mod contents, and a successful overwrite leaves no snapshot directory behind. `tests/library.rs` already has mod-add coverage to extend rather than a new test file to create.
- Add test coverage for the per-library operation guard (§8a): calling `rebuild_library_cache` (or `install_mod_archives`/`bulk_update_mods`) on a library while one of the three is already in flight for that same library returns `LibraryOperationInProgress`, not a queued/silent success. Calling any of the three on a *different* library concurrently must succeed normally — the guard is per-library, not global.
- Add test coverage that `get_library_workspace` (a read) is not blocked while a `rebuild_library_cache` is in flight for that library (§8a) — the whole point of the scan/commit split.
- `cargo run --bin export_types` succeeds and the generated bindings match `frontend-redesign-data-api-contract.md` — treat a mismatch as a spec bug in whichever of the two documents is wrong, not something to paper over in generated code.

## 15. Acceptance Criteria

- No file in `core/` calls `std::fs`/`tokio::fs` directly (§6 exit criterion).
- `ModFS`, `FileUtils`, `Toml`, `ProcessChecker` no longer exist as structs; their logic is free functions.
- `commands/library.rs::add_mods`/`remove_mods` no longer inline service-layer orchestration.
- Every command in `commands/` matches a row in the §7 mapping table — no undocumented commands.
- The App Config never contains a library's own data (name, mods, tools, cache), and no Library DB contains another library's data or the App Config's registry/settings — each fact lives in exactly one file (§8).
- `rebuild_library_cache`'s scan phase never touches `library.db` — only its final commit does, and that commit is a short, single transaction using upsert-by-id, not a blind delete-then-insert (§8a).
- `install_mod_archives`, `bulk_update_mods`, and `rebuild_library_cache` each acquire and release a per-library operation guard around their background work; overlapping calls on the same library are rejected with `LibraryOperationInProgress`, not silently interleaved (§8a).
- `store/` is the only writer of library/mod/tool/settings state; no dual-source-of-truth window remains after a given library's §8 Rollout Sequence step 5 for that library.
- A library that is registered (`known_libraries` row exists) but never activated has no `library.db` on disk — migration is lazy, not eager (§8).
- `commands/tool.rs`'s `upsert_tool`/`delete_tool` never spawn a process; `execute_tool` never writes to `store`.
- All command outputs are `Result<T, SError>`, unchanged from today's signatures. `SError` has `#[serde(tag = "code", content = "data")]` so every variant serializes to a stable `{code, data}` shape.
- No command path sends `SError`'s `Display`-formatted message (or any other English prose) to the frontend — it's logged via `tracing::error!` at the point of return, not returned.
- No `get_backups`/`create_backup`/`restore_backup`/`remove_backup` commands exist; backup is unreachable from `commands/` (§9a).
- An upgrade that fails partway through leaves the mod exactly as it was before the upgrade started (§9a, restore-on-failure).
- A successful upgrade leaves no leftover snapshot on disk (§9a, discard-on-success) — verified by a test, not just by reading the code.
- Backend test oracle is green at every step, not just the final one.
- `cargo build`, `cargo clippy`, `cargo test` all pass clean.
