import { LibraryEmptyCard } from './library-empty-card'
import { useZipFilePicker } from '../shared/hooks/use-zip-file-picker'
import { libraryText } from '../i18n/library-text'

/**
 * Active-library-but-no-mods state (reference Modkeeper.dc.html drop zone): the whole card opens a
 * `.zip`-filtered picker; the global window drop zone (initializers) covers drag-and-drop.
 * (The reference badge lists .rar/.7z too — the backend import path only accepts .zip.)
 */
export function LibraryDropEmptyState() {
  const pickAndInstall = useZipFilePicker()

  return (
    <LibraryEmptyCard
      onClick={() => void pickAndInstall()}
      badge={libraryText.dropSupported()}
    >
      <p className="font-heading text-base font-bold text-foreground">
        {libraryText.dropTitle()}
      </p>
      <p className="text-xs font-bold text-primary underline">
        {libraryText.dropBrowse()}
      </p>
    </LibraryEmptyCard>
  )
}
