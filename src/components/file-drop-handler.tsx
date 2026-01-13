'use client'

import { useEffect } from 'react'
import { useAtomValue } from 'jotai'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ALibraryActive } from '@/store/library'
import { useLibrary } from '@/hooks/use-library'
import { getUnknownModName } from '@/utils/translation'

/**
 * Component that handles file drop events
 * Listens for dropped files and passes them to addMods command
 */
export function FileDropHandler() {
  const library = useAtomValue(ALibraryActive)
  const { add } = useLibrary()

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
          add(paths, getUnknownModName()).catch((err) => {
            console.error('Failed to add mods from file drop:', err)
          })
        }
      })
    } catch (err) {
      console.error('Failed to setup file drop listener:', err)
    }

    return () => {
      unlistenPromise?.then((unlisten) => unlisten())
    }
  }, [library, add])

  return null
}
