/*
 * Transition-only legacy-UI toggle persistence (consolidated-spec.md §10a).
 *
 * Deliberately NOT part of AppSettings: it is frontend-local and removed with the legacy tree, so
 * localStorage is its permanent home, not a MOCK-FALLBACK. Lives in its own module (not
 * settings-repository) so the state layer can import it without creating a state ↔ repository
 * import cycle.
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
