'use client'

import { useEffect } from 'react'
import { useAtomValue } from 'jotai'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { libraryAtom } from '@/store/library'
import { useMods } from '@/hooks/use-library-state'

/**
 * Component that handles file drop events
 * Listens for dropped files and passes them to addMods command
 */
export function FileDropHandler() {
  const library = useAtomValue(libraryAtom)
  const { addMods } = useMods()

  useEffect(() => {
    let unlistenPromise: Promise<() => void> | undefined

    try {
      const currentWindow = getCurrentWindow()
      unlistenPromise = currentWindow.onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
          const paths = event.payload.paths

          // Only process drops when a library is active
          if (!library) {
            console.warn('No active library, ignoring file drop')
            return
          }

          // Process the dropped files
          addMods(paths).catch((err) => {
            console.error('Failed to add mods from file drop:', err)
          })
        }
      })
    } catch (err) {
      console.error('Failed to setup file drop listener:', err)
    }

    return () => {
      if (unlistenPromise) {
        unlistenPromise
          .then((unlisten) => unlisten())
          .catch(() => {
            // Ignore errors during cleanup
          })
      }
    }
  }, [library, addMods])

  return null
}
