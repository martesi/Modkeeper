import { createFileRoute } from '@tanstack/react-router'
import { ModList } from '@/components/mod/mod-list'
import { useLibrary } from '@/hooks/use-library-state'
import { useMods } from '@/hooks/use-library-state'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { Upload, RefreshCw, FileArchive, FolderOpen } from 'lucide-react'
import { useState } from 'react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@comps/dropdown-menu'

export const Route = createFileRoute('/')({
  component: RouteComponent,
})

function RouteComponent() {
  const { library, loading, error, refresh } = useLibrary()
  const {
    addMods,
    removeMods,
    toggleMod,
    syncMods,
    loading: modsLoading,
  } = useMods()
  const [isSyncing, setIsSyncing] = useState(false)

  const handleAddModFiles = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'Archive',
            extensions: ['zip'],
          },
        ],
        title: 'Select Mod Files',
      })
      if (selected && Array.isArray(selected)) {
        await addMods(selected)
      } else if (selected && typeof selected === 'string') {
        await addMods([selected])
      }
    } catch (err) {
      console.error('Failed to add mod files:', err)
    }
  }

  const handleAddModFolder = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Mod Folder',
      })
      if (selected && typeof selected === 'string') {
        await addMods([selected])
      }
    } catch (err) {
      console.error('Failed to add mod folder:', err)
    }
  }

  const handleSync = async () => {
    setIsSyncing(true)
    try {
      await syncMods()
    } catch (err) {
      console.error('Failed to sync mods:', err)
    } finally {
      setIsSyncing(false)
    }
  }

  const handleToggleMod = async (id: string, isActive: boolean) => {
    try {
      await toggleMod(id, isActive)
    } catch (err) {
      console.error('Failed to toggle mod:', err)
    }
  }

  const handleRemoveMods = async (id: string) => {
    try {
      await removeMods([id])
    } catch (err) {
      console.error('Failed to remove mod:', err)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Trans>Loading library...</Trans>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <p className="text-destructive">{error.message}</p>
        <Button onClick={refresh} variant="outline">
          <Trans>Retry</Trans>
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">
            {library?.name || <Trans>Mod Library</Trans>}
          </h1>
          {library && (
            <p className="text-sm text-muted-foreground">
              <Trans>SPT {library.spt_version}</Trans>
              {library.is_dirty && (
                <span className="ml-2 text-warning">
                  <Trans>(Needs Sync)</Trans>
                </span>
              )}
            </p>
          )}
        </div>
        <div className="flex gap-2">
          {library && (
            <>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" disabled={modsLoading}>
                    <Upload className="size-4 mr-2" />
                    <Trans>Add Mods</Trans>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuItem onClick={handleAddModFiles}>
                    <FileArchive className="size-4 mr-2" />
                    <Trans>Add Mod Files (.zip)</Trans>
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={handleAddModFolder}>
                    <FolderOpen className="size-4 mr-2" />
                    <Trans>Add Mod Folder</Trans>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
              <Button
                variant="default"
                onClick={handleSync}
                disabled={modsLoading || isSyncing}
              >
                <RefreshCw
                  className={`size-4 mr-2 ${isSyncing ? 'animate-spin' : ''}`}
                />
                <Trans>Sync Mods</Trans>
              </Button>
            </>
          )}
        </div>
      </div>

      {library ? (
        <ModList
          library={library}
          onModToggle={handleToggleMod}
          onModRemove={handleRemoveMods}
        />
      ) : (
        <div className="flex flex-col items-center justify-center h-64 text-muted-foreground">
          <p className="text-lg mb-2">
            <Trans>No library loaded</Trans>
          </p>
          <p className="text-sm">
            <Trans>Open or create a library from the sidebar</Trans>
          </p>
        </div>
      )}
    </div>
  )
}
