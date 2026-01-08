'use client'

import { useEffect } from 'react'
import { useSetAtom } from 'jotai'
import { fetchLibraryAction } from '@/store/library-actions'

/**
 * Component that initializes the library on app start
 */
export function LibraryInit() {
  const fetchLibrary = useSetAtom(fetchLibraryAction)

  useEffect(() => {
    fetchLibrary().catch(() => {
      // Silently fail - no library is loaded yet
    })
  }, [fetchLibrary])

  return null
}
