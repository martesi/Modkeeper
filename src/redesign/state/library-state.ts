/*
 * Library state atoms (consolidated-spec.md §10 "State"), walking-skeleton slice.
 *
 * Only the atoms the slice exercises exist here; 8.4 fills in search/sort/filter/selection. The
 * shapes proven here are frozen by the 8.2 exit gate:
 *   - the workspace is a single-writer atom, written only by the repository layer;
 *   - "library busy" is DERIVED from an in-flight fire-and-track task set keyed by libraryId.
 *
 * The pending-task set is the one piece of mutable state a plain repository function needs to poke
 * from outside React, so it is written through `getDefaultStore()` here rather than exposing the
 * atom to the repository directly — store access stays in the state layer.
 */
import { atom, getDefaultStore } from 'jotai'
import type {
  LibraryWorkspace,
  LibrarySummary,
  ModSummary,
  LibraryId,
} from '../data/redesign-types'
import { isLibrarySummary } from '../data/redesign-types'

export const libraryWorkspaceAtom = atom<LibraryWorkspace | null>(null)

export const activeLibraryIdAtom = atom(
  (get) => get(libraryWorkspaceAtom)?.activeLibraryId ?? null
)

export const activeLibraryAtom = atom<LibrarySummary | null>((get) => {
  const workspace = get(libraryWorkspaceAtom)
  const activeId = workspace?.activeLibraryId
  if (!workspace || !activeId) return null
  const entry = workspace.libraries.find(
    (lib) => isLibrarySummary(lib) && lib.id === activeId
  )
  return entry && isLibrarySummary(entry) ? entry : null
})

export const modListAtom = atom<ModSummary[]>((get) => {
  const workspace = get(libraryWorkspaceAtom)
  const activeId = workspace?.activeLibraryId
  if (!workspace || !activeId) return []
  return workspace.modsByLibraryId[activeId] ?? []
})

/** An accepted-but-not-yet-completed fire-and-track operation. */
export type PendingTask = { taskId: string; libraryId: LibraryId }

export const pendingTasksAtom = atom<PendingTask[]>([])

/** True while any fire-and-track task is in flight for the active library (§10 library-busy). */
export const libraryBusyAtom = atom((get) => {
  const activeId = get(activeLibraryIdAtom)
  if (!activeId) return false
  return get(pendingTasksAtom).some((task) => task.libraryId === activeId)
})

// --- Single-writer helpers used by the repository layer (outside React) ---

const store = getDefaultStore()

export function setWorkspace(workspace: LibraryWorkspace): void {
  store.set(libraryWorkspaceAtom, workspace)
}

export function addPendingTask(task: PendingTask): void {
  store.set(pendingTasksAtom, [...store.get(pendingTasksAtom), task])
}

export function removePendingTask(taskId: string): void {
  store.set(
    pendingTasksAtom,
    store.get(pendingTasksAtom).filter((task) => task.taskId !== taskId)
  )
}
