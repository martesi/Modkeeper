import { useAtom, useAtomValue, useSetAtom } from 'jotai'
import {
  libraryAtom,
  librarySwitchAtom,
  libraryLoadingAtom,
  libraryErrorAtom,
  activeLibraryAtom,
  knownLibrariesAtom,
} from '@/store/library'
import {
  fetchLibraryAction,
  openLibraryAction,
  createLibraryAction,
  addModsAction,
  removeModsAction,
  toggleModAction,
  syncModsAction,
} from '@/store/library-actions'

export function useLibrary() {
  const library = useAtomValue(libraryAtom)
  const loading = useAtomValue(libraryLoadingAtom)
  const error = useAtomValue(libraryErrorAtom)
  const fetchLibrary = useSetAtom(fetchLibraryAction)

  return {
    library,
    loading,
    error,
    refresh: fetchLibrary,
  }
}

export function useLibrarySwitch() {
  const librarySwitch = useAtomValue(librarySwitchAtom)
  const loading = useAtomValue(libraryLoadingAtom)
  const error = useAtomValue(libraryErrorAtom)
  const active = useAtomValue(activeLibraryAtom)
  const libraries = useAtomValue(knownLibrariesAtom)
  const openLibrary = useSetAtom(openLibraryAction)
  const createLibrary = useSetAtom(createLibraryAction)

  return {
    librarySwitch,
    active,
    libraries,
    loading,
    error,
    openLibrary,
    createLibrary,
    refresh: fetchLibraryAction,
  }
}

export function useMods() {
  const addMods = useSetAtom(addModsAction)
  const removeMods = useSetAtom(removeModsAction)
  const toggleMod = useSetAtom(toggleModAction)
  const syncMods = useSetAtom(syncModsAction)
  const loading = useAtomValue(libraryLoadingAtom)
  const error = useAtomValue(libraryErrorAtom)

  return {
    addMods,
    removeMods,
    toggleMod,
    syncMods,
    loading,
    error,
  }
}
