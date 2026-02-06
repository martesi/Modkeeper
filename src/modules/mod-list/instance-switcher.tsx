'use client'

import { useState } from 'react'
import {
  ChevronsUpDown,
  FolderOpen,
  Plus,
  RefreshCw,
  Server,
  Pencil,
  X,
  Trash2,
} from 'lucide-react'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

import { Button } from '@/components/ui/button'
import { Trans } from '@lingui/react/macro'
import { useAtomValue } from 'jotai'
import { ALibraryActive, ALibraryList } from '@/store/library'
import { useLibrarySwitch } from '@/hooks/use-library-switch'
import { addLibraryFromDialog } from '@/lib/library-actions'
import { ett } from '@/utils/error'
import { RenameLibraryDialog } from './rename-library-dialog'
import { CloseLibraryDialog } from './close-library-dialog'
import { RemoveLibraryDialog } from './remove-library-dialog'
import type { LibraryDTO } from '@gen/bindings'

export function InstanceSwitcher() {
  const active = useAtomValue(ALibraryActive)
  const libraries = useAtomValue(ALibraryList)
  const { create, open, rename, close, remove, rebuildCache } =
    useLibrarySwitch()

  // Dialog states - use library to indicate open
  const [renameLibrary, setRenameLibrary] = useState<LibraryDTO>()
  const [closeLibrary, setCloseLibrary] = useState<LibraryDTO>()
  const [removeLibrary, setRemoveLibrary] = useState<LibraryDTO>()

  const handleAddLibrary = () => addLibraryFromDialog(create).catch(ett)

  const handleSwitchLibrary = (libPath: string) => open(libPath).catch(ett)

  const handleRenameConfirm = (newName: string) => {
    if (!renameLibrary) return
    rename(newName, renameLibrary.id)
      .then(() => setRenameLibrary(undefined))
      .catch(ett)
  }

  const handleCloseConfirm = () => {
    if (!closeLibrary) return
    close(closeLibrary.repoRoot)
      .then(() => setCloseLibrary(undefined))
      .catch(ett)
  }

  const handleRemoveConfirm = () => {
    if (!removeLibrary) return
    remove(removeLibrary.repoRoot)
      .then(() => setRemoveLibrary(undefined))
      .catch(ett)
  }

  if (!active) {
    return (
      <Button
        size="lg"
        variant={'outline'}
        onClick={handleAddLibrary}
        className="w-full justify-start px-2"
      >
        <div className="bg-primary/10 text-primary flex aspect-square size-8 items-center justify-center rounded-lg">
          <Plus className="size-4" />
        </div>
        <div className="grid flex-1 text-left text-sm leading-tight ml-2">
          <span className="truncate font-medium">
            <Trans>No Library</Trans>
          </span>
          <span className="truncate text-xs text-muted-foreground">
            <Trans>Click to add library</Trans>
          </span>
        </div>
      </Button>
    )
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant={'outline'}
            size="lg"
            className="w-full justify-start px-2"
          >
            <div className="flex aspect-square size-8 items-center justify-center rounded-lg">
              <Server className="size-4" />
            </div>
            <div className="grid flex-1 text-left text-sm leading-tight ml-2">
              <span className="truncate font-medium">
                {active.name || 'Unnamed Library'}
              </span>
              <span className="truncate text-xs text-muted-foreground">
                SPT {active.sptVersion}
              </span>
            </div>
            <ChevronsUpDown className="ml-auto size-4 opacity-50" />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className="w-56 rounded-lg"
          align="start"
          side="bottom"
          sideOffset={4}
        >
          <DropdownMenuLabel className="text-muted-foreground text-xs">
            <Trans>Libraries</Trans>
          </DropdownMenuLabel>
          {libraries.map((lib) => (
            <DropdownMenuSub key={lib.id}>
              <DropdownMenuSubTrigger
                onClick={() =>
                  lib.repoRoot && handleSwitchLibrary(lib.repoRoot)
                }
                className="gap-2 p-2"
              >
                {lib.name} (SPT {lib.sptVersion})
              </DropdownMenuSubTrigger>
              <DropdownMenuSubContent>
                {!(active && active.id === lib.id) && (
                  <DropdownMenuItem
                    onClick={(e) => {
                      e.stopPropagation()
                      if (lib.repoRoot) handleSwitchLibrary(lib.repoRoot)
                    }}
                    className="gap-2"
                  >
                    <FolderOpen className="size-4" />
                    <Trans>Open</Trans>
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation()
                    setRenameLibrary(lib)
                  }}
                  className="gap-2"
                >
                  <Pencil className="size-4" />
                  <Trans>Rename</Trans>
                </DropdownMenuItem>
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation()
                    rebuildCache(lib.id).catch(ett)
                  }}
                  className="gap-2"
                >
                  <RefreshCw className="size-4" />
                  <Trans>Rebuild Cache</Trans>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                {active && active.id === lib.id && (
                  <DropdownMenuItem
                    onClick={(e) => {
                      e.stopPropagation()
                      setCloseLibrary(lib)
                    }}
                    className="gap-2"
                  >
                    <X className="size-4" />
                    <Trans>Close</Trans>
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem
                  onClick={(e) => {
                    e.stopPropagation()
                    setRemoveLibrary(lib)
                  }}
                  variant="destructive"
                  className="gap-2"
                >
                  <Trash2 className="size-4" />
                  <Trans>Remove</Trans>
                </DropdownMenuItem>
              </DropdownMenuSubContent>
            </DropdownMenuSub>
          ))}
          <DropdownMenuSeparator />
          <DropdownMenuItem className="gap-2 p-2" onClick={handleAddLibrary}>
            <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
              <Plus className="size-4" />
            </div>
            <div className="text-muted-foreground font-medium">
              <Trans>Add Library</Trans>
            </div>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <RenameLibraryDialog
        open={!!renameLibrary}
        onOpenChange={(open) => !open && setRenameLibrary(undefined)}
        currentName={renameLibrary?.name || ''}
        onConfirm={handleRenameConfirm}
      />

      <CloseLibraryDialog
        open={!!closeLibrary}
        onOpenChange={(open) => !open && setCloseLibrary(undefined)}
        libraryName={closeLibrary?.name || ''}
        onConfirm={handleCloseConfirm}
        onCancel={() => setCloseLibrary(undefined)}
      />

      <RemoveLibraryDialog
        open={!!removeLibrary}
        onOpenChange={(open) => !open && setRemoveLibrary(undefined)}
        libraryName={removeLibrary?.name || ''}
        onConfirm={handleRemoveConfirm}
        onCancel={() => setRemoveLibrary(undefined)}
      />
    </>
  )
}
