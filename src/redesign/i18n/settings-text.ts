/*
 * Settings-screen i18n namespace (frontend-redesign-spec.md §10): the §12.6 row list.
 */
import { t } from '@lingui/core/macro'

export const settingsText = {
  title: () => t({ id: 'settings.header.title', message: 'Settings' }),
  subtitle: () =>
    t({
      id: 'settings.header.subtitle',
      message: 'Application settings and preferences',
    }),
  appearance: () =>
    t({ id: 'settings.appearance.label', message: 'Appearance' }),
  appearanceDescription: () =>
    t({
      id: 'settings.appearance.description',
      message: 'System follows your OS preference',
    }),
  themeSystem: () => t({ id: 'settings.appearance.system', message: 'System' }),
  themeLight: () => t({ id: 'settings.appearance.light', message: 'Light' }),
  themeDark: () => t({ id: 'settings.appearance.dark', message: 'Dark' }),
  accent: () => t({ id: 'settings.accent.label', message: 'Accent color' }),
  accentDescription: () =>
    t({
      id: 'settings.accent.description',
      message: 'The primary color used across the app',
    }),
  accentSwatch: (name: string) =>
    t({ id: 'settings.accent.swatch', message: `Use ${name} accent` }),
  language: () => t({ id: 'settings.language.label', message: 'Language' }),
  languageDescription: () =>
    t({
      id: 'settings.language.description',
      message: 'Language for the whole interface',
    }),
}
