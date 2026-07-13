/*
 * Settings state atoms (consolidated-spec.md §10 "State").
 *
 * `settingsAtom` is written only by the repository layer — full-object replace on every save (T1) —
 * through `setSettings`, mirroring the library workspace's single-writer shape. The derived atoms
 * give consumers stable read points with the spec defaults until the first load lands.
 * `useLegacyUiAtom` drives the route adapters' new/old tree choice (§10a) and persists through the
 * settings repository.
 */
import { atom, getDefaultStore } from 'jotai'
import type { AppSettings } from '../data/redesign-types'
import { loadUseLegacyUi, saveUseLegacyUi } from '../data/legacy-ui-storage'

export const settingsAtom = atom<AppSettings | null>(null)

/** Single-writer helper for the repository layer (outside React), like `setWorkspace`. */
export function setSettings(settings: AppSettings): void {
  getDefaultStore().set(settingsAtom, settings)
}

export const themeModeAtom = atom((get) => get(settingsAtom)?.theme ?? 'system')

export const accentColorAtom = atom(
  (get) => get(settingsAtom)?.accentColor ?? '#e91e63',
)

export const languageAtom = atom(
  (get) => get(settingsAtom)?.language ?? 'en-US',
)

const useLegacyUiBaseAtom = atom(loadUseLegacyUi())

/** Transition-only new/old UI switch (§10a): writes persist immediately. */
export const useLegacyUiAtom = atom(
  (get) => get(useLegacyUiBaseAtom),
  (_get, set, value: boolean) => {
    set(useLegacyUiBaseAtom, value)
    saveUseLegacyUi(value)
  },
)
