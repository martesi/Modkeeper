import { useState } from 'react'
import { FidelitySection } from '../../shared/components/fidelity-section'
import { FidelityInput } from '../../shared/components/fidelity-input'
import { FidelityButton } from '../../shared/components/fidelity-button'
import { renameLibrary } from '../../data/library-repository'
import { libraryText } from '../../i18n/library-text'
import type { LibrarySummary } from '../../data/redesign-types'

/**
 * Identity section (consolidated-spec.md §12.4): rename only — the name lives in the library's own
 * manifest, not App Config. Mount keyed by library id so the draft resets on tab switch.
 */
export function LibraryIdentitySection({
  library,
}: {
  library: LibrarySummary
}) {
  const [draft, setDraft] = useState(library.name)
  const [pending, setPending] = useState(false)
  const dirty = draft.trim() !== library.name && draft.trim() !== ''

  async function handleSave() {
    setPending(true)
    try {
      await renameLibrary({ libraryId: library.id, name: draft.trim() })
    } finally {
      setPending(false)
    }
  }

  return (
    <FidelitySection
      title={libraryText.identitySection()}
      description={
        library.sptVersion
          ? libraryText.sptVersion(library.sptVersion)
          : undefined
      }
    >
      <div className="flex items-center gap-2">
        <FidelityInput
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          aria-label={libraryText.nameLabel()}
        />
        <FidelityButton
          variant="secondary"
          disabled={!dirty}
          busy={pending}
          onClick={() => void handleSave()}
        >
          {libraryText.saveName()}
        </FidelityButton>
      </div>
    </FidelitySection>
  )
}
