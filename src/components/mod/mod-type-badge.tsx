'use client'

import { Trans } from '@lingui/react/macro'
import { Badge } from '@comps/badge'
import type { Mod } from '@gen/bindings'

interface ModTypeBadgeProps {
  type: Mod['mod_type']
}

export function ModTypeBadge({ type }: ModTypeBadgeProps) {
  if (type === 'Both') {
    return (
      <>
        <Badge
          variant="outline"
          className="bg-green-500/20 text-green-700 dark:text-green-400"
        >
          <Trans>Server</Trans>
        </Badge>
        <Badge
          variant="outline"
          className="bg-blue-500/20 text-blue-700 dark:text-blue-400"
        >
          <Trans>Client</Trans>
        </Badge>
      </>
    )
  }

  const colorClass =
    type === 'Client'
      ? 'bg-blue-500/20 text-blue-700 dark:text-blue-400'
      : type === 'Server'
        ? 'bg-green-500/20 text-green-700 dark:text-green-400'
        : 'bg-gray-500/20 text-gray-700 dark:text-gray-400'

  return (
    <Badge variant="outline" className={colorClass}>
      {type === 'Client' && <Trans>Client</Trans>}
      {type === 'Server' && <Trans>Server</Trans>}
      {type === 'Unknown' && <Trans>Unknown</Trans>}
    </Badge>
  )
}
