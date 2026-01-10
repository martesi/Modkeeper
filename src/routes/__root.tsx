import { createRootRoute, Outlet } from '@tanstack/react-router'
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from '@comps/sidebar.tsx'
import { AppSidebar } from '@comps/app-sidebar.tsx'
import { Separator } from '@comps/separator.tsx'
import { LibraryInit } from '@/components/library-init'
import { FileDropHandler } from '@/components/file-drop-handler'
import { BreadcrumbNav } from '@/components/breadcrumb-nav'

export const Route = createRootRoute({
  component: RootComponent,
})

function RootComponent() {
  return (
    <SidebarProvider>
      <LibraryInit />
      <FileDropHandler />
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-12">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator
              orientation="vertical"
              className="mr-2 data-[orientation=vertical]:h-4"
            />
            <BreadcrumbNav />
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <Outlet />
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
