import { createFileRoute, Link, useRouter } from '@tanstack/react-router'
import { useAtomValue } from 'jotai'
import { ALibraryActive } from '@/store/library'
import { useLibrary } from '@/hooks/use-library'
import { Button } from '@comps/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@comps/tabs'
import { Trans } from '@lingui/react/macro'
import { ArrowLeft, Package, Trash2 } from 'lucide-react'
import { useState, useMemo } from 'react'
import { useBoolean } from 'ahooks'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
import { ConfirmPopover } from '@comps/confirm-popover'
import { Switch } from '@comps/switch'
import { HeaderPortal } from '@/components/header-portal'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@comps/alert-dialog'
import { OverviewTab } from '@/components/mod/mod-details/overview-tab'
import { DependenciesTab } from '@/components/mod/mod-details/dependencies-tab'
import { DocumentationTab } from '@/components/mod/mod-details/documentation-tab'
import { BackupsTab } from '@/components/mod/mod-details/backups-tab'
import { LinksTab } from '@/components/mod/mod-details/links-tab'
import { formatTimestamp } from '@/utils/mod'
import { ett } from '@/utils/error'

export const Route = createFileRoute('/library/$id')({
  component: ModDetailsComponent,
  loader: async ({ params: { id } }) => {
    const [backups, documentation] = await Promise.all([
      commands
        .getBackups(id)
        .then(ur)
        .catch((v) => {
          ett(v)
          return []
        }),
      commands
        .getModDocumentation(id)
        .then(ur)
        .catch(() => null),
    ])

    return { backups, documentation }
  },
})

function ModDetailsComponent() {
  const { id } = Route.useParams()
  const library = useAtomValue(ALibraryActive)
  const { toggle, remove } = useLibrary()
  const { backups, documentation } = Route.useLoaderData()
  const [
    showRemoveDialog,
    { setTrue: setShowRemoveDialogTrue, set: setShowRemoveDialog },
  ] = useBoolean()
  const [restoreTimestamp, setRestoreTimestamp] = useState<string | null>(null)
  const router = useRouter()

  const mod = useMemo(() => {
    if (!library?.mods) return null
    return library.mods[id] || null
  }, [library, id])

  const handleToggle = () => {
    if (!mod) return
    toggle(id, !mod.isActive).catch(ett)
  }

  const handleRemoveConfirm = () => {
    if (!id) return
    if (!mod) return
    remove([id])
      .then(() => router.navigate({ to: '/library', replace: true }))
      .catch(ett)
  }

  const handleRestoreBackupClick = (timestamp: string) => {
    if (!id) return
    setRestoreTimestamp(timestamp)
  }

  const handleRestoreBackupConfirm = () => {
    if (!id || !restoreTimestamp) return
    const timestamp = restoreTimestamp
    setRestoreTimestamp(null)
    ur(commands.restoreBackup(id, timestamp))
      .then(() => router.invalidate())
      .catch(ett)
  }

  if (!mod) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-destructive">
          <Trans>Mod not found</Trans>
        </p>
        <Link to="/library">
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
      <HeaderPortal>
        <div className="flex items-center gap-2">
          <ConfirmPopover
            open={showRemoveDialog}
            onOpenChange={setShowRemoveDialog}
            title={<Trans>Remove Mod</Trans>}
            description={
              mod ? (
                <Trans>
                  Are you sure you want to remove &quot;{mod.name}&quot;? This
                  action cannot be undone.
                </Trans>
              ) : (
                ''
              )
            }
            confirmLabel={<Trans>Remove</Trans>}
            variant="destructive"
            onConfirm={handleRemoveConfirm}
            trigger={
              <Button
                variant="outline"
                size="icon"
                onClick={setShowRemoveDialogTrue}
              >
                <Trash2 className="size-4" />
              </Button>
            }
            side="top"
          />
          <Switch checked={mod.isActive} onCheckedChange={handleToggle} />
        </div>
      </HeaderPortal>

      {/* Header */}
      <div className="flex items-center gap-4 mb-2 w-full">
        <Link to="..">
          <Button variant="ghost" size="icon">
            <ArrowLeft className="size-4" />
          </Button>
        </Link>
        <div className="shrink-0">
          {mod.iconData ? (
            <img
              src={mod.iconData}
              alt={mod.name}
              className="size-12 rounded"
            />
          ) : (
            <Package className="size-12 text-muted-foreground" />
          )}
        </div>
        <div className="flex-1">
          <h1 className="text-3xl font-bold truncate">{mod.name}</h1>
        </div>
      </div>

      <AlertDialog
        open={!!restoreTimestamp}
        onOpenChange={(open) => {
          if (!open) {
            setRestoreTimestamp(null)
          }
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Restore Backup</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              {restoreTimestamp && (
                <Trans>
                  Are you sure you want to restore backup from{' '}
                  {formatTimestamp(restoreTimestamp)}? This will replace the
                  current mod state.
                </Trans>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleRestoreBackupConfirm}>
              <Trans>Restore</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

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
            <DocumentationTab documentation={documentation} loading={false} />
          </TabsContent>
        )}

        <TabsContent value="backups" className="space-y-4">
          <BackupsTab backups={backups} onRestore={handleRestoreBackupClick} />
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
