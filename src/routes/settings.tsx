import { createFileRoute } from '@tanstack/react-router'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { useState } from 'react'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
import { msg, t } from '@lingui/core/macro'
import { Loader2, Copy, Check } from 'lucide-react'

export const Route = createFileRoute('/settings')({
  component: RouteComponent,
  staticData: {
    breadcrumb: () => t(msg`Settings`),
  },
})

function RouteComponent() {
  const [gameRoot, setGameRoot] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [copiedPath, setCopiedPath] = useState<string | null>(null)

  const handleCreateSimulationGameRoot = async () => {
    setLoading(true)
    setError(null)
    try {
      // Prompt user for base path (optional)
      let basePath: string | null = null
      try {
        const { open } = await import('@tauri-apps/plugin-dialog')
        const selected = await open({
          directory: true,
          multiple: false,
          title: 'Select Base Path (optional)',
        })
        if (selected && typeof selected === 'string') {
          basePath = selected
        }
      } catch (err) {
        // User cancelled or error - continue with temp directory
        console.log('No base path selected, using temp directory')
      }

      const result = await ur(commands.createSimulationGameRoot(basePath))
      setGameRoot(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      setGameRoot(null)
    } finally {
      setLoading(false)
    }
  }

  const handleCopyPath = async (path: string) => {
    try {
      await navigator.clipboard.writeText(path)
      setCopiedPath(path)
      setTimeout(() => setCopiedPath(null), 2000)
    } catch (err) {
      console.error('Failed to copy path:', err)
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">
          <Trans>Settings</Trans>
        </h1>
        <p className="text-sm text-muted-foreground mt-1">
          <Trans>Application settings and test utilities</Trans>
        </p>
      </div>

      <div className="space-y-4">
        <div className="border rounded-lg p-6 space-y-4">
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

          <Button
            onClick={handleCreateSimulationGameRoot}
            disabled={loading}
            variant="default"
          >
            {loading ? (
              <>
                <Loader2 className="size-4 mr-2 animate-spin" />
                <Trans>Creating...</Trans>
              </>
            ) : (
              <Trans>Create Simulation Game Root</Trans>
            )}
          </Button>

          {error && (
            <div className="p-4 border border-destructive/50 rounded-lg bg-destructive/10">
              <p className="text-sm text-destructive font-medium">
                <Trans>Error</Trans>
              </p>
              <p className="text-sm text-destructive mt-1">{error}</p>
            </div>
          )}

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
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
