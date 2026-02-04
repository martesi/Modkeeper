import { createFileRoute, Link, useRouter } from '@tanstack/react-router'
import { useAtomValue } from 'jotai'
import { ALibraryActive } from '@/store/library'
import { useLibrary } from '@/hooks/use-library'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { ArrowLeft } from 'lucide-react'
import { useState, useMemo } from 'react'
import { commands } from '@gen/bindings'
import { ur } from '@/utils/result'
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
import { formatTimestamp } from '@/utils/mod'
import { ett } from '@/utils/error'
import { ModDetailHeader } from '@/components/mod/mod-details/mod-detail-header'
import { ModDetailSidebar } from '@/components/mod/mod-details/mod-detail-sidebar'
import { MarkdownContent } from '@/components/mod/markdown-content'
import { Badge } from '@comps/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@comps/card'
import { Empty, EmptyDescription } from '@comps/empty'
import { Checkbox } from '@comps/checkbox'
import { Label } from '@comps/label'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@comps/dialog'
import { Input } from '@comps/input'
import { msg, t } from '@lingui/core/macro'

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
  const [restoreTimestamp, setRestoreTimestamp] = useState<string | null>(null)
  const [restoreConfig, setRestoreConfig] = useState(false)
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [backupName, setBackupName] = useState('')
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

  const handleCreateBackupClick = () => {
    if (!id || !mod) return
    setBackupName(t(msg`Manual Backup`))
    setCreateDialogOpen(true)
  }

  const handleCreateBackupConfirm = () => {
    if (!id || !backupName.trim()) return
    setCreateDialogOpen(false)
    ur(commands.createBackup(id, backupName.trim()))
      .then(() => router.invalidate())
      .catch(ett)
  }

  const handleRemoveBackup = (timestamp: string) => {
    if (!id) return
    ur(commands.removeBackup(id, timestamp))
      .then(() => router.invalidate())
      .catch(ett)
  }

  const handleRestoreBackupClick = (timestamp: string) => {
    if (!id) return
    setRestoreTimestamp(timestamp)
    setRestoreConfig(false) // Default to false
  }

  const handleRestoreBackupConfirm = () => {
    if (!id || !restoreTimestamp) return
    const timestamp = restoreTimestamp
    setRestoreTimestamp(null)
    ur(commands.restoreBackup(id, timestamp, restoreConfig))
      .then(() => router.invalidate())
      .catch(ett)
  }

  if (!mod) {
    return (
      <Empty
        title={<Trans>Mod not found</Trans>}
        description={<Trans>The mod you are looking for does not exist.</Trans>}
        className="h-full"
      >
        <Link to="/library">
          <Button variant="outline" className="mt-4">
            <ArrowLeft className="size-4 mr-2" />
            <Trans>Back to Library</Trans>
          </Button>
        </Link>
      </Empty>
    )
  }

  const manifest = mod.manifest

  return (
    <div className="space-y-6 max-w-7xl mx-auto pb-10">
      {/* Back Link */}
      <div>
        <Link
          to=".."
          className="text-sm text-muted-foreground hover:text-foreground flex items-center gap-1 w-fit"
        >
          <ArrowLeft className="size-4" />
          <Trans>Back to Library</Trans>
        </Link>
      </div>

      {/* Header */}
      <ModDetailHeader
        mod={mod}
        onToggle={handleToggle}
        onRemove={handleRemoveConfirm}
      />

      {/* Main Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Left Column: Content */}
        <div className="lg:col-span-2 space-y-6">
          <Card className="min-h-[400px]">
            <CardHeader>
              <CardTitle>
                <Trans>Overview</Trans>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {documentation ? (
                <MarkdownContent content={documentation} />
              ) : manifest?.description ? (
                <p className="whitespace-pre-wrap text-muted-foreground leading-relaxed">
                  {manifest.description}
                </p>
              ) : (
                <Empty className="border-none h-40">
                  <EmptyDescription>
                    <Trans>No overview available</Trans>
                  </EmptyDescription>
                </Empty>
              )}

              {/* Effects */}
              {manifest?.effects && manifest.effects.length > 0 && (
                <div className="pt-6 border-t">
                  <h3 className="text-sm font-semibold mb-3">
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

              {/* Compatibility */}
              {manifest?.compatibility && (
                <div className="pt-6 border-t space-y-4">
                  <h3 className="text-sm font-semibold">
                    <Trans>Compatibility</Trans>
                  </h3>
                  {manifest.compatibility.include && (
                    <div>
                      <span className="text-xs text-muted-foreground block mb-2">
                        <Trans>Includes</Trans>
                      </span>
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
                      <span className="text-xs text-muted-foreground block mb-2">
                        <Trans>Excludes</Trans>
                      </span>
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
            </CardContent>
          </Card>
        </div>

        {/* Right Column: Sidebar */}
        <div className="lg:col-span-1">
          <ModDetailSidebar
            mod={mod}
            allMods={library?.mods || {}}
            backups={backups}
            onCreateBackup={handleCreateBackupClick}
            onRestoreBackup={handleRestoreBackupClick}
            onRemoveBackup={handleRemoveBackup}
          />
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
                <div className="space-y-4">
                  <p>
                    <Trans>
                      Are you sure you want to restore backup from{' '}
                      {formatTimestamp(restoreTimestamp)}? This will replace the
                      current mod state.
                    </Trans>
                  </p>
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="restore-config"
                      checked={restoreConfig}
                      onCheckedChange={(c) => setRestoreConfig(!!c)}
                    />
                    <Label htmlFor="restore-config">
                      <Trans>Restore client configuration</Trans>
                    </Label>
                  </div>
                </div>
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

      <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Create Backup</Trans>
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="backup-name">
                <Trans>Backup Name</Trans>
              </Label>
              <Input
                id="backup-name"
                value={backupName}
                onChange={(e) => setBackupName(e.target.value)}
                placeholder="Enter backup name"
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateBackupConfirm()
                }}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setCreateDialogOpen(false)}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button onClick={handleCreateBackupConfirm}>
              <Trans>Create</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
