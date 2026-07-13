import type { ReactNode } from 'react'
import '../styles/fidelity.css'
import { RedesignErrorBoundary } from './redesign-error-boundary'
import { RedesignInitializers } from './redesign-initializers'
import { DesktopShell } from '../shell/desktop-shell'

/**
 * Root of the redesign tree (consolidated-spec.md §8.3).
 *
 * Pulls in the fidelity design tokens, catches render crashes at the top (§13), mounts startup
 * wiring, and frames the routed content in the shell. Mounted by the `__root.tsx` route adapter
 * with the router's Outlet as children (unless the legacy-UI toggle is on, §10a).
 */
export function RedesignRoot({ children }: { children: ReactNode }) {
  return (
    <RedesignErrorBoundary>
      <RedesignInitializers />
      <DesktopShell>{children}</DesktopShell>
    </RedesignErrorBoundary>
  )
}
