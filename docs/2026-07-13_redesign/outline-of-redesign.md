# Outline of Redesign

Status: Living document. Reframes and restages `docs/2026-07-09_redesign/outline-of-redesign.md`
for this round — it does not replace that document's technical content (endpoint tables, DTO
shapes, screen specs, the full C1-C15/T1 resolution history), only its scope and implementation
sequence. Where this doc is silent, the 2026-07-09 docs still apply as written.

Companion docs:

- `docs/2026-07-13_redesign/current-implementation-audit.md` — factual baseline, read from source.
- `docs/2026-07-09_redesign/backend-redesign-spec.md` — endpoint tables, DTOs, App Config
  mechanics, still authoritative except where §2 below overrides.
- `docs/2026-07-09_redesign/design-review.md` — full C1-C15/T1 resolution history, still
  authoritative for any backend decision not overridden below.
- `docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md` — TS contract, still valid
  minus the tool-registry rows (deferred, see §2).
- `docs/2026-07-09_redesign/frontend-redesign-spec.md` — screen specs and file tree, still valid;
  this doc only restructures *when* things get built, not what they look like.

## 1. Why This Revision Exists

The 2026-07-09 doc set framed both tracks as one "redesign." Clarified this round:

- **The backend track is a refactor, not a redesign.** The goal is FP-by-default so a function can
  be checked in isolation instead of requiring the reader to hold a whole OOP object's mutable
  state in their head — a legibility goal for a solo maintainer, not an architecture-for-scale
  ideology. Anything that isn't restructuring existing logic (a new subsystem, a new dependency, a
  new command surface) is out of scope for this track by definition, not just by preference.
- **The product goal is minimal closure, not feature completeness.** This app is a portfolio/resume
  piece now, not a public product — SPT's mod distribution site (the Forge) doesn't allow
  AI-written apps, which closes off the original public-distribution path, and the author no longer
  actively plays the game. A finished, coherent, demonstrable codebase beats a larger one with more
  features. Scope removals in this doc set (the manifest system, the backup browsing UI) are
  deliberate scope cuts made against that goal — the removed code isn't broken, it's just more than
  this version of the app needs to carry.
- **The frontend track is the primary effort** and gets a more deliberate build sequence this round
  (§3) — the author has stronger opinions here than on backend architecture, so the frontend track
  is staged to be checkable layer-by-layer rather than built file-by-file across a flat 13-step list.

## 2. Deltas From `docs/2026-07-09_redesign/*`

Concrete changes from the 2026-07-09 spec set, collected in one place so nothing is missed:

1. **Executable tool registry is deferred out of this pass entirely.** `backend-redesign-spec.md`
   §9 (`commands/tool.rs`, `core/tool_service.rs`, `tools.toml`, icon validation pipeline) and its
   frontend counterpart (Configure Tool dialog, Manage Library's tools section content) are not
   built this round. Not cancelled — revisit as a separate initiative later if the app's goals
   change. Nothing in this outline assumes the tool registry exists.
2. **The `GameOrServerRunning` guard is kept, not dropped.** Reverses `design-review.md` C2. C2's
   rationale (the check doesn't generalize to Linux) doesn't apply to this app's actual context:
   single-platform personal use, not a public cross-platform product. The guard works today on
   Windows and protects the author's own game/server from mid-sync corruption — dropping it would
   trade a real, currently-working protection for portability this app doesn't need yet.
3. **`bulk_update_mods` (enable/disable/delete) is a plain blocking call, not fire-and-track.**
   Reverses part of `design-review.md` C1/`backend-redesign-spec.md` §8a's "stays fire-and-track for
   all three actions" decision. Enable/disable is metadata-only and delete only touches the
   selected mods — neither scans a full tree, so neither needs the client-minted-`taskId`/event-bus
   machinery. Fire-and-track is now reserved for exactly three commands: `install_mod_archives`,
   `rebuild_library_cache`, `sync_mods` — the ones that actually extract archives or walk the whole
   tree.
4. **Comment removal is its own later phase (Phase 6 below), not a rule enforced during the
   refactor.** `purpose-of-redesign.md`'s "Reduce Unintended Distraction" is unchanged as a stated
   goal, but a blanket "remove all comments while touching each file" rule risks exactly what it
   would take out today: `linker.rs`'s comment explaining why Windows directory symlinks need a
   `remove_dir`-then-`remove_file` fallback encodes a non-obvious platform constraint, not restated
   logic. Comment cleanup happens once, after the structure stops moving, with that distinction
   applied deliberately rather than mechanically.
5. **Frontend implementation is staged by architectural layer, not by flat file sequence.** See §5.

## 3. Phase 1 — Audit (Complete)

Findings live in `docs/2026-07-13_redesign/current-implementation-audit.md`, itself grounded in the
2026-05-10 audit findings carried forward unchanged in `docs/2026-07-09_redesign/outline-of-redesign.md`
Phase 1.

## 4. Phase 2 — Backend: FP Refactor

Purpose restated per §1: make functions independently checkable, not redesign the architecture.
Behavior must not change as a side effect of restructuring — the existing test suite
(`tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs`) is the correctness oracle at every step,
exactly as `backend-redesign-spec.md` §3 already specifies.

**In scope, unchanged from `backend-redesign-spec.md` except the deltas in §2 above:**

- 4.1 FP conversion: `core/mod_fs.rs` → DTO + free functions (`resolve_id`, `infer_mod_type`);
  `utils/file.rs`, `utils/toml.rs`, `utils/process.rs` drop their struct wrappers
  (`FileUtils`/`Toml`/`ProcessChecker`).
- 4.2 Low-level FS separation: relocate `core/decompression.rs`'s `extract` into `utils/file.rs`
  (or a new `utils/archive.rs`), delete `core/decompression.rs` once moved.
- 4.3 Core/service/interface boundary enforcement: extract `commands/library.rs::add_mods`'s and
  `remove_mods`'s inlined orchestration into `library_service.rs` (`backend-redesign-spec.md` §5).
- 4.4 Endpoint renaming (`verb_noun`) matching `frontend-redesign-data-api-contract.md`, minus the
  tool commands (deferred, §2 item 1).
- 4.5 App Config: replace `confy` with plain `toml` (fixes today's silently-discarded load/save
  failures) — `known_libraries` + `app_state` + `settings`, atomic writes, per
  `backend-redesign-spec.md` §8, unchanged from that spec.
- 4.6 Comment removal — **removed from this phase**, see Phase 6.

**Kept exactly as implemented today, not touched by this phase:**

- `GameOrServerRunning` guard (§2 item 2).
- `Arc<Mutex<Option<Library>>>` lock model, the dirty flag / `deployStale`, the Sync button as a
  distinct explicit deploy step (`design-review.md` C1/C3, unchanged).
- Manifest removal (`1.7_mod-manifest-removal.md`) and the backup-to-internal-snapshot collapse
  (`backend-redesign-spec.md` §9a) — both still in scope. These are deliberate scope cuts against
  the minimal-closure goal (§1), not bug fixes, and are not reopened by this revision.

**Fire-and-track machinery (`backend-redesign-spec.md` §8a), rescoped to three commands:**

`install_mod_archives`, `rebuild_library_cache`, and `sync_mods` keep the client-minted `taskId` +
persistent event bus + completion-event correlation exactly as specified. `bulk_update_mods` drops
out of this list (§2 item 3) — it becomes a normal `Result<LibraryWorkspace, SError>` call, the same
shape as `create_library`/`activate_library`/`rename_library`/`delete_library`.

**Deferred, not built this pass:** the executable tool registry in full (§2 item 1).

## 5. Phase 3 — Frontend Redesign, Staged by Layer (Complete)

`frontend-redesign-spec.md`'s screen specs, design tokens, and acceptance criteria are unchanged.
What changes is the build order: a design-system stage, then a single vertical slice that proves the
cross-layer contracts end-to-end, then three horizontal stages that build out against contracts the
slice has already validated. The slice exists so a wrong atom/repository/UI contract surfaces once —
cheaply, against one screen — instead of at the end when every screen already depends on it.

### 5.1 Structure Stage — design system, shared primitives, Storybook

Deliverables:

- `src/redesign/styles/fidelity.css` — tokens + utilities (`frontend-redesign-spec.md` §6,
  unchanged).
- `src/redesign/shared/components/*` — `fidelity-panel`, `fidelity-button`, `fidelity-icon-button`,
  `fidelity-input`, `fidelity-section`, `confirm-dialog`.
- **Storybook for these shared primitives — new this revision.** Scoped to `shared/components/*`
  only, not screens or business logic:
  - `@storybook/react-vite` as the builder, matching the existing Vite + React 19 toolchain (no new
    bundler introduced).
  - Config at `.storybook/main.ts` / `.storybook/preview.ts`; stories colocated next to each
    component (`fidelity-button.stories.tsx` beside `fidelity-button.tsx`, etc.), not in a separate
    top-level `stories/` tree.
  - Each shared primitive gets a default story plus one story per visually distinct state
    (disabled, focus-visible, error/destructive) — enough to eyeball the whole design system in
    isolation before any screen consumes it.
  - `bun run storybook` script added in this stage (devDependency + script), not deferred to
    verification.
- **Exit criterion:** every shared primitive renders in Storybook standalone, with no dependency on
  `redesign-types.ts`, repositories, or Jotai state. If a component needs business data to render,
  it isn't a shared primitive — it belongs in the Composition stage instead.

### 5.2 Walking Skeleton — one screen, all layers

Before the horizontal build-out, cut one thin vertical slice through every layer to validate the
contracts the horizontal stages would otherwise only discover at the end (§5's rationale). Scope: the
Library screen reduced to a single mod card plus the execution bar — enough to exercise every
cross-layer seam once, nothing more.

Deliverables (thin, one instance each — not the full build-out, which stays in 5.3-5.5):

- A minimal route + shell mount (thin slice of 5.3) so the slice renders in the real app, not just
  Storybook.
- One repository + one atom pair for the mod list (thin slice of 5.4), backed by `example-data.ts`
  via the real-first/mock-fallback path — so the `MOCK-FALLBACK` mechanism is exercised, not just
  declared.
- One **plain** call (`toggleModStatus`) with its local per-click pending state, and one
  **fire-and-track** call (`syncMods` or `rebuildLibraryCache`) with its client-minted `taskId` +
  event-bus registration — so both repository shapes are proven against a real consumer.
- The derived **"library busy"** state wired producer-to-consumer: produced by the fire-and-track
  call, read by both the shell affordance and the card's disabled/pending treatment. This is the one
  contract §5's analysis flagged as split across three horizontal stages; the slice collapses it into
  one checkable path.
- One mod card + toggle + execution bar (thin slice of 5.5) consuming the above.

- **Exit criterion:** clicking the toggle on the real card updates through the real atom/repository
  path; triggering the fire-and-track call flips "library busy" and both the shell and the card
  reflect it; killing the real backend call falls back to mock without a crash. Once green, the atom
  shape, repository shape, and library-busy contract are frozen — 5.3-5.5 build out against them, not
  toward them. If the slice forces a shape change here, that is the plan working as intended.

### 5.3 Global Stage — routes, store, i18n scaffolding

Everything app-wide and screen-independent:

- `src/redesign/app/*` — `redesign-root.tsx`, `redesign-error-boundary.tsx`,
  `redesign-initializers.tsx`.
- `src/redesign/shell/*` — `desktop-shell.tsx`, `app-background.tsx`, `app-header.tsx`,
  `bottom-navigation.tsx`, `page-title.tsx` (`frontend-redesign-spec.md` §5).
- Route adapters (`frontend-redesign-spec.md` §4's table) — thin, mount `RedesignRoot` /
  `LibraryScreen` / `SettingsScreen`.
- `src/redesign/state/*` — `library-state.ts`, `settings-state.ts` (atom shapes only at this stage;
  full wiring to real data happens in 5.4, minus the single atom already proven in the 5.2 slice).
- `src/redesign/i18n/*` — `common-text.ts`, `library-text.ts`, `settings-text.ts`, `error-text.ts`.
  Skip populating `tool-text.ts` with real copy — the tool registry is deferred (§2 item 1); leave
  the file empty or omit it until that work is picked back up.
- The new/old UI runtime toggle (`frontend-redesign-spec.md` §5a) — shell-level, lives here.
- **Exit criterion:** the app boots to a working shell — header, nav, routing all functional —
  before any real or mock data flows through it.

### 5.4 Business Layer Stage — data, repositories, state wiring

Everything that decides what the UI shows, independent of how it's drawn:

- `src/redesign/data/*` — `redesign-types.ts`, `example-data.ts`, `library-repository.ts`,
  `settings-repository.ts` (`tool-repository.ts` stubbed only if trivial; real content deferred with
  the tool registry, §2 item 1).
- Real-first/mock-fallback rule (`frontend-redesign-spec.md` §7) — `MOCK-FALLBACK` tag discipline,
  unchanged.
- `bulkUpdateMods` / `toggleModStatus` wired as plain awaited calls (§2 item 3) — no client-minted
  `taskId`, no event-bus registration. Local per-click pending state only: set the instant the
  button is clicked, cleared when the promise settles — gives instant feedback even when the call
  is queued behind a heavy operation's lock hold, without a backend-driven accept/complete cycle.
- `rebuildLibraryCache` / `installZipArchives` / `syncMods` keep the fire-and-track repository shape
  exactly as `frontend-redesign-spec.md` §7 specifies (client-minted `taskId`, register-before-
  invoke, `listenWorkspaceEvent`).
- A derived **"library busy"** state — true whenever a fire-and-track task is pending for the
  active library — surfaced for the Global stage's shell/execution bar to read. This is what turns
  an unexplained delay (e.g. a toggle queued behind a large install) into a visible "library busy"
  affordance instead of a silent hang.
- Wire `state/library-state.ts` and `state/settings-state.ts` atoms to this repository layer.
- **Exit criterion:** every atom from 5.3 is backed by a real repository call or its mock fallback;
  no screen-level component exists yet beyond the 5.2 slice's single card, but the data/state layer
  is fully functional against `example-data.ts` on its own.

### 5.5 Composition Stage — screens, dialogs, cards

Everything previously covered in one pass by `frontend-redesign-spec.md` §9, now built last, on top
of working structure/global/business layers:

- Library composition (`library-screen.tsx`, `library-content.tsx`, empty states, toolbar, execution
  bar, grid, cards) — §9.1-9.3.
- Manage Library dialog — §9.4, with its tools section rendering empty/hidden per the deferred tool
  registry (§2 item 1) rather than built out this pass.
- Configure Tool dialog — **not built this pass**, deferred with the tool registry.
- Settings screen — §9.6, minus any tool-related rows.
- **Exit criterion:** `frontend-redesign-spec.md` §14's acceptance criteria, scoped down by §2's
  deltas (no tool-registry UI, no fire-and-track on `bulk_update_mods`).

## 6. Phase 4 — Logging Fix (Complete)

Unchanged from `docs/2026-07-09_redesign/outline-of-redesign.md` Phase 4: backend structured
logging + file sink for `tauri-plugin-log`, frontend global error boundary at `__root.tsx`, no
silent `.catch(() => {})` on IPC calls. Carried forward as-is.

As landed: `tracing`'s `log` feature forwards every `tracing::*` event into the `log` facade
(no separate tracing subscriber — the unused `tracing-subscriber`/`tracing-appender` deps are
dropped), and `tauri-plugin-log` sinks to stdout plus a `modkeeper` file in the platform log dir.
Every command logs its English `Display` line via `commands::log_err` at the point it returns
`Err` (§7g of the consolidated spec); fire-and-track outcome failures are logged where their
completion events are built. The frontend side (error boundary at the redesign root, no silent
catches) landed with the Global stage; the legacy tree's remaining `.catch(() => null)` rides out
with the later legacy-cleanup pass.

## 7. Phase 5 — Verification

Unchanged in substance from `docs/2026-07-09_redesign/outline-of-redesign.md` Phase 5, adjusted for
§2's deltas:

- **5.1 Build & Smoke Test:** `cargo build`/`clippy` clean, `bun run build`/`lint` clean, backend
  test oracle green throughout Phase 2. Add: `bun run storybook` builds clean — checked
  continuously from the Structure stage onward, not just at the end.
- **5.2 Functional Verification:** core workflows unchanged, minus tool configure/execute
  (deferred). Manual checks per `frontend-redesign-spec.md` §13, scoped down for the deferred tool
  dialog.
- **5.3 Document Decisions:** OOP-vs-FP decisions, endpoint contract, i18n key convention — as
  before.

## 8. Phase 6 — Comment Removal

Run once, after Phases 2-5 land and the codebase has stopped moving structurally (§2 item 4). Batch
pass, file by file, not ad hoc. Exempt comments that encode non-obvious platform/runtime
constraints (the `linker.rs` Windows symlink fallback is the concrete example already on record) —
strip comments that restate what the code already says, keep comments that explain why.

## 9. Deferred / Cut, Collected

- **Executable tool registry** — backend `tool.rs`/`tool_service.rs`/`tools.toml`/icon pipeline,
  frontend Configure Tool dialog + Manage Library's tools content. Deferred, not cancelled.
- **SQLite Library DB** — already cancelled in `design-review.md` C4; still cancelled here.
- **Fire-and-track wrapping for `bulk_update_mods`** — replaced by a plain blocking call plus the
  local pending-state + "library busy" UI pattern (§5.3).
