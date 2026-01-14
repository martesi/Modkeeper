import { createFileRoute, Link } from '@tanstack/react-router'
import { useAtomValue } from 'jotai'
import { ALibraryActive } from '@/store/library'
import { useLibrary } from '@/hooks/use-library'
import { Button } from '@comps/button'
import { Badge } from '@comps/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@comps/tabs'
import { Trans } from '@lingui/react/macro'
import { ArrowLeft, Package, Trash2 } from 'lucide-react'
import { useState, useEffect, useMemo } from 'react'
import { commands, ModBackup } from '@gen/bindings'
import { ur } from '@/utils/result'
import { msg, t } from '@lingui/core/macro'
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
import { ModTypeBadge } from '@/components/mod/mod-type-badge'
import { OverviewTab } from '@/components/mod/mod-details/overview-tab'
import { DependenciesTab } from '@/components/mod/mod-details/dependencies-tab'
import { DocumentationTab } from '@/components/mod/mod-details/documentation-tab'
import { BackupsTab } from '@/components/mod/mod-details/backups-tab'
import { LinksTab } from '@/components/mod/mod-details/links-tab'
import { formatTimestamp } from '@/utils/mod'
import { ett } from '@/utils/error'
import { tDivider } from '@/utils/translation'
import { ModVersion } from '@/components/mod/mod-version'

export const Route = createFileRoute('/$id')({
  component: ModDetailsComponent,
  staticData: {
    breadcrumb: () => t(msg`Mod Details`),
  },
  loader: async ({ params: { id } }) => {
    const backups = await commands
      .getBackups(id)
      .then(ur)
      .catch((v) => {
        ett(v)
        return []
      })

    return { backups }
  },
})

function ModDetailsComponent() {
  const { id } = Route.useParams()
  const library = useAtomValue(ALibraryActive)
  const { toggle, remove } = useLibrary()
  const [documentation, setDocumentation] = useState<string | null>(null)
  const [loadingDocs, setLoadingDocs] = useState(false)
  const [showRemoveDialog, setShowRemoveDialog] = useState(false)
  const [showRestoreDialog, setShowRestoreDialog] = useState(false)
  const [restoreTimestamp, setRestoreTimestamp] = useState<string | null>(null)
  const { backups } = Route.useLoaderData()
  const router = useRouter()

  const mod = useMemo(() => {
    if (!library?.mods) return null
    return library.mods[id] || null
  }, [library, id])

  useEffect(() => {
    if (mod?.manifest?.documentation && id) {
      setLoadingDocs(true)
      ur(commands.getModDocumentation(id))
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

  const handleToggle = async () => {
    if (!mod) return
    try {
      await toggle(id, !mod.is_active)
    } catch (err) {
      console.error('Failed to toggle mod:', err)
    }
  }

  const handleRemoveClick = () => {
    if (!mod) return
    setShowRemoveDialog(true)
  }

  const handleRemoveConfirm = async () => {
    if (!mod) return
    setShowRemoveDialog(false)
    try {
      await remove([id])
      window.history.back()
    } catch (err) {
      console.error('Failed to remove mod:', err)
    }
  }

  const handleRestoreBackupClick = (timestamp: string) => {
    if (!id) return
    setRestoreTimestamp(timestamp)
    setShowRestoreDialog(true)
  }

  const handleRestoreBackupConfirm = async () => {
    if (!id || !restoreTimestamp) return
    setShowRestoreDialog(false)
    try {
      await ur(commands.restoreBackup(id, restoreTimestamp))
      router.invalidate()
      setRestoreTimestamp(null)
    } catch (err) {
      console.error('Failed to restore backup:', err)
      setRestoreTimestamp(null)
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
                <ModVersion mod={mod} />
              </div>
            </div>
            <div className="flex gap-2">
              <ModTypeBadge type={mod.mod_type} />
              <Badge variant={mod.is_active ? 'default' : 'secondary'}>
                {mod.is_active ? (
                  <Trans>Active</Trans>
                ) : (
                  <Trans>Inactive</Trans>
                )}
              </Badge>
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          <Button onClick={handleToggle} variant="outline">
            {mod.is_active ? (
              <Trans>Deactivate</Trans>
            ) : (
              <Trans>Activate</Trans>
            )}
          </Button>
          <Button onClick={handleRemoveClick} variant="destructive">
            <Trash2 className="size-4 mr-2" />
            <Trans>Remove</Trans>
          </Button>
        </div>
      </div>

      <AlertDialog open={showRemoveDialog} onOpenChange={setShowRemoveDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Remove Mod</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              {mod && (
                <Trans>
                  Are you sure you want to remove "{mod.name}"? This action
                  cannot be undone.
                </Trans>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleRemoveConfirm}
              variant="destructive"
            >
              <Trans>Remove</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={showRestoreDialog}
        onOpenChange={(open) => {
          setShowRestoreDialog(open)
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
            <DocumentationTab
              documentation={documentation}
              loading={loadingDocs}
            />
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
