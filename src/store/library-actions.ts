import { atom } from 'jotai'
import type { Getter, Setter } from 'jotai'
// import { commands } from '@/lib/api'
// import { unwrapResult } from '@/lib/result'
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
      // Simulate network delay
      await new Promise((resolve) => setTimeout(resolve, 500))
      console.log('Opening library from path:', path)
      const switchData = mockDataStore.getLibrarySwitch()
      if (!switchData) throw new Error('No library switch data')
      return switchData
    },
    (_get, set, switchData) => updateLibrarySwitchState(_get, set, switchData),
  ),
)

export const createLibraryAction = atom(
  null,
  withAsyncState(async (requirement: LibraryCreationRequirement) => {
    // Simulate network delay
    await new Promise((resolve) => setTimeout(resolve, 800))
    const newLibrary = generateMockLibrary({
      name: requirement.name,
      modCount: 0, // New library starts with no mods
      isDirty: false,
    })
    return mockDataStore.addLibrary(newLibrary)
  }, updateLibrarySwitchState),
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
