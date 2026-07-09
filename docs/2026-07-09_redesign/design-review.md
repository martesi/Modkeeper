# Design Review — 2026-07-09 Redesign Docs vs. Current Codebase

Status: Review. Findings for the authors of `backend-redesign-spec.md`,
`frontend-redesign-data-api-contract.md`, and `frontend-redesign-spec.md` to resolve before
Phase 2 implementation starts.

Method: every doc in this folder was read against the actual `src-tauri/src/` and `src/` trees
(not against the 1.x audits' description of them). File/line references below are to the code as
of this review.

Verdict up front: the doc set is unusually rigorous about wire shapes, migration sequencing, and
file-by-file targets — that part holds up. Where it fails is one level lower: **the runtime model
that all of §8a's concurrency promises depend on is never specified**, and several load-bearing
claims contradict either each other or the code they claim to have verified. Findings 1–4 are
design-breaking and need spec changes, not implementation-time judgment calls. A proposed model
that resolves them coherently is at the end (§R).

---

## Critical — the spec contradicts itself or cannot work as written

### 1. The per-library operation guard rejects normal mod toggling

`backend-redesign-spec.md` §8a: `bulk_update_mods` acquires the per-library guard; an overlapping
call on the same library "fails immediately with `LibraryOperationInProgress`" — explicitly "not
queuing."

`frontend-redesign-spec.md` §9.3: a toggle is a one-element `bulk_update_mods`, and while it's
pending "the affected card shows a pending/disabled state" — *the affected card*. Every other
card stays interactive.

Put together: a user toggles mod A, then toggles mod B half a second later — a completely normal
interaction the UI deliberately permits — and the second call is rejected with an error. There is
no UI state in which rapid sequential toggles are prevented, and the contract's own rationale for
bulk-only toggling ("no redundant `set_mod_enabled`") guarantees toggles are frequent, small, and
overlapping. The guard as specified turns the most common interaction in the app into an error
path.

The two documents each look correct alone; they are incoherent together. Reject-on-overlap is the
right semantics for a second *rebuild* while one is running. It is the wrong semantics for
serializing small mutations.

**Fix:** replace the reject-guard with a per-library serial job queue (§R.2). Quick jobs
(toggle/enable/disable/delete) enqueue and run in order; each emits its own completion event.
Rejection is reserved for enqueueing a heavy job (rebuild, install) while an identical heavy job
is already queued or running. `LibraryOperationInProgress` survives but is only reachable for the
heavy-op duplicate case.

### 2. Workspace assembly makes the lazy migration eager — and the spec's own acceptance criterion catch it

`backend-redesign-spec.md` §8 "Assembling `get_library_workspace`": building the workspace means
"opening **every** known library's DB (running the lazy migration above for any that haven't been
touched yet)."

Same document, §15 acceptance criteria: "A library that is registered (`known_libraries` row
exists) but never activated has no `library.db` on disk — migration is lazy, not eager."

`get_library_workspace` is the replacement for `init` (§7 table) — it runs at every app startup.
Therefore on first startup of the new version, every registered library is opened and migrated.
The acceptance criterion is unsatisfiable as long as workspace assembly migrates. The §14 test
("a second fixture library that's never activated must NOT get a `library.db`") will fail against
a correct implementation of §8 — and an implementer will "fix" it by weakening whichever side they
read second.

There's a second, worse consequence hiding in the same paragraph: opening a library today is not a
cheap read. `Library::load` (`core/library.rs:67`) runs `version::fetch_and_validate` against the
game root and hard-fails on `UnsupportedSPTVersion` or a missing game directory. Assembly that
*opens* every library inherits every library's failure modes — one library on an unplugged drive
or behind a game update and the spec gives no answer for what `get_library_workspace` returns.

**Fix:** assembly never migrates and never *opens* in the `Library::load` sense. For each
registered library: if `library.db` exists, read it (read-only); if only `manifest.toml` exists,
read that (read-only — this is exactly what `get_known_library_summary` does today,
`core/library_service.rs:129`); if neither is reachable, report the library as unreachable
(finding 3). Migration stays where the spec's own migration section put it: inside
`activate_library` only.

### 3. The domain model has no representation for a library that can't be opened

Libraries live on disk, deliberately so ("a library's data travels with the library"). Disks get
unplugged, game roots get deleted, manifests get corrupted, SPT gets updated past the supported
range. The current code already meets all of these — and silently drops the library from the list
(`get_known_library_summary` logs and skips, `core/library_service.rs:141`) or refuses to open it
at all (`version::fetch_and_validate` failure aborts `Library::load`).

The new contract carries this blind spot forward: `LibrarySummary` has no health/availability
field, `LibraryWorkspace` has no partial-failure channel, and no `SError` variant distinguishes
"this library is temporarily unreachable" from "this request was invalid." A registry whose whole
point is durable, portable library registration cannot make a registered library silently vanish
from the UI because a drive letter changed.

The version-validation behavior is its own sub-problem: because `Library::load` hard-fails on an
unsupported SPT version, a game update can lock the user out of the mod manager *exactly when they
need it* (to disable/remove now-incompatible mods).

**Fix:** add availability to the contract:

```ts
export type LibrarySummary = {
  // ...existing fields...
  status: 'ready' | 'legacy' | 'unreachable' | 'corrupt' | 'unsupported_version'
  statusDetail?: string
}
```

`legacy` = registered, valid, not yet migrated (finding 2's read-only manifest path). Workspace
assembly always succeeds and reports per-library status; only `activate_library` on a non-`ready`/
non-`legacy` library returns an error. Version validation becomes advisory: it sets
`unsupported_version` status (blocking deploy-type operations if desired) instead of blocking
load/open.

### 4. The concurrency section redesigns locking without ever saying what happens to the current runtime model — whose OOP justification the same spec dissolves

Today the entire backend serializes on one `Arc<Mutex<Option<Library>>>`
(`core/registry.rs:14`). Every command funnels through `with_lib_arc(_mut)`
(`utils/thread.rs`), and the lock is held **across all filesystem I/O** — `add_mods` holds it for
the full extract/backup/copy of every archive (`commands/library.rs:43`). One in-memory `Library`
holds the mods map, the cache, and a dirty flag; there is no concept of a second open library.

§8a promises: three background operations per library that don't block reads, a per-library guard,
and `get_library_workspace` that reads *every* library concurrently with in-flight operations.
None of that is expressible in the current model, and the spec never says what replaces it. Is
there still an in-memory `Library` per library? Several at once? If in-memory state remains, a
"non-blocking read" during a background install either sees stale state or needs exactly the lock
§8a says isn't held. The spec answers the *DB* locking question in detail while leaving the
*process* state model — the thing that actually serializes this app today — unaddressed.

Meanwhile §4's decision table keeps `core/library.rs` "OOP (justified)" because it "encapsulates
dirty-flag, cache, paths." But this same spec deletes each of those reasons: the dirty flag dies
with immediate-sync-on-toggle (§2 out-of-scope note + §7 `sync_mods` removal), and the cache and
mods map move into `library.db` (§8). The justification is quoted from a 2026-05-10 audit of a
design this document replaces. After Phase 2, `Library` as an encapsulated-mutable-state object
has nothing left to encapsulate.

**Fix:** specify the runtime model explicitly — see §R.1. Short version: `library.db` becomes the
*only* authority for mod/tool/cache state (no in-memory mirror), `Library` shrinks to an immutable
handle (id + resolved paths), and the registry becomes a map of open handles plus one job worker
per library. Reads never take a long-lived lock because there is no in-memory mutable state to
protect — SQLite's own short write transactions (§8a's commit phase) are the only serialization.

---

## High — wrong or unverified claims that will surface during implementation

### 5. `ModSummary.iconDataUrl` survives in the contract, but the redesign deletes its only source — and the redesigned UI never displays it

The only mechanism that produces a mod icon today: `manifest.icon` (a filename inside the mod's
manifest, `models/mod_dto.rs:73`) → `dto_builder::build_frontend_dto` → `utils/icon.rs::
load_icon_as_data_uri`. Audit 1.7 deletes the manifest system; §4's table deletes `dto_builder.rs`.
After Phase 2 there is no way to know which file in a mod is its icon. The Library DB schema
hand-waves this — `mods.icon_data_url` is annotated "backend-derived from the mod's own files
during install/rebuild" — but no rule for that derivation exists anywhere in the doc set.

And it doesn't need to: the redesigned mod card is title-only with a *category* icon tinted by
`ModType` (`frontend-redesign-spec.md` §9.3) — no per-mod image anywhere in the new UI.

**Fix:** drop `iconDataUrl` from `ModSummary` and `icon_data_url` from the `mods` table. If mod
icons return later, they return with a defined derivation rule. (Tools keep theirs — that flow is
fully specified in §9.) Consider `sourcePath` next to it: contract §8 resolves to keep it "for
display and opener actions," but no redesigned screen displays a mod source path either.



### 6. Library identity gets minted twice

Libraries already have stable UUIDs: `Library::create` mints one and persists it in
`manifest.toml` (`core/library.rs:50`), and `find_library_by_id` resolves by it today. The App
Config migration (§8) nevertheless says: "mint a new stable `LibraryId` (`uuid`)" for each path
transcribed from confy — without reading the manifest sitting at that path. The Library DB then
stores `library_meta.library_id` which "mirrors the App Config's known_libraries.id."

So after migration, a library's own files self-describe with a freshly minted ID while its legacy
manifest carries the original one. Now run the spec's own re-adoption flow: `delete_library
(deleteFiles: false)` removes the registry row and leaves `library.db` on disk "so re-adding the
same gameRoot later picks it back up" (§7). Picks it back up under which ID — the one in
`library_meta`, or a third newly minted one? The spec establishes two sources of identity and
never says which wins.

**Fix:** one rule: **the library's own ID is authoritative.** App Config's `known_libraries.id` is
a cached copy of it, never the origin. Migration reads the existing manifest `id` (the manifest is
right there — migration already knows the path); minting happens only when no readable identity
exists. Re-adding a directory that contains a `library.db` adopts `library_meta.library_id`.

### 7. Immediate-sync-on-toggle silently drops two safety behaviors and never states its cost

Current sync (`sync_mods`) is: full `cleanup::purge` (walk the entire `BepInEx/plugins` +
`SPT/user/mods` trees removing our symlinks) + full redeploy + collision check — and it is gated by
`is_game_or_server_running` (`commands/library.rs:90`). The redesign routes toggling through
`bulk_update_mods`, which "must commit to disk immediately," and:

- **The `GameOrServerRunning` guard appears nowhere in the new spec.** Not in §7's table, §8a, or
  §15. Creating/removing symlinks under a running game is exactly the case this guard exists for.
  Today it protects `sync_mods` and `remove_library`; the redesign removes `sync_mods` and never
  re-attaches the check to the operations that inherit its filesystem effects
  (`bulk_update_mods`, `install_mod_archives`, `delete_library`, `rebuild_library_cache`'s
  normalize step — see finding 8).
- **Collision handling on toggle is unspecified.** `deployment::deploy` fails with `FileCollision`
  when two enabled mods provide the same file. Under commit-on-toggle, enabling a conflicting mod
  must fail *that toggle* — representable via the completion event's per-mod `failures`, but the
  spec never says so, and never says whether a mid-batch collision rolls back the earlier mods in
  the same bulk call.
- **The cost model is unstated.** If "commit" means today's purge-everything-redeploy-everything,
  every single toggle is a full walk of the game root — the expensive shape §8a carefully avoids
  for rebuild's scan. `find_mod_links` (`core/deployment.rs:193`) already computes exactly the
  incremental link/unlink set for one mod; the spec should require incremental commit for toggles
  and reserve full purge+deploy for rebuild.

### 8. §8a's "scan phase touches no state" claim is false against the code it describes

§8a: "Walking the game root/mod folders and hashing files (the slow part) is pure filesystem read
+ in-memory compute. It can't block or conflict with anything else."

The actual rebuild (`library_service::rebuild_library_cache`, `core/library_service.rs:181`) first
calls `normalize_mod_folders` (`core/cache.rs:29`) which **renames mod folders on disk** and then
**deletes those mods' backup directories**. That is not a pure read, and the mutation is in the
slow phase, not the commit. Two concrete consequences the spec must address rather than define
away:

- A concurrent `get_library_workspace` (explicitly *not* blocked by the guard) can return
  `installedPath`s that the in-flight rebuild just renamed.
- Renaming an enabled mod's folder dangles every game-dir symlink pointing into it. This is a
  latent bug **today** (rebuild persists but never re-syncs; links dangle until the next manual
  Sync) — and the redesign deletes the manual Sync that currently papers over it. The rebuild
  pipeline needs an explicit "re-link renamed enabled mods" step in its commit phase.

**Fix:** restate §8a honestly: scan = hash/inventory (pure); *reconcile* = normalize renames +
backup cleanup + re-link + DB commit, all inside the guarded, serialized portion. The
upsert-not-blind-replace guidance stands; the purity claim doesn't.

### 9. `create_simulation_game_root` ships in release builds and is invisible to the endpoint audit

`commands/test.rs` carries a doc comment saying "only available in debug builds," but there is no
`#[cfg(debug_assertions)]` anywhere: `commands.rs:3` exports the module unconditionally and
`lib.rs:47` registers it in `collect_commands!` unconditionally (only the comment says "debug
only"). Release builds expose an IPC command that writes dummy `EscapeFromTarkov.exe`/
`SPT.Server.exe` files and directory trees to an arbitrary caller-supplied path.

§7's command mapping table — the artifact §15 checks "no undocumented commands" against — doesn't
contain this command, so the audit that "verified against the current file, lines 18–62" for
`add_mods` missed an entire command file. **Fix:** cfg-gate it (module registration included) or
delete it, and add the row to §7 either way so the acceptance criterion is checkable.

---

## Medium — spec gaps that should be closed on paper before they're closed in code

### 10. The §4 decision table claims completeness it doesn't have

The table says all file targets "were checked against `src-tauri/src/` as it exists today," yet
classifies nothing for: `utils/icon.rs`, `utils/id.rs`, `utils/time.rs`, `utils/thread.rs`,
`core/mod_documentation.rs`, `commands/test.rs`, or anything under `models/`. Three of these are
directly load-bearing for the spec's own plans:

- `utils/icon.rs` is the existing implementation of exactly what §9's tool-icon section specifies
  — and it has every property §9 step 2–3 exists to prevent (extension-sniffed MIME, zero
  validation, no size cap). §9 says the codebase has no image handling; it does, and it's the
  wrong kind. It should be listed as refactor-or-replace, not be absent.
- `utils/thread.rs` (`with_lib_arc*`) is the current concurrency model (finding 4) and disappears
  or transforms under any honest §8a design.
- `core/mod_documentation.rs` dies with 1.7 but is only implied dead, never listed.

### 11. The init-timeout watchdog will kill the redesigned app

`lib.rs:116`: if the `init` command isn't called within 10 seconds of setup, the backend calls
`std::process::exit(1)`. §7 renames `init` → `get_library_workspace` and never mentions
`init_called`. Implement §7 as written and the app hard-exits ten seconds after every launch.

Also note what this watchdog *is*: a silent `process::exit` with no dialog, no frontend
notification, and (until Phase 4 lands a file sink) no persisted log — the purpose doc's "fatal
error on frontend might be eaten" complaint, implemented in the backend on purpose. Phase 4 should
own replacing it with a visible failure (error window / OS dialog), not just relocating the flag.

### 12. App Config write-path error handling is still unspecified — the spec only fixed the read path

§8's confy critique is about `load()` silently defaulting. The same file's `save()` is
`let _ = confy::store(...)` (`config/global.rs:27`) — a failed write of the entire library
registry is silently discarded today, and the spec's toml-direct replacement only specifies
read-side behavior. Specify the write side: atomic write (temp file + rename over), and a surfaced
`StoreError` on failure — this file is about to be the only copy of the known-libraries registry.

### 13. Fire-and-track has no staleness rule, and completion events are more expensive than they look

Two gaps in the event design:

- **Missed events.** If the webview reloads (dev hot-reload, error recovery) between
  `OperationAccepted` and the completion event, the UI holds pending state forever — nothing in
  either frontend doc says pending state must be reconciled on the next workspace load. One
  sentence fixes it: any `get_library_workspace` response (or any completion event for that
  library) clears all local pending state for that library.
- **Event payload cost.** Every completion event carries a full `LibraryWorkspace`, and per §8,
  assembling one means reading every known library. Combined with finding 1's queue, a burst of
  toggles produces N full workspace assemblies. With finding 2's fix (read-only, no migration, no
  version fetch) this is probably tolerable; without it, it's N × (open + migrate + validate) per
  toggle burst. Worth a stated decision either way — e.g. completion events could carry only the
  affected library's slice plus `activeLibraryId`.

### 14. Small contract/spec drift, collected

- `settings-repository.ts` (`frontend-redesign-spec.md` §7) declares `loadSettings(): Settings` —
  synchronous. The command it must eventually wrap is `get_settings(): Promise<AppSettings>`. The
  signature can't survive the swap it was designed to survive; make it async now.
- `frontend-redesign-spec.md` §7's `WorkspaceEvent` omits `cacheStatus` on
  `cache_rebuild_completed`, omits `action` on `bulk_update_completed`, and types `failures` as
  `unknown[]` — all present and typed in the contract §6. For docs whose stated virtue is
  "field names already match," they should match each other.
- The `errorText` lookup rule "decapitalize the variant name" breaks on acronym-leading variants:
  `IOError` → `iOError`. Specify a real mapping (explicit table keyed by exact `code` string) —
  it's the fallback-proofed lookup anyway, so exactness costs nothing.
- The comment-removal policy (purpose doc, §10) should exempt constraint comments that encode
  non-obvious platform behavior. Concrete example: `linker.rs:61`'s explanation of why Windows
  directory symlinks need `remove_dir`-then-`remove_file` fallback — delete that comment and the
  next refactor "simplifies" it into a bug no test on a dev machine with symlinks enabled will
  catch.
- `SError` gains `Deserialize`-relevant shape questions under `#[serde(tag, content)]`: multi-field
  tuple variants (`InvalidLibrary(String, String)`) serialize `data` as a positional array. The
  frontend already special-cases this destructuring today (`src/lib/error.ts`). Fine — but the
  contract's `data?: unknown` note should say "arrays for multi-field variants" so `errorText`
  interpolation code is written against the real shape.

---

## §R. Proposed runtime model — the piece the spec is missing

This resolves findings 1–4 with one coherent design instead of four patches. It changes no wire
contract except the additions already argued above (`LibrarySummary.status`, `iconDataUrl`
removal).

### R.1 SQLite is the only state authority; `Library` becomes a handle

After a library's cutover (§8 Rollout step 5), there is **no in-memory mirror** of mods, tools,
cache, or enabled-state. The `Library` struct is replaced by an immutable handle:

```rust
pub struct LibraryHandle {
    pub id: LibraryId,
    pub library_root: Utf8PathBuf,   // where library.db lives
    pub game_root: Utf8PathBuf,
    pub spt_rules: SPTPathRules,     // derived, immutable
    // no mods map, no cache, no dirty flag, no &mut methods
}
```

`AppRegistry` becomes:

```rust
pub struct AppRegistry {
    pub app_config: Mutex<AppConfig>,            // small, in-memory, write-through
    pub libraries: Mutex<HashMap<LibraryId, Arc<LibraryRuntime>>>,
}

pub struct LibraryRuntime {
    pub handle: LibraryHandle,
    pub jobs: JobQueue,              // R.2
}
```

Reads (`get_library_workspace`, any query) open a connection, read, close — no app-level lock is
ever held across I/O, because there is no shared mutable in-memory state to protect. This is what
makes §8a's "reads are never blocked" true *by construction* instead of by a lock-discipline rule
someone has to remember. It also deletes the entire `with_lib_arc*` pattern and the
`spawn_blocking`-holding-a-mutex shape that every current command copies (19 occurrences across
`commands/`).

The §4 decision table entry for `core/library.rs` changes from "OOP (justified)" to "dissolved:
DTO-style handle + free functions" — which is also more honest to the spec's own FP-by-default
rule, since the justification it cites no longer exists post-§8.

### R.2 A per-library serial job queue replaces the reject-guard

Each `LibraryRuntime` owns one background worker (an `mpsc` channel + one `spawn_blocking` loop,
or equivalent). All mutating operations are jobs:

- **Quick jobs** — toggle/enable/disable/delete batches, tool upserts if desired — are always
  accepted and run strictly in order. Rapid sequential toggles just queue; each emits its own
  completion event. Finding 1 disappears.
- **Heavy jobs** — `rebuild_library_cache`, `install_mod_archives` — are accepted unless an
  identical heavy job is already queued/running for that library, in which case
  `LibraryOperationInProgress` (kept, now reachable only where rejection is actually the right
  UX).
- Every FS-mutating job re-checks `GameOrServerRunning` at execution time (not just submission
  time) — restoring the guard finding 7 shows was dropped, and putting it at the only point where
  it can't be raced.
- The queue is drained by one worker per library, so mutations on the same library are serialized
  without any lock held during reads, and operations on different libraries are naturally
  concurrent. The §8a upsert-not-replace commit guidance stays as crash-backstop, unchanged.

`OperationAccepted` semantics are unchanged: promise resolves when the job is validated and
enqueued; the completion event reports the outcome.

### R.3 Toggle commits are incremental; rebuild owns the full resync

- `bulk_update_mods` commit = `find_mod_links`-based unlink of disabled/deleted mods + targeted
  link of enabled mods + collision check scoped to the change (a new enabled mod colliding with the
  currently-enabled set fails *that mod* into the event's `failures`, other mods in the batch
  proceed).
- `rebuild_library_cache` = scan (pure inventory/hash, honestly pure this time) → reconcile
  (normalize renames, re-link renamed enabled mods, backup-dir cleanup, one short DB transaction).
  Finding 8's dangling-symlink bug gets fixed by the re-link step rather than inherited.

### R.4 Assembly is read-only; migration happens exactly once, in `activate_library`

Per finding 2/3: `get_library_workspace` reads `library.db` where present, reads `manifest.toml`
(read-only) for `legacy` libraries, and reports `unreachable`/`corrupt`/`unsupported_version`
status instead of skipping or failing. No migration, no `version::fetch_and_validate`, no game-root
touch during assembly. `activate_library` is the only migration trigger, exactly as §8's migration
section already says — the assembly paragraph is what changes.

### R.5 Identity: the library speaks for itself

`library_meta.library_id` (or legacy `manifest.toml` `id`) is authoritative; App Config caches it.
Migration transcribes, never mints, unless nothing readable exists. Re-adding a directory adopts
the identity found inside it. One rule, no reconciliation cases.

---

## Priority order for resolving

| # | Finding | Where the fix lands |
|---|---------|--------------------------------|
| 1 | Guard rejects rapid toggles | backend-spec §8a (replace with R.2), frontend-spec §9.3 note |
| 2 | Eager migration via assembly | backend-spec §8 assembly paragraph (R.4) |
| 3 | No unavailable-library model | contract §5 (`LibrarySummary.status`), backend-spec §8 |
| 4 | Runtime model unspecified / stale OOP justification | backend-spec new section (R.1), §4 table |
| 5 | Mod `iconDataUrl` is sourceless and unused | contract §5/§7, backend-spec §8 schema |
| 6 | Double-minted library identity | backend-spec §8 migration (R.5) |
| 7 | Dropped game-running guard, unspecified toggle cost/collisions | backend-spec §7/§8a (R.2, R.3) |
| 8 | Scan-phase purity claim false; rebuild dangles symlinks | backend-spec §8a (R.3) |
| 9 | `create_simulation_game_root` in release + undocumented | code (`cfg`-gate) + backend-spec §7 row |
| 10 | Unaudited files in §4 table | backend-spec §4 |
| 11 | Init watchdog kills renamed app | backend-spec §7 `init` row + Phase 4 |
| 12 | Config save errors swallowed | backend-spec §8 Crate Choice |
| 13 | Event staleness + payload cost | contract §6, frontend-spec §7 |
| 14 | Assorted drift (sync `loadSettings`, event shapes, decapitalize rule, comment policy, `data` arrays) | respective docs |
