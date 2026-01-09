'use client'

import * as React from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@comps/dialog'
import { Trans, msg } from '@lingui/react/macro'
import { t } from '@lingui/core/macro'
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

  const handleSelectLibrary = React.useCallback(async () => {
    try {
      setError(null)
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: t(msg`Select Library Directory`),
      })

      // Ignore if no path received
      if (!selected || typeof selected !== 'string') {
        onOpenChange(false)
        return
      }

      try {
        await openLibrary(selected)
        onSuccess?.()
        onOpenChange(false)
      } catch (err) {
        if (err instanceof Error) {
          setError(err.message)
        } else {
          setError('Failed to open library')
        }
      }
    } catch (err) {
      // User cancelled or error opening dialog
      onOpenChange(false)
    }
  }, [openLibrary, onOpenChange, onSuccess])

  // Automatically open directory picker when dialog opens
  React.useEffect(() => {
    if (isOpen && !loading) {
      handleSelectLibrary()
    }
  }, [isOpen, loading, handleSelectLibrary])

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
          {loading && (
            <div className="text-sm text-muted-foreground">
              <Trans>Processing...</Trans>
            </div>
          )}
          {error && <div className="text-destructive text-sm">{error}</div>}
        </div>
      </DialogContent>
    </Dialog>
  )
}
