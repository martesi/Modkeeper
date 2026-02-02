import { useState } from 'react'
import { FileDropHandler } from '@/components/file-drop-handler'
import { LibraryInit } from '@/components/library-init'
import { SettingsInit } from '@/components/settings-init'
import { AppNavigation } from '@/components/app-navigation'
import { HeaderPortalContext } from '@/utils/header-portal-context'
import { createRootRoute, Outlet } from '@tanstack/react-router'

export const Route = createRootRoute({
  component: RootComponent,
})

function RootComponent () {
  const [container, setContainer] = useState<HTMLElement | null>(null)

  return (
    <>
      <LibraryInit />
      <SettingsInit />
      <FileDropHandler />
      <header className="flex h-16 shrink-0 items-center gap-2 transition-[width,height] ease-linear">
        <div className="flex items-center gap-2 px-4 w-full">
          <AppNavigation />
          <div className="w-full flex justify-end" ref={setContainer} />
        </div>
      </header>
      <HeaderPortalContext value={container}>
        <div className="flex flex-1 flex-col gap-4 p-4">
          <Outlet />
        </div>
      </HeaderPortalContext>
    </>
  )
}
