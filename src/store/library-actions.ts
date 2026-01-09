import { atom } from 'jotai'
import type { Getter, Setter } from 'jotai'
import { commands } from '@gen/bindings'
import { unwrapResult } from '@/lib/result'
import { mockDataStore, generateMockLibrary } from '@/lib/mock-data'
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

// Action atoms using FP patterns (using mock data)
export const fetchLibraryAction = atom(
  null,
  withAsyncState(
    async () => {
      // Simulate network delay
      await new Promise((resolve) => setTimeout(resolve, 300))
      const library = mockDataStore.getActiveLibrary()
      if (!library) throw new Error('No active library')
      return library
    },
    (_get, set, library) => set(libraryAtom, library),
  ),
)

export const openLibraryAction = atom(
  null,
  withAsyncState(
    async (path: string) => {
      return await unwrapResult(commands.openLibrary(path))
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
    return await unwrapResult(commands.createLibrary(requirement))
  }, updateLibrarySwitchState),
)

export const initAction = atom(
  null,
  withAsyncState(
    async () => {
      return await unwrapResult(commands.init())
    },
    (_get, set, switchData) => updateLibrarySwitchState(_get, set, switchData),
  ),
)

export const addModsAction = atom(
  null,
  withAsyncState(async (paths: string[]) => {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 1000))
    console.log('Adding mods from paths:', paths)

    const updated = mockDataStore.updateActiveLibrary((library) => {
      // Generate new mods for each path
      const newMods = paths.map(() => {
        const mod = generateMockLibrary({ modCount: 1 }).mods
        return Object.values(mod)[0]
      })

      const updatedMods = { ...library.mods }
      newMods.forEach((mod) => {
        if (mod) {
          updatedMods[mod.id] = mod
        }
      })

      return {
        ...library,
        mods: updatedMods,
        is_dirty: true,
      }
    })

    if (!updated) throw new Error('Failed to update library')
    return updated
  }, updateLibraryState),
)

export const removeModsAction = atom(
  null,
  withAsyncState(async (ids: string[]) => {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 500))
    console.log('Removing mods with ids:', ids)

    const updated = mockDataStore.updateActiveLibrary((library) => {
      const updatedMods = { ...library.mods }
      ids.forEach((id) => {
        delete updatedMods[id]
      })

      return {
        ...library,
        mods: updatedMods,
        is_dirty: true,
      }
    })

    if (!updated) throw new Error('Failed to update library')
    return updated
  }, updateLibraryState),
)

export const toggleModAction = atom(
  null,
  withAsyncState(async (id: string, isActive: boolean) => {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 300))
    console.log('Toggling mod:', id, 'to', isActive)

    const updated = mockDataStore.updateActiveLibrary((library) => {
      const mod = library.mods[id]
      if (!mod) return library

      return {
        ...library,
        mods: {
          ...library.mods,
          [id]: {
            ...mod,
            is_active: isActive,
          },
        },
        is_dirty: true,
      }
    })

    if (!updated) throw new Error('Failed to update library')
    return updated
  }, updateLibraryState),
)

export const syncModsAction = atom(
  null,
  withAsyncState(async () => {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 1500))
    console.log('Syncing mods...')

    const updated = mockDataStore.updateActiveLibrary((library) => {
      return {
        ...library,
        is_dirty: false, // Clear dirty flag after sync
      }
    })

    if (!updated) throw new Error('Failed to update library')
    return updated
  }, updateLibraryState),
)
