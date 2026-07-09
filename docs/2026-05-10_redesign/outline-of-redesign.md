# Outline of Redesign

> **Superseded.** See
> [`docs/2026-07-09_redesign/outline-of-redesign.md`](../2026-07-09_redesign/outline-of-redesign.md).
> Phase 2 now includes the SQLite state/index layer and tool registry, and Phase 3 no longer
> contradicts `frontend-redesign-spec.md` about editing `src/modules/*` in place. Kept for history.

## Phase 1 — Audit & Assessment

Audit and create files in `docs/2026-05-10_redesign/audits/`.

### 1.1 Audit Codebase Structure

- Map every file in `src-tauri/src/` and classify as OOP or FP.
- Identify files that mix paradigms without justification.
- Document which files legitimately need OOP (stateful structs) and which should be pure FP pipelines.

### 1.2 Audit Dependencies

- **Backend (Cargo.toml):** Review each crate for necessity, maintenance status, and duplication. Flag anything unused or replaceable by std.
- **Frontend (package.json):** Review each npm package. Identify dead dependencies, duplicated functionality, and packages that can be dropped.

### 1.3 Audit Logging

- Trace backend log flow: identify where logs are created, how they're transported to the frontend, and where they get lost.
- Trace frontend error handling: identify where fatal errors are caught and where they're swallowed silently.
- Document every gap found.

### 1.4 Audit Tests

- Map test name and its corresponding module, is it necessary, is it useful?
- Identify tests that can be removed or refactored.
- Document ones that is not useful or necessary

---

## Phase 2 — Backend Restructure

### 2.1 Establish Paradigm Standard

- Define the rule: **FP by default**. OOP only when a module manages mutable state that must be encapsulated (e.g., `AppRegistry`, `Library`).
- For each module classified in 1.1, decide: refactor to FP, keep as OOP with documented justification, or split.
- Apply the decision across `src-tauri/src/core/`.

### 2.2 Separate Low-Level FS from Business Logic

- Extract all raw filesystem operations from `mod_fs.rs` into `utils/file.rs` (or a new `utils/fs.rs` if scope warrants it).
- `mod_fs.rs` retains only mod-aware business logic that composes the low-level FS utils.
- Ensure no other core module directly calls `std::fs` or `tokio::fs` — they go through utils.

### 2.3 Enforce Core / Service / Interface Boundaries

Define and enforce the three layers:

| Layer | Location | Responsibility | Rules |
|-------|----------|---------------|-------|
| **Interface** | `commands/` | Receive IPC params, validate input shape, call a service, return result. | No business logic. No direct FS. |
| **Service** | `core/*_service.rs` | Orchestrate core functions into pipelines. May have dedicated pipeline files for complex jobs. | No direct IPC types. Takes plain params. |
| **Core / Util** | `core/*.rs`, `utils/` | Single-purpose functions. Pure where possible. | No orchestration. No service-level composition. |

Action items:
- Audit `global.rs` and `library.rs` in `commands/` — extract any business logic into services.
- Audit `global_service.rs` and `library_service.rs` — extract any low-level operations into core/util.
- If a service pipeline is complex enough, give it a dedicated file (e.g., `core/deployment_pipeline.rs`).

### 2.4 Build Endpoint Principles

Define a contract for all Tauri commands:

1. **Naming:** `verb_noun` pattern (e.g., `add_mod`, `sync_library`, `get_config`).
2. **Input:** Commands receive only serializable DTOs or primitives. No raw state handles.
3. **Output:** Commands return `Result<T, AppError>` consistently.
4. **Scope:** Each command maps to exactly one service call.
5. **Grouping:** Commands are grouped by domain (`global.rs`, `library.rs`) — no catch-all files.

Audit existing commands against these rules and refactor violations.

### 2.5 Remove Unnecessary Comments

- Strip all comments from backend code.
- If removing a comment reveals that the code is unclear without it, refactor the code for clarity instead of keeping the comment.

---

## Phase 3 — Frontend Redesign

### 3.1 UI Redesign

#### 3.1.1 Design System Alignment

- Choose one design language and commit to it. If Fluent is kept, follow it fully. If dropped, remove all Fluent artifacts.
- Ensure all shadcn/ui components are styled consistently with the chosen language.
- Guarantee contrast ratios meet accessibility standards in both transparent and opaque backgrounds.

#### 3.1.2 Layout Fixes

- Make the header sticky/fixed so it remains visible on scroll.
- Replace or restyle the scrollbar to match the design system (use CSS custom scrollbar or a component like `simplebar`).

#### 3.1.3 Component Audit

- Review `src/components/ui/` — remove unused primitives, ensure remaining ones are consistently themed.
- Review `src/modules/` — ensure feature modules don't duplicate layout concerns.

### 3.2 i18n Rules

Establish and enforce:

1. **Single source of truth:** Each unique user-visible string has exactly one translation key.
2. **Key naming convention:** `domain.context.descriptor` (e.g., `library.dialog.confirmRemove`).
3. **No inline duplicates:** Before creating a new key, search existing translations. Reuse existing keys when the meaning is identical.
4. **Extraction discipline:** Run `lingui extract` as a pre-commit check (already in `lefthook.yml`) and review new keys for duplicates.
5. **Centralized catalog review:** Periodically audit `locales/` for orphaned or duplicate entries.

---

## Phase 4 — Logging Fix

### 4.1 Backend Logging

- Ensure all service-layer functions emit structured logs (at minimum: `info` on entry/exit, `error` on failure).
- Verify the Tauri log plugin is correctly forwarding logs to the frontend console / log file.
- Add a log level configuration option.

### 4.2 Frontend Error Handling

- Implement a global error boundary at the root route (`__root.tsx`) that catches and displays fatal React errors.
- Ensure all IPC calls (`commands.ts` bindings) have consistent error handling — no silent `.catch(() => {})`.
- Surface errors to the user through a toast/notification system.

---

## Phase 5 — Verification

### 5.1 Build & Smoke Test

- Ensure `cargo build` passes with no warnings.
- Ensure `bun run build` (Vite) passes with no errors.
- Ensure `bun run lint` and `cargo clippy` pass clean.

### 5.2 Functional Verification

- Verify core workflows: add library, add mod, remove mod, sync, deploy.
- Verify UI renders correctly with and without transparent backgrounds.
- Verify logs appear in both dev console and log files.

### 5.3 Document Decisions

- Record each OOP-vs-FP decision with rationale.
- Record the endpoint contract as a living reference.
- Record the i18n key convention as a living reference.
