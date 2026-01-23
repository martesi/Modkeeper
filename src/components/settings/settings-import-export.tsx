import { useState, useRef } from 'react'
import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { Download, Upload } from 'lucide-react'
import { exportSettings, importSettings } from '@/lib/settings-storage'
import { useTheme } from 'next-themes'
import { changeLocale } from '@/utils/i18n'
import { toast } from 'sonner'

export function SettingsImportExport() {
  const [importing, setImporting] = useState(false)
  const { setTheme } = useTheme()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const handleExport = () => {
    exportSettings()
      .then(() => {
        toast.success('Settings exported successfully')
      })
      .catch((err) => {
        console.error('Failed to export settings:', err)
        toast.error('Failed to export settings')
      })
  }

  const handleImportClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    setImporting(true)
    importSettings(file)
      .then((settings) => {
        // Apply imported settings
        setTheme(settings.theme)
        return changeLocale(settings.language).then(() => settings)
      })
      .then((settings) => {
        // Apply primary color
        if (settings.primaryColor !== '#default') {
          document.documentElement.style.setProperty(
            '--primary',
            settings.primaryColor
          )
        } else {
          document.documentElement.style.removeProperty('--primary')
        }
        toast.success('Settings imported successfully')
      })
      .catch((err) => {
        console.error('Failed to import settings:', err)
        toast.error(
          err instanceof Error ? err.message : 'Failed to import settings'
        )
      })
      .finally(() => {
        setImporting(false)
        // Reset input
        if (fileInputRef.current) {
          fileInputRef.current.value = ''
        }
      })
  }

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-lg font-semibold mb-4">
          <Trans>Backup & Restore</Trans>
        </h2>
        <p className="text-sm text-muted-foreground mb-4">
          <Trans>
            Export your settings to a file or import previously exported
            settings.
          </Trans>
        </p>
      </div>

      <div className="flex gap-3">
        <Button
          onClick={handleExport}
          variant="outline"
          className="flex-1"
        >
          <Download className="size-4 mr-2" />
          <Trans>Export Settings</Trans>
        </Button>
        <Button
          onClick={handleImportClick}
          variant="outline"
          className="flex-1"
          disabled={importing}
        >
          <Upload className="size-4 mr-2" />
          {importing ? (
            <Trans>Importing...</Trans>
          ) : (
            <Trans>Import Settings</Trans>
          )}
        </Button>
        <input
          ref={fileInputRef}
          type="file"
          accept=".json"
          onChange={handleFileChange}
          className="hidden"
          aria-label="Import settings file"
        />
      </div>
    </div>
  )
}
