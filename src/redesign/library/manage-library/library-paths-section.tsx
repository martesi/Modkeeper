import { toast } from 'sonner'
import { Copy, FolderOpen } from 'lucide-react'
import { FidelitySection } from '../../shared/components/fidelity-section'
import { FidelityIconButton } from '../../shared/components/fidelity-icon-button'
import { openGameRoot, openLibraryRoot } from '../../shared/utils/app-opener'
import { libraryText } from '../../i18n/library-text'
import type { LibrarySummary } from '../../data/redesign-types'

/**
 * Paths section (consolidated-spec.md §12.4): game root + library root, each with copy-to-clipboard
 * (with feedback) and open-in-explorer via the app-opener wrapper.
 */
export function LibraryPathsSection({ library }: { library: LibrarySummary }) {
  return (
    <FidelitySection title={libraryText.pathsSection()}>
      <PathRow
        label={libraryText.gameRootLabel()}
        path={library.gameRoot}
        onOpen={() => void openGameRoot(library)}
      />
      <PathRow
        label={libraryText.libraryRootLabel()}
        path={library.libraryRoot}
        onOpen={() => void openLibraryRoot(library)}
      />
    </FidelitySection>
  )
}

function PathRow({
  label,
  path,
  onOpen,
}: {
  label: string
  path: string
  onOpen: () => void
}) {
  async function handleCopy() {
    await navigator.clipboard.writeText(path)
    toast.success(libraryText.pathCopied())
  }

  return (
    <div className="flex items-center gap-2 rounded-[var(--mk-radius-control)] border border-[var(--mk-outline)] bg-[var(--mk-surface)] px-3 py-2">
      <div className="min-w-0 flex-1">
        <p className="text-xs text-[var(--mk-text-muted)]">{label}</p>
        <p className="truncate text-sm" title={path}>
          {path}
        </p>
      </div>
      <FidelityIconButton
        size="sm"
        aria-label={libraryText.copyPath()}
        title={libraryText.copyPath()}
        onClick={() => void handleCopy()}
      >
        <Copy />
      </FidelityIconButton>
      <FidelityIconButton
        size="sm"
        aria-label={libraryText.openInExplorer()}
        title={libraryText.openInExplorer()}
        onClick={onOpen}
      >
        <FolderOpen />
      </FidelityIconButton>
    </div>
  )
}
