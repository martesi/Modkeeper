import { useAtomValue } from 'jotai'
import { RefreshCw } from 'lucide-react'
import { FidelityPanel } from '../shared/components/fidelity-panel'
import { FidelityButton } from '../shared/components/fidelity-button'
import { activeLibraryAtom, libraryBusyAtom } from '../state/library-state'
import { syncMods } from '../data/library-repository'
import { libraryText } from '../i18n/library-text'

/**
 * Library execution bar (consolidated-spec.md §12.3), walking-skeleton slice.
 *
 * Proves the FIRE-AND-TRACK repository shape against a real consumer: Sync calls `syncMods`, which
 * mints a taskId and registers a pending task, flipping "library busy". The button is highlighted
 * while `deployStale` (the explicit-deploy signal, C3) and shows the busy affordance until the
 * matching `sync_completed` event clears the task.
 */
export function LibraryExecutionBar() {
  const activeLibrary = useAtomValue(activeLibraryAtom)
  const libraryBusy = useAtomValue(libraryBusyAtom)

  if (!activeLibrary) return null

  return (
    <FidelityPanel
      radius="control"
      className="flex items-center justify-between gap-3 p-3"
    >
      <span className="text-sm text-[var(--mk-text-muted)]">
        {libraryBusy
          ? libraryText.syncing()
          : activeLibrary.deployStale
            ? libraryText.deployStale()
            : libraryText.deployUpToDate()}
      </span>
      <FidelityButton
        variant={activeLibrary.deployStale ? 'primary' : 'secondary'}
        busy={libraryBusy}
        onClick={() => void syncMods(activeLibrary.id)}
      >
        <RefreshCw />
        {libraryText.sync()}
      </FidelityButton>
    </FidelityPanel>
  )
}
