# Technical Insights & Critical Enhancements: Modkeeper Redesign

This document provides a critical analysis of the Product Requirement Document (PRD) for the Modkeeper UI redesign. It identifies product gaps, highlights complex modding edge cases (specifically for environments like Escape from Tarkov / SPT-AKI), discusses Tauri engineering challenges, and proposes advanced AI-driven architecture expansions.

---

## 1. Product & UX Design Gaps Analysis

While the proposed **Fidelity Modern** design system achieves a premium visual style, several crucial functional requirements for high-level mod management are missing from the current UI mocks:

### 1.1 Lack of a Mod Detail View / Configuration Panel
*   **The Issue:** The **Title Only View** represents mods as simple cards with category icons and toggle switches. However, mod files often contain custom settings (e.g., config JSONs for SAIN, graphics presets).
*   **The Gap:** There is no interface to view mod descriptions, author metadata, compatibility notes, readme files, or to edit configuration files directly.
*   **Recommendation:** Implement a slide-out drawer or double-click modal that displays rich markdown details from the mod's readme, and provides a built-in text/JSON editor with syntax highlighting for mod configuration files.

### 1.2 Lack of a Load Order Manager
*   **The Issue:** In many game modding setups (and specifically SPT-AKI), the order in which mods load determines compatibility. Client-side plugins (BepInEx) and server-side mods resolve conflicts based on alphabetical ordering or custom manifest dependencies.
*   **The Gap:** The current grid view has no visual representation of load order, nor does it allow users to manually re-order mods.
*   **Recommendation:** Add a toggle to switch the grid view into a sequential list view. In this list view, allow drag-and-drop handles for manual sorting, and display load indices (e.g. `[01]`, `[02]`, `[03]`).

### 1.3 No Executable Output Logger (Live Console)
*   **The Issue:** Running `Aki.Server.exe` launches an active local Node.js server. Diagnosing mod conflicts, loading warnings, or server exceptions relies entirely on reading this command-line stream.
*   **The Gap:** Currently, clicking "Launch" simply spins up the background process. If the server fails to load or throws an exception due to a mod conflict, the user is left in the dark.
*   **Recommendation:** Integrate a collapsible Terminal drawer at the bottom of the main interface (using `xterm.js` or a streaming text viewer). This drawer should display stdout/stderr logs from running tools (like SPT Server) in real-time, complete with search and search filters.

### 1.4 Missing Conflict / File Collision Visualizer
*   **The Issue:** Multiple mods may try to modify or overwrite the same game files (e.g., specific asset bundles or BepInEx dlls).
*   **The Gap:** Toggling a mod card does not show which specific files are active, nor does it warn if another enabled mod overrides the same file.
*   **Recommendation:** Highlight file-level collisions with warning icons on affected cards, allowing users to hover and see which files overlap, and select which mod takes priority (overriding target files).

---

## 2. Technical & Engineering Considerations (Tauri / Rust)

### 2.1 File System Manipulation: Symlinks vs. Hardlinks vs. File Copying
To enable or disable mods without duplicating gigabytes of texture and audio files, Modkeeper must resolve how it handles files on disk:
*   **Symlinks:** Standard practice. However, creating symlinks on Windows requires Administrator privileges unless the user has enabled Windows Developer Mode.
*   **Hardlinks:** Do not require admin rights, but only work for files (not directories) and must exist on the same physical drive volume as the target game root.
*   **Copying:** Highly compatible, but slow and consumes substantial disk space.
*   **Proposed Strategy:** Implement a fallback pipeline in the Rust utility layer:
    1.  Attempt to create a Symlink.
    2.  If permissions are denied, check if Developer Mode can be requested or fallback to Hardlinks (for files) and Directory junctions.
    3.  If junctions fail, fallback to a Copy operation and show a banner warning the user of potential disk space issues, suggesting they enable Windows Developer Mode.

### 2.2 Background Process Lifecycle Management
*   **The Challenge:** When launching `Aki.Server.exe` as a child process via Tauri's `std::process::Command`, the process can outlive the Modkeeper application if Modkeeper crashes or is closed. This causes port conflicts (port 6969) on subsequent launches.
*   **Proposed Strategy:**
    *   Maintain a global state handle in Rust for all spawned processes.
    *   Implement a cleanup hook on application exit (`tauri::RunEvent::ExitRequested`) that sends termination signals (`SIGTERM` / `taskkill`) to running child processes.
    *   Implement a background process watcher that polls process health and reports status (Running, Stopped, Crashed with Exit Code) to the frontend via Tauri events (`emit`).

---

## 3. Advanced AI-Driven Architecture Extensions

Integrating AI into the core mod management loop can turn Modkeeper from a simple file organizer into an intelligent assistant:

```
+-------------------------------------------------------------------------------+
|                               AI AGENT PIPELINE                               |
|                                                                               |
|   [Archive Drop] --> [Layout Parser] ---> [Category Tagging] --+              |
|                                                                |              |
|                                                                v              |
|   [LLM Conflict Resolver] <--- [Log Streaming] <--- [Dependency Solver]       |
+-------------------------------------------------------------------------------+
```

### 3.1 AI-Driven Mod Archive Layout Parser (Logical Structure)
Instead of relying on rigid, regex-based file path extraction, a local lightweight layout parser or decision tree model inspects archive contents. It identifies where critical mod markers are located (e.g. `package.json` for server mods, `.dll` files for BepInEx plugins) and maps the installation commands:
*   If `package.json` has `akiVersion` key $\rightarrow$ Target: `/user/mods/<mod_name>/`
*   If DLL exists inside a `plugins` or `patchers` folder $\rightarrow$ Target: `/BepInEx/plugins/` or `/BepInEx/patchers/`
*   If folder structure is flat $\rightarrow$ Search for nested asset names, reference a database of common structures, and place files correctly.

### 3.2 Machine Learning Dependency & Load Order Resolver
*   **The Solution:** Build a topological sorting engine that parses mod manifests (`package.json` load order constraints) and uses a local heuristic model.
*   **How it Works:** The AI cross-references the user's current modlist against a community conflict matrix database. It identifies known incompatibilities, calculates the optimal loading indices to minimize load-time overwrite errors, and offers a single-click "Optimize Load Order" button.

### 3.3 Semantic Translation (i18n) Assistant
*   To resolve the issue noted in `purpose-of-redesign.md` (AI generating duplicated translation keys and utils), integrate a pre-commit or dev-time tool.
*   The tool uses embeddings to compare new user-facing strings against existing translated catalog keys (`locales/*.json`). If a key with a semantic similarity of $>90\%$ is found (e.g. "Select All" vs "Check All"), it suggests reusing the existing key instead of generating a new one.

---

## 4. Architectural Action Items for Next Sprint

1.  **Frontend Layout Refactoring:**
    *   Replace standard grid cards with a flexible layout supporting both Grid (Title Only) and List (Detailed metadata/sorting) modes.
    *   Create a collapsable Console terminal drawer.
2.  **Backend Rust Restructuring:**
    *   Standardize filesystem operations inside `src-tauri/src/utils/fs.rs`.
    *   Build a robust `ProcessManager` in Rust to track child processes and route their stdout/stderr streams to Tauri event streams.
3.  **Tauri Commands & IPC Setup:**
    *   Verify all newly written commands follow the `verb_noun` pattern and return structured `Result<T, AppError>` models.
