'use client'

import * as React from 'react'
import { ChevronsUpDown, Plus, Server } from 'lucide-react'

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar'
import { Trans } from '@lingui/react/macro'
import { useLibrarySwitch } from '@/hooks/use-library-state'
import { CreateLibraryDialog } from '@/components/library/create-library-dialog'
import { OpenLibraryDialog } from '@/components/library/open-library-dialog'

export function InstanceSwitcher() {
  const { isMobile } = useSidebar()
  const { librarySwitch, loading } = useLibrarySwitch()
  const [createDialogOpen, setCreateDialogOpen] = React.useState(false)
  const [openDialogOpen, setOpenDialogOpen] = React.useState(false)

  const active = librarySwitch?.active
  const libraries = librarySwitch?.libraries ?? []

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
            onClick={() => setOpenDialogOpen(true)}
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
                <Trans>Click to open or create</Trans>
              </span>
            </div>
          </SidebarMenuButton>
          <OpenLibraryDialog
            open={openDialogOpen}
            onOpenChange={setOpenDialogOpen}
            onSuccess={() => {
              // Library will be refreshed by the hook
            }}
          />
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
                <DropdownMenuItem
                  key={lib.id}
                  onClick={() => {
                    // TODO: Implement library switching
                    // This would need a command to switch libraries
                    console.log('Switch to library:', lib.id)
                  }}
                  className="gap-2 p-2"
                >
                  {lib.name || 'Unnamed Library'} (SPT {lib.spt_version})
                </DropdownMenuItem>
              ))}
              <DropdownMenuSeparator />
              <DropdownMenuItem
                className="gap-2 p-2"
                onClick={() => setOpenDialogOpen(true)}
              >
                <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                  <Server className="size-4" />
                </div>
                <div className="text-muted-foreground font-medium">
                  <Trans>Open Library</Trans>
                </div>
              </DropdownMenuItem>
              <DropdownMenuItem
                className="gap-2 p-2"
                onClick={() => setCreateDialogOpen(true)}
              >
                <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                  <Plus className="size-4" />
                </div>
                <div className="text-muted-foreground font-medium">
                  <Trans>Create Library</Trans>
                </div>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <CreateLibraryDialog
        open={createDialogOpen}
        onOpenChange={setCreateDialogOpen}
        onSuccess={() => {
          // Library will be refreshed by the hook
        }}
      />
      <OpenLibraryDialog
        open={openDialogOpen}
        onOpenChange={setOpenDialogOpen}
        onSuccess={() => {
          // Library will be refreshed by the hook
        }}
      />
    </>
  )
}
