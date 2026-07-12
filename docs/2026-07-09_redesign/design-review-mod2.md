# Design Review Response, Round 2 — Resolving `design-review-mod-review.md` (M1–M10)

Status: Draft. Author resolutions for the ten findings `design-review-mod-review.md` raised
against `design-review-mod.md`'s C1–C15/T1. Each section below is the authoritative replacement
for the C-section it corrects; everything in `design-review-mod.md` not named here stands
unchanged. These decisions are what fold back into `backend-redesign-spec.md` /
`frontend-redesign-data-api-contract.md` / `frontend-redesign-spec.md` / `purpose-of-redesign.md` /
`outline-of-redesign.md` for Phase 2/3 implementation.

---

## M1. C8, corrected — a rebuild rename dangles every link of the renamed mod, not one

M1 is right: `execute_recursive_link` never links `mods/<mod_id>` as a whole — it links at the
highest uniquely-owned path *inside* `mods/<mod_id>/...`, per game-relative path component, so one
mod typically produces several links at varying depths, all sourced from inside that mod's folder.

**The drawback, stated plainly:** because every one of those links' source path is
`mods/<mod_id>/...`, renaming that folder (`normalize_mod_folders`, `core/cache.rs:69`) invalidates
the source path for **all** of that mod's links simultaneously — not just a single top-level link.
A mod that produced five links at five different depths loses all five the instant rebuild renames
its folder, and every one of them stays dangling in the live game/SPT directory until the user runs
the explicit deploy step. For a deployed, currently-running game, that's a wider breakage window
than "one dangling link" — every file the mod contributed can vanish from the game's view at once,
not just its top-level entry point.

**Fix applied:** delete C8's "matters less in practice" paragraph entirely — its premise was false.
The decision underneath it is unaffected and still stands: rebuild only rewrites the library's own
record; recovery is rebuild-then-explicit-deploy, same as any other on-disk change made out from
under a live deployment. What changes is only the accounting of *how much* dangles in the interim —
replace the deleted paragraph with the drawback statement above so the spec text that lands
describes the real blast radius instead of the false "one link" framing.

## M2. C1, corrected — scope fire-and-track writes to the active library; state the real guarantees

Suggestion, in three parts, matching the three gaps M2 raised:

**(a) Where non-active-library mutations serialize.** The `Arc<Mutex<Option<Library>>>` C1 keeps
only ever holds the *active* library — there is no in-memory `Library` for any other registered
library, so this mutex structurally cannot serialize writes to a non-active one no matter how it's
used. Rather than inventing a second serialization mechanism for a path nothing actually needs
concurrent-safe today, **scope `bulk_update_mods` (and `install_mod_archives`,
`rebuild_library_cache`) to the active library only**: the command validates `libraryId ===
activeLibraryId` and rejects otherwise with a plain validation error, before touching anything. The
contract's existing `libraryId` field on `bulk_update_mods` (§6) is kept — but its purpose is
corrected: it's an assertion input for that validation and the id the completion event reports
against, not a key into a "per-library operation guard" (the contract's current comment describing
it that way is stale now that C1 has already dropped that guard — reword it). `rename_library` and
`rebuild_library_cache`'s existing optional-non-active-`library_id` path keeps its current,
already-racy last-`persist()`-wins behavior unchanged — that's a pre-existing characteristic of
today's code, not a new regression this redesign introduces, and the UI only ever fires one such
call at a time per library, so it's out of scope here rather than silently inherited as a promise.

**(b) The real ordering guarantee.** Drop "queues in order, same as today" — `parking_lot::Mutex`
doesn't guarantee FIFO. State instead: two overlapping calls on the *same* active-library mod
resolve to whichever acquires the lock last, order not guaranteed to match submission order. This
is acceptable specifically because `bulk_update_mods` toggles are absolute `is_active: bool` sets,
not increments (`core/mod_manager.rs:83`) — the outcome of a race between two overlapping calls is
always one of the two states the user actually asked for, never a corrupted third state. If a
future change makes ordering matter (e.g. a delta-style operation), this guarantee stops being
sufficient and needs an actual queue at that point — not before.

**(c) Reads block behind in-flight active-library mutations.** Say so explicitly: `add_mods`-style
heavy operations hold the mutex for their full filesystem duration, so `get_library_workspace` (or
its eventual replacement) blocks for that same duration when called against the active library.
§8a's "the guard does not block `get_library_workspace` or any other read" line and the contract's
non-blocking language are corrected to state this cost plainly rather than imply reads are free
under the kept model — metadata-only operations (toggle) block reads for a negligible interval;
`install_mod_archives`/`rebuild_library_cache` can block them for longer, and that's an accepted
cost of keeping the single-mutex model (C4), not a hidden regression.

## M3. C9, corrected — no developer settings section ships; the old panel is deleted outright

Per direction: don't implement a developer settings section at all — it was never actually part of
the UI design, regardless of what `frontend-redesign-spec.md`'s text currently says.

**Decision:**
- Strike `settings/developer-settings-row.tsx` from §4's file tree, strike §9.6's "Developer row
  includes the new/old UI switch..." bullet, and strike §5a's "surfaced as a row in the Settings
  developer section (§9.6)" clause — there is no developer section in the redesigned UI.
- The new/old UI toggle (§5a) still needs a home — §5a's own constraint (runtime, not build-time,
  so switching doesn't require a rebuild) still holds. It becomes a single utility control on the
  existing Settings screen's general row list, explicitly labeled as a transition-only control, not
  a whole section built to host it. It gets deleted along with the rest of the legacy-UI toggle
  machinery once the redesign fully replaces the old UI — it was never meant to be permanent.
- `src/modules/settings/developer-settings.tsx` (the Test Game Root panel) has nothing to be
  preserved into, because nothing in the actual UI design has a slot for it. **Decision: delete it
  outright**, along with the backend `create_simulation_game_root` command per C9's original plan —
  this is an explicit, recorded exception to §3's "don't delete old feature files during the
  switch-over" rule for this one file: §3 protects files whose *feature* survives into the redesign
  in some form; this one's feature (a developer settings section) doesn't exist in the design at
  all, so there is nothing for the preservation rule to keep alive.

This makes C9's original conclusion ("not carried forward in any form") correct again, on the
correct grounds (the design never had a developer section, not the false "confirmed no developer
section" premise M3 flagged) — deletion happens for the reason given here, not the reason C9 gave
originally.

## M4. C4 — clarify the deviation in spec; don't edit `purpose-of-redesign.md`

Per direction: `purpose-of-redesign.md` is not edited. `backend-redesign-spec.md` §3's precedence
line ("If the two disagree, `purpose-of-redesign.md` wins," line 59) gets an explicit note directly
beside it recording that C4's SQLite cancellation is a **deliberate, called-out exception** to that
line, pointing back to C4 for the rationale — so the doc set's disagreement with itself is
documented as an intentional, reasoned deviation instead of the silent contradiction M4 flagged.
`purpose-of-redesign.md`'s SQLite section is left as written, read from here forward as the
historical case *for* the proposal rather than a live requirement the rest of the set must satisfy.

The rest of C4's downstream accounting still needs the edits M4 identified, since those are
implementation docs (not the purpose doc the direction above is scoped to):

- **`outline-of-redesign.md`** — Phase 2.5 (drop the Library DB migration item; the App Config
  plain-file migration it also describes stays) and Phase 2.6 (its `tools` table dependency is
  replaced by the decision below).
- **`frontend-redesign-data-api-contract.md`** — §3's two-tier description and §7's Library DB SQL
  sketch describe the cancelled design and are struck; the wire contract itself is unaffected (C4
  already established the split is invisible to the frontend).

**Tool storage, decided (this was a blocker for §9, not an optional nicety):** tools are persisted
in a `tools.toml` sibling file per library, alongside `manifest.toml`/`cache.toml` — mirroring the
existing two-file split rather than growing `manifest.toml` with an unrelated concern, and
consistent with C4's own "a consolidated single-file replacement is a separate, smaller decision"
line (this isn't that consolidation; it's the minimum needed to give tool registration a file to
live in now that `library.db` doesn't exist). `core/tool_service.rs` reads/writes it directly, the
same way `library_service.rs` already reads/writes `manifest.toml`/`cache.toml`.

## M5. C3 — the Sync button stays, unchanged in place; it highlights when deploy state is stale

Per direction: the manual Sync button is **not** removed by this redesign — it keeps its existing
role as the explicit, user-triggered deploy step C3 already established, and gains a highlighted/
accent visual state whenever the deployed state is stale relative to recorded mod state.

This reverses the backend spec's prior acceptance and fills in the five gaps M5 raised:

- **`backend-redesign-spec.md` §2 Out of Scope's line accepting `1.9_backend-redesign-audit.md`
  §2.B ("remove the manual Sync button, commit on toggle") is struck.** That sync-pipeline
  redefinition is reversed by C3/this section, not implemented as originally accepted.
- **Contract §6 gets a fourth fire-and-track row**, keeping the existing command name and guard
  posture (no `GameOrServerRunning` check, per C2's accepted drop):
  ```ts
  sync_mods(input: { libraryId: LibraryId }): Promise<OperationAccepted>
  // Fire-and-track (fourth operation) — completion via listen_workspace_event's
  // 'sync_completed'. Walks and relinks the whole tree, same cost profile as
  // rebuild_library_cache, which is why it qualifies under the "touches
  // potentially many files" criterion alongside the other three.
  ```
  "Non-Blocking Operations" (§6) is corrected from "exactly three" to four, and a matching
  `WorkspaceEvent` variant (`sync_completed`) is added alongside the other three.
- **Deploy-staleness gets its own field, not an overload of `cacheStatus`.** `cacheStatus`
  describes the cache/manifest rebuild machinery, a different concern from whether the *deployed*
  symlinks match recorded mod state. `LibrarySummary` gains `deployStale: boolean` — `true` exactly
  when `Library::is_dirty` is true today. The Sync button reads this field directly: highlighted
  when `deployStale`, quiet otherwise.
- **The button's screen, decided:** `library-execution-bar.tsx` — deploy state is an execution
  concern (does the running game match the library), not a mod-browsing concern, so it belongs
  next to whatever other execution-bar actions exist, not in `mod-grid-toolbar.tsx`.
- **`bulk_update_mods`'s fire-and-track scoping keeps its uniform treatment across all three
  actions, restated reason:** enable/disable is now metadata-only (cheap enough to be a fast
  blocking call on its own), but `delete` still unlinks and removes files, and a single command
  with three actions is kept fire-and-track uniformly for one predictable client-side handling
  path rather than a per-action split — consistency, not per-action cost, is the reason now that
  C3 makes the actions' underlying costs diverge.

## M6. C15 — contract/spec change list, pending-vs-optimistic, and the task registry, added

- **Contract/spec changes required by client-minted `taskId`, listed explicitly:** `bulk_update_mods`
  (and the other three fire-and-track inputs) gain a `taskId: string` field; all four
  `WorkspaceEvent` variants carry it; contract §6's "never needs a ... correlation step" rationale
  sentence is deleted, since C15 introduces exactly that correlation step by design.
- **Reconciled with §9.3: pending, not optimistic.** The registered bus handler's job is "clear
  pending state and reconcile against the completion event's fresh workspace," matching §9.3's
  existing pending/disabled-until-completion pattern — not a canceller/rollback for an optimistic
  update that doesn't exist. C15's "canceller/rollback for its optimistic UI change" language is
  corrected to this.
- **In-flight task registry, specified:** a `taskId → status` map lives on `AppRegistry` (the same
  place the active-library `Arc<Mutex<Option<Library>>>` already lives), inserted when a
  fire-and-track command is accepted and removed on completion. A resubmitted `taskId` still present
  in the map is rejected with a new `SError::TaskIdInUse(String)` variant. This is the "queue" C15
  referred to under C1's model — a bookkeeping map alongside the mutex, not a job queue replacing it.

## M7. C12 — free function, not a `FileUtils` method; `lib.rs` startup call site covered

`atomic_write` is specified as a free function in `utils/file.rs` (`pub fn atomic_write(...)`),
consistent with §4 dissolving the `FileUtils` struct entirely — not `FileUtils::atomic_write`.

The call-site audit list gains `lib.rs::load_initial_library`'s `config.save()`
(`lib.rs:107`), run on the startup background thread before any command exists to propagate a
`Result` to. Its handling: log the failure at that call site, and surface it as a toast on the
*next* successful init (a small pending-warning flag the frontend reads once from the first
`get_library_workspace`/init response) rather than blocking or failing startup over a config write
that already has a fallback (defaults) if genuinely unreadable.

## M8. C5 — reuse `InvalidLibrary`, don't add `CorruptLibrary`; catch both read-path errors

`SError::InvalidLibrary(String, String)` already exists and is already raised by
`validate_library_structure` for the corrupt-manifest case. **Decision: reuse `InvalidLibrary`,
don't add `CorruptLibrary`** — the distinction M8 raised (structural problems vs. unparseable data)
isn't one the frontend needs to act on differently yet (C13's stub-object model treats any unreadable
library the same way, path-only), so a second variant would exist without a consumer. If that
distinction becomes load-bearing later, split it out then, with a stated reason at that point.

Detection at `read_library_manifest` is corrected to catch **both** read-path failures the toml
utilities can produce — `IOError` (`utils/toml.rs:14`, unreadable file) and `ParseError`
(`utils/toml.rs:15`, unparseable content) — not just `ParseError`; both map to `InvalidLibrary` for
this purpose.

## M9. C6 — corrected: `icon_data` is a live pipeline being removed, not dead weight

`dto_builder::build_frontend_dto` (`core/dto_builder.rs:14-21`) populates `icon_data` from
`manifest.icon` on every DTO build today — it's the live producer this decision removes, not an
already-`None` field. C6's decision is unchanged (drop `iconDataUrl`/`icon_data`, default icon by
`ModType`), but the description is corrected so the implementer treats this as a **visible
regression to check**: mods that show a custom icon today will show the default category icon
after this change, and that's the expected, accepted UX change — not a bug to chase down during
implementation.

## M10. C10 — corrected: the tool-icon refactor changes `utils/icon.rs`'s input contract

`load_icon_as_data_uri` today is path-in → extension-sniffed MIME → data-URI out. §9's tool-icon
flow is base64 *bytes* in (already read by the frontend) → content-validated → resized/capped →
stored representation — a different input shape, not "exactly this capability." Keep-and-refactor
is still the right verdict, but the refactor changes the function's input contract from a filesystem
path to raw bytes, and MIME detection moves from extension-matching to content-sniffing (the
`image` crate decode §9 already plans) — the row is corrected so nobody refactors it by preserving
the path-and-extension shape and calling it done.

---

## Summary of decisions requiring doc edits

| # | Decision | Docs touched |
|---|---|---|
| M1 | Rebuild rename dangles *all* of a mod's links, not one; delete the false paragraph | `design-review-mod.md` C8 |
| M2 | Scope fire-and-track writes to active library; unordered-mutex + absolute-set guarantee stated; reads block behind in-flight mutation, stated | C1, §8a, contract §6 comment |
| M3 | No developer settings section anywhere; old panel + backend command deleted outright; UI-toggle relocated to a plain settings row | frontend-redesign-spec.md §4/§5a/§9.6 |
| M4 | Deviation from `purpose-of-redesign.md` recorded in spec, not edited into purpose; outline/contract SQLite text struck; tools persist in `tools.toml` | backend-redesign-spec.md §3, outline-of-redesign.md 2.5/2.6, contract §3/§7 |
| M5 | Sync button kept, unchanged in place, highlights on `deployStale`; §2 Out of Scope line struck; contract gets 4th fire-and-track op + `deployStale` field | backend-redesign-spec.md §2, contract §6/`LibrarySummary` |
| M6 | `taskId`/event changes listed; pending-not-optimistic reconciled with §9.3; task registry specified on `AppRegistry` | contract §6, frontend-redesign-spec.md §9.3, backend-redesign-spec.md |
| M7 | `atomic_write` as free function; `lib.rs` startup call site handling specified | backend-redesign-spec.md §4/§12 area |
| M8 | Reuse `InvalidLibrary`; catch `IOError` and `ParseError` both | models/error.rs plan (C5) |
| M9 | Icon pipeline described as live, removal is a stated visible regression | C6 |
| M10 | Icon refactor's input-contract change (path → bytes, extension → content sniff) stated | C10 |
