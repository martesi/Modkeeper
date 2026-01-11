'use client'

import { Trans } from '@lingui/react/macro'
import { MarkdownContent } from '@/components/mod/markdown-content'

interface DocumentationTabProps {
  documentation: string | null
  loading: boolean
}

export function DocumentationTab({
  documentation,
  loading,
}: DocumentationTabProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Trans>Loading documentation...</Trans>
      </div>
    )
  }

  if (!documentation) {
    return (
      <div className="flex items-center justify-center h-64 text-muted-foreground">
        <Trans>No documentation available</Trans>
      </div>
    )
  }

  return (
    <div className="border rounded-lg p-6">
      <MarkdownContent content={documentation} />
    </div>
  )
}
