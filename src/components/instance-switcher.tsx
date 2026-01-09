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
import { addLibraryFromDialog } from '@/lib/library-actions'

export function InstanceSwitcher() {
  const { isMobile } = useSidebar()
  const { librarySwitch, loading, createLibrary } = useLibrarySwitch()

  const active = librarySwitch?.active
  const libraries = librarySwitch?.libraries ?? []

  const handleAddLibrary = React.useCallback(async () => {
    try {
      await addLibraryFromDialog(createLibrary)
    } catch (err) {
      // Error is already logged in the function
      // You might want to show a toast notification here
    }
  }, [createLibrary])

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
    </>
  )
}
