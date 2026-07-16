import { isTauri } from '@tauri-apps/api/core'

/**
 * Browser-prototype page surface (fix_plan_0.md §6): the reference's radial page gradient, for
 * environments with no OS surface (browser prototype, Storybook). In Tauri it renders NOTHING —
 * the window is transparent and `apply_window_effect`'s mica IS the background; painting anything
 * here would block it.
 */
export function AppBackground() {
  if (isTauri()) return null
  return (
    <div
      aria-hidden
      className="mk-page-gradient fixed inset-0 -z-10 transition-colors"
    />
  )
}
