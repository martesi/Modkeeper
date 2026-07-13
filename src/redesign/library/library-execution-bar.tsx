import { useAtomValue } from 'jotai'
import { FolderCog, RefreshCw, Upload } from 'lucide-react'
import { FidelityPanel } from '../shared/components/fidelity-panel'
import { FidelityButton } from '../shared/components/fidelity-button'
import { FidelityIconButton } from '../shared/components/fidelity-icon-button'
import { activeLibraryAtom, libraryBusyAtom } from '../state/library-state'
import { syncMods } from '../data/library-repository'
import { useZipFilePicker } from '../shared/hooks/use-zip-file-picker'
import { libraryText } from '../i18n/library-text'

/**
 * Library execution bar (consolidated-spec.md §12.3): deploy status + the actions that operate on
 * the library as a whole — import archives, manage libraries, and the explicit Sync step (C3),
 * highlighted while `deployStale`. Sync is FIRE-AND-TRACK: it flips "library busy" until the
 * matching `sync_completed` event clears the task.
 */
export function LibraryExecutionBar({
  onManageLibraries,
}: {
  onManageLibraries: () => void
}) {
  const activeLibrary = useAtomValue(activeLibraryAtom)
  const libraryBusy = useAtomValue(libraryBusyAtom)
  const pickAndInstall = useZipFilePicker()

  if (!activeLibrary) return null

  return (
    <FidelityPanel
      radius="control"
      className="flex flex-wrap items-center justify-between gap-3 p-3"
    >
      <span className="text-sm text-[var(--mk-text-muted)]">
        {libraryBusy
          ? libraryText.syncing()
          : activeLibrary.deployStale
            ? libraryText.deployStale()
            : libraryText.deployUpToDate()}
      </span>
      <div className="flex items-center gap-2">
        <FidelityIconButton
          variant="secondary"
          aria-label={libraryText.manageLibraries()}
          title={libraryText.manageLibraries()}
          onClick={onManageLibraries}
        >
          <FolderCog />
        </FidelityIconButton>
        <FidelityButton
          variant="secondary"
          disabled={libraryBusy}
          onClick={() => void pickAndInstall()}
        >
          <Upload />
          {libraryText.addMods()}
        </FidelityButton>
        <FidelityButton
          variant={activeLibrary.deployStale ? 'primary' : 'secondary'}
          busy={libraryBusy}
          onClick={() => void syncMods(activeLibrary.id)}
        >
          <RefreshCw />
          {libraryText.sync()}
        </FidelityButton>
      </div>
    </FidelityPanel>
  )
}
