'use client'

import * as React from 'react'
import { Button } from '@comps/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@comps/dialog'
import { Trans } from '@lingui/react/macro'
import { useLibrarySwitch } from '@/hooks/use-library-state'

interface OpenLibraryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess?: () => void
}

export function OpenLibraryDialog({
  open: isOpen,
  onOpenChange,
  onSuccess,
}: OpenLibraryDialogProps) {
  const { openLibrary, loading } = useLibrarySwitch()
  const [error, setError] = React.useState<string | null>(null)

  const handleSelectLibrary = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Library Directory',
      })
      if (selected && typeof selected === 'string') {
        setError(null)
        try {
          await openLibrary(selected)
          onSuccess?.()
          onOpenChange(false)
        } catch (err) {
          setError(err instanceof Error ? err.message : 'Failed to open library')
        }
      }
    } catch (err) {
      console.error('Failed to select library:', err)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Open Library</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Select the library directory to open</Trans>
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {error && <div className="text-destructive text-sm">{error}</div>}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={loading}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={handleSelectLibrary} disabled={loading}>
              <Trans>Select Library</Trans>
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  )
}
