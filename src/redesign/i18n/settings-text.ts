/*
 * Settings-screen i18n namespace (frontend-redesign-spec.md §10). The Composition stage (8.5) adds
 * the row copy (appearance, accent, language, legacy-UI switch).
 */
import { t } from '@lingui/core/macro'

export const settingsText = {
  title: () => t({ id: 'settings.header.title', message: 'Settings' }),
  subtitle: () =>
    t({
      id: 'settings.header.subtitle',
      message: 'Application settings and preferences',
    }),
}
