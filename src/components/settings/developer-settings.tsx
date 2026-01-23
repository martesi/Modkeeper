import { useState } from 'react'
import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { Copy, Check, Library } from 'lucide-react'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
import { useLibrarySwitch } from '@/hooks/use-library-switch'
import { addLibraryFromDialog } from '@/lib/library-actions'
import { toast } from 'sonner'

export function DeveloperSettings() {
  const [gameRoot, setGameRoot] = useState<string | null>(null)
  const [copiedPath, setCopiedPath] = useState<string | null>(null)
  const { create } = useLibrarySwitch()

  const handleCreateSimulationGameRoot = () => {
    import('@tauri-apps/plugin-dialog')
      .then(({ open }) =>
        open({
          directory: true,
          multiple: false,
          title: 'Select Base Path (optional)',
        })
      )
      .then((selected) => {
        const basePath =
          selected && typeof selected === 'string' ? selected : null
        return ur(commands.createSimulationGameRoot(basePath))
      })
      .then((result) => {
        setGameRoot(result)
        toast.success('Game root created successfully')
      })
      .catch((err) => {
        const errorMessage =
          err instanceof Error ? err.message : 'Failed to create game root'
        toast.error(errorMessage)
        setGameRoot(null)
      })
  }

  const handleCopyPath = (path: string) => {
    navigator.clipboard
      .writeText(path)
      .then(() => {
        setCopiedPath(path)
        setTimeout(() => setCopiedPath(null), 2000)
      })
      .catch((err) => {
        console.error('Failed to copy path:', err)
        toast.error('Failed to copy path')
      })
  }

  const handleCreateLibrary = () => {
    if (!gameRoot) return

    addLibraryFromDialog(create, gameRoot)
      .then(() => {
        toast.success('Library created successfully')
      })
      .catch((err) => {
        const errorMessage =
          err instanceof Error ? err.message : 'Failed to create library'
        toast.error(errorMessage)
      })
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-lg font-semibold mb-2">
          <Trans>Test Game Root</Trans>
        </h2>
        <p className="text-sm text-muted-foreground mb-4">
          <Trans>
            Create a simulation game root structure for testing purposes
          </Trans>
        </p>
      </div>

      <Button onClick={handleCreateSimulationGameRoot} variant="default">
        <Trans>Create Simulation Game Root</Trans>
      </Button>

      {gameRoot && (
        <div className="space-y-3 mt-4 p-4 border rounded-lg bg-muted/50">
          <div>
            <div className="flex items-center justify-between mb-1">
              <p className="text-sm font-semibold">
                <Trans>Game Root</Trans>
              </p>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => handleCopyPath(gameRoot)}
                className="h-7 w-7"
              >
                {copiedPath === gameRoot ? (
                  <Check className="size-3 text-green-600" />
                ) : (
                  <Copy className="size-3" />
                )}
              </Button>
            </div>
            <p className="text-sm text-muted-foreground font-mono break-all">
              {gameRoot}
            </p>
          </div>

          <Button
            onClick={handleCreateLibrary}
            variant="default"
            className="w-full"
          >
            <Library className="size-4 mr-2" />
            <Trans>Create Library from Game Root</Trans>
          </Button>
        </div>
      )}
    </div>
  )
}
