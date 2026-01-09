'use client'

import { useEffect } from 'react'
import { useSetAtom } from 'jotai'
import { initAction } from '@/store/library-actions'

/**
 * Component that initializes the library on app start
 * Calls the init command which shows the window and returns synced state
 */
export function LibraryInit() {
  const init = useSetAtom(initAction)

  useEffect(() => {
    init().catch(() => {
      // Silently fail - no library is loaded yet
    })
  }, [init])

  return null
}
