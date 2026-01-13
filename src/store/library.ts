import { atom } from 'jotai'
import type { LibraryDTO, LibrarySwitch } from '@gen/bindings'

// Base atoms
export const libraryAtom = atom<LibraryDTO | null>(null)
export const librarySwitchAtom = atom<LibrarySwitch | null>(null)
export const libraryLoadingAtom = atom<boolean>(false)
export const libraryErrorAtom = atom<Error | null>(null)

// Derived atoms
export const activeLibraryAtom = atom(
  (get) => get(librarySwitchAtom)?.active ?? null,
)

export const knownLibrariesAtom = atom(
  (get) => get(librarySwitchAtom)?.libraries ?? [],
)

export const ALibrarySwitch = atom<LibrarySwitch>()
export const ALibraryActive = atom(
  (g) => g(ALibrarySwitch)?.active || void 0,
  (g, s, value: LibraryDTO) => {
    s(ALibrarySwitch, {
      libraries: g(ALibrarySwitch)?.libraries ?? [],
      active: value,
    })
  },
)
export const ALibraryList = atom((g) => g(ALibrarySwitch)?.libraries || [])
