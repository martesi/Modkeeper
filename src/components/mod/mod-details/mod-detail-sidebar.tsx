import { Mod, ModBackup } from '@gen/bindings'
import {
  Link as LinkIcon,
  Heart,
  Code,
  CheckCircle2,
  History,
  Plus,
} from 'lucide-react'
import { Trans } from '@lingui/react/macro'
import { Button } from '@comps/button'
import { formatTimestamp } from '@/utils/mod'
import { useMemo } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@comps/card'
import { Empty, EmptyDescription, EmptyMedia } from '@comps/empty'

interface ModDetailSidebarProps {
  mod: Mod
  backups: ModBackup[]
  onCreateBackup: () => void
  onRestoreBackup: (timestamp: string) => void
}

export function ModDetailSidebar({
  mod,
  backups,
  onCreateBackup,
  onRestoreBackup,
}: ModDetailSidebarProps) {
  const links = mod.manifest?.links || []

  const dependencies = useMemo(() => {
    const { dependencies: deps } = mod.manifest || {}
    if (!deps) return []
    if (Array.isArray(deps)) return deps
    return Object.entries(deps).map(([id, version]) => ({ id, version }))
  }, [mod])

  return (
    <div className="space-y-6">
      {/* Details Card */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">
            <Trans>Mod Details</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <div className="flex justify-between py-1 border-b border-dashed border-border/50">
            <span className="text-muted-foreground">
              <Trans>Mod ID</Trans>
            </span>
            <span
              className="font-mono bg-muted px-1.5 rounded text-xs py-0.5 max-w-[150px] truncate"
              title={mod.id}
            >
              {mod.id}
            </span>
          </div>
          <div className="flex justify-between py-1 border-b border-dashed border-border/50">
            <span className="text-muted-foreground">
              <Trans>Version</Trans>
            </span>
            <span>{mod.manifest?.version || '-'}</span>
          </div>
          <div className="flex justify-between py-1 border-b border-dashed border-border/50">
            <span className="text-muted-foreground">
              <Trans>SPT Version</Trans>
            </span>
            <span>{mod.manifest?.sptVersion || '-'}</span>
          </div>
        </CardContent>
      </Card>

      {/* Links Card */}
      {links.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">
              <Trans>Links</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {links.map((link, i) => (
              <a
                key={i}
                href={link.url}
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-3 text-sm p-2 rounded hover:bg-muted transition-colors border"
              >
                <div className="bg-primary/10 p-1.5 rounded text-primary">
                  {getLinkIcon(link.type)}
                </div>
                <span>{link.name || link.type || 'Link'}</span>
              </a>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Dependencies Card */}
      {dependencies.length > 0 && (
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">
              <Trans>Dependencies</Trans>
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 max-h-60 overflow-y-auto">
            {dependencies.map((dep) => (
              <div
                key={dep.id}
                className="flex items-center gap-2 text-sm p-2 bg-muted/30 rounded border"
              >
                <CheckCircle2 className="size-4 text-green-500 shrink-0" />
                <span className="truncate flex-1" title={dep.id}>
                  {dep.id}
                </span>
                <span className="text-xs text-muted-foreground shrink-0">
                  {dep.version}
                </span>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Backups Card */}
      <Card>
        <CardHeader className="pb-3 flex flex-row items-center justify-between">
          <CardTitle className="text-sm font-medium">
            <Trans>Backups</Trans>
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            {backups.slice(0, 5).map((backup) => (
              <div
                key={backup.timestamp}
                className="flex items-center gap-3 p-3 border rounded-lg bg-background hover:bg-muted transition-colors cursor-pointer group"
                onClick={() => onRestoreBackup(backup.timestamp)}
              >
                <div className="bg-blue-500/10 p-2 rounded-lg text-blue-500">
                  <History className="size-4" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium text-sm truncate">
                    {backup.name || <Trans>Backup</Trans>}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {formatTimestamp(backup.timestamp)}
                  </p>
                </div>
              </div>
            ))}
            {backups.length === 0 && (
              <Empty className="border-none h-32 p-0">
                <EmptyMedia variant="icon">
                  <History className="size-5 text-muted-foreground" />
                </EmptyMedia>
                <EmptyDescription>
                  <Trans>No backups yet</Trans>
                </EmptyDescription>
              </Empty>
            )}
            {backups.length > 5 && (
              <p className="text-xs text-center text-muted-foreground">
                <Trans>and {backups.length - 5} more...</Trans>
              </p>
            )}
          </div>

          <Button className="w-full" onClick={onCreateBackup}>
            <Plus className="size-4 mr-2" />
            <Trans>Create Backup</Trans>
          </Button>
        </CardContent>
      </Card>
    </div>
  )
}

function getLinkIcon(type?: string | null) {
  switch (type?.toLowerCase()) {
    case 'code':
    case 'source':
      return <Code className="size-4" />
    case 'discord':
      return <Heart className="size-4" /> // Lucide doesn't have Discord, using Heart or Message
    case 'website':
    default:
      return <LinkIcon className="size-4" />
  }
}
