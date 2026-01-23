'use client'

import { useEffect } from 'react'
import { useTheme } from 'next-themes'
import { loadSettings } from '@/lib/settings-storage'
import { changeLocale } from '@/utils/i18n'

/**
 * Component that initializes settings on app start
 * Applies saved theme, primary color, and language
 */
export function SettingsInit() {
  const { setTheme } = useTheme()

  useEffect(() => {
    const settings = loadSettings()

    // Apply theme
    if (settings.theme) {
      setTheme(settings.theme)
    }

    // Apply primary color
    if (settings.primaryColor && settings.primaryColor !== '#default') {
      document.documentElement.style.setProperty('--primary', settings.primaryColor)
    }

    // Apply language
    changeLocale(settings.language).catch((err) => {
      console.error('Failed to change locale on init:', err)
    })
  }, [setTheme])

  return null
}
