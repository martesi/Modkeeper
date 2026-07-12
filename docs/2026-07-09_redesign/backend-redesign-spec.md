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
didn't exist in the 2026-05-10 scope: the plain-file App Config layer (stable library registry,
app state, backend-owned settings — the per-library SQLite Library DB originally planned alongside
it is cancelled, see `design-review.md` C4), the executable tool registry, and an endpoint
contract that actually matches what `frontend-redesign-data-api-contract.md` expects to call.

## 2. Refactor Scope

### In Scope

- FP-by-default refactor of `core/mod_fs.rs` and the `utils/*` "struct with static methods" files.
- Low-level FS extraction from `mod_fs.rs` and `core/decompression.rs` into `utils/file.rs`.
- Core/service/interface boundary enforcement across `commands/` and `core/*_service.rs`.
- `verb_noun` endpoint contract, matching `frontend-redesign-data-api-contract.md` exactly.
- Plain-file `store/` module for the App Config (library registry, app state, settings). Per-library state stays in that library's own plain files (`manifest.toml`/`cache.toml`, plus the new `tools.toml` — §8).
- Executable tool registry: data model, store, service, commands, process spawning.
- Collapse the user-facing backup/restore feature into an internal upgrade-safety mechanism — see §9a.
- Comment removal, as a batch pass after structural moves settle.
- Dependency cleanup: remove `help`; evaluate `confy` removal once the App Config migration lands.

### Out of Scope

- Changing on-disk mod/game content layout or the deployment/linking algorithm itself (`core/deployment.rs`, `core/linker.rs`) — only their call boundaries move, not their logic.
- The sync-pipeline redefinition in `1.9_backend-redesign-audit.md` §2.B (remove the manual Sync button, commit on toggle) is **reversed, not implemented** (`design-review.md` C3/M5): toggles persist *mod state* only (cheap, always committed); deployment stays a distinct, explicit, user-triggered `sync_mods` step, and the manual Sync button stays — highlighted when the deployed state is stale (see §7's `sync_mods` row and the contract's `deployStale` field).

## 3. Execution Method

This restates `purpose-of-redesign.md`'s "Backend: Mechanical Refactor In Place" at
implementation granularity. If the two disagree, `purpose-of-redesign.md` wins.

**Recorded exception to that precedence line:** `purpose-of-redesign.md`'s "Adopt SQLite as the
Library State and Index Layer" section is deliberately **not** implemented by this spec — the
SQLite Library DB migration is cancelled per `design-review.md` C4 (see there for the full
rationale). The purpose doc is intentionally left as written and reads, for that section only, as
the historical case for the proposal rather than a live requirement. This is a deliberate,
called-out deviation, not an oversight or a silent contradiction.

1. **Lock the oracle.** `tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs` must be green before any file in §5–§9 is touched. Where `1.7_mod-manifest-removal.md`'s behavior change (hash-only mod ID) hasn't landed yet, land it first, as its own commit, so the oracle reflects the actually-intended behavior before structural work starts.
2. **Port mechanically, then refine.** Moving `ModFS`'s methods to free functions (§4) is two separate commits: (a) move the code verbatim into its new shape, get it compiling and green; (b) clean up naming/structure. Don't do both in one commit — a red test after a combined commit doesn't tell you whether the move or the redesign broke it.
3. **Batch mechanical violations.** For the endpoint audit (§7) and comment removal (§10), produce the full violation list first, then fix in grouped passes by category, not one-by-one as noticed.
4. **Review adversarially.** Each of §5 (boundary enforcement), §7 (endpoint contract), and §8 (SQLite migration) gets an independent pass — someone other than whoever wrote the change — specifically checking for a boundary violation, a silently changed behavior, or a test that was weakened to pass rather than fixed.

## 4. FP/OOP Decision Table

Finalized from `1.1_codebase-structure.md` and `1.5_justification.md`, plus classification for
the two files that postdate that audit.

| File | Decision | Notes |
|------|----------|-------|
| `core/library.rs` | OOP (justified) | Encapsulates dirty-flag, cache, paths. Forcing FP here means threading every field through every function call — rejected per `1.5_justification.md`. Re-affirmed by `design-review.md` C1/C3/C4: the dirty flag survives (deployment stays an explicit step) and the cache/mods map stay in-memory (SQLite cancelled), so the justification still holds post-Phase 2. |
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
| `utils/icon.rs` | **Refactor — the input contract changes** | Loses its only current caller (mod icons die with `iconDataUrl`, `design-review.md` C6), but §9's tool-icon flow still needs the base64/data-URI capability. Not a shape-preserving refactor: today it's path-in → extension-sniffed MIME → data-URI out; §9's flow is base64 *bytes* in (read by the frontend, never a backend file read) → content-validated (`image` crate decode) → resized/size-capped → stored representation. MIME detection moves from extension-matching to content-sniffing (`design-review.md` C10/M10). |
| `utils/id.rs` | Keep, unchanged | `hash_id` (blake3 + base64url) is the hash-based mod identification mechanism `1.7` standardizes on. |
| `utils/time.rs` | Keep, unchanged | Single trivial helper (`get_unix_timestamp`). |
| `utils/thread.rs` | Keep, unchanged | `with_lib_arc`/`with_lib_arc_mut` are the kept lock model (§8a, `design-review.md` C1). The boilerplate-reduction idea from the 1.9 audit stays open as a later helper-ergonomics change, not a deletion. |
| `core/mod_documentation.rs` | **Delete** | `read_documentation` reads `manifest.documentation`, deleted entirely by `1.7` — nothing left for it to read. |
| `commands/test.rs` | **Delete** | `create_simulation_game_root` is deleted outright (not cfg-gated), along with the `pub mod test;` export, the `collect_commands!` entry, and `models/test.rs` (`design-review.md` C9). See the §7 row. |
| `models/*` | Classified individually | `models/mod_dto.rs` shrinks under `1.7` (manifest-derived fields drop, plus `icon_data` per C6); `models/error.rs` gains `TaskIdInUse` (§8a) and `ConfigSaveFailed` (§8); `models/global.rs`/`models/config.rs` evolve with the confy→toml migration (§8); `models/test.rs` deletes per C9; `models/library.rs`, `models/paths.rs` unchanged; `models/mod_backup.rs` shrinks per §9a. |

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
| `init` | `commands/global.rs` | `get_library_workspace` | `commands/global.rs` | Returns full `LibraryWorkspace`, not `LibrarySwitch`. **Must carry over `init`'s `state.init_called.store(true, ...)` call (the `lib.rs` startup watchdog) into this command's body** — the rename must not orphan it (`design-review.md` C11). Assembly is read-only (§8): a registered library whose manifest can't be read appears as a path-only stub, never dropped and never eagerly opened. |
| `open_library` | `commands/global.rs` | `activate_library` | `commands/global.rs` | Takes `libraryId`, not a path. Looks up `library_root` from the App Config's `known_libraries`, opens that library (`Library::load` against its plain files, §8), then writes `active_library_id` to the App Config's `app_state` section. Activation is where open-time errors surface: `UnsupportedSPTVersion` stays a hard-fail here, and an unreadable manifest/cache maps to `InvalidLibrary` (`design-review.md` C5). |
| `create_library` | `commands/global.rs` | `create_library` | `commands/global.rs` | Add optional `libraryRoot` input per contract §5, item 5; derive via existing `derive_library_root` (`core/library_service.rs`) when omitted. Writes a new `known_libraries` entry (App Config) *and* creates that library's own files (`manifest.toml` etc., §8) — both, since a registered library with no files or library files with no registry entry are both invalid states. Adding a directory that already contains a readable manifest adopts the `id` found inside it, never mints a new one (`design-review.md` C7). |
| `close_library` | `commands/global.rs` | *(removed)* | — | No longer a distinct concept once `activeLibraryId` lives in the App Config's `app_state` section — "closing" is just clearing that key. Confirm with product before deleting; if kept, it isn't part of the new contract, so it would be a non-contract escape hatch. |
| `remove_library` | `commands/global.rs` | `delete_library` | `commands/library.rs` | Splits into `deleteFiles: true` (this command's old behavior — deletes the `known_libraries` row *and* the `library_root` directory, manifest/cache/tools files included) vs. `deleteFiles: false` (new — entry-only, deletes only the `known_libraries` row, leaves `library_root` and everything in it on disk so re-adding the same `gameRoot` later picks it back up — under the `id` still sitting in its manifest, per `design-review.md` C7). These must be two distinct code paths in the service, not a shared function with a flag that's easy to get backwards given the destructive stakes. |
| `rename_library` | `commands/library.rs` | `rename_library` | `commands/library.rs` | Writes to that library's own `manifest.toml` `name` — the App Config's `known_libraries` never stores a name (§8), so there's no second place to update. Drop the optional-library-id dual-path logic noted in `1.6_api-design.md` §5 now that IDs are non-optional. |
| `rebuild_library_cache` | `commands/library.rs` | `rebuild_library_cache` | `commands/library.rs` | Fire-and-track: takes a client-minted `taskId`, validates it targets the active library, registers the task (§8a), returns `OperationAccepted`; emits `cache_rebuild_completed` (carrying the `taskId`) on completion. Runs under the active-library mutex for its full duration (§8a). Folder normalization renames only — the `remove_all_backups` coupling is removed, and rebuild does **not** re-link or deploy; a rename dangles all of that mod's deployed links until the user runs `sync_mods` (`design-review.md` C8). |
| `get_library` | `commands/library.rs` | *(folded into `get_library_workspace`)* | — | Per-mod-list data becomes part of the workspace response. |
| `add_mods` | `commands/library.rs` | `install_mod_archives` | `commands/library.rs` | Fire-and-track (contract §6): takes a client-minted `taskId`, validates active-library scope (§8a), returns `OperationAccepted`; emits `mod_install_completed` (carrying the `taskId`) with per-archive failures on completion — service must collect per-archive errors instead of failing the whole call on the first `SError`. Upgrading an existing mod (destination folder already exists) takes a transient pre-overwrite snapshot and discards or restores it per §9a — invisible to the frontend either way. |
| `remove_mods` | `commands/library.rs` | `bulk_update_mods` (`action: 'delete'`) | `commands/library.rs` | Fire-and-track, see `toggle_mod` row below — same command, same task machinery (§8a). |
| `toggle_mod` | `commands/library.rs` | `bulk_update_mods` (`action: 'enable'\|'disable'`, one-element `modIds`) | `commands/library.rs` | No separate `set_mod_enabled` — matches the contract's resolved item 3. Fire-and-track (contract §6): takes a client-minted `taskId`, returns `OperationAccepted`, emits `bulk_update_completed` (carrying the `taskId`) with per-mod failures on completion. Commits *mod state* immediately (flip `is_active`, `mark_dirty()`, `persist()` — metadata-only, cheap, never lost); it does **not** deploy — no symlinks are touched, no collision check runs; that's the explicit `sync_mods` step below (`design-review.md` C3). Stays fire-and-track for all three actions even though enable/disable is now cheap: one predictable client-side handling path (consistency, not per-action cost — `delete` still unlinks and removes files). |
| `sync_mods` | `commands/library.rs` | `sync_mods` | `commands/library.rs` | **Kept** — the explicit, user-triggered deploy step (`design-review.md` C3/M5; reverses the earlier §2.B acceptance, see §2 Out of Scope). Fire-and-track (the fourth operation): takes a client-minted `taskId`, validates active-library scope, returns `OperationAccepted`, emits `sync_completed` on completion. Wraps `Library::sync` (purge → full redeploy → collision check → mark clean); same cost profile as a rebuild, which is why it qualifies for fire-and-track. No `GameOrServerRunning` guard — deliberately dropped as an accepted risk (`design-review.md` C2). Frontend surface: the Sync button on the execution bar, highlighted when `LibrarySummary.deployStale` is true. |
| `reveal_mod` | `commands/library.rs` | *(removed — frontend-side)* | — | Frontend's `app-opener.ts` calls `@tauri-apps/plugin-opener` directly (`frontend-redesign-spec.md` §7); no backend command needed for this one. |
| `get_backups`, `create_backup`, `restore_backup`, `remove_backup` | `commands/library.rs` | *(removed — no longer commands)* | — | Resolved: there is no user-facing backup feature in the redesign — it existed to support the old mod detail page's backup history/restore UI, which is gone (no detail route, per `frontend-redesign-spec.md` §2 Out of Scope). Backup becomes a purely internal upgrade-safety mechanism with no IPC surface at all. See §9a. |
| `get_mod_documentation` | `commands/library.rs` | *(removed)* | — | Per `1.7_mod-manifest-removal.md`. |
| `create_simulation_game_root` | `commands/test.rs` | *(removed — deleted outright)* | — | Was missing from earlier drafts of this table (finding 9). Deleted, not cfg-gated: `commands/test.rs`, the `pub mod test;` export in `commands.rs`, the `collect_commands!` entry in `lib.rs`, and `models/test.rs` all go, along with the old frontend panel built on it (`src/modules/settings/developer-settings.tsx` — a recorded exception to the frontend preservation rule, `design-review.md` C9). |
| `apply_window_effect` | `commands/global.rs` | `apply_window_effect` | `commands/global.rs` | Unchanged — window chrome, not workspace data, not part of the contract's domain model. |
| *(new)* | — | `upsert_tool`, `delete_tool`, `execute_tool` | `commands/tool.rs` | See §9. |
| *(new)* | — | `get_settings`, `save_settings` | `commands/global.rs` | Backed by the App Config's `settings` section (§8), not `confy`, once migrated. Settings are backend-owned; mutating commands return the **full** settings object and the frontend replaces its atom wholesale — full-replacement, never a client-side merge (`design-review.md` T1). |
| *(new — not a command)* | — | `listen_workspace_event` | n/a | This is a frontend-side wrapper (`frontend-redesign-spec.md`/contract §6) around the standard Tauri event API, not a `#[tauri::command]`. Backend side is `app_handle.emit(...)` calls from `rebuild_library_cache`, `install_mod_archives`, `bulk_update_mods`, and `sync_mods` when each finishes (§8a) — exactly the four fire-and-track operations, no others. Every completion event carries the operation's `taskId` (§8a). No new specta command needed for this row. |

## 8. Library State (Plain Files) and App Config (Plain File)

Two storage tiers, neither of them SQLite. Earlier revisions of this section specified a
per-library SQLite `library.db` replacing `manifest.toml`/`cache.toml`; that migration is
**cancelled** (`design-review.md` C4): the access patterns don't need a query engine, the
in-process lock model §8a keeps already serializes writers, and the costs were concrete — a new
dependency, migration mechanics to write and test, an N-file-open cost at every workspace
assembly, and a real loss of text-editor transparency, cutting against "a library's data travels
with the library." App-level state (which libraries are known, which is active, app settings)
lives in one plain App Config file; each library's own state stays in its own plain files —
`manifest.toml`, `cache.toml`, and the new `tools.toml` for the tool registry (§9). The shape for
both tiers is in `frontend-redesign-data-api-contract.md` §7; this section covers the Rust-side
module layout, the confy-vs-toml decision, the write-path fix, and the migration mechanics.

### Module Layout

New top-level module, peer to `core/`, `commands/`, `utils/`, `models/` — App Config only:

```text
src-tauri/src/store/
  mod.rs                -- loads/saves the App Config
  app_config.rs          -- App Config: known_libraries, app_state, settings (plain TOML file)
```

`store/*` is the low-level tier for app-level persistence, the same role `utils/file.rs` plays for
the filesystem: plain functions, no orchestration. `app_config.rs` functions take a handle to the
single, in-memory `AppConfig` struct (loaded once at startup, saved on write — see Crate Choice
below; no "connection" concept, it's a file, not a database). Per-library persistence stays where
it lives today: `core/library.rs` reads/writes that library's `manifest.toml`/`cache.toml` via
`utils/toml.rs`, and the new `core/tool_service.rs` reads/writes `tools.toml` the same way (§9) —
there is no `library_store/` module. Services (`library_service.rs`, new `tool_service.rs`) call
into `store/*` for app-level state, never the other way around, and `commands/` never touches
`store/` directly (§5).

### File Locations

- **App Config:** the app's own data directory (already reachable via the existing `directories`
  crate dependency — same place `confy`'s `GlobalConfig` lives today), e.g.
  `app_data_dir/config.toml`. Loaded once at startup, held in memory, written back on change
  (atomically — see Write Path below).
- **Library files:** inside that library's own mod-manager directory — `manifest.toml` and
  `cache.toml` exactly where they are today, plus the new `tools.toml` as their sibling, next to
  the existing `mods/`, `backups/`, `staging/` directories (`LibPathRules`, `models/paths.rs`).
  `library_root` defaults to `game_root/.mod_keeper` (`derive_library_root`,
  `core/library_service.rs`) but can be overridden — the files always live wherever that library's
  `library_root` actually resolves to, not at a fixed path.

### Crate Choice

**No database dependency.** The `rusqlite`/`rusqlite_migration` additions from earlier drafts are
cancelled with the Library DB itself (`design-review.md` C4). Per-library files keep using
`utils/toml.rs` (post-§4, free functions).

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

### Write Path: Atomic and Surfaced

The confy critique above covers the read side; the write side is currently worse:
`GlobalConfig::save` is `let _ = confy::store(...)` (`config/global.rs:26-28`) — any write failure
(disk full, permissions, path gone) is silently discarded, and `save()` returns `()` so no call
site can know. Once this file carries stable library ids (C7) and settings (T1), a silently lost
write means losing registry entries. The fix (`design-review.md` C12/M7):

1. New `SError` variant `ConfigSaveFailed(String)` — distinct, so the frontend can show "your
   settings/library list may not have saved" specifically (§11 table).
2. `pub fn save(&self)` becomes `pub fn save(&self) -> Result<(), SError>`. Audit and update every
   call site (`library_service.rs`, `global_service.rs`, any command calling `config.save()`) to
   propagate the error into the command's `Result<_, SError>` return. **Including**
   `lib.rs::load_initial_library`'s `config.save()` (`lib.rs:107`), which runs on the startup
   background thread before any command exists to propagate through — its handling: log the
   failure at the call site, and surface it as a toast on the next successful init via a small
   pending-warning flag the frontend reads once from the first `get_library_workspace` response.
   Don't block or fail startup over a config write that has a defaults fallback.
3. The write is atomic: serialize to a temp file in the same directory, then `std::fs::rename`
   over the real path (atomic for same-volume renames on both Windows and Linux) — a crash
   mid-write never leaves a half-written `config.toml`. Implemented as a **free function**
   `pub fn atomic_write(...)` in `utils/file.rs` (not a `FileUtils::` method — §4 dissolves that
   struct), so `manifest.toml`/`cache.toml`/`tools.toml` writes can reuse it if the same
   crash-safety is wanted there.
4. This lands as part of the confy→toml migration below, not as a separate pass — explicit
   serialization control and explicit write control are needed together.
5. Read-path behavior on parse failure (log + start fresh vs. hand-written migration) stays the
   open choice already stated under Crate Choice; this subsection settles only the write side.

### Migration: Old Config → App Config

Runs once, at startup, before anything else touches `store/`:

1. On first launch of the new version, if `known_libraries` is empty and a `confy` `GlobalConfig`
   file exists, read it (`library_last`, `library_recent` — both plain paths, no IDs today).
2. For each path, **read the `id` already persisted in that library's `manifest.toml`**
   (`Library::create` mints and persists it, `core/library.rs:50`) and insert a `known_libraries`
   row `{ id, library_root: path, last_opened_at }`. Mint a new `LibraryId` (`uuid` — already a
   dependency) **only** when no readable manifest exists at that path. The library's own manifest
   id is authoritative; the App Config's copy is a cache of it, never the origin — re-adding a
   directory that still has a readable manifest adopts the id found inside it (`design-review.md`
   C7). `library_last` becomes the `app_state` row `active_library_id`; `library_recent` entries
   get no special ordering column beyond `last_opened_at`, which the frontend can sort by if it
   wants an MRU list.
3. This step does **not** open any library in the `Library::load` sense — reading the manifest id
   is a plain TOML read, no version fetch, no game-root touch. (There is no per-library data
   migration at all anymore: `manifest.toml`/`cache.toml` remain the live format, C4.)
4. Leave the old `confy` file in place until §12's `confy` removal step; don't delete it as part of
   this migration in case the new version needs to be rolled back.

### Assembling `get_library_workspace`

Building `LibraryWorkspace` means reading the App Config's `known_libraries`, then performing a
**read-only** read of each known library's `manifest.toml`/`cache.toml`/`tools.toml` — exactly the
shape `get_known_library_summary` has today (`core/library_service.rs:129`) — to fill in
`modsByLibraryId`/`toolsByLibraryId` and derive each `LibrarySummary`'s `cacheStatus` and
`deployStale` (`deployStale` is `true` exactly when that library's persisted dirty flag is set —
`design-review.md` C3/M5; `modCount`/`enabledModCount` aren't part of the DTO at all — the
frontend computes those from `modsByLibraryId`). Assembly **never** calls `Library::load`, never
runs `version::fetch_and_validate`, and never migrates anything — it inherits no library's
failure modes. A library whose manifest can't be read (either `IOError` or `ParseError` from the
toml utilities — both, `design-review.md` C5/M8) is reported as a **path-only stub** (contract §5:
an object carrying only the registered `path`), not silently dropped (the current `filter_map`
behavior) and not a workspace-level failure. Cost: reading N small file sets per assembly —
acceptable for realistic library counts (single digits to low tens); if that stops being true, the
fallback is a denormalized summary cache in the App Config, refreshed on write — don't build that
preemptively.

### Rollout Sequence

1. Add `store/` (App Config) alongside the existing `confy` persistence — additive, not a cutover.
   `1.4_tests.md`'s oracle must stay green through this step.
2. Land the App Config migration (above) — ids adopted from each library's manifest, minted only
   where no readable manifest exists. This unblocks `activate_library`/`rename_library`/etc.,
   which need stable IDs.
3. Land `tools.toml` and the tool registry (§9) — net-new, no legacy data.
4. Cut settings over to the App Config (`get_settings`/`save_settings`, T1) once the file exists
   and the write path (above) is in.
5. After rollout, remove `confy` (§12) once nothing reads it — not before.
   `manifest.toml`/`cache.toml` were never leaving; there is no per-library cutover step.

## 8a. Non-Blocking Operations, the Kept Lock Model, and Task Tracking

Four operations are fire-and-track per the contract §6: `install_mod_archives`,
`bulk_update_mods`, `rebuild_library_cache`, and `sync_mods`. This section is the backend half of
that. Earlier drafts specified a per-library operation guard that rejected overlapping calls with
`LibraryOperationInProgress` — that design is **dropped** (`design-review.md` finding 1/C1): it
turned rapid sequential toggles, the app's most common interaction, into an error path.

### The lock model is the existing one, kept deliberately

- The `Arc<Mutex<Option<Library>>>` (`core/registry.rs:14`) and the `with_lib_arc(_mut)` pattern
  (`utils/thread.rs`) stay exactly as they are — `design-review.md` C1 rejected the
  handle-plus-job-queue replacement (§R.1/R.2). Overlapping mutating calls serialize by blocking
  on the mutex, in whatever order they acquire it, like every mutating command does today. There
  is no reject-on-overlap and no `LibraryOperationInProgress` variant.
- **Scope: fire-and-track writes operate on the active library only.** The mutex only ever holds
  the *active* library, so it structurally cannot serialize writes to a non-active one. All four
  fire-and-track commands validate `libraryId == activeLibraryId` and reject otherwise with a
  plain validation error, before touching anything. Their `libraryId` input is an assertion and
  the id completion events report against — not a key into a guard. `rename_library`'s existing
  optional-non-active-`library_id` path keeps its current, already-racy last-`persist()`-wins
  behavior — pre-existing, single-caller-at-a-time in practice, recorded as out of scope rather
  than silently inherited as a promise.
- **Ordering guarantee, stated honestly:** `parking_lot::Mutex` permits barging — it is not FIFO.
  Two overlapping calls touching the same mod resolve to whichever acquires the lock last, order
  not guaranteed to match submission order. Acceptable because `bulk_update_mods` toggles are
  absolute `is_active: bool` sets, not increments (`core/mod_manager.rs:83`) — a race's outcome
  is always one of the states the user asked for, never a corrupted third state. If a future
  change makes ordering matter (a delta-style operation), that's the point an actual queue gets
  built — not before.
- **Reads block behind in-flight active-library mutations — a stated, accepted cost.** Heavy
  operations (`install_mod_archives`, `rebuild_library_cache`, `sync_mods`) hold the mutex for
  their full filesystem duration, so `get_library_workspace` blocks for that same duration when
  called against the active library. Metadata-only operations (toggle) block reads for a
  negligible interval. This replaces earlier drafts' "the guard does not block reads" promise,
  which was unsatisfiable under the kept model — the contract's non-blocking language is scoped
  accordingly.

### Task tracking: client-minted `taskId` (`design-review.md` C15/M6)

How the frontend knows which completion event belongs to which submitted action:

- Every fire-and-track command input carries a **frontend-generated** `taskId` (uuid). The id is
  client-minted because waiting for an accept response to hand one back reopens the race — a fast
  task could complete before that round trip resolves. The frontend registers its handler in a
  single, persistent event bus (initialized once at startup, same sequence as the C11 watchdog
  handoff) *before* the invoke goes out.
- **In-flight task registry:** a `taskId → status` map lives on `AppRegistry` (the same struct
  holding the active-library mutex), inserted when a fire-and-track command is accepted, removed
  on completion. A `taskId` resubmitted while still present in the map is rejected with the new
  `SError::TaskIdInUse(String)` variant — collision is a client bug to surface, not paper over.
  This is a bookkeeping map alongside the mutex, not a job queue replacing it.
- **Two-phase signal:** the command's own invoke `Result` is the *accept* signal — synchronous
  validation failures (no active library, non-active `libraryId`, bad input, `TaskIdInUse`) reject
  immediately with no event. The completion event, matched by `taskId`, is the *outcome* signal
  for requests that passed validation and executed. All four `WorkspaceEvent` variants carry the
  `taskId`.
- **Dropped events on a stale bus are accepted.** If the frontend reloads, its dispatch map is
  wiped; a completion event for an unregistered `taskId` is silently dropped. A user who reloads
  mid-operation may see a stale view until the next read — self-correcting, and cheaper than
  reconciliation machinery for that window.

### Rebuild's shape, restated honestly (finding 8)

Rebuild is not "pure scan, then short commit" as earlier drafts claimed: `normalize_mod_folders`
(`core/cache.rs:29`) **renames mod folders on disk** during the slow phase. Under the kept model
that's safe — the whole rebuild holds the active-library mutex, so no concurrent read or write
observes a half-renamed state. Two decisions bound what rebuild does (`design-review.md` C8):

- The `remove_all_backups` call is **removed** from the normalization step
  (`core/library_service.rs:193`) — folder normalization renames the folder, removes the old
  mod-map entry, and re-keys to the resolved id, nothing else. Orphaned backups are left on disk;
  cleaning them up is a separate, explicit, user-visible action if ever wanted.
- Rebuild does **not** re-link or deploy. Deployment links at the highest uniquely-owned path
  *inside* `mods/<mod_id>/...` (one mod → several links at varying depths), so renaming the
  folder dangles **all** of that mod's deployed links at once, until the user runs the explicit
  `sync_mods` step. That is the accepted cost, stated: rebuild rewrites the library's *record*;
  recovery from any on-disk change made out from under a live deployment is
  rebuild-then-explicit-deploy.

`install_mod_archives` keeps its per-archive partial-failure model (contract §6): each archive is
extracted/staged and its mod recorded independently, so a failure on archive 3 of 5 doesn't roll
back or block archives 1, 2, 4, 5.

## 9. Executable Tool Registry

- `commands/tool.rs`: `upsert_tool`, `delete_tool`, `execute_tool` — interface layer only.
- `core/tool_service.rs`: reads/writes that library's `tools.toml` (a sibling of `manifest.toml`/`cache.toml`, §8 — a tool belongs to one library, per the contract's `toolsByLibraryId`) for config CRUD, the same way `library_service.rs` reads/writes manifest/cache; and process spawning for `execute_tool`.
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
3. Normalize it (at minimum, cap dimensions/file size so a user-selected 4K icon doesn't bloat `tools.toml` — exact limits are an implementation detail, not specified here).
4. Produce the stored representation and write it to the tool's `icon_data_url` field in `tools.toml` (contract §7) — a `data:` URL is the simplest choice (self-contained, no separate asset-serving path needed), but this is a backend implementation detail per the contract; the frontend must keep treating `iconDataUrl` as opaque either way.

Step 2's implementation is a **refactor of `utils/icon.rs`, not a parallel new path**
(`design-review.md` C10/M10): the codebase's one existing piece of image handling,
`load_icon_as_data_uri`, is path-in → extension-sniffed MIME → data-URI out with no validation and
no size cap — exactly what steps 2–3 exist to prevent, and its only caller (mod icons) dies with
C6. Refactor it so its input contract is raw bytes (not a filesystem path) and MIME comes from
content sniffing. Add the `image` crate for decode/validate/resize; flag this as a recommendation
to confirm before implementation, not a hard requirement — a narrower hand-rolled magic-byte check
would also satisfy step 2 if pulling in `image` is undesirable.

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
variants needed elsewhere in this document (`InvalidToolIcon` for §9, `ConfigSaveFailed` for §8's
write path, `TaskIdInUse` for §8a) get a `code` for free the moment they're added — nothing to
remember to update here. One deliberate non-addition: there is no `CorruptLibrary` variant — an
unreadable/unparseable library maps to the existing `InvalidLibrary(String, String)`, with
detection at `read_library_manifest` catching **both** read-path failures (`IOError` and
`ParseError` from `utils/toml.rs`), per `design-review.md` C5/M8.

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
- No database dependency is added — the SQLite Library DB and its `rusqlite`/`rusqlite_migration` additions are cancelled (§8, `design-review.md` C4). `toml` is already a dependency and covers both the App Config and the per-library files.
- Add `image` (or a narrower hand-rolled alternative — see §9) for tool icon validation/normalization.
- Drop `confy` (§8 Crate Choice) once §8's Rollout Sequence step 5 confirms nothing reads it anymore — the decision is made, this is just sequencing the removal, not re-litigating whether to.

## 13. Implementation Sequence

1. Lock the test oracle (Execution Method item 1); land the `1.7` manifest-removal behavior change first if not already done.
2. FP refactor: `core/mod_fs.rs`, `utils/file.rs`, `utils/toml.rs`, `utils/process.rs` (§4), mechanically then refined.
3. Low-level FS separation: relocate `core/decompression.rs`'s `extract` (§6).
4. Fix the known `add_mods`/`remove_mods` boundary violation (§5) — this both closes the flagged violation and becomes the template for the endpoint rework in step 5.
5. Add `store/` (App Config) additively (§8 Rollout Sequence step 1), including the atomic,
   surfaced write path (`atomic_write` in `utils/file.rs`, `save() -> Result`, §8).
6. Land the App Config migration (§8 Rollout Sequence step 2): `confy` → `known_libraries`/`app_state`, IDs adopted from each library's `manifest.toml` (minted only when no readable manifest exists, C7).
7. Add the tool registry against `tools.toml` (§9, §8 Rollout Sequence step 3) — no legacy data.
8. Cut settings over to the App Config (`get_settings`/`save_settings`, §8 Rollout Sequence step 4, T1).
9. Endpoint rework (§7): rename, regroup into `commands/library.rs` / `commands/global.rs` / `commands/tool.rs`, add the `#[serde(tag = "code", content = "data")]` attribute to `SError` (§11), move `init_called.store` into the startup command (C11), delete `commands/test.rs`/`models/test.rs` (C9).
10. Implement the fire-and-track task machinery (§8a): active-library-scope validation, the `taskId` registry on `AppRegistry`, `TaskIdInUse` rejection; wire `rebuild_library_cache`, `install_mod_archives`, `bulk_update_mods`, and `sync_mods` to emit their completion events (`cache_rebuild_completed`, `mod_install_completed`, `bulk_update_completed`, `sync_completed`) carrying the `taskId`. Remove the `remove_all_backups` call from rebuild's normalize step (C8). Add `deployStale` to `LibrarySummary` (C3/M5).
11. Comment removal batch pass (§10).
12. Dependency cleanup (§12), including §8 Rollout Sequence step 6 (`confy`/legacy manifest-cache read-path removal).
13. Run `cargo run --bin export_types` and hand off to `frontend-redesign-spec.md` §12 step 13.

## 14. Verification Plan

- `cargo build`, `cargo clippy` clean throughout, not just at the end.
- `tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs` green after every step in §13, per Execution Method item 1.
- Per `1.4_tests.md`: refactor `tests/library.rs`/`tests/mod_fs.rs` manifest-dependent cases to hash-only assertions (already scoped by `1.7`); remove `test_collect_files_excludes_manifest`.
- Add test coverage for the App Config migration: a fixture `confy` `GlobalConfig` with `library_last`/`library_recent` migrates to the expected `known_libraries`/`app_state` entries, with IDs **adopted from each library's `manifest.toml`** — a fixture path with a readable manifest keeps its manifest id; only a fixture path with no readable manifest gets a freshly minted one (§8, C7).
- Add test coverage for read-only workspace assembly (§8, C13): a registered library whose `manifest.toml` is unreadable (one fixture missing the file, one with unparseable content — both cases) appears in the workspace as a path-only stub, is never dropped, and listing never calls `Library::load`/`version::fetch_and_validate` (an unsupported-SPT-version fixture must still list fully and only fail on `activate_library`).
- Add test coverage for the App Config write path (§8, C12): a failed save surfaces `ConfigSaveFailed` at the command boundary, and an interrupted write never leaves a half-written `config.toml` (temp-file + rename).
- Add test coverage for `execute_tool`'s detached-spawn behavior on Windows (`1.9_backend-redesign-audit.md` §4, Windows Verification).
- Add test coverage for §9a's upgrade-safety mechanism: a failed overwrite restores the prior mod contents, and a successful overwrite leaves no snapshot directory behind. `tests/library.rs` already has mod-add coverage to extend rather than a new test file to create.
- Add test coverage for the task machinery (§8a): a fire-and-track call reusing a `taskId` still in flight returns `TaskIdInUse`; a call whose `libraryId` is not the active library is rejected with a validation error before touching anything; two overlapping `bulk_update_mods` calls on the same mod both complete and the final state is one of the two requested states (absolute-set semantics under an unordered mutex).
- Add test coverage that `bulk_update_mods` does not deploy (§7, C3): after a toggle, no symlinks in the game/SPT directories change and the library's dirty flag (`deployStale`) is set until `sync_mods` runs; `sync_mods` clears it.
- Add test coverage that rebuild's folder normalization no longer deletes backups (§8a, C8): a renamed mod's backup directories survive the rebuild, under the old name.
- `cargo run --bin export_types` succeeds and the generated bindings match `frontend-redesign-data-api-contract.md` — treat a mismatch as a spec bug in whichever of the two documents is wrong, not something to paper over in generated code.

## 15. Acceptance Criteria

- No file in `core/` calls `std::fs`/`tokio::fs` directly (§6 exit criterion).
- `ModFS`, `FileUtils`, `Toml`, `ProcessChecker` no longer exist as structs; their logic is free functions.
- `commands/library.rs::add_mods`/`remove_mods` no longer inline service-layer orchestration.
- Every command in `commands/` matches a row in the §7 mapping table — no undocumented commands.
- The App Config never contains a library's own data (name, mods, tools, cache), and no library's own files contain another library's data or the App Config's registry/settings — each fact lives in exactly one file (§8).
- Library identity is single-sourced: the App Config's `known_libraries.id` is always a copy of the id in that library's `manifest.toml`; migration and re-adding adopt, never re-mint, wherever a readable manifest exists (§8, C7).
- `install_mod_archives`, `bulk_update_mods`, `rebuild_library_cache`, and `sync_mods` each take a client-minted `taskId`, validate active-library scope, register/release the task on `AppRegistry`, and emit their completion event (carrying the `taskId`) when done; a `taskId` reused while in flight is rejected with `TaskIdInUse` (§8a). There is no `LibraryOperationInProgress` and no reject-on-overlap for normal mutations.
- `bulk_update_mods` never creates or removes symlinks; `sync_mods` is the only deployment path, and `LibrarySummary.deployStale` is `true` exactly when recorded mod state has diverged from deployed state (§7, C3/M5).
- Rebuild's folder normalization renames only — it deletes no backup directories and performs no re-linking (§8a, C8).
- `store/` is the only writer of App Config state (registry, app state, settings); per-library state is written only through the existing manifest/cache path and `tool_service`'s `tools.toml` path (§8).
- App Config writes are atomic (temp file + rename via `utils/file.rs::atomic_write`) and surfaced (`save() -> Result<(), SError>` propagated at every call site, with the `lib.rs` startup site's log-and-toast-later handling) (§8, C12/M7).
- Workspace assembly is read-only: it never opens (`Library::load`) or version-validates any library, and a registered library whose manifest can't be read surfaces as a path-only stub, never silently dropped (§8, C13).
- `commands/tool.rs`'s `upsert_tool`/`delete_tool` never spawn a process; `execute_tool` never writes to `store`.
- All command outputs are `Result<T, SError>`, unchanged from today's signatures. `SError` has `#[serde(tag = "code", content = "data")]` so every variant serializes to a stable `{code, data}` shape.
- No command path sends `SError`'s `Display`-formatted message (or any other English prose) to the frontend — it's logged via `tracing::error!` at the point of return, not returned.
- No `get_backups`/`create_backup`/`restore_backup`/`remove_backup` commands exist; backup is unreachable from `commands/` (§9a).
- `create_simulation_game_root`, `commands/test.rs`, and `models/test.rs` no longer exist in any build (C9).
- An upgrade that fails partway through leaves the mod exactly as it was before the upgrade started (§9a, restore-on-failure).
- A successful upgrade leaves no leftover snapshot on disk (§9a, discard-on-success) — verified by a test, not just by reading the code.
- Backend test oracle is green at every step, not just the final one.
- `cargo build`, `cargo clippy`, `cargo test` all pass clean.
