/*
 * Settings state atoms (consolidated-spec.md §10 "State"), Global-stage shapes.
 *
 * Shapes only at this stage: `settingsAtom` is written by the settings repository in 8.4
 * (full-object replace on every save, T1); the derived atoms give consumers stable read points with
 * the spec defaults until then. `useLegacyUiAtom` is live now — it drives the route adapters'
 * new/old tree choice (§10a) and persists through the settings repository.
 */
import { atom } from 'jotai'
import type { AppSettings } from '../data/redesign-types'
import { loadUseLegacyUi, saveUseLegacyUi } from '../data/settings-repository'

export const settingsAtom = atom<AppSettings | null>(null)

export const themeModeAtom = atom(
  (get) => get(settingsAtom)?.theme ?? 'system'
)

export const accentColorAtom = atom(
  (get) => get(settingsAtom)?.accentColor ?? '#e91e63'
)

export const languageAtom = atom(
  (get) => get(settingsAtom)?.language ?? 'en-US'
)

const useLegacyUiBaseAtom = atom(loadUseLegacyUi())

/** Transition-only new/old UI switch (§10a): writes persist immediately. */
export const useLegacyUiAtom = atom(
  (get) => get(useLegacyUiBaseAtom),
  (_get, set, value: boolean) => {
    set(useLegacyUiBaseAtom, value)
    saveUseLegacyUi(value)
  }
)
