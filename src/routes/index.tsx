import { createFileRoute } from '@tanstack/react-router'
import { ModList } from '@/components/mod/mod-list'
import { useLibrary } from '@/hooks/use-library'
import { useAtomValue } from 'jotai'
import { ALibraryActive } from '@/store/library'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { Upload, RefreshCw, FileArchive, FolderOpen } from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@comps/dropdown-menu'
import { msg, t } from '@lingui/core/macro'
import { tArchive, tSelectModFiles, tSelectModFolder, tUnknownModName } from '@/utils/translation'
import { HeaderPortal } from '@/components/header-portal'
import { ButtonGroup } from '@/components/ui/button-group'

export const Route = createFileRoute('/')({
  component: RouteComponent,
  staticData: {
    breadcrumb: () => t(msg`Library`),
  },
})

function RouteComponent () {
  const library = useAtomValue(ALibraryActive)
  const { add, sync } = useLibrary()

  const handleAddModFiles = async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog')
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: tArchive(),
            extensions: ['zip'],
          },
        ],
        title: tSelectModFiles(),
      })
      const unknownModName = tUnknownModName()
      if (selected && Array.isArray(selected)) {
        await add(selected, unknownModName)
      } else if (selected && typeof selected === 'string') {
        await add([selected], unknownModName)
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
        title: tSelectModFolder(),
      })
      const unknownModName = tUnknownModName()
      if (selected && typeof selected === 'string') {
        await add([selected], unknownModName)
      }
    } catch (err) {
      console.error('Failed to add mod folder:', err)
    }
  }


  return (
    <div className="space-y-4">
      {library && <HeaderPortal>
        <ButtonGroup>
          <ButtonGroup>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size={'icon'}>
                  <Upload className="" />
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
          </ButtonGroup>
          <ButtonGroup>
            <Button variant={library?.is_dirty ? "default" : "outline"} size="icon" onClick={sync}><RefreshCw /></Button>
          </ButtonGroup>
        </ButtonGroup>
      </HeaderPortal>}
      {library ? (
        <ModList />
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
