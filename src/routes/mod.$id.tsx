import { createFileRoute, Link } from '@tanstack/react-router'
import { useLibrary } from '@/hooks/use-library-state'
import { useMods } from '@/hooks/use-library-state'
import { Button } from '@comps/button'
import { Badge } from '@comps/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@comps/tabs'
import { Trans } from '@lingui/react/macro'
import { ArrowLeft, Package, Trash2, ChevronRight } from 'lucide-react'
import { MarkdownContent } from '@/components/mod/markdown-content'
import { useState, useEffect, useMemo } from 'react'
import { commands } from '@gen/bindings'
import { unwrapResult } from '@/lib/result'
import type { Mod, ModManifest, Dependencies } from '@gen/bindings'

export const Route = createFileRoute('/mod/$id')({
  component: ModDetailsComponent,
})

function ModDetailsComponent() {
  const { id } = Route.useParams()
  const { library } = useLibrary()
  const { toggleMod, removeMods } = useMods()
  const [documentation, setDocumentation] = useState<string | null>(null)
  const [backups, setBackups] = useState<string[]>([])
  const [loadingDocs, setLoadingDocs] = useState(false)
  const [loadingBackups, setLoadingBackups] = useState(false)

  const mod = useMemo(() => {
    if (!library?.mods) return null
    return library.mods[id] || null
  }, [library, id])

  useEffect(() => {
    if (mod?.manifest?.documentation && id) {
      setLoadingDocs(true)
      unwrapResult(commands.getModDocumentation(id))
        .then((docs) => {
          setDocumentation(docs)
        })
        .catch((err) => {
          console.error('Failed to load documentation:', err)
          setDocumentation(null)
        })
        .finally(() => {
          setLoadingDocs(false)
        })
    }
  }, [id, mod?.manifest?.documentation])

  useEffect(() => {
    if (id) {
      setLoadingBackups(true)
      unwrapResult(commands.getBackups(id))
        .then((backupList) => {
          setBackups(backupList)
        })
        .catch((err) => {
          console.error('Failed to load backups:', err)
          setBackups([])
        })
        .finally(() => {
          setLoadingBackups(false)
        })
    }
  }, [id])

  const handleToggle = async () => {
    if (!mod) return
    try {
      await toggleMod(id, !mod.is_active)
    } catch (err) {
      console.error('Failed to toggle mod:', err)
    }
  }

  const handleRemove = async () => {
    if (!mod) return
    if (confirm(`Are you sure you want to remove "${mod.name}"?`)) {
      try {
        await removeMods([id])
        window.history.back()
      } catch (err) {
        console.error('Failed to remove mod:', err)
      }
    }
  }

  const handleRestoreBackup = async (timestamp: string) => {
    if (!id) return
    if (confirm(`Are you sure you want to restore backup from ${timestamp}?`)) {
      try {
        await unwrapResult(commands.restoreBackup(id, timestamp))
        // Reload backups after restore
        const backupList = await unwrapResult(commands.getBackups(id))
        setBackups(backupList)
      } catch (err) {
        console.error('Failed to restore backup:', err)
      }
    }
  }

  if (!mod) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-destructive">
          <Trans>Mod not found</Trans>
        </p>
        <Link to="/">
          <Button variant="outline">
            <ArrowLeft className="size-4 mr-2" />
            <Trans>Back to Library</Trans>
          </Button>
        </Link>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex items-start gap-4">
          <Link to="/">
            <Button variant="ghost" size="icon">
              <ArrowLeft className="size-4" />
            </Button>
          </Link>
          <div>
            <div className="flex items-center gap-2 mb-2">
              {mod.icon_data ? (
                <img
                  src={mod.icon_data}
                  alt={mod.name}
                  className="size-12 rounded"
                />
              ) : (
                <Package className="size-12 text-muted-foreground" />
              )}
              <div>
                <h1 className="text-3xl font-bold">{mod.name}</h1>
                {mod.manifest && (
                  <p className="text-sm text-muted-foreground">
                    v{mod.manifest.version} by{' '}
                    {Array.isArray(mod.manifest.author)
                      ? mod.manifest.author.join(', ')
                      : mod.manifest.author}
                  </p>
                )}
              </div>
            </div>
            <div className="flex gap-2">
              <ModTypeBadge type={mod.mod_type} />
              <Badge variant={mod.is_active ? 'default' : 'secondary'}>
                {mod.is_active ? <Trans>Active</Trans> : <Trans>Inactive</Trans>}
              </Badge>
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          <Button onClick={handleToggle} variant="outline">
            {mod.is_active ? <Trans>Deactivate</Trans> : <Trans>Activate</Trans>}
          </Button>
          <Button onClick={handleRemove} variant="destructive">
            <Trash2 className="size-4 mr-2" />
            <Trans>Remove</Trans>
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <Tabs defaultValue="overview" className="w-full">
        <TabsList>
          <TabsTrigger value="overview">
            <Trans>Overview</Trans>
          </TabsTrigger>
          {mod.manifest?.dependencies && (
            <TabsTrigger value="dependencies">
              <Trans>Dependencies</Trans>
            </TabsTrigger>
          )}
          {mod.manifest?.documentation && (
            <TabsTrigger value="documentation">
              <Trans>Documentation</Trans>
            </TabsTrigger>
          )}
          <TabsTrigger value="backups">
            <Trans>Backups</Trans>
          </TabsTrigger>
          {mod.manifest?.links && mod.manifest.links.length > 0 && (
            <TabsTrigger value="links">
              <Trans>Links</Trans>
            </TabsTrigger>
          )}
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <OverviewTab mod={mod} />
        </TabsContent>

        {mod.manifest?.dependencies && (
          <TabsContent value="dependencies" className="space-y-4">
            <DependenciesTab dependencies={mod.manifest.dependencies} />
          </TabsContent>
        )}

        {mod.manifest?.documentation && (
          <TabsContent value="documentation" className="space-y-4">
            <DocumentationTab
              documentation={documentation}
              loading={loadingDocs}
            />
          </TabsContent>
        )}

        <TabsContent value="backups" className="space-y-4">
          <BackupsTab
            backups={backups}
            loading={loadingBackups}
            onRestore={handleRestoreBackup}
          />
        </TabsContent>

        {mod.manifest?.links && mod.manifest.links.length > 0 && (
          <TabsContent value="links" className="space-y-4">
            <LinksTab links={mod.manifest.links} />
          </TabsContent>
        )}
      </Tabs>
    </div>
  )
}

function ModTypeBadge({ type }: { type: Mod['mod_type'] }) {
  const colorClass =
    type === 'Client'
      ? 'bg-blue-500/20 text-blue-700 dark:text-blue-400'
      : type === 'Server'
        ? 'bg-green-500/20 text-green-700 dark:text-green-400'
        : type === 'Both'
          ? 'bg-purple-500/20 text-purple-700 dark:text-purple-400'
          : 'bg-gray-500/20 text-gray-700 dark:text-gray-400'

  return (
    <Badge variant="outline" className={colorClass}>
      {type === 'Client' && <Trans>Client</Trans>}
      {type === 'Server' && <Trans>Server</Trans>}
      {type === 'Both' && <Trans>Both</Trans>}
      {type === 'Unknown' && <Trans>Unknown</Trans>}
    </Badge>
  )
}

function OverviewTab({ mod }: { mod: Mod }) {
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

function DependenciesTab({ dependencies }: { dependencies: Dependencies }) {
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

function DocumentationTab({
  documentation,
  loading,
}: {
  documentation: string | null
  loading: boolean
}) {
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

function BackupsTab({
  backups,
  loading,
  onRestore,
}: {
  backups: string[]
  loading: boolean
  onRestore: (timestamp: string) => void
}) {
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

function LinksTab({ links }: { links: ModManifest['links'] }) {
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

function formatTimestamp(timestamp: string): string {
  try {
    const date = new Date(timestamp)
    return date.toLocaleString()
  } catch {
    return timestamp
  }
}

function getLinkTypeName(type: string | undefined | null): string {
  switch (type) {
    case 'code':
      return 'Source Code'
    case 'discord':
      return 'Discord'
    case 'website':
      return 'Website'
    case 'documentation':
      return 'Documentation'
    default:
      return 'Link'
  }
}
