/*
 * `.zip` import picker (consolidated-spec.md §12.2): opens the native archive picker and feeds the
 * selection into the fire-and-track install. The dialog itself already filters to `.zip`, so no
 * second filter pass is needed here — the drop zone is the surface that needs one.
 */
import { useCallback } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { useAtomValue } from 'jotai'
import { activeLibraryIdAtom } from '../../state/library-state'
import { installZipArchives } from '../../data/library-repository'
import { libraryText } from '../../i18n/library-text'

async function pickZipArchives(): Promise<string[]> {
  if (!isTauri()) {
    // MOCK-FALLBACK: no native picker in the browser prototype — pretend one archive was chosen so
    // the install flow stays exercisable end-to-end.
    console.info('[redesign] no native file picker, simulating a selection')
    return ['C:/downloads/Example Mod.zip']
  }
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selection = await open({
    multiple: true,
    filters: [{ name: libraryText.zipArchive(), extensions: ['zip'] }],
    title: libraryText.selectModArchives(),
  })
  if (!selection) return []
  return Array.isArray(selection) ? selection : [selection]
}

/** Returns a callback that picks `.zip` archives and installs them into the active library. */
export function useZipFilePicker(): () => Promise<void> {
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)

  return useCallback(async () => {
    if (!activeLibraryId) return
    const paths = await pickZipArchives()
    if (paths.length === 0) return
    await installZipArchives({ libraryId: activeLibraryId, paths })
  }, [activeLibraryId])
}
