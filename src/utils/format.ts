import { Mod } from '@gen/bindings'
import { tDivider } from './translation'

export function formatAuthor(mod: Mod) {
  if (!mod.manifest?.author) return
  if (Array.isArray(mod.manifest.author)) {
    return mod.manifest.author.join(tDivider())
  }
  return mod.manifest.author
}
