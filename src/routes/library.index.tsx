import { createFileRoute } from '@tanstack/react-router'
import { useAtomValue } from 'jotai'
import { useLegacyUiAtom } from '@/redesign/state/settings-state'
import { WalkingSkeletonScreen } from '@/redesign/library/walking-skeleton-screen'
import { ModList } from '@/modules/mod-list/mod-list'
import { useLibrary } from '@/hooks/use-library'
import { ALibraryActive } from '@/store/library'
import { Button } from '@comps/button'
import { Trans } from '@lingui/react/macro'
import { Upload, RefreshCw, FileArchive, FolderOpen } from 'lucide-react'
import { ett } from '@/utils/error'
import { isString } from 'remeda'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@comps/dropdown-menu'
import {
  tArchive,
  tSelectModFiles,
  tSelectModFolder,
  tUnknownModName,
  tBackup,
} from '@/utils/translation'
import { HeaderPortal } from '@/components/header-portal'
import { ButtonGroup } from '@/components/ui/button-group'
import { InstanceSwitcher } from '@/modules/mod-list/instance-switcher'

/**
 * Library index adapter (consolidated-spec.md §10 route table + §10a toggle). The redesign branch
 * mounts the 8.2 walking skeleton until 8.5's LibraryScreen replaces it. Original content
 * preserved at docs/2026-07-13_redesign/reference/current-frontend/routes/library.index.tsx.
 */
export const Route = createFileRoute('/library/')({
  component: RouteComponent,
})

function RouteComponent() {
  const useLegacyUi = useAtomValue(useLegacyUiAtom)
  return useLegacyUi ? <LegacyLibraryIndex /> : <WalkingSkeletonScreen />
}

function LegacyLibraryIndex() {
  const library = useAtomValue(ALibraryActive)
  const { add, sync } = useLibrary()

  const handleAddModFiles = () => {
    import('@tauri-apps/plugin-dialog')
      .then(({ open }) =>
        open({
          multiple: true,
          filters: [{ name: tArchive(), extensions: ['zip'] }],
          title: tSelectModFiles(),
        }),
      )
      .then(handleImportMod)
      .catch(ett)
  }

  const handleAddModFolder = () => {
    import('@tauri-apps/plugin-dialog')
      .then(({ open }) =>
        open({
          directory: true,
          multiple: false,
          title: tSelectModFolder(),
        }),
      )
      .then(handleImportMod)
      .catch(ett)
  }

  function handleImportMod(select?: string | string[] | null) {
    if (!select) return Promise.resolve()
    if (isString(select)) select = [select]
    return add(select, tUnknownModName(), tBackup())
  }

  return (
    <div className="space-y-4">
      <HeaderPortal>
        <div className="flex items-center gap-2">
          <ButtonGroup>
            <InstanceSwitcher />
          </ButtonGroup>
          {library && (
            <>
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
              <Button
                variant={library?.isDirty ? 'default' : 'outline'}
                size="icon"
                onClick={sync}
              >
                <RefreshCw />
              </Button>
            </>
          )}
        </div>
      </HeaderPortal>
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
