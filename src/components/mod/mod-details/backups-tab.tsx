'use client'

import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { formatTimestamp } from '@/utils/mod'

interface BackupsTabProps {
  backups: string[]
  loading: boolean
  onRestore: (timestamp: string) => void
}

export function BackupsTab({
  backups,
  loading,
  onRestore,
}: BackupsTabProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Trans>Loading backups...</Trans>
      </div>
    )
  }

  if (backups.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No backups available</Trans>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <h3 className="text-lg font-semibold mb-4">
        <Trans>Available Backups</Trans>
      </h3>
      <div className="space-y-2">
        {backups.map((timestamp) => (
          <div
            key={timestamp}
            className="flex items-center justify-between p-3 border rounded-lg"
          >
            <div>
              <p className="font-medium">{formatTimestamp(timestamp)}</p>
              <p className="text-sm text-muted-foreground font-mono">
                {timestamp}
              </p>
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onRestore(timestamp)}
            >
              <Trans>Restore</Trans>
            </Button>
          </div>
        ))}
      </div>
    </div>
  )
}
