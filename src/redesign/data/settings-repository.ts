/*
 * Settings repository (consolidated-spec.md §10), 8.3 slice.
 *
 * This stage only carries the transition-only legacy-UI toggle (§10a). It is frontend-local by
 * design — the backend's AppSettings never owns it, and it is removed with the legacy tree in the
 * later cleanup pass — so localStorage is its permanent home, not a MOCK-FALLBACK. 8.4 grows this
 * file into loadSettings/saveSettings (get_settings/save_settings, full-object replace per T1) and
 * applyTheme/applyAccent/applyLanguage.
 */

const LEGACY_UI_KEY = 'modkeeper-use-legacy-ui'

export function loadUseLegacyUi(): boolean {
  if (typeof window === 'undefined') return false
  return localStorage.getItem(LEGACY_UI_KEY) === 'true'
}

export function saveUseLegacyUi(value: boolean): void {
  if (typeof window === 'undefined') return
  localStorage.setItem(LEGACY_UI_KEY, String(value))
}
