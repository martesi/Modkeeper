import { useAtomValue } from 'jotai'
import { visibleModsAtom } from '../state/library-state'
import { ModTitleCard } from './mod-title-card'
import { libraryText } from '../i18n/library-text'

/**
 * Title-only mod grid (consolidated-spec.md §12.3): 1/2/3 columns by width, stable card heights.
 * Renders the derived visible list — filter/search/sort live in the state layer.
 */
export function ModGrid() {
  const mods = useAtomValue(visibleModsAtom)

  if (mods.length === 0) {
    return (
      <p className="py-12 text-center text-sm text-[var(--mk-text-muted)]">
        {libraryText.noVisibleMods()}
      </p>
    )
  }

  return (
    <div className="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
      {mods.map((mod) => (
        <ModTitleCard key={mod.id} mod={mod} />
      ))}
    </div>
  )
}
