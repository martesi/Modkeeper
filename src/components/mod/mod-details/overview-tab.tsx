'use client'

import { Trans } from '@lingui/react/macro'
import { Badge } from '@comps/badge'
import type { Mod } from '@gen/bindings'

interface OverviewTabProps {
  mod: Mod
}

export function OverviewTab({ mod }: OverviewTabProps) {
  const manifest = mod.manifest

  return (
    <div className="space-y-4">
      {manifest?.description && (
        <div>
          <h3 className="text-lg font-semibold mb-2">
            <Trans>Description</Trans>
          </h3>
          <p className="text-muted-foreground">{manifest.description}</p>
        </div>
      )}

      <div className="grid grid-cols-2 gap-4">
        <div>
          <h4 className="text-sm font-semibold mb-1">
            <Trans>Mod ID</Trans>
          </h4>
          <p className="text-sm text-muted-foreground font-mono">{mod.id}</p>
        </div>
        {manifest?.sptVersion && (
          <div>
            <h4 className="text-sm font-semibold mb-1">
              <Trans>SPT Version</Trans>
            </h4>
            <p className="text-sm text-muted-foreground">{manifest.sptVersion}</p>
          </div>
        )}
      </div>

      {manifest?.effects && manifest.effects.length > 0 && (
        <div>
          <h3 className="text-lg font-semibold mb-2">
            <Trans>Effects</Trans>
          </h3>
          <div className="flex gap-2 flex-wrap">
            {manifest.effects.map((effect, idx) => (
              <Badge key={idx} variant="outline">
                {effect}
              </Badge>
            ))}
          </div>
        </div>
      )}

      {manifest?.compatibility && (
        <div>
          <h3 className="text-lg font-semibold mb-2">
            <Trans>Compatibility</Trans>
          </h3>
          {manifest.compatibility.include && (
            <div className="mb-2">
              <h4 className="text-sm font-semibold mb-1">
                <Trans>Includes</Trans>
              </h4>
              <div className="flex gap-2 flex-wrap">
                {manifest.compatibility.include.map((item, idx) => (
                  <Badge key={idx} variant="secondary">
                    {item}
                  </Badge>
                ))}
              </div>
            </div>
          )}
          {manifest.compatibility.exclude && (
            <div>
              <h4 className="text-sm font-semibold mb-1">
                <Trans>Excludes</Trans>
              </h4>
              <div className="flex gap-2 flex-wrap">
                {manifest.compatibility.exclude.map((item, idx) => (
                  <Badge key={idx} variant="destructive">
                    {item}
                  </Badge>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
