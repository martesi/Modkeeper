import { useAtomValue } from 'jotai'
import { modListAtom } from '../state/library-state'
import { LibraryExecutionBar } from './library-execution-bar'
import { LibraryDropEmptyState } from './library-drop-empty-state'
import { ModGridToolbar } from './mod-grid-toolbar'
import { ModGrid } from './mod-grid'

/**
 * Active-library content (consolidated-spec.md §12.2/§12.3): execution bar on top, then either the
 * `.zip` drop empty state (zero mods) or the toolbar + title-only grid.
 */
export function LibraryContent({
  onManageLibraries,
}: {
  onManageLibraries: () => void
}) {
  const mods = useAtomValue(modListAtom)

  return (
    <div className="flex flex-col gap-4">
      <LibraryExecutionBar onManageLibraries={onManageLibraries} />
      {mods.length === 0 ? (
        <LibraryDropEmptyState />
      ) : (
        <>
          <ModGridToolbar />
          <ModGrid />
        </>
      )}
    </div>
  )
}
