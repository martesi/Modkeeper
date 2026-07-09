# Purpose of Redesign

Status: Living document. Supersedes `docs/2026-05-10_redesign/purpose-of-redesign.md`.

Changes since 2026-05-10: added the SQLite state/index layer and executable tool registry as
in-scope backend requirements, and added an explicit Execution Strategy section so the backend
and frontend tracks stop being planned as if they were the same kind of change.

## Backend

### Build Standard of the Diagram

The current codebase is mixed with OOP and FP, without a clear line on why one is needed. For ideal flow, FP is the best choice. However, if OOP is needed for a good reason, it should stay, and we accept the category to be OOP.

### Form a Clear Boundary for Core, Service, and Interface

The current service level doesn't have a clear boundary of what should be done and what should not. A service should take parameters from interfaces, call core functions and utils, and finish jobs with a pipeline. If the job is complicated, it can have its own dedicated file for its pipeline. The core and util functions should strictly hold to one purpose only, avoiding monoliths.

### Separate Low-Level FS and Higher-Level Business

In `ModFS`, there are low-level FS operations mixed with high-level business logic. Separating both would give a better view of the logic and business hierarchy.

### Reduce Unintended Distraction

Remove all comments as they are not necessary to understand the context. If they are needed, the code might have grown too complex and needs separation.

### Build Principles for Endpoints

The original endpoint design doesn't follow a specific set of rules. One can set the scope and intent generally, avoiding deviation over a large range.

### Adopt SQLite as the Library State and Index Layer

A library's own mod, tool, and cache/index state currently lives in scattered manifest and cache files. That state should move into a SQLite database inside that library's own mod-manager directory under its game root, replacing that library's `manifest.toml`/`cache.toml`. A library's data travels with the library, not with the app installation. Files on disk stay authoritative for actual game and mod content — SQLite never stores content itself, only registration, state, and derived index data (e.g. cache/scan results).

This does not apply to app-level state (the list of known libraries, which one is active, app settings). That state is separate from any single library and small enough that it doesn't need a database — see the next section.

### Keep App-Level State as a Plain Config File

The list of known libraries, the active selection, and app settings are small and flat — nothing about them benefits from SQLite's query/index capabilities the way a library's mod list does. This stays a single config file, not a database. Whether it's `confy` (current) or the `toml` crate directly is a `backend-redesign-spec.md` decision, not a purpose-level one, but either way it's one file, not a SQLite database — don't scale the tooling to the size of the data.

Settings specifically move out of frontend `localStorage` and into this same file: some settings (theme, for window vibrancy at startup) are needed before the frontend has loaded, so they must be readable by the backend at startup, which `localStorage` can't provide.

### Add an Executable Tool Registry

A library can register external executables (servers, launchers, editors) by name, icon, executable path, and launch arguments. Configuring a tool and executing a tool are separate concerns and must be separate endpoints — saving a tool's config must never launch it, and launching a tool must never fail silently.

## Frontend

### Redesign UI

This is the core of this design. The previous design doesn't look coherent enough — it is a mix of Fluent Design and the standard web application aesthetic. In the new design, the contrast should be guaranteed, with or without a transparent background.

The second issue that needs to be addressed is that the header is not fixed, and the scrollbar doesn't look right.

### Set Rules for i18n

AI tends to create new utils all the time, and the same goes for text that needs translating. While `lingui` provides great convenience, we still need to avoid repeated translations for the same content used in different places.

## Execution Strategy

The backend and frontend redesigns are structurally different kinds of change, so they follow different execution strategies. Downstream planning docs must not blur the two.

### Backend: Mechanical Refactor In Place

The backend restructure is a refactor of the existing codebase, not a rewrite. The existing test suite (after the Phase 1 test audit) is the correctness oracle for the whole effort: refactor toward the FP/service/interface boundaries incrementally, keep tests passing at every step, and expand coverage where the audit finds gaps instead of deferring correctness checks to the end. Behavior should not change as a side effect of restructuring — a passing test that starts failing because of a structural change is a regression, not an acceptable cost.

### Frontend: Build Fresh, Verify Continuously

The frontend redesign is built as new files in a dedicated directory. Existing frontend files are reference material only during this phase — they are not edited or deleted. Old routes become thin adapters that mount the new implementation only once it exists and is verified; removing the old files is a separate, later, explicit task, not part of the redesign itself.

The frontend does not wait for the backend to be fully finished before touching real data. As soon as the backend stubs a command matching the new contract, the frontend calls it directly — not the old `src/gen/bindings.ts` surface, the new one. Where the backend isn't implemented yet, or returns something too empty to usefully check the UI against (an empty mod list when the point is to check grid layout, for instance), the repository falls back to example data, clearly marked so it's easy to find and delete once the real path is complete. This lets both sides be checked against each other continuously instead of as two isolated efforts joined at the very end.

A runtime toggle between the new and old UI lets both be compared side by side for functional parity while the redesign is in progress — mechanism in `frontend-redesign-spec.md`.

## General

### Audit the Current Codebase and Structure

### Audit Dependencies and Their Necessity

### Logging Fix

The current backend doesn't yield logs correctly, and fatal error on frontend might be eaten.
