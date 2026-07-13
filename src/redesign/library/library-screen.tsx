import { useState } from 'react'
import { useAtomValue } from 'jotai'
import { PageTitle } from '../shell/page-title'
import { activeLibraryAtom } from '../state/library-state'
import { libraryText } from '../i18n/library-text'
import { LibraryActivateEmptyState } from './library-activate-empty-state'
import { LibraryContent } from './library-content'
import { ManageLibraryDialog } from './manage-library/manage-library-dialog'

/**
 * Library screen (consolidated-spec.md §12.1–§12.3): branches between the activate empty state
 * (no active library — no toolbar renders) and the working content. Owns the Manage Library
 * dialog's open state, reachable from both branches.
 */
export function LibraryScreen() {
  const activeLibrary = useAtomValue(activeLibraryAtom)
  const [manageOpen, setManageOpen] = useState(false)

  return (
    <div className="flex flex-col gap-6">
      <PageTitle
        title={libraryText.title()}
        subtitle={activeLibrary?.name ?? libraryText.subtitleEmpty()}
      />

      {activeLibrary ? (
        <LibraryContent onManageLibraries={() => setManageOpen(true)} />
      ) : (
        <LibraryActivateEmptyState
          onManageLibraries={() => setManageOpen(true)}
        />
      )}

      <ManageLibraryDialog open={manageOpen} onOpenChange={setManageOpen} />
    </div>
  )
}
