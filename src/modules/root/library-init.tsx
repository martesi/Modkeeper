'use client'

import { ALibrarySwitch } from '@/store/library'
import { createSetter } from '@/utils/function'
import { themeToIsDark } from '@/utils/theme'
import { commands } from '@gen/bindings'
import { useSetAtom } from 'jotai'
import { useTheme } from 'next-themes'
import { useEffect } from 'react'

/**
 * Component that initializes the library on app start
 * Calls the init command which shows the window and returns synced state
 * Also applies window effects based on the current theme
 */
export function LibraryInit() {
  const set = useSetAtom(ALibrarySwitch)
  const { resolvedTheme } = useTheme()

  const init = createSetter(commands.init, set)

  // Apply window effect when theme changes
  useEffect(() => {
    commands.applyWindowEffect(themeToIsDark(resolvedTheme) ?? null)
  }, [resolvedTheme])

  // Initialize the library on mount
  useEffect(() => {
    init()
  }, [init])

  return null
}
