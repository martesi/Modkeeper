import { useState } from 'react'
import { useAtomValue } from 'jotai'
import * as Dialog from '@radix-ui/react-dialog'
import { isTauri } from '@tauri-apps/api/core'
import { Plus, RefreshCw, Trash2, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { FidelityButton } from '../../shared/components/fidelity-button'
import { FidelityIconButton } from '../../shared/components/fidelity-icon-button'
import {
  activeLibraryIdAtom,
  libraryListAtom,
  pendingTasksAtom,
} from '../../state/library-state'
import {
  activateLibrary,
  createLibrary,
  deleteLibrary,
  rebuildLibraryCache,
} from '../../data/library-repository'
import { isLibrarySummary } from '../../data/redesign-types'
import type {
  LibraryEntry,
  LibraryStub,
  LibrarySummary,
} from '../../data/redesign-types'
import { libraryText } from '../../i18n/library-text'
import { commonText } from '../../i18n/common-text'
import { LibraryIdentitySection } from './library-identity-section'
import { LibraryPathsSection } from './library-paths-section'
import { DeleteLibraryConfirmDialog } from './delete-library-confirm-dialog'

/**
 * Manage Library dialog (consolidated-spec.md §12.4): strong-glass body, top tabs across every
 * registered entry (readable summaries AND path-only stubs, C13) plus a dashed plus tab that picks
 * a game folder and creates+selects. The tools section is NOT rendered — the tool registry is
 * deferred (§3). Rebuild Cache is fire-and-track and only valid for the ACTIVE library (§7e
 * rejects non-active libraryIds), so it is disabled elsewhere with a hint.
 */
export function ManageLibraryDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const libraries = useAtomValue(libraryListAtom)
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)
  const [selectedKey, setSelectedKey] = useState<string | null>(null)

  // Selection is resolved against the CURRENT list every render, so a deleted entry or a fresh
  // workspace can never leave a dangling tab selected.
  const selected =
    libraries.find((entry) => entryKey(entry) === selectedKey) ??
    libraries.find(
      (entry) => isLibrarySummary(entry) && entry.id === activeLibraryId,
    ) ??
    libraries[0] ??
    null

  async function handleAddLibrary() {
    const gameRoot = await pickGameRootFolder()
    if (!gameRoot) return
    const workspace = await createLibrary({ gameRoot })
    // create+select (§12.4): the created library becomes the active one.
    if (workspace.activeLibraryId) setSelectedKey(workspace.activeLibraryId)
  }

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            'fixed inset-0 z-50 bg-black/40 backdrop-blur-sm',
            'data-[state=open]:animate-in data-[state=open]:fade-in-0',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0',
          )}
        />
        <Dialog.Content
          className={cn(
            'mk-glass-strong fixed left-1/2 top-1/2 z-50 w-[min(40rem,calc(100vw-2rem))]',
            '-translate-x-1/2 -translate-y-1/2 rounded-[var(--mk-radius-dialog)] p-6',
            'border border-[var(--mk-outline)] bg-[var(--mk-surface-strong)] text-[var(--mk-text)]',
            'shadow-[var(--mk-shadow-panel)]',
            'data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95',
          )}
        >
          <div className="flex flex-col gap-5">
            <div className="flex items-center justify-between gap-3">
              <Dialog.Title className="text-lg font-semibold">
                {libraryText.manageDialogTitle()}
              </Dialog.Title>
              <Dialog.Close asChild>
                <FidelityIconButton size="sm" aria-label={commonText.close()}>
                  <X />
                </FidelityIconButton>
              </Dialog.Close>
            </div>

            <div className="flex flex-wrap items-center gap-2" role="tablist">
              {libraries.map((entry) => {
                const key = entryKey(entry)
                const isSelected =
                  selected !== null && key === entryKey(selected)
                return (
                  <button
                    key={key}
                    type="button"
                    role="tab"
                    aria-selected={isSelected}
                    onClick={() => setSelectedKey(key)}
                    className={cn(
                      'mk-focus-ring h-9 max-w-48 truncate rounded-[var(--mk-radius-control)] border px-3 text-sm transition-colors',
                      isSelected
                        ? 'border-[var(--mk-primary)] bg-[var(--mk-state-hover)] font-medium'
                        : 'border-[var(--mk-outline)] bg-[var(--mk-surface-container)] hover:bg-[var(--mk-surface-container-hover)]',
                    )}
                  >
                    {isLibrarySummary(entry)
                      ? entry.name
                      : libraryText.unreadableLibrary()}
                  </button>
                )
              })}
              <button
                type="button"
                onClick={() => void handleAddLibrary()}
                aria-label={libraryText.addLibraryTab()}
                title={libraryText.addLibraryTab()}
                className={cn(
                  'mk-focus-ring inline-flex h-9 items-center justify-center rounded-[var(--mk-radius-control)]',
                  'border-2 border-dashed border-[var(--mk-outline)] px-3 text-[var(--mk-text-muted)] transition-colors',
                  'hover:border-[var(--mk-primary)] hover:text-[var(--mk-primary)]',
                )}
              >
                <Plus className="size-4" aria-hidden />
              </button>
            </div>

            {selected === null ? null : isLibrarySummary(selected) ? (
              <LibraryDetails key={selected.id} library={selected} />
            ) : (
              <StubDetails stub={selected} />
            )}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

function LibraryDetails({ library }: { library: LibrarySummary }) {
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)
  const pendingTasks = useAtomValue(pendingTasksAtom)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [activating, setActivating] = useState(false)

  const isActive = library.id === activeLibraryId
  const busy = pendingTasks.some((task) => task.libraryId === library.id)

  async function handleActivate() {
    setActivating(true)
    try {
      await activateLibrary(library.id)
    } finally {
      setActivating(false)
    }
  }

  return (
    <>
      <LibraryIdentitySection library={library} />
      <LibraryPathsSection library={library} />
      {/* Tools section deliberately absent — tool registry deferred (§3). */}

      <div className="flex flex-wrap items-center justify-between gap-2 border-t border-[var(--mk-outline)] pt-4">
        <div className="flex items-center gap-2">
          <FidelityButton
            variant="secondary"
            disabled={!isActive}
            busy={busy}
            title={isActive ? undefined : libraryText.rebuildNeedsActive()}
            onClick={() => void rebuildLibraryCache(library.id)}
          >
            <RefreshCw />
            {libraryText.rebuildCache()}
          </FidelityButton>
          <FidelityButton
            variant="destructive"
            onClick={() => setConfirmDelete(true)}
          >
            <Trash2 />
            {libraryText.deleteLibrary()}
          </FidelityButton>
        </div>
        <FidelityButton
          disabled={isActive}
          busy={activating}
          onClick={() => void handleActivate()}
        >
          {isActive ? libraryText.activated() : libraryText.activate()}
        </FidelityButton>
      </div>

      <DeleteLibraryConfirmDialog
        library={library}
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        onDeleted={() => {}}
      />
    </>
  )
}

/** Registered-but-unreadable entry (C13): bare path, remove-only. */
function StubDetails({ stub }: { stub: LibraryStub }) {
  const [pending, setPending] = useState(false)

  async function handleRemove() {
    setPending(true)
    try {
      // The stub has no id, only its registered path — the path is the only handle the frontend
      // can pass for the entry-only (never destructive) removal.
      await deleteLibrary({ libraryId: stub.path, deleteFiles: false })
    } finally {
      setPending(false)
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="rounded-[var(--mk-radius-control)] border border-[var(--mk-outline)] bg-[var(--mk-surface)] px-3 py-2">
        <p className="text-xs text-[var(--mk-text-muted)]">
          {libraryText.unreadableLibraryHint()}
        </p>
        <p className="truncate text-sm" title={stub.path}>
          {stub.path}
        </p>
      </div>
      <div className="flex justify-end">
        <FidelityButton
          variant="destructive"
          busy={pending}
          onClick={() => void handleRemove()}
        >
          <Trash2 />
          {libraryText.removeEntry()}
        </FidelityButton>
      </div>
    </div>
  )
}

function entryKey(entry: LibraryEntry): string {
  return isLibrarySummary(entry) ? entry.id : entry.path
}

async function pickGameRootFolder(): Promise<string | null> {
  if (!isTauri()) {
    // MOCK-FALLBACK: no native folder picker in the browser prototype — simulate a selection so
    // the create flow stays exercisable end-to-end.
    console.info('[redesign] no native folder picker, simulating a selection')
    return `C:/SPT-instances/instance-${Date.now() % 1000}`
  }
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selection = await open({
    directory: true,
    multiple: false,
    title: libraryText.selectGameRoot(),
  })
  return typeof selection === 'string' ? selection : null
}
