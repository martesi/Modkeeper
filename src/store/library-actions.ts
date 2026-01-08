import { atom } from 'jotai'
import { commands } from '@/lib/api'
import { unwrapResult } from '@/lib/result'
import {
  libraryAtom,
  librarySwitchAtom,
  libraryLoadingAtom,
  libraryErrorAtom,
} from './library'
import type { LibraryDTO, LibrarySwitch } from '@gen/bindings'

// Action atoms
export const fetchLibraryAction = atom(null, async (get, set) => {
  set(libraryLoadingAtom, true)
  set(libraryErrorAtom, null)
  try {
    const library = await unwrapResult(commands.getLibrary())
    set(libraryAtom, library)
    return library
  } catch (err) {
    const error = err instanceof Error ? err : new Error('Failed to load library')
    set(libraryErrorAtom, error)
    set(libraryAtom, null)
    throw error
  } finally {
    set(libraryLoadingAtom, false)
  }
})

export const openLibraryAction = atom(
  null,
  async (get, set, path: string): Promise<LibrarySwitch> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const switchData = await unwrapResult(commands.openLibrary(path))
      set(librarySwitchAtom, switchData)
      set(libraryAtom, switchData.active ?? null)
      return switchData
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to open library')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
)

export const createLibraryAction = atom(
  null,
  async (
    get,
    set,
    requirement: { gameRoot: string; repoRoot: string; name: string }
  ): Promise<LibrarySwitch> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const switchData = await unwrapResult(commands.createLibrary(requirement))
      set(librarySwitchAtom, switchData)
      set(libraryAtom, switchData.active ?? null)
      return switchData
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to create library')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
)

export const addModsAction = atom(
  null,
  async (get, set, paths: string[]): Promise<LibraryDTO> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const library = await unwrapResult(commands.addMods(paths, null))
      set(libraryAtom, library)
      // Update library switch if it exists
      const switchData = get(librarySwitchAtom)
      if (switchData?.active) {
        set(librarySwitchAtom, {
          ...switchData,
          active: library,
        })
      }
      return library
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to add mods')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
)

export const removeModsAction = atom(
  null,
  async (get, set, ids: string[]): Promise<LibraryDTO> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const library = await unwrapResult(commands.removeMods(ids, null))
      set(libraryAtom, library)
      // Update library switch if it exists
      const switchData = get(librarySwitchAtom)
      if (switchData?.active) {
        set(librarySwitchAtom, {
          ...switchData,
          active: library,
        })
      }
      return library
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to remove mods')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
)

export const toggleModAction = atom(
  null,
  async (get, set, id: string, isActive: boolean): Promise<LibraryDTO> => {
    set(libraryLoadingAtom, true)
    set(libraryErrorAtom, null)
    try {
      const library = await unwrapResult(commands.toggleMod(id, isActive))
      set(libraryAtom, library)
      // Update library switch if it exists
      const switchData = get(librarySwitchAtom)
      if (switchData?.active) {
        set(librarySwitchAtom, {
          ...switchData,
          active: library,
        })
      }
      return library
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to toggle mod')
      set(libraryErrorAtom, error)
      throw error
    } finally {
      set(libraryLoadingAtom, false)
    }
  }
)

export const syncModsAction = atom(null, async (get, set): Promise<LibraryDTO> => {
  set(libraryLoadingAtom, true)
  set(libraryErrorAtom, null)
  try {
    const library = await unwrapResult(commands.syncMods(null))
    set(libraryAtom, library)
    // Update library switch if it exists
    const switchData = get(librarySwitchAtom)
    if (switchData?.active) {
      set(librarySwitchAtom, {
        ...switchData,
        active: library,
      })
    }
    return library
  } catch (err) {
    const error = err instanceof Error ? err : new Error('Failed to sync mods')
    set(libraryErrorAtom, error)
    throw error
  } finally {
    set(libraryLoadingAtom, false)
  }
})
