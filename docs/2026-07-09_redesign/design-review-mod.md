# Design Review Response — Clarifications on Findings 1, 4, 7

Status: Draft. Author clarifications against `design-review.md`, addressed to that doc's
findings 1 ("guard rejects rapid toggles"), 4 ("runtime model unspecified"), and 7 ("dropped
game-running guard, unspecified toggle cost/collisions").

This does not replace §R of `design-review.md`; it narrows which parts of §R are actually being
adopted and which are deferred.

---

## C1. `bulk_update_mods` keeps the current lock model — R.1/R.2 is not being adopted wholesale

`design-review.md` §R.1/R.2 proposes dissolving `Library` into an immutable handle and replacing
the in-process mutex with a per-library job queue, on the premise that the current model can't
express finding 4's concurrency promises.

Clarification: `bulk_update_mods` is not being redesigned onto that new runtime. It runs as a
single command call that does its series of work (validate → mutate in-memory `Library` → persist)
under the existing `Arc<Mutex<Option<Library>>>` (`core/registry.rs:14`), the same
`with_lib_arc_mut` pattern already in use (`utils/thread.rs:6`). The current lock model already
prevents write competition as-is — two overlapping `bulk_update_mods` calls serialize on the mutex
like every other mutating command does today. Finding 4's concern that "no in-memory mutable
state exists to protect" doesn't apply here because the in-memory `Library` isn't being removed.

This also changes finding 1's fix. The reject-guard `design-review.md` objects to
(`LibraryOperationInProgress`, "not queuing") is a separate, additional mechanism layered on top of
locking — not the lock itself. Dropping that separate reject-check and letting overlapping calls
simply block on `Mutex::lock()` (as every other command already does) removes the reject-on-overlap
behavior finding 1 flags, without needing R.2's per-library job queue. Rapid sequential toggles
queue by blocking on the mutex, in order, same as today.

**What's out of scope here:** the *boilerplate* cost of this model — every command still clones an
`Arc`, locks it, and wraps work in `spawn_blocking` (`utils/thread.rs`, ~19 call sites per
`design-review.md` R.1). That duplication was already called out as its own problem (1.9 backend
audit, "Strip all boilerplate... reduce command boilerplate from ~15 lines to 2 lines"). It's a
real issue but a distinct one from write-competition safety, and isn't being solved by this
clarification — it stays open, tracked separately from findings 1/4.

## C2. `GameOrServerRunning` guard is deliberately dropped, not an oversight

`design-review.md` finding 7 reads the guard's absence from the new spec as a gap needing R.2's
"re-check at execution time" fix.

Clarification: the guard is being dropped on purpose, and the reason is narrower than "TOCTOU in
general" — it's a cross-platform detection problem. Today's check
(`utils/process.rs::ProcessChecker::is_running`) works by matching a process's canonical exe path
(`p.exe()`) against known canonical target paths (`core/registry.rs:32`
`is_game_or_server_running`). That's stable on Windows because the game/server ships as a `.exe`
with a fixed, resolvable path. It isn't stable the same way on Linux: the server is commonly
launched through a `.sh` script that then execs the real binary (or runs it under a wrapper), so
there's no single canonical exe path to match the way there is on Windows — the same detection
strategy doesn't generalize across platforms.

This doesn't mean detection is impossible on Linux, just that path-based matching isn't it. Matching
on process *name* (or an equivalent name-based heuristic) instead of the resolved exe path could
still find a running instance — but that's a materially different, less precise mechanism (name
collisions, truncated `comm` values, no guarantee the script's process name matches the binary's),
not a drop-in fix to the existing path-comparison code. Re-checking at execution time (R.2's
proposal) doesn't address this at all — the gap isn't *when* the check runs, it's that the check
itself has no cross-platform-consistent implementation yet.

So: the guard is dropped for now as a deliberate scope cut, not reintroduced with R.2's timing fix.
It's open for revisiting once a name-based (or otherwise platform-agnostic) detection strategy is
actually specified and shown consistent on both platforms. Until then, no guard ships, and that
should be stated as an accepted risk in the spec rather than left as an implicit gap.

## C3. Dirty-flag semantics — deployment is decoupled from mod-state edits, deferred to the user

The dirty flag (`Library::is_dirty`, `core/library.rs:25`) stays, and its meaning is narrower than
"unsaved change" — it specifically marks that the *deployed* state (the symlinks in the game/SPT
mod directories) no longer matches the *recorded* mod state (which mods are enabled). A toggle
always saves immediately; it just doesn't always deploy immediately.

This matches the current implementation exactly:

- `mod_manager::toggle_mod` (`core/mod_manager.rs:83-92`) flips `is_active` in memory, calls
  `mark_dirty()`, then `library.persist()` — which writes the manifest/cache TOML
  (`core/library.rs:135-139`). That's cheap: no filesystem walk, no symlink creation/removal, no
  collision check. The toggle's own state is never lost, regardless of how large or rapid a batch
  of toggles is — every one lands in the persisted manifest.
- `Library::sync` (`core/library.rs:143-149`) is the separate, explicit operation that actually
  does the expensive work: `cleanup::purge` (unlink everything) → `deployment::deploy` (relink
  everything, where the collision check lives) → `mark_clean` → `persist`. This is exactly the
  operation the current `sync_mods` command wraps (`commands/library.rs:89`), gated by
  `is_game_or_server_running` (C2).

So the model is: **edits to mod state are cheap and always committed; deployment (the expensive,
collision-checked, symlink-touching part) is a distinct, user-triggered step.** This is a deliberate
design choice, not a missing piece — it's what lets a user make a large-scale change (many toggles,
an install, a bulk disable) without paying deployment's cost per-edit, while never risking losing
what they changed.

This resolves finding 7 more directly than the "incremental commit" fix `design-review.md` R.3
proposes, and it corrects a premise upstream of that finding:

- **Cost model**: answered. Toggling never runs purge+redeploy; only an explicit sync/deploy action
  does. There's no per-toggle full-tree-walk cost to avoid, because toggling doesn't walk the tree.
- **Collision handling**: answered the same way it works today — collisions surface at
  deploy-time (`deployment::deploy`), not at toggle-time, because toggle-time doesn't deploy.
- **This means `frontend-redesign-spec.md` §9.3's "must commit to disk immediately" and
  `design-review.md` finding 4's "`sync_mods` removal" are both describing a model that isn't the
  one being built.** `bulk_update_mods` commits *mod state* immediately; it does not deploy
  immediately, and an explicit deploy/sync command (whatever `sync_mods` is renamed to) is not
  being removed — it's the same guarded, expensive step it is today, just no longer triggered
  implicitly by every toggle. The frontend and backend specs need that correction before Phase 2
  implementation, not just this review.
- Finding 1 remains resolved independently by C1 (lock model), unaffected by this — the two were
  always orthogonal, as noted before; this section replaces the earlier open question with a
  settled answer instead.

## C4. Decision: cancel the SQLite migration for this redesign

`backend-redesign-spec.md` §8 proposes replacing each library's `manifest.toml`/`cache.toml` with a
per-library `library.db` (SQLite via `rusqlite` + `rusqlite_migration`). Reviewed the tradeoff
against what the docs actually claim versus what the redesign actually needs:

- **The stated justifications don't specifically require SQL.** "Two files into one"
  (`purpose-of-redesign.md:33`) is satisfied by any single structured file — it argues for
  consolidation, not a database. "Sophistication" (mods/tools/cache as distinct concerns) argues
  for schema/structure, which a single file with sectioned tables (e.g. TOML with `[mods]`/
  `[tools]`/`[cache]`) provides equally well. Nothing in `get_library_workspace`'s actual access
  pattern (`backend-redesign-spec.md:257-268`) exercises query/join capability — it reads whole
  tables per library, and even derived counts (`modCount`/`enabledModCount`) are pushed to the
  frontend to compute rather than queried. The one genuine SQL-specific win — atomic, crash-safe
  per-row commits — is real, but it's a narrower justification than what's written, and (per C3)
  toggle-time writes are already metadata-only and cheap, so the write-amplification framing
  overstates the problem at this app's realistic scale.
- **§8a's concurrency argument leans on a guarantee the redesign isn't actually using.** Its case
  for needing "SQLite serializes writers regardless" (`backend-redesign-spec.md:299`) assumes the
  in-process locking model was being replaced (R.1/R.2). C1 settled that it isn't — `bulk_update_mods`
  keeps the existing `Arc<Mutex<Option<Library>>>`, which already serializes every write to a given
  library within this single-instance app. SQLite's own writer arbitration is then mostly redundant
  with a lock the app is keeping anyway.
- **Costs are concrete and immediate:** a new dependency (`rusqlite` bundled +
  `rusqlite_migration`), the accepted N-small-file-open cost at every workspace assembly
  (`backend-redesign-spec.md:265-268`), a full migration path (old manifest/cache → DB, §8's lazy
  migration) that has to be written and tested either way, and a real loss of transparency — a
  library's state stops being something a user can open in a text editor, cutting against the
  redesign's own "a library's data travels with the library" philosophy.

**Decision: cancel the SQLite migration for this redesign.** It brings more new surface area
(dependency, migration mechanics, per-library file-open cost, debuggability loss) than it resolves,
given the in-process lock model (C1) is staying and the access patterns don't need a query engine.
Library state stays in structured plain files (`manifest.toml`/`cache.toml`, or a consolidated
single-file replacement if that's still wanted for the "two files into one" goal — a separate,
smaller decision). This can be revisited later if the app's shape changes — larger library counts,
a genuine need for cross-entity queries, or a move to a model where the in-process mutex no longer
serializes writers (e.g. if R.1/R.2 is adopted after all) would all strengthen the case again.

**Downstream effect on other findings:** every finding and fix in `design-review.md` that assumed
`library.db`/SQLite is now moot for this redesign and needs re-grounding against whichever plain-file
design replaces it, specifically:
- **#2** (eager migration via assembly) and **#4's R.4** (lazy migration only in `activate_library`)
  — restate against manifest/cache files, not `library.db`.
- **#6** (double-minted library identity) — the identity source becomes `manifest.toml`'s `id`
  again outright, not `library_meta.library_id`; C4 simplifies this rather than resolving it via a
  new rule.
- **#8** (scan-phase purity / rebuild reconcile step) — the commit-phase mechanics need restating
  as a file write (with the same atomic temp-file+rename pattern finding 12 already proposes for
  the App Config), not a SQLite transaction/upsert.
- **#13** (event payload cost tied to assembly cost) — assembly's cost model changes from "open N
  SQLite files" to "read N manifest/cache file pairs"; likely cheaper, but restate rather than
  assume.
- `backend-redesign-spec.md` §8/§8a and the `store/` module layout (lines 154-320) need a rewrite
  pass to reflect this before Phase 2 implementation, not just a note in this review.

## C5. Library open errors: distinct variants for unsupported version and corrupt library

Finding 3 flagged that no `SError` variant distinguishes "this library is temporarily unreachable/
broken" from "this request was invalid," and that `get_known_library_summary` silently drops
libraries it can't read (`core/library_service.rs:133-136`, `filter_map` over
`Library::read_library_manifest`). Decision: add the missing error variants rather than adopt the
full `LibrarySummary.status` field finding 3 proposed — narrower scope, same gap closed at the
point where it actually bites (opening/activating a library).

- **Unsupported version** already exists and already fires: `SError::UnsupportedSPTVersion(String)`
  (`models/error.rs:7`), raised by `version::fetch_and_validate` (`core/version.rs:76`) and called
  from both `Library::create` and `Library::load` (`core/library.rs:47,76`). No new variant needed
  here — this one was already correctly typed, just currently a hard-fail on every load rather than
  a reportable status. Keeping it a hard-fail on `activate`/`open` is acceptable as a narrower fix
  than finding 3's full proposal: a user can't do anything useful with a library manager that
  claims to support an SPT version the library doesn't have.
- **Corrupt library** has no dedicated variant today — a manifest/cache parse failure surfaces as
  generic `SError::ParseError` (`utils/toml.rs:9,15`), indistinguishable from any other parse
  failure in the app (a bad mod archive, a malformed registry.json, etc.). Add a distinct variant,
  e.g. `SError::CorruptLibrary(String, String)` (path, underlying reason), raised specifically when
  `Library::read_library_manifest`/`Library::load` fails to parse the manifest or cache at a path
  that otherwise looks like a library (directory exists, expected files present but unreadable).
  This lets the frontend show "this library's files are corrupted" instead of a generic parse
  error, without committing to the broader per-library status/degrade model finding 3 proposed.
- `get_known_library_summary`'s silent `filter_map` drop stays out of scope for this decision — it
  keeps working exactly as today (skip what can't be read) until/unless the broader status-field
  question from finding 3 is revisited separately.

## C6. Remove `iconDataUrl` from `ModSummary` — UI falls back to a default icon

Adopts finding 5's fix directly. The only producer of a mod icon was `manifest.icon` →
`utils/icon.rs::load_icon_as_data_uri`, and 1.7's manifest removal deletes the source field; the
redesigned mod card is title-only with a category icon tinted by `ModType`
(`frontend-redesign-spec.md` §9.3), so there's nothing left to display it anyway.

**Decision:** drop `iconDataUrl` from `ModSummary` and the `icon_data_url`/`icon_data` fields
backing it (`models/mod_dto.rs` DTO, `Mod.icon_data` set to `None` at every construction site
already, e.g. `core/library_service.rs:218`). The UI uses a default icon (by `ModType`) for every
mod card — no per-mod image, no derivation rule needed. `utils/icon.rs` doesn't die with this
change though — see C10, it's still needed for tool icons (§9).

## C7. Library identity: `manifest.toml`'s `id` is authoritative

Adopts finding 6's fix (R.5) directly, and it's simpler now that C4 cancelled the SQLite/`library.db`
side of the question — there's no `library_meta.library_id` to reconcile against. The only identity
that exists today is the one already minted in `core/library.rs:50`
(`id: uuid::Uuid::new_v4().to_string()`, persisted in the manifest). 

**Decision:** that id stays the single source of truth. Wherever the App Config gains a stable
`known_libraries` registry (independent of the SQLite decision — this was always a plain-file
change per `purpose-of-redesign.md:37-39`), migrating an existing path into that registry must read
the id already sitting in that library's manifest, not mint a new one. Minting only happens for a
library that genuinely has no manifest yet (brand new). Re-adding a directory that still has a
readable manifest adopts the id found inside it. One rule, no reconciliation cases — matches R.5
as originally proposed, just without the `library_meta` half that no longer exists.

## C8. Rebuild cache: rename folders only, stop deleting backups in the same step

Finding 8 flagged that `rebuild_library_cache`'s folder-normalization step does more than rename —
it also deletes backup directories for renamed mods, coupling an unrelated destructive action to a
path-fixup step. Confirmed in code: `library_service::rebuild_library_cache`
(`core/library_service.rs:181-226`) step 2 calls
`mod_backup::remove_all_backups(&library.lib_paths, &renamed.old_name)` (line 193) for every
renamed mod, right after `normalize_mod_folders` (`core/cache.rs:29`) does the actual rename.

**Decision:** remove the `remove_all_backups` call from that step. Rebuild's folder-normalization
should only do what its name says — rename the folder, remove the old mod-map entry, and re-key the
new one to the resolved id (lines 189-221 otherwise unchanged). Backups for a renamed mod are left
on disk under the old name; they become orphaned relative to the mod's new id, but nothing deletes
data a user might still want (a backup taken before the rename). If orphaned backups ever need
cleanup, that's a separate, explicit, user-visible action — not a side effect of a cache rebuild.

**Finding 8's other half — dangling symlinks after rename — is expected behavior, not a gap.**
Clarified: rebuild only rewrites the library's own *record* (mod map, cache) — it is not, and isn't
meant to be, a fix for live game deployment. If a rename (or any other on-disk change to a deployed
mod, including a user manually deleting a file inside it) leaves the running game in a broken state,
that's an expected consequence of editing files out from under a live deployment, not something
rebuild is responsible for catching or repairing. The recovery path is the same one C3 already
established: rebuild the cache, then run the explicit sync/deploy step — deployment was already a
separate, user-triggered action, and this is just another case that falls out of that model rather
than a new one.

This also matters less in practice than finding 8 implied, because deployment links the mod's
**top-level folder**, not individual files inside it — a symlink from the game/SPT mod directory
points at `library_root/mods/<mod_id>` as a whole. Changes to files *inside* that folder are already
live through the existing link without any resync. The only case that actually breaks the link is
the top-level folder itself being renamed (which is exactly what rebuild's normalization step does),
and that case is already covered by the rebuild-then-redeploy expectation above — no separate
re-link step needed inside rebuild itself.

## C9. Remove `create_simulation_game_root` entirely

Finding 9: this command has no `#[cfg(debug_assertions)]` anywhere — `commands.rs:3`
(`pub mod test;`) and `lib.rs`'s `collect_commands!` (currently line 47, `create_simulation_game_root`)
both register it unconditionally, so release builds ship an IPC command that writes dummy
`EscapeFromTarkov.exe`/`SPT.Server.exe` files to a caller-supplied path.

**Decision:** delete it, not cfg-gate it. That means removing:
- `commands/test.rs` (the command itself) and `pub mod test;` from `commands.rs:3`.
- The `create_simulation_game_root` import and `collect_commands!` entry in `lib.rs`.
- `models/test.rs`, which is already an empty stub (`// Test models module - currently empty as
  test structs have been removed`) — nothing left depending on it.

**Frontend dependency, resolved:** `src/modules/settings/developer-settings.tsx` is built entirely
around this command (`commands.createSimulationGameRoot`, the whole "Test Game Root" panel) and
would otherwise be left dead by this deletion. Confirmed: the redesigned UI has no developer
settings section at all, so `developer-settings.tsx` is deleted in the same change, along with
wherever it's mounted in the current settings module — not carried forward in any form.

## C10. Filling in finding 10's file-classification gaps

Finding 10: `utils/icon.rs`, `utils/id.rs`, `utils/time.rs`, `utils/thread.rs`,
`core/mod_documentation.rs`, `commands/test.rs`, and everything under `models/` were never
classified in `backend-redesign-spec.md` §4's table despite the table claiming full coverage.
Classifications, given the decisions already made above:

| File | Classification | Why |
|---|---|---|
| `utils/icon.rs` | **Refactor, not delete.** | Loses its mod-icon caller (C6 removes `iconDataUrl`), but `frontend-redesign-spec.md` §9's tool-icon flow needs exactly this capability. Finding 10's own complaint stands and must actually be fixed on this pass: MIME is extension-sniffed (`match icon_path.extension()`, `utils/icon.rs:13-20`) rather than content-sniffed, and there's no size cap before base64-encoding into memory. Fix both as part of wiring it to tool icons, not as a separate ticket. |
| `utils/id.rs` | **Keep, unchanged.** | `hash_id` (blake3 + base64url) is the hash-based mod identification mechanism 1.7 standardizes on. No redesign document proposes changing it. |
| `utils/time.rs` | **Keep, unchanged.** | Single trivial helper (`get_unix_timestamp`), no redesign touches timestamp handling. |
| `utils/thread.rs` | **Keep, unchanged (supersedes design-review's R.1 reclassification).** | Design-review proposed reclassifying this "dissolved" once `Library` becomes an immutable handle (R.1). C1 rejected R.1 — `with_lib_arc`/`with_lib_arc_mut` (`utils/thread.rs:6,15`) stay exactly as they are. The boilerplate-reduction idea from the 1.9 audit is still worth doing later (C1's noted open item) but that's a helper-ergonomics change, not a deletion. |
| `core/mod_documentation.rs` | **Delete.** | `read_documentation` (`core/mod_documentation.rs:7-33`) reads `manifest.documentation`, a field on `ModManifest` that 1.7's manifest removal deletes entirely. This has no legs after 1.7 lands — finding 10 was right that it's only "implied dead." |
| `commands/test.rs` | **Delete.** | Per C9. |
| `models/*` | **Mostly unchanged, no group verdict needed** — classify what actually moves: `models/mod_dto.rs` shrinks under 1.7 (manifest-derived fields drop, plus `icon_data` per C6) and gains nothing from the cancelled SQLite plan (C4); `models/error.rs` gains C5's `CorruptLibrary` variant; `models/global.rs`/`models/config.rs` are the App Config shapes and evolve with whatever confy→toml migration still proceeds (C4 didn't cancel that, only the Library DB); `models/test.rs` deletes per C9; `models/library.rs`, `models/paths.rs`, `models/mod_backup.rs` are unaffected by anything decided here. |

## C11. Fixing finding 11: the init-timeout watchdog must track whatever command actually loads the workspace

Confirmed current code is correct today: `commands/global.rs::init` (lines 159-166) is the only
place `state.init_called.store(true, ...)` is called, and it's called first thing inside that
command. The bug finding 11 describes is conditional on the spec's plan to rename `init` →
`get_library_workspace` (`backend-redesign-spec.md` §7) without carrying this line over — if that
rename still happens under whatever's left of the spec after C4, the `store(true, ...)` call must
move with it into the new command's body, not stay attached to the old name.

**Fix:** whichever command ends up being the frontend's one startup/workspace-fetch call — renamed
or not — must be the one that calls `init_called.store(true, Ordering::Relaxed)`. Add this as an
explicit line item in `backend-redesign-spec.md` §7's row for that command, not left implicit. Separately,
finding 11's second point stands on its own regardless of naming: `start_init_timeout_checker`
(`lib.rs:116-127`) responds to a timeout with a silent `std::process::exit(1)` — no dialog, no
frontend notification, nothing persisted. That should become a visible failure (error window or OS
dialog) as part of Phase 4's logging work, independent of whatever the command is named.

## C12. Proposed plan for finding 12: surfaced, atomic App Config writes

Confirmed current state: `GlobalConfig::save` (`config/global.rs:26-28`) is
`let _ = confy::store("Modkeeper", CONFIG_NAME, self);` — any write failure (disk full, permissions,
path gone) is silently discarded, and every call site (e.g. `config.save()` in
`core/library_service.rs:123`) has no way to know it happened because `save()` returns `()`. This is
about to matter more once the App Config also carries stable library ids (C7) and settings (T1) —
losing a write silently means losing registry entries, not just a convenience file.

This is unaffected by C4 — the App Config was always staying a plain file (`purpose-of-redesign.md:37-39`),
and its confy→toml migration (`backend-redesign-spec.md:209-220`) is a separate decision from the
cancelled Library DB.

**Proposed plan:**
1. Add a dedicated error variant, e.g. `SError::ConfigSaveFailed(String)` (or reuse `IOError`/
   `ParseError` if a new variant feels like overkill — but a distinct variant lets the frontend show
   "your settings/library list may not have saved" specifically, which a generic IO error doesn't).
2. Change the signature from `pub fn save(&self)` to `pub fn save(&self) -> Result<(), SError>`.
   This is a breaking change to every call site — audit and update each one (`library_service.rs`,
   `global_service.rs`, and any command that calls `config.save()`) to propagate the error up to the
   command's `Result<_, SError>` return instead of ignoring it.
3. Implement the write itself as atomic: serialize to a temp file in the same directory as the
   target (`config.toml.tmp` or similar), then `std::fs::rename` the temp file over the real path.
   Rename-over is atomic on both Windows and Linux for same-volume renames, so a crash or failure
   mid-write never leaves a half-written `config.toml` — the old file stays valid until the new one
   is fully written. Add this as a `FileUtils::atomic_write` helper in `utils/file.rs` (alongside
   the existing `copy_recursive`) rather than inlining temp-file handling in `config/global.rs`,
   since C-whatever-replaces-`library.db` (if `manifest.toml`/`cache.toml` want the same crash-safety
   after C4) can reuse it.
4. This lands as part of the already-planned confy→toml direct migration (`backend-redesign-spec.md:209-220`),
   not as a separate pass — the toml crate gives explicit control over serialization, the atomic
   write gives explicit control over the write itself; both are needed together, not either alone.
5. Read-path behavior (log + reset vs. hand-written migration on parse failure) is already an open
   choice per the existing spec text — this plan doesn't resolve that, only the write side finding
   12 was actually about.

## T1. App settings ownership — backend (App Config) vs. frontend (`localStorage`)

**Decision: confirmed backend-owned.** Settings move into the App Config alongside
`known_libraries`/`app_state` (same file, same "one place for all app-level state" principle) —
the full migration option below, not the middle-ground.

**Frontend integration pattern:**
- Whichever command is the frontend's startup call (C11 — `init` or its eventual replacement)
  returns the full settings blob as part of what it loads; the frontend stores it in a Jotai atom.
- User-initiated changes dispatch through backend commands (not a local atom write followed by a
  separate save call) — the command performs the change and its response returns the **full**
  settings object, which replaces the Jotai atom wholesale.
- This is full-replacement, not merge: the frontend never constructs the next settings state
  itself from a partial update. The backend is the sole authority on what the settings look like
  after a change, which sidesteps any drift between an optimistic local update and what actually
  got persisted (the same "don't hand-construct state the backend already computed" principle
  `get_library_workspace`-style responses use elsewhere in this redesign).
- This also resolves finding 14's `loadSettings()` sync-signature complaint outright — there's no
  longer a synchronous `loadSettings()` to reconcile with an async command, because settings access
  was never going to be synchronous once it's fetched at init and read from the atom afterward.

The tradeoff analysis below is kept for reference (why this was the right call), superseded by the
decision above rather than left open.

**Current state:** entirely frontend-owned. `src/lib/settings-storage.ts` reads/writes
`localStorage` directly, validated with a zod schema (`SettingsSchema`), synchronous
(`loadSettings(): Settings`, no `Promise`), with client-side JSON import/export via `Blob`/download.
The backend has no involvement at all today.

**Proposed state** (`backend-redesign-spec.md:151`, `purpose-of-redesign.md:41`): `get_settings`/
`save_settings` commands backed by a `settings` section of the App Config file, alongside
`known_libraries`/`app_state`.

**Why the proposal exists at all:** a specific, narrow need — some settings (theme, for window
vibrancy) must be readable by the backend *before* the frontend has loaded, so the native window
can be created with the right chrome from the first frame instead of flashing the wrong theme.
`localStorage` is a webview construct; the Rust process can't read it before the webview exists.
That's a real constraint the current design can't satisfy at all.

**What backend ownership buys beyond that:**
- One place for all app-level state (library registry, active selection, settings), consistent with
  the doc's own principle that each fact lives in exactly one file — instead of `known_libraries`
  living in the App Config while settings live in a separate, backend-invisible store.
- Gets C12's atomic-write/surfaced-error treatment for free, instead of settings having their own
  ad hoc persistence story forever (today's `localStorage.setItem` has no failure handling at all,
  though for `localStorage` that's a much smaller risk than for a registry file).

**What it costs:**
- `loadSettings()`'s synchronous signature can't survive the move — finding 14 already flags this
  (`frontend-redesign-spec.md` §7 declares it sync while the command it wraps is
  `Promise<AppSettings>`). Every settings read in the frontend becomes async, which touches any UI
  code that currently assumes settings are available synchronously on first render.
- Settings become a Rust-typed shape that needs `export_types` re-run on every change, instead of a
  self-contained TS zod schema editable without touching the backend at all.
- Import/export (today: client-side `Blob`/download, `src/lib/settings-storage.ts:57-91`) either
  needs to keep operating on a shape now owned by Rust (round-tripping through IPC to read/write
  it) or gets reimplemented backend-side — not free either way.
- Startup sequencing gets more coupled: getting the "read theme before first frame" benefit actually
  requires the window-creation code path to consume the App Config *before* building the window,
  which is a real Rust-side startup-order dependency, not just "add two commands."

**The middle ground (mirroring only the startup-relevant setting) was considered and rejected:**
it would have avoided some of the costs above, but at the price of reintroducing "two sources of
truth for overlapping data" — the same problem the rest of this redesign is deliberately avoiding
elsewhere (C7's single library-identity source, the one-file-per-fact principle §8 already states).
Full backend ownership is the more consistent choice given that principle, even though it means
paying the async-signature and `export_types` costs for settings that didn't strictly need them.

## C13. Known-library list: unreadable libraries collapse to a path-only object — resolves findings 2 and 3

Extends C5 with the frontend-facing shape, and settles findings 2 and 3 together rather than
adopting either finding's original fix (the `LibrarySummary.status` enum, or eager-migrating
assembly) wholesale.

**Decision:** the known-library list is a list of either a full summary or a minimal stub — not a
full summary with a `status`/`statusDetail` field bolted on. A library that can't be read (missing
folder, corrupt manifest, drive unplugged) shows up as an object containing only its registered
`path` — every other field simply isn't there, rather than being present-but-empty or explained by
an error code. The UI renders that stub as a bare path with no name, mod count, or actions beyond
remove.

**Recovery is manual and coarse, on purpose:** there is no in-app repair/retry/refresh-this-entry
action. If the library becomes readable again (user reconnects the drive, restores the folder), the
user restarts the app to have it re-read at next startup; if it's actually gone or intentionally
abandoned, the user removes the stub and re-adds the library fresh. No polling, no per-entry
re-validation, no live status transitions to design or implement — this is what makes the model
simple enough to not need finding 3's fuller status enum.

**Why this resolves finding 2 as well, not just finding 3:** finding 2's actual bug was assembly
eager-opening/migrating every registered library on every startup. Checked against current code:
`get_known_library_summary` (`core/library_service.rs:129-141`) already only reads `manifest.toml`
— it never calls `Library::load`/`version::fetch_and_validate`, so it never triggers the SPT-version
hard-fail (`core/library.rs:47,76`) at listing time regardless. That means an unsupported-SPT-version
library still lists successfully today (manifest read succeeds; only *activating* it fails) — finding
2's eager-migration risk was specific to the SQLite assembly design C4 already cancelled, and this
plain-file read-only listing behavior was never actually broken. The only piece that needed a
decision was what happens when the manifest read itself fails, which this section answers: fall
back to the path-only stub instead of `filter_map`'s current silent drop (`core/library_service.rs:133`).
Unsupported-version libraries still list fully (their manifest reads fine) and only surface
`SError::UnsupportedSPTVersion` (C5) if the user tries to activate them — consistent with today's
behavior, not a new restriction.

## C14. Is `ModType` still meaningful? Yes — keep it, don't fold into the default-icon decision

Checked whether `ModType` (`Client`/`Server`/`Both`/`Unknown`) is still doing real work after 1.7's
manifest removal and C6's icon removal, since both of those turned out to be manifest-sourced or
purely decorative.

**It's neither.** `ModType` is derived structurally, not from manifest data: `infer_mod_type`
(`core/mod_fs.rs:69-79`) checks which of `client_plugins`/`server_mods` a mod's actual files live
under and classifies from that — it's computed from the same file-placement facts deployment itself
depends on, not something 1.7's manifest deletion touches at all. The contract already documents
that this is real, retained logic: `frontend-redesign-data-api-contract.md:78-80` — "Still used...
Backend-side it's still real logic, not vestigial manifest [data]" — and lists two live consumers:
the mod-grid toolbar's type filter, and each mod card's category icon tint.

**Decision: keep `ModType`, don't drop it.** It isn't in the same category as `iconDataUrl` (C6) —
that was an orphaned per-mod image with no source and no display target. `ModType` has both: a
real source (file-placement inference) and a real target (filter + category icon tint). Dropping it
would also remove the type filter, which nothing asked for. The "default icon for all mods" framing
doesn't apply here since the category icon *is* the default-icon mechanism C6 already settled on —
it's tinted by `ModType`, not replaced by dropping `ModType`.

## C15. Fire-and-track completion signaling: client-owned task ids over a central event bus

Settles finding 13's staleness question and gives the concrete correlation mechanism the "quick vs.
heavy job" split (C1) needed but didn't specify: how the frontend knows which completion event
belongs to which submitted action, without adopting R.2's per-library job queue.

**The id is frontend-owned, not backend-minted.** The frontend generates the `taskId` (uuid) and
includes it in the command call itself (e.g. `bulk_update_mods(taskId, ...)`), rather than waiting
for an accept response to hand one back. Reason: the accept response is itself async, and waiting
for it before registering interest in the id reopens exactly the race this is trying to close — a
fast backend task could complete before that round-trip resolves.

**A single, persistent event bus — not per-task `listen()` calls — is what actually closes the
race.** The bus is initialized once at startup, as part of the same startup sequence C11 already
covers (`init` or its eventual replacement), and stays subscribed to one completion event for the
app's lifetime. An optimistic updater registers its handler (the canceller/rollback for its
optimistic UI change) in the bus's `taskId → handler` map *synchronously, before* the command
invoke goes out — so the handler exists before the request is even sent, regardless of how fast the
backend responds. Client id ownership alone doesn't guarantee this; a fresh `listen()` per task
would still race. The bus is the actual fix; the id is what makes the bus's dispatch table work.

**Two-phase signal, not one:**
- The command's own invoke `Result` is the *accept* signal — synchronous validation failures (no
  active library, bad input) reject immediately, with no `taskId` ceremony at all. A request that
  doesn't even want the library yet doesn't need to go through the task machinery to say so.
- The completion event, matched by `taskId` through the bus, is the *outcome* signal for requests
  that passed validation and were actually queued/executed.

**Backend rejects id reuse.** A `taskId` resubmitted while its original task is still in flight is
an error, not silently accepted or clobbered — the backend treats collision as a client bug to
surface, not something to paper over.

**Dropped events on a stale bus are an accepted edge case, not a bug to fix.** If the frontend
reloads, the bus and its dispatch map are wiped — any completion event that arrives afterward for a
`taskId` with no registered handler is silently dropped. No backend-side reconciliation, no
persisted pending-task list across reload. The accepted cost: a user who reloads mid-operation may
see a stale library view until their next explicit action (or workspace refetch) picks up the
now-completed change — narrow, self-correcting on the next real read, and simpler than building
reconciliation machinery for a reload-during-a-background-task window.
