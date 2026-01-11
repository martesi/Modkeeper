'use client'

import { Trans } from '@lingui/react/macro'
import { ChevronRight } from 'lucide-react'
import type { ModManifest } from '@gen/bindings'
import { getLinkTypeName } from '@/utils/mod'

interface LinksTabProps {
  links: ModManifest['links']
}

export function LinksTab({ links }: LinksTabProps) {
  if (!links || links.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No links available</Trans>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <h3 className="text-lg font-semibold mb-4">
        <Trans>External Links</Trans>
      </h3>
      <div className="space-y-2">
        {links.map((link, idx) => (
          <a
            key={idx}
            href={link.url}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors"
          >
            <div>
              <p className="font-medium">
                {link.name || getLinkTypeName(link.link_type)}
              </p>
              <p className="text-sm text-muted-foreground truncate">
                {link.url}
              </p>
            </div>
            <ChevronRight className="size-4 text-muted-foreground" />
          </a>
        ))}
      </div>
    </div>
  )
}
