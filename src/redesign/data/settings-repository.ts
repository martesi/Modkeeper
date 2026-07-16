/*
 * Settings repository (consolidated-spec.md §10).
 *
 * Settings are backend-owned in App Config (T1): the initial value rides the workspace from
 * `get_library_workspace`, and every mutation goes through `saveSettings`, which returns the FULL
 * object — the atom is replaced wholesale, never patched. The appliers (`applyTheme` side of theme
 * lives with next-themes in the initializers; accent/language live here) turn the saved value into
 * visible effect.
 */
import { isTauri } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { i18n } from '@lingui/core'
import { commands } from '@gen/bindings'
import { changeLocale } from '@/utils/i18n'
import type { AppSettings } from './redesign-types'
import { resolveCommandError } from '../shared/hooks/use-command-error'
import { contrastOn } from '../shared/utils/contrast'
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
 * Accent is applied as shadcn-variable overrides on the root element (fix_plan_0.md §3):
 * `--primary` and `--ring` carry the hex, `--primary-foreground` is derived from its luminance
 * (§4) so light accents keep readable text. Hover/active shades need no vars — the shadcn `/90`
 * opacity idiom derives them. The default accent clears the overrides so the stylesheet wins.
 */
export function applyAccent(accentColor: string): void {
  const style = document.documentElement.style
  if (accentColor === DEFAULT_ACCENT) {
    for (const name of ['--primary', '--primary-foreground', '--ring']) {
      style.removeProperty(name)
    }
    return
  }
  style.setProperty('--primary', accentColor)
  style.setProperty('--primary-foreground', contrastOn(accentColor))
  style.setProperty('--ring', accentColor)
}

export async function applyLanguage(language: string): Promise<void> {
  if (i18n.locale === language) return
  try {
    await changeLocale(language)
  } catch (error) {
    // Configs written before the backend default was fixed persist bare "en"; there is no such
    // catalog (lingui locales are region-tagged, e.g. en-US), so fall back instead of warning
    // on every boot.
    if (language === 'en-US') throw error
    console.warn(`[redesign] no catalog for "${language}", falling back to en-US`, error)
    await changeLocale('en-US')
  }
}
