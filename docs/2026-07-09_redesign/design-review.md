# Design Review — 2026-07-09 Redesign Docs vs. Current Codebase (Consolidated)

Status: Resolved. This document consolidates the full review cycle into one place:

1. The original review — findings 1–14 plus the §R runtime-model proposal (Part 1 below, kept
   verbatim with a resolution pointer per finding).
2. The author clarifications C1–C15/T1 (formerly `design-review-mod.md`).
3. The review of those clarifications, M1–M10 (formerly `design-review-mod-review.md`).
4. The final resolutions of M1–M10 (formerly `design-review-mod2.md`).

The three follow-up documents are superseded by this one and removed. Every M-correction is folded
directly into its C-section in Part 2, so Part 2 is the authoritative, final form of each decision —
there is no separate correction layer left to cross-reference. These decisions are reflected in
`backend-redesign-spec.md`, `frontend-redesign-data-api-contract.md`, `frontend-redesign-spec.md`,
and `outline-of-redesign.md` for Phase 2/3 implementation. Per C4/M4, `purpose-of-redesign.md` is
deliberately **not** edited — its SQLite section reads from here forward as the historical case
*for* that proposal, and `backend-redesign-spec.md` §3 records the deviation.

Method (original review): every doc in this folder was read against the actual `src-tauri/src/` and
`src/` trees. Method (clarification review): every code citation in the clarifications was
re-verified against the code, and every decision checked for consistency against the doc set.

---

# Part 1 — Original Findings

Verdict up front (as originally written): the doc set is unusually rigorous about wire shapes,
migration sequencing, and file-by-file targets — that part holds up. Where it fails is one level
lower: **the runtime model that all of §8a's concurrency promises depend on is never specified**,
and several load-bearing claims contradict either each other or the code they claim to have
verified.

Note on reading Part 1: each finding's original **Fix:** paragraph is the reviewer's first
proposal, kept for the record. The *final* decision is the Resolution pointer under each finding —
several proposals (notably §R.1/R.2's job-queue runtime and the `LibrarySummary.status` enum) were
**not** adopted.

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

**Fix (original proposal):** replace the reject-guard with a per-library serial job queue (§R.2).

> **Resolution: C1.** The reject-guard is dropped, but not in favor of R.2's job queue — the
> existing `Arc<Mutex<Option<Library>>>` model is kept and overlapping calls simply block on the
> mutex. Fire-and-track mutations are scoped to the active library.

### 2. Workspace assembly makes the lazy migration eager — and the spec's own acceptance criterion catches it

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

**Fix (original proposal):** assembly never migrates and never *opens* in the `Library::load`
sense; read-only manifest reads, report unreachable libraries per finding 3, migration only in
`activate_library` (§R.4).

> **Resolution: C13 (and C4).** The SQLite migration this finding's eager-migration risk was
> specific to is cancelled outright (C4). Assembly was already read-only against `manifest.toml`
> in the current code; the one real gap — what happens when the manifest read itself fails — is
> answered by C13's path-only stub, not by a status enum.

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

**Fix (original proposal):** add a `status: 'ready' | 'legacy' | 'unreachable' | 'corrupt' |
'unsupported_version'` field (+ `statusDetail`) to `LibrarySummary`; version validation becomes
advisory.

> **Resolution: C5 + C13.** The full status enum is not adopted. Unreadable libraries surface as a
> path-only stub object in the known-library list (C13); the error surface at open/activate time
> reuses `SError::InvalidLibrary` for corrupt libraries and keeps `UnsupportedSPTVersion` a
> hard-fail on activate (C5). Recovery is manual and coarse by design.

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

**Fix (original proposal):** §R.1 — `library.db` becomes the only state authority, `Library`
shrinks to an immutable handle, per-library job workers.

> **Resolution: C1 + C3 + C4.** R.1/R.2 are rejected. The in-memory `Library` and its mutex stay;
> the dirty flag survives (C3 keeps deployment as an explicit step, so `is_dirty` remains
> load-bearing); the SQLite migration is cancelled (C4), so `core/library.rs`'s OOP justification
> holds again. The real costs of the kept model (reads block behind in-flight mutations, unordered
> mutex) are stated explicitly in C1 instead of being hidden.

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

> **Resolution: C6.** Adopted directly (with M9's correction that this deletes a *live* pipeline,
> not a dead field — a visible, accepted UX regression). `sourcePath` stays per contract §8.

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
a cached copy of it, never the origin. Migration reads the existing manifest `id`; minting happens
only when no readable identity exists. Re-adding a directory adopts the identity found inside it.

> **Resolution: C7.** Adopted — and simpler than proposed, since C4 cancels `library.db`, leaving
> `manifest.toml`'s `id` as the only identity that exists.

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

> **Resolution: C2 + C3.** The premise is corrected upstream: commit-on-toggle is not the model
> being built. Toggles persist *mod state* only (cheap, always committed); deployment stays a
> distinct, explicit, user-triggered `sync_mods` step, so the cost and collision questions answer
> themselves (collisions surface at deploy time, as today). The `GameOrServerRunning` guard is
> deliberately dropped as an accepted, documented risk (C2) — a cross-platform detection problem,
> not a timing problem.

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

**Fix (original proposal):** restate §8a honestly (scan pure, reconcile guarded); add a re-link
step to rebuild's commit phase.

> **Resolution: C8.** Rebuild stays record-only: no re-link step is added. A rebuild rename
> dangles **all** of the renamed mod's links (per M1's corrected topology) until the user runs the
> explicit deploy step — an accepted, stated cost, recoverable via rebuild-then-sync (C3's model,
> whose Sync button survives per C3/M5). The backup-deletion coupling is removed (C8).

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

> **Resolution: C9.** Deleted outright (not cfg-gated), along with `models/test.rs` and the
> frontend panel built on it — see C9 for the §3-preservation-rule exception this required.

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

> **Resolution: C10.** All seven classified (with M10's correction on the `utils/icon.rs` refactor's
> input-contract change). `utils/thread.rs` stays unchanged since C1 keeps the lock model.

### 11. The init-timeout watchdog will kill the redesigned app

`lib.rs:116`: if the `init` command isn't called within 10 seconds of setup, the backend calls
`std::process::exit(1)`. §7 renames `init` → `get_library_workspace` and never mentions
`init_called`. Implement §7 as written and the app hard-exits ten seconds after every launch.

Also note what this watchdog *is*: a silent `process::exit` with no dialog, no frontend
notification, and (until Phase 4 lands a file sink) no persisted log — the purpose doc's "fatal
error on frontend might be eaten" complaint, implemented in the backend on purpose. Phase 4 should
own replacing it with a visible failure (error window / OS dialog), not just relocating the flag.

> **Resolution: C11.** Adopted — the `init_called.store` call moves with whatever command becomes
> the startup call, as an explicit §7 line item; the silent exit becomes a visible failure in
> Phase 4.

### 12. App Config write-path error handling is still unspecified — the spec only fixed the read path

§8's confy critique is about `load()` silently defaulting. The same file's `save()` is
`let _ = confy::store(...)` (`config/global.rs:27`) — a failed write of the entire library
registry is silently discarded today, and the spec's toml-direct replacement only specifies
read-side behavior. Specify the write side: atomic write (temp file + rename over), and a surfaced
`StoreError` on failure — this file is about to be the only copy of the known-libraries registry.

> **Resolution: C12.** Adopted, with M7's corrections (free function, not `FileUtils::`; the
> `lib.rs` startup call site gets its own handling).

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

> **Resolution: C15.** Client-minted `taskId` + a single persistent event bus, with dropped events
> after a reload accepted as a self-correcting edge case. Assembly cost is re-grounded by C4
> (read N manifest/cache file pairs, no migration, no version fetch).

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

> **Resolution:** the `loadSettings` sync-signature point is resolved structurally by T1
> (settings become backend-owned and async by construction). The remaining bullets stand as
> written and land as small edits in their respective docs.

---

## §R. Proposed runtime model — status: NOT adopted (except R.5)

This section proposed resolving findings 1–4 with one coherent design: SQLite as the only state
authority with `Library` reduced to an immutable handle (R.1), a per-library serial job queue
replacing the reject-guard (R.2), incremental toggle commits (R.3), read-only assembly with
migration only in `activate_library` (R.4), and library-self-described identity (R.5).

Final status per the clarification round:

- **R.1 — rejected** (C1, C4). The `Arc<Mutex<Option<Library>>>` model and in-memory `Library`
  stay; the SQLite migration is cancelled, which also restores `core/library.rs`'s OOP
  justification.
- **R.2 — rejected** (C1, C15/M6). No job queue; overlapping mutations block on the mutex. The
  reject-guard is dropped, and the only new backend bookkeeping is C15's in-flight `taskId`
  registry — a map alongside the mutex, not a queue replacing it.
- **R.3 — superseded** (C3). There is no commit-on-toggle to make incremental: toggles persist mod
  state only, and deployment stays a separate explicit step. The re-link-on-rebuild step is
  likewise not added (C8).
- **R.4 — moot** (C4, C13). No `library.db` exists to migrate; assembly is read-only manifest
  reads with a path-only stub for unreadable libraries.
- **R.5 — adopted** (C7), simplified: `manifest.toml`'s `id` is the single identity source; no
  `library_meta` half exists after C4.

The original R.1–R.5 text is preserved in git history (`design-review.md` prior to this
consolidation) if the rejected design needs revisiting — C4 explicitly names the conditions that
would strengthen the SQLite/queue case again.

---

# Part 2 — Final Resolutions (C1–C15, T1)

Authoritative form of each decision, with the M1–M10 corrections folded in. Where a section below
disagrees with anything in Part 1, this part wins.

## C1. `bulk_update_mods` keeps the current lock model — R.1/R.2 not adopted *(incorporates M2)*

`bulk_update_mods` is not being redesigned onto a new runtime. It runs as a single command call
that does its series of work (validate → mutate in-memory `Library` → persist) under the existing
`Arc<Mutex<Option<Library>>>` (`core/registry.rs:14`), the same `with_lib_arc_mut` pattern already
in use (`utils/thread.rs:6`). The reject-guard finding 1 flags (`LibraryOperationInProgress`,
"not queuing") is a separate mechanism layered on top of locking — it is dropped, and overlapping
calls simply block on `Mutex::lock()` like every other mutating command does today.

**Scope: fire-and-track writes operate on the active library only.** The mutex only ever holds the
*active* library — it structurally cannot serialize writes to a non-active one. Rather than invent
a second serialization mechanism for a path nothing needs concurrent-safe today,
`bulk_update_mods`, `install_mod_archives`, `rebuild_library_cache`, and `sync_mods` (C3) validate
`libraryId === activeLibraryId` and reject otherwise with a plain validation error, before touching
anything. The contract's `libraryId` field on these calls is kept, but its purpose is corrected:
it's an assertion input for that validation and the id the completion event reports against — not
a key into a per-library operation guard (that guard no longer exists). `rename_library`'s
existing optional-non-active-`library_id` path keeps its current, already-racy
last-`persist()`-wins behavior unchanged — a pre-existing characteristic of today's code, not a
new regression, and the UI only ever fires one such call at a time per library, so it's recorded
as out of scope rather than silently inherited as a promise.

**The real ordering guarantee.** `parking_lot::Mutex` doesn't guarantee FIFO — two overlapping
calls on the *same* active-library mod resolve to whichever acquires the lock last, order not
guaranteed to match submission order. This is acceptable specifically because `bulk_update_mods`
toggles are absolute `is_active: bool` sets, not increments (`core/mod_manager.rs:83`) — the
outcome of a race is always one of the two states the user actually asked for, never a corrupted
third state. If a future change makes ordering matter (e.g. a delta-style operation), this
guarantee stops being sufficient and needs an actual queue at that point — not before.

**Reads block behind in-flight active-library mutations — stated cost, not hidden.** Heavy
operations (`install_mod_archives`, `rebuild_library_cache`, `sync_mods`) hold the mutex for their
full filesystem duration, so `get_library_workspace` (or its eventual replacement) blocks for that
same duration when called against the active library. Metadata-only operations (toggle) block
reads for a negligible interval. §8a's former "the guard does not block `get_library_workspace` or
any other read" promise and the contract's non-blocking language are corrected to state this
plainly — an accepted cost of keeping the single-mutex model (C4), not a hidden regression.

**Open, tracked separately:** the *boilerplate* cost of this model — every command clones an
`Arc`, locks it, and wraps work in `spawn_blocking` (~19 call sites). Already called out by the
1.9 backend audit as its own helper-ergonomics problem; not solved here, stays open.

## C2. `GameOrServerRunning` guard is deliberately dropped, not an oversight

The guard is dropped on purpose, and the reason is narrower than "TOCTOU in general" — it's a
cross-platform detection problem. Today's check (`utils/process.rs::ProcessChecker::is_running`)
matches a process's canonical exe path (`p.exe()`) against known canonical target paths
(`core/registry.rs:32` `is_game_or_server_running`). That's stable on Windows because the
game/server ships as a `.exe` with a fixed, resolvable path. It isn't stable the same way on
Linux: the server is commonly launched through a `.sh` script that execs the real binary (or runs
it under a wrapper), so there's no single canonical exe path to match — the same detection
strategy doesn't generalize across platforms.

Detection isn't impossible on Linux — matching on process *name* could find a running instance —
but that's a materially different, less precise mechanism (name collisions, truncated `comm`
values, no guarantee the script's process name matches the binary's), not a drop-in fix.
Re-checking at execution time (§R.2's proposal) doesn't address this at all — the gap isn't *when*
the check runs, it's that the check has no cross-platform-consistent implementation yet.

So: the guard is dropped for now as a deliberate scope cut. It's open for revisiting once a
name-based (or otherwise platform-agnostic) detection strategy is specified and shown consistent
on both platforms. Until then, no guard ships, and the spec states that as an accepted risk rather
than leaving an implicit gap.

## C3. Deployment is decoupled from mod-state edits; the Sync button stays *(incorporates M5)*

The dirty flag (`Library::is_dirty`, `core/library.rs:25`) stays, and its meaning is narrower than
"unsaved change" — it marks that the *deployed* state (the symlinks in the game/SPT mod
directories) no longer matches the *recorded* mod state (which mods are enabled). A toggle always
saves immediately; it just doesn't always deploy immediately.

This matches the current implementation exactly:

- `mod_manager::toggle_mod` (`core/mod_manager.rs:83-92`) flips `is_active` in memory, calls
  `mark_dirty()`, then `library.persist()` — writes the manifest/cache TOML
  (`core/library.rs:135-139`). Cheap: no filesystem walk, no symlink creation/removal, no
  collision check. A toggle's state is never lost regardless of batch size or speed.
- `Library::sync` (`core/library.rs:143-149`) is the separate, explicit operation doing the
  expensive work: `cleanup::purge` → `deployment::deploy` (where the collision check lives) →
  `mark_clean` → `persist`. This is what the `sync_mods` command wraps (`commands/library.rs:89`).

So the model: **edits to mod state are cheap and always committed; deployment (the expensive,
collision-checked, symlink-touching part) is a distinct, user-triggered step.** A user can make a
large-scale change without paying deployment's cost per-edit, and never risks losing what they
changed. Cost model: answered — toggling never runs purge+redeploy. Collision handling: answered —
collisions surface at deploy time, as today.

**The Sync button is NOT removed by this redesign** (reversing the backend spec's prior acceptance
of `1.9_backend-redesign-audit.md` §2.B). Full change set, per M5:

- `backend-redesign-spec.md` §2 Out of Scope's line accepting §2.B ("remove the manual Sync
  button, commit on toggle") is struck. That redefinition is reversed, not implemented.
- Contract §6 gains a **fourth fire-and-track operation**, keeping the existing command name and
  guard posture (no `GameOrServerRunning` check, per C2):

  ```ts
  sync_mods(input: { taskId: string; libraryId: LibraryId }): Promise<OperationAccepted>
  // Fire-and-track (fourth operation) — completion via listen_workspace_event's
  // 'sync_completed'. Walks and relinks the whole tree, same cost profile as
  // rebuild_library_cache, which is why it qualifies under the "touches
  // potentially many files" criterion alongside the other three.
  ```

  "Non-Blocking Operations" is corrected from "exactly three" to four, and a matching
  `WorkspaceEvent` variant (`sync_completed`) is added.
- **Deploy-staleness gets its own field, not an overload of `cacheStatus`.** `cacheStatus`
  describes the cache/manifest rebuild machinery — a different concern from whether deployed
  symlinks match recorded mod state. `LibrarySummary` gains `deployStale: boolean` — `true`
  exactly when `Library::is_dirty` is true today. The Sync button reads this field directly:
  highlighted/accented when `deployStale`, quiet otherwise.
- **The button's screen:** `library-execution-bar.tsx` — deploy state is an execution concern
  (does the running game match the library), not a mod-browsing concern, so it belongs next to
  other execution-bar actions, not in `mod-grid-toolbar.tsx`.
- **`bulk_update_mods` stays fire-and-track for all three actions, restated reason:**
  enable/disable is now metadata-only (cheap enough to be a fast blocking call on its own), but
  `delete` still unlinks and removes files, and a single command with three actions is kept
  fire-and-track uniformly for one predictable client-side handling path — consistency, not
  per-action cost, is the reason now that the actions' underlying costs diverge.

This corrects a premise upstream of finding 7: `frontend-redesign-spec.md` §9.3's "must commit to
disk immediately" and finding 4's "`sync_mods` removal" both described a model that isn't the one
being built. `bulk_update_mods` commits *mod state* immediately; it does not deploy.

## C4. The SQLite migration is cancelled for this redesign *(incorporates M4)*

`backend-redesign-spec.md` §8 proposed replacing each library's `manifest.toml`/`cache.toml` with
a per-library `library.db` (SQLite via `rusqlite` + `rusqlite_migration`). Reviewed against what
the docs claim versus what the redesign needs:

- **The stated justifications don't specifically require SQL.** "Two files into one" is satisfied
  by any single structured file. "Sophistication" (mods/tools/cache as distinct concerns) argues
  for schema/structure, which sectioned TOML provides equally well. Nothing in
  `get_library_workspace`'s access pattern exercises query/join capability — it reads whole tables
  per library, and derived counts are pushed to the frontend. The one genuine SQL win — atomic,
  crash-safe per-row commits — is real but narrow, and (per C3) toggle-time writes are already
  metadata-only and cheap, so the write-amplification framing overstates the problem at this app's
  realistic scale.
- **§8a's concurrency argument leaned on a guarantee the redesign isn't using.** Its case for
  "SQLite serializes writers regardless" assumed the in-process locking model was being replaced
  (R.1/R.2). C1 settled that it isn't — the `Arc<Mutex<Option<Library>>>` already serializes every
  write within this single-instance app; SQLite's writer arbitration would be redundant.
- **Costs are concrete and immediate:** a new dependency, the N-small-file-open cost at every
  workspace assembly, a full migration path to write and test, and a real loss of transparency —
  a library's state stops being user-openable in a text editor, cutting against "a library's data
  travels with the library."

**Decision: cancel the SQLite migration.** Library state stays in structured plain files
(`manifest.toml`/`cache.toml`; a consolidated single-file replacement remains a separate, smaller
decision). Revisit if the app's shape changes — larger library counts, genuine cross-entity query
needs, or a move off the in-process mutex would all strengthen the case again.

**Documentation of the deviation (M4):** `purpose-of-redesign.md` is **not** edited — its SQLite
section reads from here forward as the historical case for the proposal, not a live requirement.
Instead, `backend-redesign-spec.md` §3's precedence line ("If the two disagree,
`purpose-of-redesign.md` wins") carries an explicit note recording this cancellation as a
deliberate, called-out exception, pointing back to this section.

**Tool storage, decided (a blocker for §9, not a nicety):** tools persist in a **`tools.toml`
sibling file per library**, alongside `manifest.toml`/`cache.toml` — mirroring the existing
two-file split rather than growing `manifest.toml` with an unrelated concern. `core/tool_service.rs`
reads/writes it directly, the same way `library_service.rs` already reads/writes manifest/cache.

**Downstream effects (all applied):**

- `backend-redesign-spec.md` §8/§8a and the `store/` module layout: rewritten against plain files.
- `outline-of-redesign.md` Phase 2.5 (Library DB migration item dropped; App Config plain-file
  migration stays) and Phase 2.6 (`tools` table → `tools.toml`).
- `frontend-redesign-data-api-contract.md` §3's two-tier description and §7's Library DB SQL
  sketch: struck/replaced. The wire contract itself is unaffected — the split was always invisible
  to the frontend.
- Findings 2, 6, 8, 13's SQLite-specific mechanics: re-grounded against plain files (see their
  resolution pointers in Part 1).

## C5. Library open errors: reuse `InvalidLibrary` for corrupt libraries *(incorporates M8)*

Finding 3 flagged that no `SError` variant distinguishes "this library is broken" from "this
request was invalid," and that `get_known_library_summary` silently drops unreadable libraries
(`core/library_service.rs:133-136`). Decision: close the gap at the point it bites (opening/
activating), not with the full `LibrarySummary.status` proposal.

- **Unsupported version** already exists and already fires: `SError::UnsupportedSPTVersion(String)`
  (`models/error.rs:7`), raised by `version::fetch_and_validate` (`core/version.rs:76`) from both
  `Library::create` and `Library::load` (`core/library.rs:47,76`). Keeping it a hard-fail on
  `activate`/`open` is acceptable as the narrower fix — a user can't do anything useful with a
  library manager that claims to support an SPT version the library doesn't have.
- **Corrupt library: reuse `SError::InvalidLibrary(String, String)`, don't add `CorruptLibrary`.**
  `InvalidLibrary` already exists (`models/error.rs:22-23`) and `validate_library_structure`
  already returns it for exactly the corrupt-manifest case (`core/library_service.rs:31-36`). The
  structural-vs-unparseable distinction isn't one the frontend acts on differently yet (C13's
  stub model treats any unreadable library the same way), so a second variant would exist without
  a consumer. Split it out later if that distinction becomes load-bearing, with a stated reason.
- **Detection at `read_library_manifest` catches both read-path failures** the toml utilities
  produce — `IOError` (`utils/toml.rs:14`, unreadable file) and `ParseError` (`utils/toml.rs:15`,
  unparseable content) — both mapping to `InvalidLibrary` for this purpose. Not just `ParseError`.
- `get_known_library_summary`'s silent `filter_map` drop is superseded by C13's path-only stub.

## C6. Remove `iconDataUrl` from `ModSummary` — UI falls back to a category icon *(incorporates M9)*

Adopts finding 5's fix directly. The redesigned mod card is title-only with a category icon tinted
by `ModType` (`frontend-redesign-spec.md` §9.3), and 1.7's manifest removal deletes the icon's
source field — there's nothing left to display it anyway.

**Decision:** drop `iconDataUrl` from `ModSummary` and the `icon_data_url`/`icon_data` fields
backing it. **This deletes a live pipeline, not dead weight (M9):**
`dto_builder::build_frontend_dto` (`core/dto_builder.rs:14-21`) populates `icon_data` from
`manifest.icon` on every DTO build today. Mods that show a custom icon today will show the default
category icon after this change — that is the expected, accepted UX regression, not a bug to chase
during implementation. `utils/icon.rs` does not die with this change — see C10; it's still needed
for tool icons (§9), with a changed input contract.

## C7. Library identity: `manifest.toml`'s `id` is authoritative

Adopts finding 6's fix (R.5) directly — simpler now that C4 cancelled `library.db`, since there's
no `library_meta.library_id` to reconcile against. The only identity that exists is the one minted
in `core/library.rs:50` (`uuid::Uuid::new_v4().to_string()`, persisted in the manifest).

**Decision:** that id is the single source of truth. Wherever the App Config gains a stable
`known_libraries` registry, migrating an existing path into it must read the id already sitting in
that library's manifest, not mint a new one. Minting only happens for a library that genuinely has
no manifest yet. Re-adding a directory that still has a readable manifest adopts the id found
inside it. One rule, no reconciliation cases.

## C8. Rebuild cache: rename folders only; stop deleting backups; renames dangle every link *(incorporates M1)*

Finding 8 flagged that `rebuild_library_cache`'s folder-normalization step also deletes backup
directories for renamed mods (`core/library_service.rs:181-226`, `remove_all_backups` at line 193,
right after `normalize_mod_folders`, `core/cache.rs:29`).

**Decision:** remove the `remove_all_backups` call from that step. Folder normalization only does
what its name says — rename the folder, remove the old mod-map entry, re-key to the resolved id.
Backups for a renamed mod are left on disk under the old name; orphaned relative to the new id,
but nothing deletes data a user might still want. If orphaned backups ever need cleanup, that's a
separate, explicit, user-visible action.

**Dangling symlinks after rename are expected behavior — with the real blast radius stated (M1).**
`deployment::execute_recursive_link` (`core/deployment.rs:119-157`) never links `mods/<mod_id>` as
a whole — it links at the highest uniquely-owned path *inside* `mods/<mod_id>/...` per
game-relative path component, so one mod typically produces several links at varying depths, all
sourced from inside that mod's folder. Because every one of those links' source path is
`mods/<mod_id>/...`, renaming that folder invalidates the source path for **all** of that mod's
links simultaneously. A mod that produced five links at five depths loses all five the instant
rebuild renames its folder, and every one stays dangling in the live game/SPT directory until the
user runs the explicit deploy step. For a deployed, currently-running game, every file the mod
contributed can vanish from the game's view at once.

That is the accepted cost, stated plainly: rebuild only rewrites the library's own *record* — it
is not, and isn't meant to be, a fix for live game deployment. If a rename (or any on-disk change
made out from under a live deployment) leaves the running game broken, the recovery path is the
one C3 established: rebuild the cache, then run the explicit sync/deploy step.

## C9. Remove `create_simulation_game_root` entirely; no developer settings section exists *(incorporates M3)*

Finding 9 confirmed: no `#[cfg(debug_assertions)]` anywhere — `commands.rs:3` and `lib.rs`'s
`collect_commands!` both register the command unconditionally, so release builds ship an IPC
command that writes dummy exe files to a caller-supplied path.

**Decision: delete it, not cfg-gate it.** Removals:

- `commands/test.rs` and `pub mod test;` from `commands.rs:3`.
- The `create_simulation_game_root` import and `collect_commands!` entry in `lib.rs`.
- `models/test.rs` (already an empty stub).

**Frontend side (M3): no developer settings section ships anywhere — it was never actually part of
the UI design**, regardless of what `frontend-redesign-spec.md`'s text previously said:

- `settings/developer-settings-row.tsx` is struck from §4's file tree; §9.6's "Developer row"
  bullet and §5a's "surfaced as a row in the Settings developer section" clause are struck.
- The new/old UI toggle (§5a) still needs a home — its runtime-not-build-time constraint holds. It
  becomes a single utility control on the existing Settings screen's general row list, explicitly
  labeled as a transition-only control, deleted along with the rest of the legacy-UI toggle
  machinery once the redesign fully replaces the old UI.
- `src/modules/settings/developer-settings.tsx` (the Test Game Root panel) is **deleted outright**
  along with the backend command — an explicit, recorded exception to §3's "don't delete old
  feature files during the switch-over" rule for this one file. §3 protects files whose *feature*
  survives into the redesign in some form; this one's feature (a developer settings section)
  doesn't exist in the design at all, so there is nothing for the preservation rule to keep alive.
  (Deleting the backend command and re-running `export_types` would leave the old file
  uncompilable anyway — deletion is the resolution, chosen deliberately rather than by accident.)

## C10. Finding 10's file-classification gaps, filled *(incorporates M10)*

| File | Classification | Why |
|---|---|---|
| `utils/icon.rs` | **Refactor — the input contract changes.** | Loses its mod-icon caller (C6), but §9's tool-icon flow still needs the base64/data-URI capability. The refactor is not shape-preserving (M10): today it's path-in → extension-sniffed MIME → data-URI out (`utils/icon.rs:8-27`); §9's tool flow is base64 *bytes* in (read by the frontend, never a backend file read) → content-validated (the `image` crate decode §9 plans) → resized/size-capped → stored representation. MIME detection moves from extension-matching to content-sniffing. Don't "refactor" it by preserving the path-and-extension shape. |
| `utils/id.rs` | **Keep, unchanged.** | `hash_id` (blake3 + base64url) is the hash-based mod identification mechanism 1.7 standardizes on. |
| `utils/time.rs` | **Keep, unchanged.** | Single trivial helper (`get_unix_timestamp`). |
| `utils/thread.rs` | **Keep, unchanged.** | §R.1's "dissolved" reclassification is rejected with R.1 itself (C1) — `with_lib_arc`/`with_lib_arc_mut` stay exactly as they are. The boilerplate-reduction idea from the 1.9 audit stays open as a helper-ergonomics change, not a deletion. |
| `core/mod_documentation.rs` | **Delete.** | `read_documentation` reads `manifest.documentation`, deleted entirely by 1.7. |
| `commands/test.rs` | **Delete.** | Per C9. |
| `models/*` | **Classified individually.** | `models/mod_dto.rs` shrinks under 1.7 (manifest-derived fields drop, plus `icon_data` per C6); `models/error.rs` gains C15's `TaskIdInUse` and C12's save-failure variant (no `CorruptLibrary` — C5/M8); `models/global.rs`/`models/config.rs` evolve with the confy→toml migration (which C4 did not cancel); `models/test.rs` deletes per C9; `models/library.rs`, `models/paths.rs`, `models/mod_backup.rs` unaffected by anything decided here (`mod_backup.rs` still shrinks per backend spec §9a). |

## C11. The init-timeout watchdog must track whatever command actually loads the workspace

Current code is correct today: `commands/global.rs::init` (lines 159-166) is the only caller of
`state.init_called.store(true, ...)`, first thing in the command. The bug finding 11 describes is
conditional on the spec's `init` → `get_library_workspace` rename — if that rename happens, the
`store(true, ...)` call must move with it into the new command's body.

**Fix:** whichever command is the frontend's one startup/workspace-fetch call — renamed or not —
calls `init_called.store(true, Ordering::Relaxed)`, recorded as an explicit line item in
`backend-redesign-spec.md` §7's row for that command. Separately, `start_init_timeout_checker`
(`lib.rs:116-127`) responds to a timeout with a silent `std::process::exit(1)` — that becomes a
visible failure (error window or OS dialog) as part of Phase 4's logging work.

## C12. Surfaced, atomic App Config writes *(incorporates M7)*

Current state: `GlobalConfig::save` (`config/global.rs:26-28`) is
`let _ = confy::store("Modkeeper", CONFIG_NAME, self);` — any write failure is silently discarded,
and `save()` returns `()` so no call site can know. This matters more once the App Config carries
stable library ids (C7) and settings (T1) — a silently lost write means losing registry entries.
Unaffected by C4: the App Config was always staying a plain file, and its confy→toml migration is
separate from the cancelled Library DB.

**Plan:**

1. Add a dedicated error variant, e.g. `SError::ConfigSaveFailed(String)` — a distinct variant
   lets the frontend show "your settings/library list may not have saved" specifically.
2. `pub fn save(&self)` → `pub fn save(&self) -> Result<(), SError>`. Audit and update every call
   site (`library_service.rs`, `global_service.rs`, any command calling `config.save()`) to
   propagate the error to the command's `Result<_, SError>` return.
   **Including (M7):** `lib.rs::load_initial_library`'s `config.save()` (`lib.rs:107`), which runs
   on the startup background thread before any command exists to propagate through. Its handling:
   log the failure at that call site and surface it as a toast on the next successful init (a
   small pending-warning flag the frontend reads once from the first startup-call response) —
   don't block or fail startup over a config write that has a defaults fallback.
3. Atomic write: serialize to a temp file in the same directory, then `std::fs::rename` over the
   real path (atomic for same-volume renames on both Windows and Linux), so a crash mid-write
   never leaves a half-written `config.toml`. **Specified as a free function
   `pub fn atomic_write(...)` in `utils/file.rs` (M7)** — not a `FileUtils::` method, consistent
   with §4 dissolving the `FileUtils` struct. Reusable if `manifest.toml`/`cache.toml`/`tools.toml`
   want the same crash-safety after C4.
4. Lands as part of the already-planned confy→toml migration, not a separate pass.
5. Read-path behavior (log + reset vs. hand-written migration on parse failure) stays an open
   choice per the existing spec text — this plan only settles the write side.

## C13. Known-library list: unreadable libraries collapse to a path-only stub — resolves findings 2 and 3

The known-library list is a list of either a full summary or a minimal stub — not a full summary
with a `status`/`statusDetail` field bolted on. A library that can't be read (missing folder,
corrupt manifest, unplugged drive) shows up as an object containing only its registered `path` —
every other field simply isn't there. The UI renders that stub as a bare path with no name, mod
count, or actions beyond remove.

**Recovery is manual and coarse, on purpose:** no in-app repair/retry/refresh action. If the
library becomes readable again, the user restarts the app; if it's gone or abandoned, the user
removes the stub and re-adds fresh. No polling, no per-entry re-validation, no live status
transitions — this is what keeps the model simple enough to not need finding 3's status enum.

**Why this also resolves finding 2:** finding 2's eager-migration risk was specific to the SQLite
assembly design C4 cancelled. `get_known_library_summary` (`core/library_service.rs:129-141`)
already only reads `manifest.toml` — never `Library::load`/`version::fetch_and_validate` — so
listing never triggers the SPT-version hard-fail. An unsupported-version library lists fully
(manifest reads fine) and only surfaces `SError::UnsupportedSPTVersion` (C5) on activation —
consistent with today, not a new restriction. The only piece needing a decision was what happens
when the manifest read itself fails: fall back to the path-only stub instead of `filter_map`'s
current silent drop (`core/library_service.rs:133`).

## C14. `ModType` is still meaningful — keep it

Checked whether `ModType` (`Client`/`Server`/`Both`/`Unknown`) still does real work after 1.7's
manifest removal and C6's icon removal. It does: `infer_mod_type` (`core/mod_fs.rs:69-79`) derives
it structurally from which of `client_plugins`/`server_mods` a mod's files live under — computed
from the same file-placement facts deployment depends on, untouched by 1.7. Two live consumers:
the mod-grid toolbar's type filter and each mod card's category icon tint
(`frontend-redesign-data-api-contract.md`).

**Decision: keep `ModType`.** It has a real source (file-placement inference) and a real target
(filter + tint). The category icon *is* the default-icon mechanism C6 settled on — tinted by
`ModType`, not replaced by dropping it.

## C15. Fire-and-track completion signaling: client-owned task ids over a central event bus *(incorporates M6)*

Settles finding 13's staleness question and specifies how the frontend knows which completion
event belongs to which submitted action, without R.2's job queue.

**The id is frontend-owned, not backend-minted.** The frontend generates the `taskId` (uuid) and
includes it in the command call itself. Waiting for an accept response to hand one back would
reopen the race this closes — a fast backend task could complete before that round-trip resolves.

**A single, persistent event bus — not per-task `listen()` calls — is what closes the race.**
Initialized once at startup (same startup sequence C11 covers), subscribed to one completion event
for the app's lifetime. The submitter registers its handler in the bus's `taskId → handler` map
*synchronously, before* the command invoke goes out — so the handler exists before the request is
sent, regardless of backend speed. Client id ownership alone doesn't guarantee this; a fresh
`listen()` per task would still race. The bus is the fix; the id is what makes its dispatch table
work.

**Pending, not optimistic (M6, reconciled with frontend spec §9.3).** The registered handler's job
is "clear pending state and reconcile against the completion event's fresh workspace" — matching
§9.3's pending/disabled-until-completion pattern. There is no optimistic update and therefore no
canceller/rollback.

**Two-phase signal:**

- The command's own invoke `Result` is the *accept* signal — synchronous validation failures
  (no active library, bad input, non-active `libraryId` per C1) reject immediately.
- The completion event, matched by `taskId` through the bus, is the *outcome* signal for requests
  that passed validation and actually executed.

**In-flight task registry (M6), specified:** a `taskId → status` map lives on `AppRegistry` (the
same place the active-library `Arc<Mutex<Option<Library>>>` lives), inserted when a fire-and-track
command is accepted, removed on completion. A resubmitted `taskId` still present in the map is
rejected with a new `SError::TaskIdInUse(String)` variant — collision is a client bug to surface,
not something to paper over. This is a bookkeeping map alongside the mutex, not a job queue
replacing it.

**Contract/spec changes this requires (M6), all applied:** the four fire-and-track inputs gain a
`taskId: string` field; all four `WorkspaceEvent` variants carry it; contract §6's "never needs a
… correlation step" rationale sentence is deleted, since this introduces exactly that correlation
step by design.

**Dropped events on a stale bus are an accepted edge case.** If the frontend reloads, the bus and
its dispatch map are wiped; a completion event arriving for an unregistered `taskId` is silently
dropped. Accepted cost: a user who reloads mid-operation may see a stale library view until the
next explicit action or workspace refetch — narrow, self-correcting, and simpler than
reconciliation machinery for a reload-during-background-task window.

## T1. App settings ownership — backend-owned, full-replacement updates

**Decision: backend-owned.** Settings move into the App Config alongside
`known_libraries`/`app_state` (same file, same "one place for all app-level state" principle).

**Frontend integration pattern:**

- The frontend's startup call (C11 — `init` or its replacement) returns the full settings blob;
  the frontend stores it in a Jotai atom.
- User-initiated changes dispatch through backend commands (not a local atom write plus a separate
  save) — the command performs the change and returns the **full** settings object, which replaces
  the atom wholesale.
- Full-replacement, not merge: the frontend never constructs the next settings state from a
  partial update. The backend is the sole authority on post-change settings, sidestepping drift
  between an optimistic local update and what actually persisted.
- This resolves finding 14's `loadSettings()` sync-signature complaint outright — settings access
  was never going to be synchronous once it's fetched at init and read from the atom afterward.

**Why (kept for reference):** the narrow forcing constraint is that some settings (theme, for
window vibrancy) must be readable by the backend *before* the webview exists — `localStorage`
can't satisfy that at all. Backend ownership additionally gives one place for all app-level state
and C12's atomic-write treatment for free. Costs accepted: async settings reads everywhere,
`export_types` on every settings-shape change, import/export reimplemented against the Rust-owned
shape, and a real Rust-side startup-order dependency (window creation consumes the App Config
before building the window). The middle ground (mirroring only the startup-relevant setting) was
rejected: it reintroduces two sources of truth for overlapping data — the exact problem this
redesign avoids elsewhere (C7, one-file-per-fact).

---

# Part 3 — Resolution Map

| Finding | Resolution | Outcome |
|---|---|---|
| 1 | C1 | Reject-guard dropped; mutex blocking kept; fire-and-track scoped to active library; real guarantees stated |
| 2 | C4 + C13 | SQLite cancelled; assembly stays read-only; unreadable → path-only stub |
| 3 | C5 + C13 | No status enum; `InvalidLibrary` reused (covers IOError + ParseError); stub model |
| 4 | C1 + C3 + C4 | R.1/R.2 rejected; lock model and dirty flag kept; costs stated |
| 5 | C6 | `iconDataUrl` dropped from mods (live-pipeline deletion, accepted regression); tools keep theirs |
| 6 | C7 | `manifest.toml` `id` authoritative; migration transcribes, never mints (unless nothing readable) |
| 7 | C2 + C3 | Guard dropped as accepted risk; deployment stays explicit; Sync button stays (`deployStale` highlight) |
| 8 | C8 | Backup deletion decoupled from rename; rename dangles all of a mod's links until explicit deploy — accepted |
| 9 | C9 | Command + `models/test.rs` + frontend panel deleted outright; recorded §3 exception |
| 10 | C10 | All unclassified files classified; `utils/icon.rs` refactor changes its input contract |
| 11 | C11 | `init_called` moves with the startup command; silent exit → visible failure (Phase 4) |
| 12 | C12 | `save()` returns `Result`; `atomic_write` free function; `lib.rs` startup call site handled |
| 13 | C15 | Client-minted `taskId` + persistent bus + `AppRegistry` task registry + `TaskIdInUse`; reload drop accepted |
| 14 | T1 + doc edits | `loadSettings` async via T1; event-shape/errorText/comment-policy/data-array bullets land as small doc edits |
