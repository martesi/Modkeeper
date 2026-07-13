import { LibraryEmptyCard } from './library-empty-card'
import { useZipFilePicker } from '../shared/hooks/use-zip-file-picker'
import { libraryText } from '../i18n/library-text'

/**
 * Active-library-but-no-mods state (consolidated-spec.md §12.2): the whole card opens a
 * `.zip`-filtered picker; the global window drop zone (initializers) covers drag-and-drop.
 */
export function LibraryDropEmptyState() {
  const pickAndInstall = useZipFilePicker()

  return (
    <LibraryEmptyCard onClick={() => void pickAndInstall()} badge=".zip">
      <p className="text-sm text-[var(--mk-text-muted)]">
        {libraryText.dropZipPrompt()}
      </p>
    </LibraryEmptyCard>
  )
}
