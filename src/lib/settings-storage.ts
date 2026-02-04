import { z } from 'zod'

const SettingsSchema = z.object({
  theme: z.enum(['system', 'light', 'dark']),
  primaryColor: z.string(),
  language: z.string(),
})

export type Settings = z.infer<typeof SettingsSchema>

const SETTINGS_KEY = 'modkeeper-settings'

const DEFAULT_SETTINGS: Settings = {
  theme: 'system',
  primaryColor: '#default',
  language: 'en-US',
}

export function loadSettings(): Settings {
  if (typeof window === 'undefined') {
    return DEFAULT_SETTINGS
  }

  const stored = localStorage.getItem(SETTINGS_KEY)
  if (!stored) {
    return DEFAULT_SETTINGS
  }

  const parsed = (() => {
    try {
      return JSON.parse(stored) as unknown
    } catch {
      return undefined
    }
  })()
  if (parsed === undefined) {
    console.error('Failed to parse settings, using defaults')
    return DEFAULT_SETTINGS
  }
  const result = SettingsSchema.safeParse(parsed)
  if (!result.success) {
    console.warn('Invalid settings format, using defaults:', result.error)
    return DEFAULT_SETTINGS
  }
  return result.data
}

export function saveSettings(settings: Settings): void {
  if (typeof window === 'undefined') {
    return
  }

  SettingsSchema.parse(settings)
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

export async function exportSettings(): Promise<void> {
  const settings = loadSettings()
  const json = JSON.stringify(settings, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `modkeeper-settings-${new Date().toISOString().split('T')[0]}.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export async function importSettings(file: File): Promise<Settings> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = (e) => {
      const text = e.target?.result as string
      const parsed = JSON.parse(text)
      const result = SettingsSchema.safeParse(parsed)

      if (result.success) {
        saveSettings(result.data)
        resolve(result.data)
      } else {
        reject(new Error('Invalid settings file format'))
      }
    }
    reader.onerror = () => {
      reject(new Error('Failed to read file'))
    }
    reader.readAsText(file)
  })
}
