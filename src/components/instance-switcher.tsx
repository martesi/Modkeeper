'use client'

import * as React from 'react'
import { ChevronsUpDown, Plus, Server, Pencil, X, Trash2 } from 'lucide-react'

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
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Trans } from '@lingui/react/macro'
import { useLibrarySwitch } from '@/hooks/use-library-state'
import { addLibraryFromDialog } from '@/lib/library-actions'

export function InstanceSwitcher() {
  const { isMobile } = useSidebar()
  const {
    librarySwitch,
    loading,
    createLibrary,
    openLibrary,
    renameLibrary,
    closeLibrary,
    removeLibrary,
  } = useLibrarySwitch()

  const active = librarySwitch?.active
  const libraries = librarySwitch?.libraries ?? []

  // State for rename dialog
  const [renameDialogOpen, setRenameDialogOpen] = React.useState(false)
  const [renameLibraryId, setRenameLibraryId] = React.useState<string | null>(
    null,
  )
  const [renameValue, setRenameValue] = React.useState('')

  // State for close confirmation
  const [closeDialogOpen, setCloseDialogOpen] = React.useState(false)
  const [closeLibraryInfo, setCloseLibraryInfo] = React.useState<{
    id: string
    name: string
    repoRoot: string
  } | null>(null)

  // State for remove confirmation
  const [removeDialogOpen, setRemoveDialogOpen] = React.useState(false)
  const [removeLibraryInfo, setRemoveLibraryInfo] = React.useState<{
    id: string
    name: string
    repoRoot: string
  } | null>(null)

  const handleAddLibrary = React.useCallback(async () => {
    try {
      await addLibraryFromDialog(createLibrary)
    } catch (err) {
      // Error is already logged in the function
      // You might want to show a toast notification here
    }
  }, [createLibrary])

  const handleSwitchLibrary = React.useCallback(
    async (libPath: string) => {
      try {
        await openLibrary(libPath)
      } catch (err) {
        console.error('Failed to switch library:', err)
        // You might want to show a toast notification here
      }
    },
    [openLibrary],
  )

  const handleRenameClick = React.useCallback((lib: (typeof libraries)[0]) => {
    setRenameLibraryId(lib.id)
    setRenameValue(lib.name || '')
    setRenameDialogOpen(true)
  }, [])

  const handleRenameConfirm = React.useCallback(async () => {
    if (!renameLibraryId || !renameValue.trim()) return

    try {
      // Only rename active library (renameLibrary only works on active library)
      if (active && active.id === renameLibraryId) {
        await renameLibrary(renameValue.trim())
      }
      setRenameDialogOpen(false)
      setRenameLibraryId(null)
      setRenameValue('')
    } catch (err) {
      console.error('Failed to rename library:', err)
    }
  }, [renameLibraryId, renameValue, active, renameLibrary])

  const handleCloseClick = React.useCallback((lib: (typeof libraries)[0]) => {
    if (!lib.repo_root) return
    setCloseLibraryInfo({
      id: lib.id,
      name: lib.name || 'Unnamed Library',
      repoRoot: lib.repo_root,
    })
    setCloseDialogOpen(true)
  }, [])

  const handleCloseConfirm = React.useCallback(async () => {
    if (!closeLibraryInfo) return

    try {
      await closeLibrary(closeLibraryInfo.repoRoot)
      setCloseDialogOpen(false)
      setCloseLibraryInfo(null)
    } catch (err) {
      console.error('Failed to close library:', err)
    }
  }, [closeLibraryInfo, closeLibrary])

  const handleRemoveClick = React.useCallback((lib: (typeof libraries)[0]) => {
    if (!lib.repo_root) return
    setRemoveLibraryInfo({
      id: lib.id,
      name: lib.name || 'Unnamed Library',
      repoRoot: lib.repo_root,
    })
    setRemoveDialogOpen(true)
  }, [])

  const handleRemoveConfirm = React.useCallback(async () => {
    if (!removeLibraryInfo) return

    try {
      await removeLibrary(removeLibraryInfo.repoRoot)
      setRemoveDialogOpen(false)
      setRemoveLibraryInfo(null)
    } catch (err) {
      console.error('Failed to remove library:', err)
    }
  }, [removeLibraryInfo, removeLibrary])

  if (loading && !active) {
    return (
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton size="lg" disabled>
            <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
              <Server className="size-4" />
            </div>
            <div className="grid flex-1 text-left text-sm leading-tight">
              <span className="truncate font-medium">
                <Trans>Loading...</Trans>
              </span>
            </div>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    )
  }

  if (!active) {
    return (
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton
            size="lg"
            onClick={handleAddLibrary}
            className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
          >
            <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
              <Plus className="size-4" />
            </div>
            <div className="grid flex-1 text-left text-sm leading-tight">
              <span className="truncate font-medium">
                <Trans>No Library</Trans>
              </span>
              <span className="truncate text-xs">
                <Trans>Click to add library</Trans>
              </span>
            </div>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    )
  }

  return (
    <>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <SidebarMenuButton
                size="lg"
                className="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
              >
                <div className="bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg">
                  <Server className="size-4" />
                </div>
                <div className="grid flex-1 text-left text-sm leading-tight">
                  <span className="truncate font-medium">
                    {active.name || 'Unnamed Library'}
                  </span>
                  <span className="truncate text-xs">
                    SPT {active.spt_version}
                  </span>
                </div>
                <ChevronsUpDown className="ml-auto" />
              </SidebarMenuButton>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="w-(--radix-dropdown-menu-trigger-width) min-w-56 rounded-lg"
              align="start"
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuLabel className="text-muted-foreground text-xs">
                <Trans>Libraries</Trans>
              </DropdownMenuLabel>
              {libraries.map((lib) => (
                <DropdownMenuSub key={lib.id}>
                  <DropdownMenuSubTrigger
                    onClick={(e) => {
                      // Switch library on click of the sub trigger
                      if (lib.repo_root) {
                        handleSwitchLibrary(lib.repo_root)
                      }
                    }}
                    className="gap-2 p-2"
                  >
                    {lib.name || 'Unnamed Library'} (SPT {lib.spt_version})
                  </DropdownMenuSubTrigger>
                  <DropdownMenuSubContent>
                    {active && active.id === lib.id && (
                      <DropdownMenuItem
                        onClick={(e) => {
                          e.stopPropagation()
                          handleRenameClick(lib)
                        }}
                        className="gap-2"
                      >
                        <Pencil className="size-4" />
                        <Trans>Rename</Trans>
                      </DropdownMenuItem>
                    )}
                    <DropdownMenuItem
                      onClick={(e) => {
                        e.stopPropagation()
                        handleCloseClick(lib)
                      }}
                      className="gap-2"
                    >
                      <X className="size-4" />
                      <Trans>Close</Trans>
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={(e) => {
                        e.stopPropagation()
                        handleRemoveClick(lib)
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
              <DropdownMenuItem
                className="gap-2 p-2"
                onClick={handleAddLibrary}
              >
                <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                  <Plus className="size-4" />
                </div>
                <div className="text-muted-foreground font-medium">
                  <Trans>Add Library</Trans>
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>

      {/* Rename Dialog */}
      <Dialog open={renameDialogOpen} onOpenChange={setRenameDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              <Trans>Rename Library</Trans>
            </DialogTitle>
            <DialogDescription>
              <Trans>Enter a new name for this library.</Trans>
            </DialogDescription>
          </DialogHeader>
          <Input
            value={renameValue}
            onChange={(e) => setRenameValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleRenameConfirm()
              }
            }}
            placeholder={active?.name || 'Library Name'}
            autoFocus
          />
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setRenameDialogOpen(false)
                setRenameLibraryId(null)
                setRenameValue('')
              }}
            >
              <Trans>Cancel</Trans>
            </Button>
            <Button
              onClick={handleRenameConfirm}
              disabled={!renameValue.trim()}
            >
              <Trans>Rename</Trans>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Close Confirmation Dialog */}
      <AlertDialog open={closeDialogOpen} onOpenChange={setCloseDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Close Library</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              {closeLibraryInfo && (
                <Trans>
                  Are you sure you want to close "{closeLibraryInfo.name}"? It
                  will be removed from your library list but files will remain
                  on disk.
                </Trans>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              onClick={() => {
                setCloseDialogOpen(false)
                setCloseLibraryInfo(null)
              }}
            >
              <Trans>Cancel</Trans>
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleCloseConfirm}>
              <Trans>Close</Trans>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Remove Confirmation Dialog */}
      <AlertDialog open={removeDialogOpen} onOpenChange={setRemoveDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              <Trans>Remove Library</Trans>
            </AlertDialogTitle>
            <AlertDialogDescription>
              {removeLibraryInfo && (
                <Trans>
                  Are you sure you want to remove "{removeLibraryInfo.name}"?
                  This will unlink all mods, remove it from your library list,
                  and delete the library directory. This action cannot be
                  undone.
                </Trans>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel
              onClick={() => {
                setRemoveDialogOpen(false)
                setRemoveLibraryInfo(null)
              }}
            >
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
    </>
  )
}
