'use client'

import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { formatTimestamp } from '@/utils/mod'
import { ModBackup } from '@gen/bindings'
import { ButtonGroup } from '@/components/ui/button-group'
import { FolderSearch } from 'lucide-react'

interface BackupsTabProps {
  backups: ModBackup[]
  onRestore: (timestamp: string) => void
}

export function BackupsTab({ backups, onRestore }: BackupsTabProps) {
  if (backups.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No backups available</Trans>
      </div>
    )
  }

  console.log(backups)

  return (
    <div className="space-y-2">
      <h3 className="text-lg font-semibold mb-4">
        <Trans>Available Backups</Trans>
      </h3>
      <div className="space-y-2">
        {backups.map(({ timestamp, path }) => (
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

            <ButtonGroup>
              <Button
                variant="outline"
                size="sm"
                onClick={() => onRestore(timestamp)}
              >
                <Trans>Restore</Trans>
              </Button>
              <Button
                onClick={() =>
                  import('@tauri-apps/plugin-opener').then(
                    ({ revealItemInDir }) => revealItemInDir(path),
                  )
                }
                variant={'outline'}
                size="sm"
              >
                <FolderSearch />
              </Button>
            </ButtonGroup>
          </div>
        ))}
      </div>
    </div>
  )
}
