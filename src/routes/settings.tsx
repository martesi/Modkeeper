import { createFileRoute } from '@tanstack/react-router'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { useState } from 'react'
import { commands } from '@gen/bindings'
import { unwrapResult } from '@/lib/result'
import type { TestGameRoot } from '@gen/bindings'
import { msg, t } from '@lingui/core/macro'
import { Loader2 } from 'lucide-react'

export const Route = createFileRoute('/settings')({
  component: RouteComponent,
  staticData: {
    breadcrumb: () => t(msg`Settings`),
  },
})

function RouteComponent() {
  const [gameRoot, setGameRoot] = useState<TestGameRoot | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleCreateSimulationGameRoot = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await unwrapResult(
        commands.createSimulationGameRoot({
          spt_version: undefined,
          base_path: null,
        }),
      )
      setGameRoot(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      setGameRoot(null)
    } finally {
      setLoading(false)
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
                <p className="text-sm font-semibold mb-1">
                  <Trans>Game Root</Trans>
                </p>
                <p className="text-sm text-muted-foreground font-mono break-all">
                  {gameRoot.game_root}
                </p>
              </div>

              {gameRoot.temp_dir_path && (
                <div>
                  <p className="text-sm font-semibold mb-1">
                    <Trans>Temp Directory Path</Trans>
                  </p>
                  <p className="text-sm text-muted-foreground font-mono break-all">
                    {gameRoot.temp_dir_path}
                  </p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
