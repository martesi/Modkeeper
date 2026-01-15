'use client'

import { Trans } from '@lingui/react/macro'
import { useMemo } from 'react'
import type { LibraryDTO, Mod } from '@gen/bindings'
import { ModCard } from './mod-card'
import { useAtomValue } from 'jotai'
import { ALibraryActive } from '@/store/library'

export function ModList () {
  const library = useAtomValue(ALibraryActive)


  const mods = useMemo(() => {
    if (!library?.mods) return []
    return Object.values(library.mods) as Mod[]
  }, [library])


  if (!library) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No library loaded</Trans>
      </div>
    )
  }

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
    <div className="space-y-4">
      {/* Mod Grid */}
      {mods.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
          <p className="text-lg mb-2">
            <Trans>No mods installed</Trans>
          </p>
          <p className="text-sm">
            <Trans>Add mods to get started</Trans>
          </p>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {mods.map((mod) => (
            <ModCard
              key={mod.id}
              mod={mod}
            />
          ))}
        </div>
      )}
    </div>
  )
}
