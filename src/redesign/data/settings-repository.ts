/*
 * Settings repository (consolidated-spec.md §10).
 *
 * Settings are backend-owned in App Config (T1): the initial value rides the workspace from
 * `get_library_workspace`, and every mutation goes through `saveSettings`, which returns the FULL
 * object — the atom is replaced wholesale, never patched. The appliers (`applyTheme` side of theme
 * lives with next-themes in the initializers; accent/language live here) turn the saved value into
 * visible effect. The transition-only legacy-UI toggle lives in `legacy-ui-storage.ts` (§10a).
 */
import { isTauri } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { i18n } from '@lingui/core'
import { commands } from '@gen/bindings'
import { changeLocale } from '@/utils/i18n'
import type { AppSettings } from './redesign-types'
import { resolveCommandError } from '../shared/hooks/use-command-error'
import { getMockWorkspace, setMockWorkspace } from './example-data'
import { setSettings } from '../state/settings-state'

export const DEFAULT_ACCENT = '#e91e63'

export async function saveSettings(
  settings: AppSettings,
): Promise<AppSettings> {
  try {
    if (isTauri()) {
      const result = await commands.saveSettings(settings)
      if (result.status === 'ok') {
        setSettings(result.data)
        return result.data
      }
      toast.error(resolveCommandError(result.error))
      console.error('[redesign] command error', result.error)
      return settings
    }
  } catch (error) {
    // MOCK-FALLBACK: backend unavailable — persist to the fixture instead.
    console.warn('[redesign] saveSettings failed, using mock', error)
  }
  // MOCK-FALLBACK: no backend present — the fixture workspace carries settings like the real one.
  const workspace = structuredClone(getMockWorkspace())
  workspace.settings = settings
  setMockWorkspace(workspace)
  setSettings(settings)
  return settings
}

/**
 * Accent is applied as CSS-variable overrides on the root element; the hover/active/state-layer
 * shades are derived with color-mix so one hex drives the whole interactive palette. The default
 * accent clears the overrides so the stylesheet's hand-tuned values win.
 */
export function applyAccent(accentColor: string): void {
  const style = document.documentElement.style
  if (accentColor === DEFAULT_ACCENT) {
    for (const name of [
      '--mk-primary',
      '--mk-primary-hover',
      '--mk-primary-active',
      '--mk-state-hover',
      '--mk-state-active',
    ]) {
      style.removeProperty(name)
    }
    return
  }
  style.setProperty('--mk-primary', accentColor)
  style.setProperty(
    '--mk-primary-hover',
    `color-mix(in srgb, ${accentColor} 90%, black)`,
  )
  style.setProperty(
    '--mk-primary-active',
    `color-mix(in srgb, ${accentColor} 80%, black)`,
  )
  style.setProperty(
    '--mk-state-hover',
    `color-mix(in srgb, ${accentColor} 12%, transparent)`,
  )
  style.setProperty(
    '--mk-state-active',
    `color-mix(in srgb, ${accentColor} 20%, transparent)`,
  )
}

export async function applyLanguage(language: string): Promise<void> {
  if (i18n.locale === language) return
  await changeLocale(language)
}
