import { LibraryEmptyCard } from './library-empty-card'
import { FidelityButton } from '../shared/components/fidelity-button'
import { libraryText } from '../i18n/library-text'

/**
 * No-active-library state (reference Modkeeper.dc.html "Activate a library" card): dashed card
 * whose action opens the Manage Library dialog. No toolbar renders in this state.
 */
export function LibraryActivateEmptyState({
  onManageLibraries,
}: {
  onManageLibraries: () => void
}) {
  return (
    <LibraryEmptyCard>
      <p className="font-heading text-base font-bold text-muted-foreground">
        {libraryText.emptyActivateTitle()}
      </p>
      <p className="max-w-md text-xs text-muted-foreground">
        {libraryText.emptyActivateBody()}
      </p>
      <FidelityButton
        className="mt-1.5 rounded-full text-xs font-extrabold uppercase tracking-[0.05em]"
        onClick={onManageLibraries}
      >
        {libraryText.manageLibraries()}
      </FidelityButton>
    </LibraryEmptyCard>
  )
}
