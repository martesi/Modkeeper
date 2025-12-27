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

export function InstanceSwitcher({ instances }: InstanceSwitcherProps) {
  const { isMobile } = useSidebar()
  const [active, setActive] = React.useState<Instance>(instances[0])

  if (!active) {
    return null
  }

  return (
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
                  {active.name ?? active.version}
                </span>
                {active.name && (
                  <span className="truncate text-xs">{active.version}</span>
                )}
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
              <Trans>SPT Instance</Trans>
            </DropdownMenuLabel>
            {/* TODO the key prop needs to be actual key*/}
            {instances.map((team) => (
              <DropdownMenuItem
                key={team.name}
                onClick={() => setActive(team)}
                className="gap-2 p-2"
              >
                <Trans>{team.name} ({team.version})</Trans>
              </DropdownMenuItem>
            ))}
            <DropdownMenuSeparator />
            <DropdownMenuItem className="gap-2 p-2">
              <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                <Plus className="size-4" />
              </div>
              <div className="text-muted-foreground font-medium">
                <Trans>Add instance</Trans>
              </div>
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </SidebarMenuItem>
    </SidebarMenu>
  )
}

export interface Instance {
  version: string
  name?: string
}

export interface InstanceSwitcherProps {
  instances: Instance[]
}
