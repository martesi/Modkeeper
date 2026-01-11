'use client'

import { useMemo } from 'react'
import { Trans } from '@lingui/react/macro'
import { Badge } from '@comps/badge'
import type { Dependencies } from '@gen/bindings'

interface DependenciesTabProps {
  dependencies: Dependencies
}

export function DependenciesTab({ dependencies }: DependenciesTabProps) {
  const depList = useMemo(() => {
    if ('Object' in dependencies) {
      return Object.entries(dependencies.Object).map(([id, version]) => ({
        id,
        version,
        optional: false,
      }))
    }
    return dependencies.Array
  }, [dependencies])

  return (
    <div className="space-y-2">
      <h3 className="text-lg font-semibold mb-4">
        <Trans>Dependencies</Trans>
      </h3>
      {depList.length === 0 ? (
        <p className="text-muted-foreground">
          <Trans>No dependencies</Trans>
        </p>
      ) : (
        <div className="space-y-2">
          {depList.map((dep, idx) => (
            <div
              key={idx}
              className="flex items-center justify-between p-3 border rounded-lg"
            >
              <div>
                <p className="font-medium">{dep.id}</p>
                <p className="text-sm text-muted-foreground">v{dep.version}</p>
              </div>
              {dep.optional && (
                <Badge variant="secondary">
                  <Trans>Optional</Trans>
                </Badge>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
