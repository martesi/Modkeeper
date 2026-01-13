import { atom } from 'jotai'
import type { Getter, Setter } from 'jotai'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
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
    async () => {
      return await ur(commands.getLibrary())
    },
    (_get, set, library) => set(libraryAtom, library),
  ),
)

export const openLibraryAction = atom(
  null,
  withAsyncState(
    async (path: string) => {
      return await ur(commands.openLibrary(path))
    },
    (_get, set, switchData) => updateLibrarySwitchState(_get, set, switchData),
  ),
)

export const createLibraryAction = atom(
  null,
  withAsyncState(async (requirement: LibraryCreationRequirement) => {
    // Backend automatically derives repoRoot from gameRoot as gameRoot/.mod_keeper
    // If .mod_keeper exists and is valid, it opens it. If invalid, it returns InvalidLibrary error.
    // The requirement.repoRoot will be overridden by the backend.
    return await ur(commands.createLibrary(requirement))
  }, updateLibrarySwitchState),
)

export const initAction = atom(
  null,
  withAsyncState(
    async () => {
      return await ur(commands.init())
    },
    (_get, set, switchData) => updateLibrarySwitchState(_get, set, switchData),
  ),
)

export const addModsAction = atom(
  null,
  withAsyncState(async (paths: string[], unknownModName: string) => {
    return await ur(commands.addMods(paths, unknownModName))
  }, updateLibraryState),
)

export const removeModsAction = atom(
  null,
  withAsyncState(async (ids: string[]) => {
    return await ur(commands.removeMods(ids))
  }, updateLibraryState),
)

export const toggleModAction = atom(
  null,
  withAsyncState(async (id: string, isActive: boolean) => {
    return await ur(commands.toggleMod(id, isActive))
  }, updateLibraryState),
)

export const syncModsAction = atom(
  null,
  withAsyncState(async () => {
    return await ur(commands.syncMods())
  }, updateLibraryState),
)

export const renameLibraryAction = atom(
  null,
  withAsyncState(async (name: string) => {
    return await ur(commands.renameLibrary(name))
  }, updateLibrarySwitchState),
)

export const closeLibraryAction = atom(
  null,
  withAsyncState(async (repoRoot: string) => {
    return await ur(commands.closeLibrary(repoRoot))
  }, updateLibrarySwitchState),
)

export const removeLibraryAction = atom(
  null,
  withAsyncState(async (repoRoot: string) => {
    return await ur(commands.removeLibrary(repoRoot))
  }, updateLibrarySwitchState),
)
