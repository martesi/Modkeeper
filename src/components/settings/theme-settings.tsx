import { useTheme } from 'next-themes'
import { useEffect, useState } from 'react'
import { Trans } from '@lingui/react/macro'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { loadSettings, saveSettings, type Settings } from '@/lib/settings-storage'
import { Palette } from 'lucide-react'

const PRESET_COLORS = [
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Green', value: '#10b981' },
  { name: 'Purple', value: '#8b5cf6' },
  { name: 'Red', value: '#ef4444' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Pink', value: '#ec4899' },
  { name: 'Cyan', value: '#06b6d4' },
  { name: 'Yellow', value: '#eab308' },
]

export function ThemeSettings() {
  const { theme, setTheme } = useTheme()
  const [primaryColor, setPrimaryColor] = useState<string>('#default')
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    const settings = loadSettings()
    setPrimaryColor(settings.primaryColor)

    // Sync theme with ThemeProvider
    if (settings.theme && theme !== settings.theme) {
      setTheme(settings.theme)
    }

    // Apply primary color if not default
    if (settings.primaryColor !== '#default') {
      document.documentElement.style.setProperty('--primary', settings.primaryColor)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const handleThemeChange = (value: 'system' | 'light' | 'dark') => {
    setTheme(value)
    const settings = loadSettings()
    saveSettings({ ...settings, theme: value })
  }

  const handlePrimaryColorChange = (color: string) => {
    setPrimaryColor(color)
    const settings = loadSettings()
    const newSettings: Settings = { ...settings, primaryColor: color }
    saveSettings(newSettings)

    // Apply primary color
    if (color !== '#default') {
      document.documentElement.style.setProperty('--primary', color)
    } else {
      // Reset to default CSS value
      document.documentElement.style.removeProperty('--primary')
    }
  }

  if (!mounted) {
    return null
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-4">
          <Trans>Theme</Trans>
        </h2>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="theme-select">
              <Trans>Appearance</Trans>
            </Label>
            <Select
              value={theme || 'system'}
              onValueChange={handleThemeChange}
            >
              <SelectTrigger id="theme-select" className="w-[180px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">
                  <Trans>System</Trans>
                </SelectItem>
                <SelectItem value="light">
                  <Trans>Light</Trans>
                </SelectItem>
                <SelectItem value="dark">
                  <Trans>Dark</Trans>
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              <Trans>
                Choose how the app looks. System follows your OS preference.
              </Trans>
            </p>
          </div>

          <div className="space-y-2">
            <Label>
              <Trans>Primary Color</Trans>
            </Label>
            <div className="flex flex-wrap gap-2">
              {PRESET_COLORS.map((preset) => (
                <button
                  key={preset.value}
                  type="button"
                  onClick={() => handlePrimaryColorChange(preset.value)}
                  className={`w-10 h-10 rounded-md border-2 transition-all ${
                    primaryColor === preset.value
                      ? 'border-foreground scale-110'
                      : 'border-border hover:border-foreground/50'
                  }`}
                  style={{ backgroundColor: preset.value }}
                  title={preset.name}
                  aria-label={`Select ${preset.name} primary color`}
                />
              ))}
              <button
                type="button"
                onClick={() => handlePrimaryColorChange('#default')}
                className={`w-10 h-10 rounded-md border-2 flex items-center justify-center transition-all ${
                  primaryColor === '#default'
                    ? 'border-foreground scale-110'
                    : 'border-border hover:border-foreground/50'
                }`}
                title="Default"
                aria-label="Reset to default primary color"
              >
                <Palette className="size-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
