import { atom } from 'jotai'
import type { LibraryDTO, LibrarySwitch } from '@gen/bindings'

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
