Here are the suggestions, grouped by concern:

---

**1. `iconData` vs `iconSrc` inconsistency**

`ModSummary` uses `iconData?: string` while `ToolSummary` uses `iconSrc?: string`. The names imply different semantics — `iconData` suggests embedded base64/binary, `iconSrc` suggests a file path or URL. The spec says icon browsing should accept any file renderable by `<img>`, so this needs to be settled consistently before implementation. Either normalise both to a stored file path (and let the frontend construct the `src`) or normalise both to a data URL. Whichever you choose, the field name should be the same on both types.

Decision: normalize as data URL when transfered to frontend.

---

**2. `isActive` on `LibrarySummary` is redundant with `activeLibraryId` on `LibraryWorkspace`**

You can always derive active status by comparing `library.id === workspace.activeLibraryId`. Keeping both creates a potential inconsistency if the backend ever returns them out of sync. I'd drop `isActive` from `LibrarySummary` and let the UI derive it.

Decision: remove redundent isActive

---

**3. `bulk_update_mods` contradicts your own destructive-operation principle**

Section 3 says *"Destructive operations must have explicit action names and flags"*, but `bulk_update_mods` bundles `'enable' | 'disable' | 'delete'` into one `action` field. A delete is structurally different from a state toggle — it should be a separate call:

```ts
bulk_set_mods_enabled(input: { modIds: ModId[], enabled: boolean }): Promise<LibraryWorkspace>
bulk_delete_mods(input: { modIds: ModId[] }): Promise<LibraryWorkspace>
```

This also makes `set_mod_enabled` redundant — a single-element `bulk_set_mods_enabled` covers it.

Decision: remove set_mod_enabled, keep bulk_update_mods as is and used as replacement of the previous.

---

**4. `rebuild_library_cache` is synchronous but the state machine implies async**

The return type is `Promise<LibraryWorkspace>`, but `CacheStatus.state` can be `'rebuilding'`, which implies a background process. If the call returns immediately with `state: 'rebuilding'`, the frontend has no defined way to know when it finishes. In Tauri you'd typically emit events. At minimum, the contract should note whether this is fire-and-return or blocking, and if it's non-blocking, sketch a `listen_cache_status` or `listen_workspace_event` mechanism. This is the most significant gap in the current contract.

---

**5. `create_library` is missing `libraryRoot`**

`LibrarySummary` has both `gameRoot` and `libraryRoot`, but `create_library` only accepts `gameRoot`. Either the backend derives `libraryRoot` from `gameRoot` (which should be documented), or the input is missing a field. If the user can choose a separate library folder, it needs to be in the input.

---

**6. `AppError` is referenced but never defined**

Section 5 says all calls return `Result<T, AppError>` but the type is never sketched. Even a minimal version would help the frontend handle errors uniformly and satisfy the toast-on-error requirement in the main spec:

```ts
export type AppError = {
  code: string        // machine-readable, e.g. 'library.not_found'
  message: string     // user-visible fallback
  context?: Record<string, unknown>
}
```

---

**7. `install_mod_archives` has no partial failure model**

If you pass three archive paths and two install successfully but one fails, the current return type (`LibraryWorkspace`) tells you nothing about which one failed. Consider either a wrapper or an extended result:

```ts
install_mod_archives(input: {
  libraryId: LibraryId
  archivePaths: string[]
}): Promise<{
  workspace: LibraryWorkspace
  failures: { archivePath: string; error: AppError }[]
}>
```

---

**8. The Open Decisions in Section 8 need resolution before mock repositories are written**

Specifically the `iconSrc`/`iconData` format choice (local path vs data URL) directly affects how the Configure Tool dialog writes and reads icon values. If mocks are written assuming a file path and the backend later returns data URLs, component props will need to change. Resolve this before implementation starts.

---

**9. Timestamp format should be specified**

Every type has `updatedAt: string` and `lastRebuiltAt?: string` without specifying the format. Just add a comment or a note that these are ISO 8601 UTC strings — it avoids ambiguity when the frontend formats them for display.

---

**10. Minor: `modCount` / `enabledModCount` on `LibrarySummary` should be noted as derived**

These must always equal `modsByLibraryId[id].length` and the filtered enabled count respectively. Worth a line in the contract making it explicit that the backend is responsible for keeping them in sync with the full list, so the UI doesn't have to re-derive them and create a second source of truth.
