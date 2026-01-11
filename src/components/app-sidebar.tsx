'use client'

import * as React from 'react'
import { useMatches, Link } from '@tanstack/react-router'

import { InstanceSwitcher } from '@comps/instance-switcher.tsx'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
  SidebarGroup,
} from '@/components/ui/sidebar'
import { Home, Settings } from 'lucide-react'
import { Trans } from '@lingui/react/macro'

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const matches = useMatches()
  const currentPath = matches[matches.length - 1]?.pathname ?? '/'

  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <InstanceSwitcher />
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton
                asChild
                isActive={currentPath === '/'}
                tooltip="Library"
              >
                <Link to="/">
                  <Home className="size-4" />
                  <span>
                    <Trans>Library</Trans>
                  </span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton
                asChild
                isActive={currentPath === '/settings'}
                tooltip="Settings"
              >
                <Link to="/settings">
                  <Settings className="size-4" />
                  <span>
                    <Trans>Settings</Trans>
                  </span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>{/* Footer content can be added here */}</SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
