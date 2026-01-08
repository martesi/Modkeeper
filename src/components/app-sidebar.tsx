'use client'

import * as React from 'react'

import { InstanceSwitcher } from '@comps/instance-switcher.tsx'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@/components/ui/sidebar'

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <InstanceSwitcher />
      </SidebarHeader>
      <SidebarContent>
        {/* Navigation items can be added here */}
      </SidebarContent>
      <SidebarFooter>
        {/* Footer content can be added here */}
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  )
}
