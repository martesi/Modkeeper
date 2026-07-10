# Review of `design-review-mod.md` — Clarifications Checked Against Code and Doc Set

Status: Review. Findings for the author of `design-review-mod.md` (the C1–C15/T1 clarifications)
to resolve before those decisions are folded back into `backend-redesign-spec.md` /
`frontend-redesign-data-api-contract.md` / `frontend-redesign-spec.md`.

Method: every code citation in `design-review-mod.md` was re-verified against `src-tauri/src/`
and `src/` as they exist today, and every decision was checked for consistency against
`design-review.md`, the three spec docs, `purpose-of-redesign.md`, and `outline-of-redesign.md`.

Verdict up front: the doc is in good shape on citations — C5, C7, C11, C12, C13, C14, and T1's
code references all check out exactly, and the decisions in C5, C7, C13, C14, and T1 are coherent
with both the code and the doc set. Where it fails is two kinds of error: **two sections rest on
factually wrong claims about the code or the specs (M1, M3), and one section's central
serialization claim doesn't hold for the command surface the redesign actually defines (M2)**.
Behind those, several decisions cancel or contradict other documents without listing them in
their downstream-effects accounting (M4–M6). More than one finding requires change, hence this
document.

---

## Critical — the clarification is wrong about the code or another doc

### M1. C8's deployment-topology claim is false — deployment does not link the mod's top-level folder

C8 states: "deployment links the mod's **top-level folder**, not individual files inside it — a
symlink from the game/SPT mod directory points at `library_root/mods/<mod_id>` as a whole," and
concludes from that: changes inside the folder are live without resync, and "the only case that
actually breaks the link is the top-level folder itself being renamed."

The code does something structurally different. `deployment::execute_recursive_link`
(`core/deployment.rs:119-157`) walks **each active mod file's game-relative path** component by
component and creates a link at the **highest uniquely-owned path**: `src =
lib_paths.mods.join(id).join(&current_path)`, `dst = game_root.join(&current_path)`
(`core/deployment.rs:143-145`). Shared ancestors (e.g. `BepInEx/plugins` itself, or any directory
two mods both populate) are materialized as real directories (`core/deployment.rs:149-153`), and
linking recurses beneath them — down to individual files when a directory is shared. There is
never a link whose target is `mods/<mod_id>` as a whole; every link's target is a path *inside*
`mods/<mod_id>` mirroring the game-relative structure, and one mod typically produces several
links at varying depths.

Consequences for C8's argument:

- **The "matters less in practice" paragraph is built on a false premise and should be deleted or
  rewritten.** Because every link source lives under `mods/<mod_id>/...`, renaming that folder
  (exactly what `normalize_mod_folders` does, `core/cache.rs:69`) dangles **every** link of that
  mod, not "only the top-level link." This makes `design-review.md` finding 8's dangling-symlink
  point *stronger* than C8 credits, not weaker.
- "Changes to files *inside* that folder are already live" is only true for files under an
  already-linked directory. A new file introducing a new uniquely-owned path, or a change in
  ownership (another mod becoming active in the same directory), requires a re-deploy to be
  reflected — which is consistent with C3's explicit-deploy model, but not with C8's "already
  live without any resync" framing.
- **The C8 decision itself can survive** — "rebuild rewrites the record; recovery is rebuild then
  the explicit sync/deploy step" is a defensible product choice that doesn't depend on the broken
  paragraph, and the `remove_all_backups` removal (line 193) is verified and unaffected. But the
  spec text that eventually lands must describe the real link topology, because "rename dangles
  everything, redeploy fixes it" and "rename dangles one link" produce very different user-facing
  breakage windows for a deployed, running game.

**Fix:** rewrite C8's third paragraph against `execute_recursive_link`'s actual behavior; state
plainly that a rebuild rename dangles all of the renamed mod's links until the user runs the
explicit deploy step, and that this is the accepted cost.

### M2. C1's "the current lock model already prevents write competition as-is" does not hold for non-active libraries — which `bulk_update_mods` explicitly targets

C1's core claim: overlapping `bulk_update_mods` calls "serialize on the mutex like every other
mutating command does today," so dropping the reject-guard and blocking on
`Arc<Mutex<Option<Library>>>` suffices.

Two problems, one of them structural:

- **Not every mutating command serializes on that mutex today.** `rename_library` and
  `rebuild_library_cache` both accept an optional `library_id`; when it resolves to a non-active
  library, they do `Library::load(&path)` → mutate → `persist()` on a blocking thread **with no
  lock held at all** (`commands/library.rs:244-249` and `282-287`). Two overlapping calls on the
  same non-active library race freely today — last `persist()` wins. The redesigned
  `bulk_update_mods` takes a required `libraryId` (contract §6) that is not restricted to the
  active library, so C1's serialization guarantee only exists if the spec *adds* the restriction
  "fire-and-track mutations only operate on the active library" or gives non-active libraries
  their own serialization. Neither is stated anywhere. The per-library guard C1 removes was, in
  the original §8a design, keyed by `libraryId` — i.e. it covered exactly the case the mutex
  doesn't.
- **"Queue by blocking on the mutex, in order" overstates what a mutex provides.** `parking_lot`'s
  `Mutex` permits barging (eventual fairness, not strict FIFO), so two rapid opposite toggles of
  the *same* mod submitted from the UI can be applied in either order. For same-mod toggles the
  final state is order-dependent. This may be acceptable (the UI toggle reflects intended final
  state and toggles are absolute `is_active: bool` sets, not flips — `core/mod_manager.rs:83`),
  but "in order, same as today" is not the mechanism; if ordering matters it needs an actual
  queue, and if it doesn't, say why (absolute-set semantics), not "the mutex orders them."

There is also an unaddressed half of finding 4 that C1 implicitly inherits: keeping the single
mutex held across filesystem I/O (`add_mods` holds it for the full extract/backup/copy,
`commands/library.rs:43-58`) means a heavy install **does** block every read, including whatever
`get_library_workspace` becomes. §8a's "the guard does not block `get_library_workspace` or any
other read" promise and the contract §6 non-blocking language are unsatisfiable under C1's kept
model unless they are rewritten too. C1 resolves write-vs-write and is silent on write-vs-read;
the silence will be read as "reads block now," and that should be an explicit stated cost, not an
inference.

**Fix:** C1 needs three additions: (a) state where non-active-library mutations serialize (scope
`bulk_update_mods` to the active library, or keep a per-`libraryId` serialization mechanism for
the non-active path); (b) replace the "in order" claim with the real guarantee (absolute-set
toggle semantics under an unordered mutex, or an actual queue); (c) state explicitly that reads
block behind in-flight mutations under the kept model, and update §8a's non-blocking promises to
match.

### M3. C9's frontend justification contradicts `frontend-redesign-spec.md` — the redesign *does* have a developer settings section

C9: "Confirmed: the redesigned UI has no developer settings section at all, so
`developer-settings.tsx` is deleted in the same change... not carried forward in any form."

The frontend spec says the opposite in three places: §4's file tree contains
`settings/developer-settings-row.tsx`; §5a places the new/old UI toggle "as a row in the Settings
developer section (§9.6)"; §9.6's rows include "Developer row includes the new/old UI switch
(§5a) as a compact utility control, **alongside anything else retained from the current developer
row**." So the premise is false, and the conclusion drawn from it ("not carried forward in any
form") contradicts a spec that explicitly contemplates carrying parts of it forward.

Separately, deleting `src/modules/settings/developer-settings.tsx` "in the same change" conflicts
with the preservation rule: §3 "Do not delete old feature files during the redesign switch-over.
Cleanup is a later, explicit task," and §5a requires the old tree to stay *mountable* during the
transition (the legacy-UI toggle renders it). The conflict is real, not pedantic: deleting the
backend command and re-running `export_types` removes `createSimulationGameRoot` from the
generated bindings, which makes the old file fail to compile — so *something* must give. That
resolution needs to be chosen explicitly: either (a) stub the old panel's simulation button
behind the deleted command (edit, not delete — smallest violation of §3), or (b) accept that the
legacy UI loses its developer panel during transition and record that as an accepted §5a
degradation. C9 currently picks (c) — delete the old file — on a premise that's wrong.

**Fix:** correct the premise (the redesign has a developer row per §5a/§9.6 — it hosts the UI
switch, not the simulation tool); decide and record the old-file handling against §3/§5a instead
of asserting deletion. The backend-side decision (delete the command, `models/test.rs`, the
`collect_commands!` entry — all verified: `commands.rs:3`, `lib.rs:47`) is unaffected and stands.

---

## High — decisions whose downstream accounting is incomplete

### M4. C4 cancels a purpose-level requirement but doesn't list `purpose-of-redesign.md` (or the tool registry's storage) in its downstream effects

The SQLite Library DB is not just `backend-redesign-spec.md` §8 — it is an in-scope requirement
in the canonical doc: `purpose-of-redesign.md` "Adopt SQLite as the Library State and Index
Layer" (and its header note listing SQLite as a change since 2026-05-10), plus
`outline-of-redesign.md` Phase 2.5, plus the contract's §3 Backend Direction and §7 Library DB
schema. `backend-redesign-spec.md` §3 states "If the two disagree, `purpose-of-redesign.md`
wins" — a clarifications doc cannot cancel a purpose-level requirement and leave the purpose doc
saying the opposite; as written, the doc set now disagrees with itself and the stated precedence
rule resolves that disagreement *against* C4.

C4's "downstream effect" list names only `design-review.md` findings and backend-spec §8/§8a.
Missing from it:

- **`purpose-of-redesign.md`** — the SQLite section must be rewritten or removed (this is the
  single most important edit, given the precedence rule).
- **`outline-of-redesign.md`** — Phase 2.5 (the migration item), Phase 2.6 (which stores tools in
  "a `tools` table (2.5)"), and the §8 Rollout Sequence steps the implementation sequence (§13
  steps 5–8) is built on.
- **`frontend-redesign-data-api-contract.md`** — §3's two-tier description, §7's entire Library DB
  SQL sketch, and §8's migration bullet all describe the cancelled design. (The wire contract
  itself survives — C4 is right that the split is invisible to the frontend — but the contract
  *document* doesn't.)
- **The tool registry loses its storage home entirely.** §9 stores tools in
  `store/library_store/tools.rs` (Library DB); the contract's `toolsByLibraryId` is filled from
  the `tools` table; §8 Rollout step 3 uses the tool registry as the per-library-DB proving
  ground. With SQLite cancelled, *nothing specifies where a tool registration is persisted*. C4's
  parenthetical "a consolidated single-file replacement... a separate, smaller decision" hides
  the fact that this decision is now a **blocker** for §9, not an optional consolidation nicety —
  tools have no legacy file to fall back to the way mods/cache fall back to
  `manifest.toml`/`cache.toml`.

**Fix:** extend C4's downstream list with the four items above, and either make the
tools-storage decision inside C4 (e.g. a `[tools]` section in the manifest or a `tools.toml`
sibling) or explicitly mark §9 blocked on it.

### M5. C3 keeps the explicit deploy step but doesn't account for the contract and UI having no deploy surface left

C3's model (cheap always-persisted toggles; deployment as a separate, explicit, user-triggered
step) is verified accurate against the current code (`core/mod_manager.rs:83-92`,
`core/library.rs:135-149`, `commands/library.rs:89-102`) and is a coherent decision. But its
"needs correction" list names only frontend-spec §9.3's commit-immediately line and the
`sync_mods` removal. The actual change set is larger:

- **Contract §6 has no deploy command.** If `sync_mods` (renamed or not) survives, it needs a row:
  its input, whether it's the fourth fire-and-track operation (it walks and relinks the whole
  tree — by the contract's own "touches potentially many files" criterion it qualifies) and thus
  a fourth `WorkspaceEvent` shape, and its `GameOrServerRunning`-adjacent error surface given C2
  drops that guard.
- **The contract has no way to represent "deployed state is stale."** Under C3 the dirty flag is
  load-bearing UI state (it's what tells the user a deploy is needed), but `LibrarySummary`
  carries only `cacheStatus`; `LibraryDTO.is_dirty` exists today and survives nowhere in the new
  contract. Either `CacheStatus` is redefined to cover deploy-staleness (it isn't that today) or
  a field is added.
- **Frontend spec §9 has no Deploy/Sync affordance on any screen** — the manual Sync button was
  deliberately removed from the design. Under C3 a user *must* have a deploy trigger; which
  screen owns it (Manage Library? the execution bar? the grid toolbar?) is unspecified.
- **Backend spec §2's Out of Scope explicitly locks in the opposite:** "the sync-pipeline
  redefinition in `1.9`... (remove the manual Sync button, commit on toggle) is **accepted as-is**."
  C3 reverses an accepted decision and should cite and strike that line, not just §7's table rows.
- **The fire-and-track scoping for `bulk_update_mods` loses its rationale for enable/disable.**
  The contract justifies fire-and-track by "these three touch potentially many files"; under C3,
  enable/disable is a metadata-only TOML write — arguably a fast blocking call like
  `rename_library` — while `delete` still unlinks and removes files. Whether `bulk_update_mods`
  stays fire-and-track for all three actions needs a restated reason (uniformity is a fine
  reason; it just has to be said, because the current stated reason becomes false).

**Fix:** expand C3's correction list to the five items above so the Phase 2 spec rewrite has the
full change set, not a third of it.

### M6. C15 changes the wire contract in ways the contract explicitly argues against, and leans on machinery no section defines

Three consistency problems:

- **The contract's event design is explicitly correlation-free.** Contract §6: each completion
  event carries the fresh workspace so the frontend "never needs a second round trip or a
  separate 'did this event belong to my request' correlation step." C15 adds client-minted
  `taskId` to command inputs and (implicitly) to completion events — that's a contract §6 and
  frontend-spec §7 change (`bulk_update_mods` input shape, all three `WorkspaceEvent` variants,
  plus the rationale sentence above must be deleted). C15's downstream accounting lists none of
  these, unlike C3/C4 which model this correctly.
- **C15 presumes optimistic updates the frontend spec doesn't have.** The registered handler is
  described as "the canceller/rollback for its optimistic UI change," but frontend-spec §9.3
  specifies the opposite pattern: pending/disabled state from `{ accepted: true }` until the
  completion event — no optimistic application, so there is nothing to roll back. Either C15 is
  also (silently) changing §9.3 to optimistic toggling, or the handler's job is "clear pending
  state and reconcile," and the section should say which.
- **The machinery C15 references doesn't exist under C1.** It attributes a "quick vs. heavy job
  split" to C1 — C1 defines no such split (it rejected R.2's queue and kept mutex blocking). It
  speaks of tasks "queued/executed" and of the backend rejecting a `taskId` "resubmitted while
  its original task is still in flight" — which requires a backend-side in-flight task registry
  (id → status map, with cleanup on completion) that no section of either doc specifies, and a
  new `SError` variant for the collision case that isn't named. Under C1's model the only "queue"
  is threads blocked on a mutex; a task registry is a real new component and needs a home.

**Fix:** add the contract/frontend-spec change list to C15; reconcile with §9.3 (pending vs.
optimistic — pick one); specify the in-flight task registry (where it lives on `AppRegistry`, its
cleanup, the collision `SError` variant) or drop the id-reuse rejection rule.

---

## Medium — smaller corrections, collected

### M7. C12 proposes `FileUtils::atomic_write` — a struct §4 dissolves

Backend spec §4 refactors `utils/file.rs` to free functions ("Drop the `FileUtils` struct"), and
nothing in `design-review-mod.md` cancels that (C10 leaves the §4 FP refactors intact). The
atomic-write helper should be specified as a free function in `utils/file.rs`, not a
`FileUtils::` method. Same section: the call-site audit list ("`library_service.rs`,
`global_service.rs`, and any command") misses `lib.rs::load_initial_library`, which calls
`config.save()` on the startup background thread (`lib.rs:107`) — a context where "propagate the
error up to the command's `Result`" has no command to propagate to; it needs its own stated
handling (log + toast-on-next-init, or similar).

### M8. C5's proposed `CorruptLibrary(String, String)` overlaps an existing variant it doesn't mention

`SError::InvalidLibrary(String, String)` already exists (`models/error.rs:22-23`) and
`validate_library_structure` already returns it for exactly the corrupt-manifest case:
"manifest.toml is invalid or unreadable" (`core/library_service.rs:31-36`). C5 should either
reuse/promote `InvalidLibrary` or state why a distinct `CorruptLibrary` is needed alongside it
(the plausible reason: `InvalidLibrary` also covers structural problems like missing directories,
and the frontend wants to distinguish "fixable structure" from "unparseable data" — but that has
to be said). Also a citation nuance that matters for implementation: `utils/toml.rs:9` is the
*write*-path serialize error; the read path produces `IOError` for an unreadable file (line 14)
and `ParseError` for unparseable content (line 15) — "corrupt library" detection at
`read_library_manifest` must catch both, not just `ParseError`.

### M9. C6 misdescribes the icon pipeline it's deleting

"`Mod.icon_data` set to `None` at every construction site already" is true of the persisted `Mod`
struct's construction sites but misleading about the feature: `dto_builder::build_frontend_dto`
(`core/dto_builder.rs:14-21`) populates `icon_data` from `manifest.icon` on every DTO build —
that is the live producer being removed, not an already-dead field. The decision is unchanged
(1.7 deletes the manifest source, §4 deletes `dto_builder.rs`); the sentence should describe
deleting a live pipeline, because "already None everywhere" invites an implementer to skip the
frontend-visible regression check (mods that show icons today stop showing them).

### M10. C10's `utils/icon.rs` row overstates the fit with the tool-icon flow

"§9's tool-icon flow needs exactly this capability" — not quite. `load_icon_as_data_uri` is
path-in → extension-sniffed MIME → data-URI out (`utils/icon.rs:8-27`); §9's tool flow is
base64 *bytes* in (read by the frontend, never a backend file read) → content-validate →
resize/cap → stored representation. After C6 removes the only path-based caller, the shared
surface is just the base64/data-URI tail. Keep-and-refactor may still be the right verdict, but
the row should say the refactor changes the function's input contract (bytes, not a path) and
that MIME comes from content sniffing (the `image` crate decode §9 already plans), so nobody
"refactors" it by keeping the path-and-extension shape.

---

## What was checked and found correct

For completeness, the following were verified and need no change: C1/C3/C5/C7/C8/C11/C12/C13/C14
and T1's file/line citations (all accurate, including `core/registry.rs:14`, `utils/thread.rs:6`,
`core/mod_manager.rs:83-92`, `core/library.rs:47,50,76,135-149`, `commands/library.rs:89-90`,
`core/library_service.rs:123,129-149,181-226` with `remove_all_backups` at 193,
`core/mod_fs.rs:69-79`, `models/error.rs:7`, `commands/global.rs:159-166`, `lib.rs:47,107,116-127`,
`config/global.rs:26-28`, `utils/icon.rs:13-20`, `core/mod_documentation.rs:7-33`,
`src/lib/settings-storage.ts` including the sync `loadSettings` and lines 57-91 import/export,
and `models/test.rs` as an empty stub). The decisions in C2 (guard dropped as an accepted,
documented risk), C5 (modulo M8), C7 (identity rule — now strictly simpler than R.5), C13
(path-only stub model, including the correct observation that finding 2's eager-migration risk
was specific to the cancelled SQLite assembly), C14 (`ModType` retained), and T1 (backend-owned
settings, full-replacement update pattern) are internally consistent and consistent with the code.

## Priority order for resolving

| # | Finding | Where the fix lands |
|---|---------|---------------------|
| M1 | Deployment topology claim false; rename dangles all links | `design-review-mod.md` C8, then whatever spec text lands |
| M2 | Mutex doesn't serialize non-active-library writes; ordering + read-blocking unstated | C1 (+ §8a rewrite scope) |
| M3 | Developer-settings premise contradicts frontend spec §4/§5a/§9.6 and §3 preservation rule | C9 |
| M4 | SQLite cancellation missing purpose/outline/contract in blast radius; tool storage unhomed | C4 (+ `purpose-of-redesign.md` edit) |
| M5 | Kept deploy step has no contract command, no dirty field, no UI surface; §2 Out of Scope contradicts | C3 |
| M6 | `taskId` contradicts correlation-free contract; optimistic-vs-pending mismatch; task registry unspecified | C15 |
| M7 | `FileUtils::atomic_write` vs §4 FP refactor; `lib.rs` save call site missed | C12 |
| M8 | `CorruptLibrary` overlaps existing `InvalidLibrary`; read-path is IOError *or* ParseError | C5 |
| M9 | icon_data described as dead; it's populated on every DTO build | C6 |
| M10 | `utils/icon.rs` "exactly this capability" overstates fit | C10 |
