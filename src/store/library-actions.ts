import { atom } from 'jotai'
import type { Getter, Setter } from 'jotai'
import { commands } from '@/lib/api'
import { unwrapResult } from '@/lib/result'
import {
  libraryAtom,
  librarySwitchAtom,
  libraryLoadingAtom,
  libraryErrorAtom,
} from './library'
import type {
  LibraryDTO,
  LibrarySwitch,
  LibraryCreationRequirement,
} from '@gen/bindings'

/**
 * Higher-order function to wrap async operations with loading/error state
 * This eliminates boilerplate and makes the code more functional
 */
function withAsyncState<Args extends any[], ReturnType>(
  operation: (...args: Args) => Promise<ReturnType>,
  updateState: (get: Getter, set: Setter, result: ReturnType) => void,
) {
  return async (
    get: Getter,
    set: Setter,
    ...args: Args
  ): Promise<ReturnType> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const result = await operation(...args)
      updateState(get, set, result)
      return result
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Operation failed')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
}

/**
 * Updates library state and syncs with library switch
 */
function updateLibraryState(get: Getter, set: Setter, library: LibraryDTO) {
  set(libraryAtom, library)
  const switchData = get(librarySwitchAtom)
  if (switchData?.active) {
    set(librarySwitchAtom, {
      ...switchData,
      active: library,
    })
  }
}

/**
 * Updates library switch state
 */
function updateLibrarySwitchState(
  _get: Getter,
  set: Setter,
  switchData: LibrarySwitch,
) {
  set(librarySwitchAtom, switchData)
  set(libraryAtom, switchData.active ?? null)
}

// Action atoms using FP patterns
export const fetchLibraryAction = atom(
  null,
  withAsyncState(
    () => unwrapResult(commands.getLibrary()),
    (_get, set, library) => set(libraryAtom, library),
  ),
)

export const openLibraryAction = atom(
  null,
  withAsyncState(
    (path: string) => unwrapResult(commands.openLibrary(path)),
    (_get, set, switchData) => updateLibrarySwitchState(_get, set, switchData),
  ),
)

export const createLibraryAction = atom(
  null,
  withAsyncState(
    (requirement: LibraryCreationRequirement) =>
      unwrapResult(commands.createLibrary(requirement)),
    updateLibrarySwitchState,
  ),
)

export const addModsAction = atom(
  null,
  withAsyncState(
    (paths: string[]) => unwrapResult(commands.addMods(paths, null as any)),
    updateLibraryState,
  ),
)

export const removeModsAction = atom(
  null,
  withAsyncState(
    (ids: string[]) => unwrapResult(commands.removeMods(ids, null as any)),
    updateLibraryState,
  ),
)

export const toggleModAction = atom(
  null,
  withAsyncState(
    (id: string, isActive: boolean) =>
      unwrapResult(commands.toggleMod(id, isActive)),
    updateLibraryState,
  ),
)

export const syncModsAction = atom(
  null,
  withAsyncState(
    () => unwrapResult(commands.syncMods(null as any)),
    updateLibraryState,
  ),
)
