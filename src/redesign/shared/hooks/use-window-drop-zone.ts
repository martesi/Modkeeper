/*
 * Global window drop zone (consolidated-spec.md §12): the whole window accepts `.zip` drops for the
 * active library. Non-zip files are rejected with a toast and NO backend call (§12.2). Mounted once
 * by the initializers; in the browser prototype there is no window drop source, so this is inert.
 */
import { useEffect } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { useAtomValue } from 'jotai'
import { toast } from 'sonner'
import { activeLibraryIdAtom } from '../../state/library-state'
import { installZipArchives } from '../../data/library-repository'
import { splitZipPaths } from '../utils/file-filters'
import { libraryText } from '../../i18n/library-text'

export function useWindowDropZone(): void {
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)

  useEffect(() => {
    if (!isTauri() || !activeLibraryId) return
    let disposed = false
    const unlistenPromise = import('@tauri-apps/api/window').then(
      ({ getCurrentWindow }) =>
        getCurrentWindow().onDragDropEvent((event) => {
          if (event.payload.type !== 'drop') return
          const { zips, rejected } = splitZipPaths(event.payload.paths)
          if (rejected.length > 0) toast.error(libraryText.nonZipRejected())
          if (zips.length > 0) {
            void installZipArchives({ libraryId: activeLibraryId, paths: zips })
          }
        }),
    )
    return () => {
      disposed = true
      void unlistenPromise.then((unlisten) => {
        if (disposed) unlisten()
      })
    }
  }, [activeLibraryId])
}
