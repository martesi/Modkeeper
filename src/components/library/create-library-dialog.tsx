'use client'

import * as React from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@comps/dialog'
import { Trans } from '@lingui/react/macro'
import { msg, t } from '@lingui/core/macro'
import { useLibrarySwitch } from '@/hooks/use-library-state'

interface CreateLibraryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess?: () => void
}

export function CreateLibraryDialog({
  open: isOpen,
  onOpenChange,
  onSuccess,
}: CreateLibraryDialogProps) {
  const { createLibrary, loading } = useLibrarySwitch()
  const [error, setError] = React.useState<string | null>(null)

  const handleSelectGameRoot = React.useCallback(async () => {
    try {
      setError(null)
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: t(msg`Select Game Root Directory`),
      })

      // Ignore if no path received
      if (!selected || typeof selected !== 'string') {
        onOpenChange(false)
        return
      }

      try {
        // Use translated "Unnamed Library" as the library name
        const libraryName = t(msg`Unnamed Library`)
        const separator = selected.includes('\\') ? '\\' : '/'
        const repoRoot = `${selected}${separator}.mod_keeper`

        // Backend automatically derives repoRoot from gameRoot as gameRoot/.mod_keeper
        // If .mod_keeper exists and is valid, it opens it. If invalid, it returns InvalidLibrary error.
        await createLibrary({
          name: libraryName,
          game_root: selected,
          repo_root: repoRoot, // Backend will override this, but we pass it for backward compatibility
        })
        onSuccess?.()
        onOpenChange(false)
      } catch (err) {
        // unwrapResult already translates errors, so we just need to extract the message
        if (err instanceof Error) {
          setError(err.message)
        } else {
          setError('Failed to create library')
        }
      }
    } catch (err) {
      // User cancelled or error opening dialog
      onOpenChange(false)
    }
  }, [createLibrary, onOpenChange, onSuccess])

  // Automatically open directory picker when dialog opens
  React.useEffect(() => {
    if (isOpen && !loading) {
      handleSelectGameRoot()
    }
  }, [isOpen, loading, handleSelectGameRoot])

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Create New Library</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Select the game root directory</Trans>
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
