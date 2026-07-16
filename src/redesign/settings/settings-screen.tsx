import { useAtomValue } from 'jotai'
import { Globe, Palette, SunMoon } from 'lucide-react'
import { PageTitle } from '../shell/page-title'
import { settingsAtom } from '../state/settings-state'
import { saveSettings, DEFAULT_ACCENT } from '../data/settings-repository'
import type { AppSettings } from '../data/redesign-types'
import { settingsText } from '../i18n/settings-text'
import { SettingRow } from './setting-row'
import { ThemeModeControl } from './theme-mode-control'
import { AccentSwatches } from './accent-swatches'
import { LanguageSelect } from './language-select'

const FALLBACK_SETTINGS: AppSettings = {
  theme: 'system',
  accentColor: DEFAULT_ACCENT,
  language: 'en-US',
}

/**
 * Settings screen (consolidated-spec.md §12.6): one centered column of rows, no tabs, no developer
 * section. Every change saves the FULL settings object (T1); the initializers' applier turns the
 * replaced atom into visible effect.
 */
export function SettingsScreen() {
  const settings = useAtomValue(settingsAtom) ?? FALLBACK_SETTINGS

  function save(patch: Partial<AppSettings>) {
    void saveSettings({ ...settings, ...patch })
  }

  return (
    <div className="mx-auto flex w-full max-w-2xl flex-col gap-3">
      <PageTitle
        title={settingsText.title()}
        subtitle={settingsText.subtitle()}
      />

      <SettingRow
        icon={SunMoon}
        label={settingsText.appearance()}
        description={settingsText.appearanceDescription()}
      >
        <ThemeModeControl
          value={settings.theme}
          onChange={(theme) => save({ theme })}
        />
      </SettingRow>

      <SettingRow
        icon={Palette}
        label={settingsText.accent()}
        description={settingsText.accentDescription()}
      >
        <AccentSwatches
          value={settings.accentColor}
          onChange={(accentColor) => save({ accentColor })}
        />
      </SettingRow>

      <SettingRow
        icon={Globe}
        label={settingsText.language()}
        description={settingsText.languageDescription()}
      >
        <LanguageSelect
          value={settings.language}
          onChange={(language) => save({ language })}
        />
      </SettingRow>
    </div>
  )
}
