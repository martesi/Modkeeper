/*
 * File-explorer opener (consolidated-spec.md §10): UI never imports the opener plugin directly —
 * it calls these wrappers, which no-op with a console note in the browser prototype.
 */
import { isTauri } from '@tauri-apps/api/core'
import type { LibrarySummary, ModSummary } from '../../data/redesign-types'

async function openPath(path: string): Promise<void> {
  if (!isTauri()) {
    // MOCK-FALLBACK: no runtime to open a file explorer in the browser prototype.
    console.info('[redesign] would open in file explorer:', path)
    return
  }
  const { openPath: open } = await import('@tauri-apps/plugin-opener')
  await open(path)
}

export function openGameRoot(library: LibrarySummary): Promise<void> {
  return openPath(library.gameRoot)
}

export function openLibraryRoot(library: LibrarySummary): Promise<void> {
  return openPath(library.libraryRoot)
}

export function openModSource(mod: ModSummary): Promise<void> {
  if (!mod.sourcePath) return Promise.resolve()
  return openPath(mod.sourcePath)
}
