# Current Implementation Audit

Status: Snapshot of the codebase as it exists today (pre-redesign), derived by reading the source
directly (`src-tauri/src/**`, `src/**`) — not from any prior planning document. Purpose: back-track
from what's actually built to the ideas and technical decisions behind it, as a factual baseline
other redesign docs can be checked against.

## 1. What the App Is

Modkeeper is a desktop mod manager for **SPT** (Single Player Tarkov, a mod for Escape from
Tarkov). A "library" pairs one **game root** (an SPT install) with one **repo root** (Modkeeper's
own storage directory for that install, `game_root/.mod_keeper` by default). The app lets a user:

- Import mods from `.zip` archives, folders, or loose files (drag-drop or file picker).
- Track mods per library, each classified as `Client` (BepInEx plugin), `Server` (SPT server mod),
  `Both`, or `Unknown`.
- Enable/disable mods without touching disk, then explicitly **sync** — which deploys enabled mods
  into the game root via symlinks and removes symlinks for disabled ones.
- View rich mod metadata (name, version, author, description, dependencies, links, effects,
  documentation) when a mod ships a `manifest/manifest.json`.
- Back up a mod's files before overwriting it, and browse/restore/delete those backups.
- Switch between multiple libraries (multiple SPT installs), rename/close/remove them, rebuild a
  library's cache if its on-disk mod folders drift from recorded state.

It's built as a Tauri v2 app: Rust backend (`src-tauri/`) doing all filesystem/domain work, React
19 + TypeScript frontend (`src/`) as the UI, talking over Tauri's IPC via `tauri-specta`-generated
bindings (`src/gen/bindings.ts`).

## 2. Domain Model

### Library

`core/library.rs::Library` is the central stateful object — one loaded at a time, held in
`AppRegistry.active_instance: Arc<Mutex<Option<Library>>>`. It owns:

- `id` (uuid, minted once at `Library::create`, persisted in that library's own `manifest.toml` —
  this is the only place a library's identity lives; nothing else mints or duplicates it).
- `mods: BTreeMap<String, Mod>` — the *recorded* state (which mods exist, which are enabled).
- `cache: LibraryCache` (`core/cache.rs`) — the *derived* state: each mod's file list and inferred
  type (`ModFS`), scanned from disk. Cache and `mods` are two different sources of truth that must
  stay reconciled (see §3.3).
- `is_dirty: bool` — set whenever mod state changes (add/remove/toggle), cleared only by
  `Library::sync()`. This is the one flag that answers "does the game root currently match what
  the user asked for."
- Path rule structs (`SPTPathRules`, `LibPathRules`, `SPTPathCanonical`) — where things live inside
  the game root and the repo root, plus canonicalized exe paths used for the running-process check.

Persistence is two plain TOML files per library, written together by `Library::persist()`:
`manifest.toml` (the `LibraryDTO` — id, name, roots, spt_version, mods) and `cache.toml` (the
`LibraryCache`). No database anywhere in the current implementation.

### Mod / ModManifest

A `Mod` (`models/mod_dto.rs`) is the *recorded* entity: id, `is_active`, `mod_type`, `name`, an
optional `ModManifest`, and transient `icon_data` (populated only when building the frontend DTO,
never persisted). `ModManifest` is a rich, author-supplied metadata shape (version, author,
description, icon filename, documentation filename, `Compatibility` include/exclude lists,
`Dependencies` (object-or-array union), `Effect` tags, `Link` list) — read from
`<mod>/manifest/manifest.json` if present. Nothing requires a manifest to exist; a mod with no
manifest is still fully functional, just with no metadata to show.

### Mod Identity

A mod's `id` is resolved by `core/mod_fs.rs::ModFS::resolve_id`, in priority order:

1. If `manifest/manifest.json` exists, its own `id` field wins.
2. Otherwise, hash-derive one: collect every file under `SPT/user/mods/<name>` (server) and every
   `.dll` under `BepInEx/plugins/**` (client), sort those relative paths, concatenate, and
   `blake3`-hash + base64url-encode the result (`utils/id.rs::hash_id`) into a ~22-char id.

This means identity today is **manifest-first, hash-fallback** — not purely hash-based. The hash
path exists specifically so mods with no manifest still get a stable, content-derived id instead
of depending on a folder name a user might rename.

### Mod Type Inference

`ModFS::infer_mod_type` classifies a mod by which known SPT subtrees its files fall under
(`BepInEx/plugins` → client, `SPT/user/mods` → server, both → `Both`, neither → `Unknown`) — purely
structural, no manifest involved.

## 3. Backend Architecture

### 3.1 Layering (as it exists, not as a target)

```
commands/          Tauri #[command] entry points — IPC boundary
  global.rs         app-level: init, open/create/close/remove library, apply_window_effect
  library.rs        active-library ops: add/remove/toggle mods, sync, backups, rename, rebuild
  test.rs           create_simulation_game_root (debug scaffolding — see §6.3)
core/               business logic
  library.rs         the Library struct itself (stateful, OOP)
  library_service.rs global_service.rs   orchestration functions taking &mut GlobalConfig/&Library
  mod_manager.rs      add_mod / remove_mod / toggle_mod — free functions over &mut Library
  mod_stager.rs       resolves raw user input (paths/archives/folders) into StagedMod values
  mod_fs.rs           ModFS: id resolution + type inference (struct with static-ish methods)
  deployment.rs       symlink planning + collision detection
  cleanup.rs          purge (remove all managed symlinks) + per-mod unlink
  linker.rs           raw symlink create/read/remove (OS-level primitive)
  cache.rs            LibraryCache: on-disk mod scan + folder-name normalization
  decompression.rs    zip extraction (zip-slip safe)
  mod_backup.rs       timestamped backup create/list/restore/remove
  mod_documentation.rs reads a mod's doc file per its manifest
  version.rs          SPT version fetch (registry.json) + semver validation (^4)
  dto_builder.rs       builds the enriched frontend DTO (manifest + icon data attached)
  registry.rs          AppRegistry: the Tauri-managed global state container
models/             DTOs, error type, path-rule structs
utils/              file/toml/icon/id/process/thread/time — mostly stateless helpers,
                    some as structs-with-static-methods (FileUtils, Toml, ProcessChecker)
config/global.rs    the actual, used GlobalConfig (confy-persisted)
```

There is no separate "service" boundary consistently enforced — some commands call a service
function that calls core (`open_library`), others inline orchestration directly in the command body
inside a `spawn_blocking` closure (`add_mods`/`remove_mods` in `commands/library.rs` build the
staged mods, install them, and build the DTO all in one closure).

### 3.2 State & Concurrency

`AppRegistry` (`core/registry.rs`) is the one piece of Tauri-managed state:

- `active_instance: Arc<Mutex<Option<Library>>>` — the currently open library, if any. Only one
  library is ever loaded at a time; switching libraries drops the old one.
- `global_config: Arc<Mutex<GlobalConfig>>` — the confy-persisted app config.
- `sys: Mutex<System>` — reused `sysinfo::System` handle for the running-process check.
- `init_called: Arc<AtomicBool>` — see §3.5.

Every mutating command follows the same shape: clone the relevant `Arc` handles, run the actual
work inside `tauri::async_runtime::spawn_blocking`, and lock the mutex for the duration of the
operation (`utils/thread.rs::with_lib_arc`/`with_lib_arc_mut`). There is no operation queue and no
overlap guard beyond the mutex itself — concurrent mutating calls simply serialize on lock
acquisition, in whatever order the OS scheduler grants it (`parking_lot::Mutex`, not FIFO).

### 3.3 The Two Sources of Truth: `mods` vs `cache`

`Library.mods` (recorded — what the user asked for) and `Library.cache.mods` (derived — what's
actually on disk, keyed by the same id) are separate maps that must be kept in step by hand at
every call site. `rebuild_library_cache` exists specifically to reconcile them when they drift:
it re-scans `mods/` on disk, renames folders that don't match their resolved id
(`cache::normalize_mod_folders`), rebuilds `cache` from scratch, and re-inserts `mods` entries for
anything renamed (preserving the old `is_active` value under the new key). This also silently
drops backups for renamed mods (`mod_backup::remove_all_backups` is called from inside the rebuild
path in `library_service::rebuild_library_cache`).

### 3.4 Deployment Model: Ownership-Map Symlinking

`core/deployment.rs::plan` is the core algorithm; `deploy` applies its recorded plan:

1. **Collision check** (`check_file_collisions`): walk every file of every *active* mod; if two
   different mods claim the same relative path, fail the whole sync with `SError::FileCollision`
   listing every conflicting path. No partial deploy on collision.
2. **Ownership map** (`build_folder_ownership_map`): for every active mod's file, walk all of its
   path *ancestors* too (not just the leaf file), recording which mod ids "touch" each path.
   `SPT/user/mods` and `BepInEx/plugins` are seeded as owned by a synthetic `"__SYSTEM__"` entry so
   they're never treated as a mod's exclusive folder.
3. **Recursive ownership plan**: walk each active mod's files path-component by path-component.
   The first ancestor uniquely owned by exactly one mod becomes one artifact at that level; the
   selected file/directory method is chosen by preflight and can be a symlink, junction, hardlink,
   or copy.
   An ancestor owned by 2+ mods is left as a real, physically-created directory instead of a link,
   so siblings from different mods can coexist inside it.

This means one mod can produce anywhere from one directory-level symlink to many file-level
symlinks, depending on whether other active mods also touch the same subtree. `find_mod_links`
(used by `remove_mod`) reads the deployment record to compute exactly what a *specific* mod
contributed, including protected system roots explicitly excluded from ever being unlinked
(`deployment::is_protected_path`).

`Library::sync()` is `plan/preflight → compatibility purge → reconcile recorded artifacts → mark
clean → persist`. `deployment.toml` records targets relative to the game root, sources relative to
the repository, artifact kinds, and directories created by deployment. The compatibility purge only
removes old unrecorded symlinks that point into the repository; current cleanup is record-based and
never overwrites an unowned game target.

### 3.5 Lifecycle & Startup

`lib.rs::run()` is staged explicitly (comments literally number the stages 1–7):

1. Register all Tauri commands (`collect_commands!`).
2. In debug builds only, re-export TypeScript bindings to `../src/gen/bindings.ts` on every launch
   (`specta-typescript` + `tauri-specta`) — the frontend's binding file is generated, not hand
   -written, and only regenerates automatically in debug.
3. Construct `AppRegistry` (loads `GlobalConfig` via confy synchronously at construction).
4. Register plugins: `opener`, `log` (info level, no file sink configured), `dialog`.
5. On Tauri's `setup` hook: attempt to load `library_last` from config **in a background thread**
   (`load_initial_library`) so it doesn't block window creation; also start a **10-second watchdog**
   (`start_init_timeout_checker`) that hard-exits the process (`std::process::exit(1)`) if the
   frontend's `init` command hasn't been called by then.
6. The frontend's `init` command (`commands/global.rs::init`) marks the watchdog satisfied, reads
   current state, and — notably — is also responsible for **showing and focusing the main window**
   (`window.show()`/`set_focus()`/`center()`). The window is not shown until the frontend
   successfully calls `init`, which is the mechanism that prevents a flash of an unstyled/unthemed
   window on launch.

### 3.6 Error Model

One flat enum, `SError` (`models/error.rs`), covers everything — no per-layer error types. It
derives `specta::Type` + `Serialize`/`Deserialize` with default (untagged-by-variant) serde
representation, and `Display` (via `derive_more`) for human-readable messages. A macro
(`impl_from!`) blanket-converts common foreign errors (`io::Error`, `semver::Error`,
`serde_json::Error`, `zip::result::ZipError`, path-strip errors) into the appropriate `SError`
variant by stringifying the underlying error. Commands return `Result<T, SError>` directly — the
raw enum crosses the Tauri IPC boundary as-is (see §4.4 for how the frontend consumes it).

### 3.7 Config: What's Live vs What's Dead

`config/global.rs::GlobalConfig` is the one actually used, confy-persisted app config — just
`library_last: Option<Utf8PathBuf>` and `library_recent: Vec<Utf8PathBuf>` (a simple MRU list of
plain paths, no ids). Loaded via `confy::load(...).unwrap_or_default()` (silently defaults on any
read/parse failure) and saved via `let _ = confy::store(...)` (write failures are silently
discarded — `save()` returns `()`, not a `Result`).

`models/config.rs::GlobalConfig` is a **second, differently-shaped, unused struct** — same type
name, different module, `last_opened_instance`/`known_instance_paths` fields — with zero references
anywhere else in the codebase. Dead code left over from an earlier shape.

## 4. Frontend Architecture

### 4.1 Routing & Shell

TanStack Router, file-based (`src/routes/*`), code-split via `.lazy.tsx` files for anything with a
heavier component tree. `__root.tsx` mounts three side-effect-only initializer components
(`LibraryInit`, `SettingsInit`, `FileDropHandler`) plus a persistent header/nav shell
(`AppNavigation`: Home + Settings icon buttons) and a `HeaderPortalContext` — child routes portal
page-specific header content (the instance switcher, add-mod menu, sync button) into that shared
header via `HeaderPortal`, rather than the header knowing about every route's needs.

Three top-level areas: `/` (redirects to `/library`), `/library` (list + empty state, mod grid),
`/library/$id` (mod detail — loader prefetches backups + documentation before render), `/settings`
(tabs: General, Developer).

### 4.2 State

Jotai, minimal: `src/store/library.ts` holds exactly one base atom, `ALibrarySwitch` (mirrors the
backend's `LibrarySwitch` DTO — `{ active, libraries }`), plus two derived read atoms
(`ALibraryActive`, `ALibraryList`). All mutation flows through a single pattern
(`utils/function.ts::createSetter`): wrap a backend command so its resolved value is both unwrapped
(`ur`, `utils/result.ts`) and written into the atom in one step — every hook that mutates library
state (`useLibrary`, `useLibrarySwitch`) is built from the same primitive, so there's exactly one
way a command's result reaches the store.

Everything else (dialog open/closed, selected mod, rename input value) is local component state —
there is no global UI/dialog atom in the current implementation.

### 4.3 Data Flow / Bindings

`src/gen/bindings.ts` is machine-generated (`tauri-specta`) from the exact same `SError`/DTO types
the backend uses — frontend and backend share type definitions by construction, not by convention.
Every command call goes through `ur()` to unwrap the `Result<T, SError>` Rust-shaped return into a
thrown-on-error promise, and failures are funneled through `ett()` (`utils/error.ts`) which — per
call site — typically logs and/or surfaces a toast built from `translateError()`.

### 4.4 Error Presentation

`lib/error.ts::translateError` pattern-matches on `SError`'s shape (bare string for unit variants,
single-key object for variants with data, e.g. `'InvalidLibrary' in error`) and returns a
Lingui-translated, human-readable string per variant. This means **today, the mapping from error
code to user-facing message lives entirely on the frontend**, keyed off the Rust enum's exact
serialized shape — the backend does not send pre-translated text, but it does send the raw variant
name and payload, which the frontend interprets directly rather than through a stable `{code,
data}` contract (default serde external tagging, not a normalized shape).

### 4.5 Feature Modules

- **`modules/mod-list/`**: `ModList` (grid), `ModCard` (icon, name, dependency-issue badge,
  reveal-in-explorer, remove w/ confirm popover, enable toggle), `InstanceSwitcher` (dropdown:
  switch/rename/rebuild-cache/close/remove per known library, "add library" entry), plus three
  focused confirm/rename dialogs.
- **`modules/mod-details/`**: full mod detail page — header (icon, name, version/author, toggle,
  reveal, remove), sidebar (id/version/SPT-version card, links card, dependency-status card,
  backups card with restore/remove), markdown-rendered documentation or manifest description as
  main content.
- **`modules/settings/`**: theme (light/dark/system + a preset/custom primary color picker that
  writes directly to a CSS custom property), language (single locale today, `en-US`), import/export
  settings as a downloaded/uploaded JSON file, and a **Developer** tab
  (`developer-settings.tsx`) that calls `create_simulation_game_root` to scaffold a fake SPT
  install for manual testing, then can create a library from it in one click.
- **`utils/dependency-check.ts`**: entirely client-side dependency resolution — reads a mod's
  manifest `dependencies` (object-or-array union), looks up each referenced id among currently
  known mods, and classifies each as Missing / Mismatch / Satisfied via `semver.satisfies`. The
  backend has no concept of dependency validation at all; it's a frontend-only, advisory feature
  layered on top of manifest data the backend merely passes through.

### 4.6 Settings Storage

`lib/settings-storage.ts` — `localStorage`, Zod-validated (`theme`, `primaryColor`, `language`),
with safe fallback to defaults on any parse/validation failure. Backend has no knowledge of these
settings at all (window vibrancy is applied by a *separate* call, `apply_window_effect`, driven by
`next-themes`'s resolved theme at runtime — the backend doesn't persist or read theme state itself).

## 5. Cross-Cutting Technical Decisions Observed

- **Symlinks, not copies.** The entire deployment model depends on OS-level symlink support
  (`std::os::windows::fs::symlink_dir/file`), which on Windows requires Developer Mode or admin
  rights — there is no fallback copy-based deploy path.
- **Manifest-first, hash-fallback identity**, not purely hash-based (§2, Mod Identity) — a stated
  simplification target in other planning docs, but not what's implemented today.
  **Correction context:** the mod's own manifest `id` field, when present, always wins over the
  content hash.
  - **Backups are tied to overwrite, not to every write.** `mod_manager::add_mod` only creates a
  backup when the destination folder already exists (i.e., this is an upgrade of an existing mod);
  a fresh install never produces a backup. Backups are timestamped and multiple can coexist per mod
  (`ModBackup`/`BackupManifest`), browsable and individually restorable/removable from the mod
  detail page — a full user-facing history feature, not an internal safety net.
- **One active library at a time**, enforced structurally by `Arc<Mutex<Option<Library>>>` holding
  at most one loaded `Library`. Switching libraries drops the previous one's in-memory state
  entirely (any unsynced/dirty state is simply discarded from memory — still recoverable from disk
  since `mods`/`cache` were persisted on every mutating call, but the in-memory `Library` object
  itself doesn't survive a switch).
- **Process-running guards exist for exactly two operations** — `sync_mods` and `remove_library` —
  both check `AppRegistry::is_game_or_server_running()` (matches live process exe paths against the
  library's canonicalized client/server exe paths via `sysinfo`) and refuse with
  `SError::GameOrServerRunning` if either is running. `add_mods` has a narrower, different check
  (`mod_stager::any_mod_tool_running`) against executables bundled *inside* the mods being staged,
  not the game itself.
- **The debug-only test command isn't actually gated.** `commands/test.rs::create_simulation_game_root`
  is documented as "only available in debug builds" but is registered unconditionally in
  `lib.rs::setup_command_handler` — no `#[cfg(debug_assertions)]` on the command itself or its
  registration (only the bindings *export* step is debug-gated). It ships in release builds as-is.
- **Two unused artifacts confirmed by direct reference search:** the `help` crate
  (`Cargo.toml` dependency, zero `use`/reference anywhere in `src-tauri/src`) and
  `models/config.rs::GlobalConfig` (a second, differently-shaped, never-referenced struct that
  happens to share a name with the real one in `config/global.rs`).
- **Backend test coverage is concentrated in three integration suites**
  (`src-tauri/tests/library.rs` 1051 lines, `mod_fs.rs` 220 lines, `linker.rs` 129 lines, plus
  shared fixtures in `tests/common/mod.rs`) — exercising `Library` lifecycle, mod id
  resolution/type inference, and raw symlink primitives. The frontend has no automated test files
  anywhere in `src/`.

## 6. Summary: The Idea Behind the Implementation

Strip away specific file layout and the intent reads as: **a single mutable "current library"
object that's the source of truth in memory, mirrored to two plain TOML files on disk, where
"enabling" a mod is a cheap recorded-state flip and "deploying" it is a separate, explicit,
potentially-expensive symlink-planning step the user triggers on purpose** — so a user can queue up
several toggles and only pay the collision-check/relink cost once. Around that core sits: a
manifest-driven metadata layer that's entirely optional (mods work with zero manifest, hash-only
identity), a backup safety net scoped to overwrite-in-place upgrades, and a thin app-level registry
(confy + plain paths) whose only job is remembering which library directories exist and which one
was open last. The frontend mirrors this shape almost directly — one `LibrarySwitch` atom, one
generated binding surface, one unwrap-and-toast error convention — rather than introducing its own
independent state model.
