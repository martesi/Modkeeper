import { LibraryEmptyCard } from './library-empty-card'
import { FidelityButton } from '../shared/components/fidelity-button'
import { libraryText } from '../i18n/library-text'

/**
 * No-active-library state (consolidated-spec.md §12.1): centered dashed card whose primary action
 * opens the Manage Library dialog. No toolbar renders in this state.
 */
export function LibraryActivateEmptyState({
  onManageLibraries,
}: {
  onManageLibraries: () => void
}) {
  return (
    <LibraryEmptyCard>
      <p className="text-sm text-[var(--mk-text-muted)]">
        {libraryText.subtitleEmpty()}
      </p>
      <FidelityButton onClick={onManageLibraries}>
        {libraryText.manageLibraries()}
      </FidelityButton>
    </LibraryEmptyCard>
  )
}
