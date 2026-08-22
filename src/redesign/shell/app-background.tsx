import { isTauri } from '@tauri-apps/api/core'

/**
 * Browser-prototype page surface (fix_plan_0.md §6): the reference's radial page gradient, for
 * environments with no OS surface (browser prototype, Storybook, and Linux). In Tauri on Windows
 * and macOS it renders NOTHING — the transparent window and native effect provide the background.
 */
export function AppBackground() {
  if (isTauri() && !navigator.userAgent.includes('Linux')) return null
  return (
    <div
      aria-hidden
      className="mk-page-gradient fixed inset-0 -z-10 transition-colors"
    />
  )
}
