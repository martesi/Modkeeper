import { ALibraryActive } from '@/store/library'
import { createSetter } from '@/utils/function'
import { commands } from '@gen/bindings'
import { useSetAtom } from 'jotai'

export function useLibrary() {
  const set = useSetAtom(ALibraryActive)

  const add = createSetter(commands.addMods, set)
  const remove = createSetter(commands.removeMods, set)
  const sync = createSetter(commands.syncMods, set)
  const toggle = createSetter(commands.toggleMod, set)

  return {
    add,
    remove,
    sync,
    toggle,
  }
}
