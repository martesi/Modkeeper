/*
 * Desktop shell (consolidated-spec.md §12): owns the full app frame below the native title bar —
 * background, sticky header, scrollable centered main, bottom-center nav dock. Toast and dialog
 * portals render to <body> (sonner Toaster is mounted app-wide in main.tsx; Radix dialogs portal
 * themselves), so the shell doesn't re-mount them.
 */
import type { ReactNode } from 'react'
import { AppBackground } from './app-background'
import { AppHeader } from './app-header'
import { BottomNavigation } from './bottom-navigation'

export function DesktopShell({ children }: { children: ReactNode }) {
  return (
    <div className="mk-scrollbar min-h-screen w-full text-[var(--mk-text)]">
      <AppBackground />
      <AppHeader />
      {/* Bottom padding keeps content clear of the fixed nav dock. */}
      <main className="mx-auto w-[min(100%,var(--mk-content-max))] px-4 pb-28 pt-8">
        {children}
      </main>
      <BottomNavigation />
    </div>
  )
}
