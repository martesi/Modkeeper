import { useState } from 'react'
import { Trans } from '@lingui/react/macro'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { changeLocale } from '@/utils/i18n'
import { loadSettings, saveSettings } from '@/lib/settings-storage'

const AVAILABLE_LOCALES = [
  { value: 'en-US', label: 'English (US)' },
  // Add more locales here as they become available
]

export function LanguageSettings() {
  const [language, setLanguage] = useState(() => loadSettings().language)

  const handleLanguageChange = (value: string) => {
    setLanguage(value)
    changeLocale(value)
      .then(() => {
        const settings = loadSettings()
        saveSettings({ ...settings, language: value })
      })
      .catch((err) => console.error('Failed to change language:', err))
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-4">
          <Trans>Language</Trans>
        </h2>

        <div className="space-y-2">
          <Label htmlFor="language-select">
            <Trans>Interface Language</Trans>
          </Label>
          <Select value={language} onValueChange={handleLanguageChange}>
            <SelectTrigger id="language-select" className="w-[180px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {AVAILABLE_LOCALES.map((locale) => (
                <SelectItem key={locale.value} value={locale.value}>
                  {locale.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <p className="text-sm text-muted-foreground">
            <Trans>Choose your preferred language for the interface.</Trans>
          </p>
        </div>
      </div>
    </div>
  )
}
