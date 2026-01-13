import { ALibrarySwitch } from '@/store/library'
import { createSetter } from '@/utils/function'
import { commands } from '@gen/bindings'
import { useSetAtom } from 'jotai'

export function useLibrarySwitch() {
  const set = useSetAtom(ALibrarySwitch)

  const open = createSetter(commands.openLibrary, set)
  const create = createSetter(commands.createLibrary, set)
  const rename = createSetter(commands.renameLibrary, set)
  const close = createSetter(commands.closeLibrary, set)
  const remove = createSetter(commands.removeLibrary, set)

  return {
    open,
    create,
    rename,
    close,
    remove,
  }
}
