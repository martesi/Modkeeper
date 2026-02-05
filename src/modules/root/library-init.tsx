'use client'

import { ALibrarySwitch } from '@/store/library'
import { createSetter } from '@/utils/function'
import { commands } from '@gen/bindings'
import { useSetAtom } from 'jotai'
import { useEffect } from 'react'

/**
 * Component that initializes the library on app start
 * Calls the init command which shows the window and returns synced state
 */
export function LibraryInit() {
  const set = useSetAtom(ALibrarySwitch)

  const init = createSetter(commands.init, set)

  useEffect(() => {
    init()
  }, [init])

  return null
}
