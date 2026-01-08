'use client'

import * as React from 'react'
import { Button } from '@comps/button'
import { Input } from '@comps/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@comps/dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@comps/tooltip'
import { Trans, msg } from '@lingui/react/macro'
import { HelpCircle } from 'lucide-react'
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
  const [name, setName] = React.useState('')
  const [gameRoot, setGameRoot] = React.useState('')
  const [error, setError] = React.useState<string | null>(null)

  // Auto-calculate library root from game root
  const libraryRoot = React.useMemo(() => {
    if (!gameRoot) return ''
    // Use path.join equivalent - for now just append /.mod_keeper
    // The backend will handle path normalization
    const separator = gameRoot.includes('\\') ? '\\' : '/'
    return `${gameRoot}${separator}.mod_keeper`
  }, [gameRoot])

  const handleSelectGameRoot = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Game Root Directory',
      })
      if (selected && typeof selected === 'string') {
        setGameRoot(selected)
      }
    } catch (err) {
      console.error('Failed to select game root:', err)
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!name.trim() || !gameRoot) {
      setError('Name and game root are required')
      return
    }

    try {
      await createLibrary({
        name: name.trim(),
        gameRoot,
        repoRoot: libraryRoot,
      })
      onSuccess?.()
      onOpenChange(false)
      // Reset form
      setName('')
      setGameRoot('')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create library')
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Create New Library</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Create a new mod library for managing SPT mods</Trans>
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="name" className="text-sm font-medium mb-1 block">
              <Trans>Library Name</Trans>
            </label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={msg`Enter library name`}
              disabled={loading}
            />
          </div>

          <div>
            <label htmlFor="gameRoot" className="text-sm font-medium mb-1 block">
              <Trans>Game Root Directory</Trans>
            </label>
            <div className="flex gap-2">
              <Input
                id="gameRoot"
                value={gameRoot}
                onChange={(e) => setGameRoot(e.target.value)}
                placeholder={msg`Select game root directory`}
                disabled={loading}
              />
              <Button
                type="button"
                variant="outline"
                onClick={handleSelectGameRoot}
                disabled={loading}
              >
                <Trans>Browse</Trans>
              </Button>
            </div>
          </div>

          <div>
            <label
              htmlFor="libraryRoot"
              className="text-sm font-medium mb-1 block flex items-center gap-2"
            >
              <Trans>Library Root Directory</Trans>
              <Tooltip>
                <TooltipTrigger asChild>
                  <HelpCircle className="size-4 text-muted-foreground cursor-help" />
                </TooltipTrigger>
                <TooltipContent>
                  <Trans>
                    The library root is automatically set to .mod_keeper in the game root
                    directory. This is because the current implementation does not support
                    different volumes for the game and library directories.
                  </Trans>
                </TooltipContent>
              </Tooltip>
            </label>
            <Input
              id="libraryRoot"
              value={libraryRoot}
              placeholder={msg`Auto-calculated from game root`}
              disabled
              readOnly
              className="bg-muted"
            />
          </div>

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
            <Button type="submit" disabled={loading}>
              <Trans>Create</Trans>
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
