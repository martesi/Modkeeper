'use client'

import { Trans } from '@lingui/react/macro'
import type { LibraryDTO } from '@gen/bindings'
import { ModCard } from './mod-card'

interface ModListProps {
  library: LibraryDTO | null
  onModToggle?: (id: string, isActive: boolean) => void
  onModRemove?: (id: string) => void
}

export function ModList({ library, onModToggle, onModRemove }: ModListProps) {
  if (!library) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No library loaded</Trans>
      </div>
    )
  }

  const mods = Object.values(library.mods)

  if (mods.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
        <p className="text-lg mb-2">
          <Trans>No mods installed</Trans>
        </p>
        <p className="text-sm">
          <Trans>Add mods to get started</Trans>
        </p>
      </div>
    )
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {mods.map((mod) => {
        if (!mod) return null
        return (
          <ModCard
            key={mod.id}
            mod={mod}
            onToggle={onModToggle}
            onRemove={onModRemove}
          />
        )
      })}
    </div>
  )
}
