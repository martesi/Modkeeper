# Outline of Redesign

Status: Living document. Supersedes `docs/2026-05-10_redesign/outline-of-redesign.md`.

Companion docs:

- `docs/2026-07-09_redesign/purpose-of-redesign.md` — why, and which execution strategy applies to which track.
- `docs/2026-07-09_redesign/backend-redesign-spec.md` — implementation-ready backend plan (Phase 2).
- `docs/2026-07-09_redesign/frontend-redesign-data-api-contract.md` — the data/API shape the frontend is built against.
- `docs/2026-07-09_redesign/frontend-redesign-spec.md` — implementation-ready frontend plan (Phase 3).

Changes since 2026-05-10:

- Phase 2 now includes the SQLite state/index layer and executable tool registry from `purpose-of-redesign.md`.
- Phase 2 execution method changed to mechanical-port-then-refactor with the test suite as a gating oracle (see "Backend Execution Method" below) — this is a direct adoption of how the Bun team ported Bun from Zig to Rust.
- Phase 3 no longer describes editing `src/modules/*` in place. It now points at the `src/redesign/` build-fresh strategy that `frontend-redesign-spec.md` actually specifies, removing the contradiction between the two documents.
- Phase 1 audit results are folded in as findings, not just as an audit checklist, since Phase 1 is complete.

---

## Phase 1 — Audit & Assessment (Complete)

Audits live in `docs/2026-05-10_redesign/audits/`. Findings that constrain later phases:

- **1.1 Codebase structure:** `core/mod_fs.rs` mixes struct-based state with static logic (`resolve_id`, `infer_mod_type`) and needs to become a plain DTO plus free functions. `utils/file.rs`, `utils/toml.rs`, `utils/process.rs` use the "struct with static methods" pattern and should become top-level functions. `core/library.rs`, `core/registry.rs`, `config/global.rs` are justified OOP (encapsulated mutable state) per `1.5_justification.md`. Two files postdate this audit and are unclassified: `core/mod_manager.rs` (already FP — free functions over `&mut Library`) and `core/decompression.rs` (already FP — single free function). Both are fine as-is.
- **1.2 Dependencies:** `help` (Cargo.toml) and `radix-ui` (package.json) are confirmed unused — remove both. `confy` can likely be dropped in favor of direct `toml` usage once the SQLite migration lands (see Phase 2.6).
- **1.3 Logging:** No global React error boundary (`__root.tsx` has no `errorComponent`), IPC calls (e.g. `commands.init()`) aren't consistently caught/toasted, and `tauri-plugin-log` has no file sink configured. These are Phase 4 blockers, not nice-to-haves — L-001 and L-002 are rated High severity.
- **1.4 Tests:** Backend integration tests (`tests/library.rs`, `tests/linker.rs`, `tests/mod_fs.rs`) are the main safety net and are rated high-value — they become the correctness oracle for Phase 2 (see below). Frontend has **zero** automated tests. This is a known, accepted gap for the Phase 3 prototype (it's UI-only, mock-data-driven); it should be revisited once Phase 3 moves off mock data.
- **1.7 Manifest removal:** Already decided and scoped — hash-only mod identification, ~230 backend lines and ~474 frontend lines removed. This lands inside Phase 2 (backend) and is a precondition for the simplified mod card in Phase 3 (no version/author/dependency fields to show because they no longer exist).

---

## Phase 2 — Backend Restructure

### Backend Execution Method

Per `purpose-of-redesign.md`'s Execution Strategy, this is a refactor, not a rewrite, and it follows a mechanical-port-then-refactor method:

1. **Lock the oracle first.** Before any structural change, get `tests/library.rs`, `tests/linker.rs`, and `tests/mod_fs.rs` green and keep them green through every step in this phase. If a test needs to change because the manifest-removal decision (1.7) changed behavior, make that change first and land it as its own commit, separate from structural refactors.
2. **Port mechanically before refining.** When moving logic out of `core/mod_fs.rs` or the `utils/*` structs (2.2 below), do the boring move first — same logic, new location/shape — get it compiling and green, then clean up. Don't redesign the logic and relocate it in the same step; that makes a failing test ambiguous (did the move break it, or the redesign?).
3. **Batch mechanical violations, don't fix ad hoc.** For cross-cutting rule enforcement (endpoint naming in 2.5, comment removal in 2.7), classify every violation against the rule first, then fix in grouped passes. Don't fix the first one you notice and move on.
4. **Review adversarially.** For each module refactor, have a second, independent pass specifically try to find a boundary violation or behavior change before it's considered done — don't rely on the person who wrote the refactor to also be the one who signs off on it.

### 2.1 Establish Paradigm Standard

- Rule: **FP by default.** OOP only when a module manages mutable state that must be encapsulated (`Library`, `AppRegistry`/`core/registry.rs`, `config/global.rs` — all already justified in `1.5_justification.md`).
- Refactor targets from 1.1: `core/mod_fs.rs` (strip `ModFS` to a DTO, promote `resolve_id`/`infer_mod_type` to free functions), `utils/file.rs` (drop `FileUtils`), `utils/toml.rs` (drop `Toml`), `utils/process.rs` (drop `ProcessChecker`).
- `core/mod_manager.rs` and `core/decompression.rs` are already compliant — no action.

### 2.2 Separate Low-Level FS from Business Logic

- Extract raw filesystem primitives out of `mod_fs.rs` into `utils/file.rs` (post-2.1, this is free functions, not `FileUtils`).
- `core/decompression.rs`'s `extract` is already a low-level, mod-unaware FS primitive — it belongs in the same low-level tier as `utils/file.rs`, not `core/`. Move it during this step.
- `mod_fs.rs` keeps only mod-aware business logic (identification, type inference) that composes the low-level utils.
- No other core module calls `std::fs`/`tokio::fs` directly — verified by grep as an exit criterion for this step, not just a stated goal.

### 2.3 Enforce Core / Service / Interface Boundaries

See `backend-redesign-spec.md` for the full layer table and file-by-file audit. Known violation to fix first: `commands/library.rs::add_mods` currently inlines staging, install, and cleanup orchestration directly inside the command body (inside a `spawn_blocking` closure) instead of delegating to a service pipeline — this is the canonical example of what 2.3 is fixing, and a good first target since it's already flagged.

### 2.4 Build Endpoint Principles

Naming (`verb_noun`), DTO-only input, `Result<T, SError>` output at the command boundary — `SError` is unchanged, returned directly, with one added `#[serde(tag = "code", content = "data")]` attribute so its wire shape is stable (see `backend-redesign-spec.md` §11; there is no separate boundary error type) — one command → one service call, domain-grouped files (`commands/global.rs`, `commands/library.rs`, plus new `commands/tool.rs` for the tool registry). Full audit table of current commands against these rules is in `backend-redesign-spec.md`.

### 2.5 Adopt SQLite as the State and Index Layer

New in this revision — see `purpose-of-redesign.md`. Scope and schema live in `backend-redesign-spec.md` §8; this phase item covers the migration itself. This is two databases, not one:

- An **App Config** (one plain config file per install, app data directory — `toml` crate directly, not `confy`, not SQLite) holding the known-library registry and active selection, plus durable settings.
- A **Library DB** (SQLite) per library, living inside that library's own mod-manager directory under its game root, holding that library's mods, tools, and cache/index data. It replaces that library's `manifest.toml`/`cache.toml`.
- Both are introduced alongside the existing `confy`/manifest/cache files (additive, not a hard cutover).
- The App Config migrates from `confy`'s `GlobalConfig` once, at startup. Each Library DB migrates from that library's own `manifest.toml`/`cache.toml` lazily, the first time that library is opened — not all registered libraries up front.
- Keep on-disk mod/game content files authoritative; SQLite stores registration, state, and derived index/cache data only.
- Cut over `commands/`/`core/*_service.rs` to read/write through the new stores once parity is verified against the test oracle, one domain/library at a time.

### 2.6 Add the Executable Tool Registry

New in this revision. `commands/tool.rs` (interface) + `core/tool_service.rs` (service) + a `tools` table (2.5). Configure and execute are separate commands (`upsert_tool`/`delete_tool` vs. `execute_tool`) per `purpose-of-redesign.md`. Full contract in `backend-redesign-spec.md`.

### 2.7 Remove Unnecessary Comments

Strip comments from backend code as a batch pass (see Backend Execution Method, item 3) after the structural moves in 2.1–2.6 settle, not before — comments in code that's about to move are wasted triage effort.

---

## Phase 3 — Frontend Redesign

### Frontend Execution Method

Per `purpose-of-redesign.md`'s Execution Strategy: build fresh under `src/redesign/`, with old `src/modules/*` files left untouched as reference material. This phase does **not** edit or delete old feature files — that cleanup is explicit and deferred. Repositories call the new backend contract directly as soon as it's stubbed, falling back to example data only where the backend isn't ready yet or returns too little to check the UI against — not "mock-only until a later migration phase." Full detail, file tree, screen specs, and acceptance criteria are in `frontend-redesign-spec.md`; this section stays at the phase-roadmap level.

### 3.1 UI Redesign

- Design system alignment: commit to the Fidelity Modern language (tokens, glass utilities, geometry) defined in `frontend-redesign-spec.md` §6, replacing the current mixed Fluent/web aesthetic.
- Layout fixes: sticky header, styled scrollbar, max-width centered content — all detailed in `frontend-redesign-spec.md` §5.
- New screens/flows this phase adds beyond the original purpose statement: bottom-center navigation dock, unified Manage Library dialog (replacing the scattered instance switcher + rename/close/remove dialogs), Configure Tool dialog, simplified single-column Settings. These exist because the SQLite/tool-registry scope (Phase 2.5–2.6) and the PRD inputs in `docs/2026-05-10_redesign/ui/` expanded the surface area beyond "restyle the existing screens" — see `frontend-redesign-spec.md` for the full screen-by-screen spec.

### 3.2 i18n Rules

Centralized translation helpers grouped by namespace object with plain member names — no `t` prefix or `tText` suffix; the namespace already signals "this is translated text" (e.g. `commonText.home()`, `commonText.cancel()`) — one key per unique string, `domain.context.descriptor` naming for the underlying Lingui key. Full convention in `frontend-redesign-spec.md` §10.

### 3.3 Testing Backlog (carried from 1.4)

The frontend has zero automated tests today, and the redesign prototype (mock data, no backend) doesn't change that. This is an accepted gap for this phase, not a blocker — but it should be picked up once Phase 3 moves off mock data and starts consuming the real API contract, not deferred indefinitely.

---

## Phase 4 — Logging Fix

Findings from 1.3 are the acceptance bar, not just a starting point:

### 4.1 Backend Logging

- Structured logs (`info` on entry/exit, `error` on failure) for service-layer functions — addresses L-004.
- Configure `tauri-plugin-log` with a file sink — addresses L-003.
- Add a log level configuration option.

### 4.2 Frontend Error Handling

- Global error boundary at `__root.tsx` — addresses L-001.
- No silent `.catch(() => {})` on IPC calls, starting with `commands.init()` — addresses L-002.
- Surface errors via toast (`sonner`).

---

## Phase 5 — Verification

### 5.1 Build & Smoke Test

- `cargo build` clean, `cargo clippy` clean.
- `bun run build` clean, `bun run lint` clean.
- Backend test oracle (1.4 suite, expanded per 2.5/2.6 additions) green throughout Phase 2, not just at the end.

### 5.2 Functional Verification

- Core workflows: add library, add mod, remove mod, toggle mod, rebuild cache, configure tool, execute tool.
- UI renders correctly with and without transparent backgrounds; manual checks per `frontend-redesign-spec.md` §13.
- Logs appear in dev console and log file; fatal frontend errors are caught by the error boundary, not swallowed.

### 5.3 Document Decisions

- OOP-vs-FP decisions recorded in `backend-redesign-spec.md` (not left as an audit table only).
- Endpoint contract and SQLite schema recorded as living references in `backend-redesign-spec.md`.
- i18n key convention recorded as a living reference in `frontend-redesign-spec.md`.
