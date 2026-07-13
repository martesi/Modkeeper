import { useState } from 'react'
import { useAtomValue } from 'jotai'
import * as DropdownMenu from '@radix-ui/react-dropdown-menu'
import { ChevronDown, Play, Square, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import { FidelityButton } from '../shared/components/fidelity-button'
import { ConfirmDialog } from '../shared/components/confirm-dialog'
import {
  activeLibraryIdAtom,
  libraryBusyAtom,
  selectedModIdsAtom,
} from '../state/library-state'
import { bulkUpdateMods } from '../data/library-repository'
import { libraryText } from '../i18n/library-text'
import { commonText } from '../i18n/common-text'

const itemClasses = cn(
  'flex cursor-pointer select-none items-center gap-2 rounded-[calc(var(--mk-radius-control)-0.375rem)]',
  'px-3 py-2 text-sm text-[var(--mk-text)] outline-none',
  'data-[highlighted]:bg-[var(--mk-state-hover)]',
)

/**
 * Bulk `ACTIONS [count]` menu (consolidated-spec.md §12.3): enable/disable/delete over the current
 * selection via `bulkUpdateMods` — plain awaited, local pending, delete behind a local confirmation.
 */
export function BulkActionsMenu() {
  const activeLibraryId = useAtomValue(activeLibraryIdAtom)
  const libraryBusy = useAtomValue(libraryBusyAtom)
  const selectedIds = useAtomValue(selectedModIdsAtom)
  const [pending, setPending] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)

  const count = selectedIds.size
  const disabled = count === 0 || libraryBusy || pending || !activeLibraryId

  async function run(action: 'enable' | 'disable' | 'delete') {
    if (!activeLibraryId) return
    setPending(true)
    try {
      await bulkUpdateMods({
        libraryId: activeLibraryId,
        modIds: [...selectedIds],
        action,
      })
    } finally {
      setPending(false)
      setConfirmDelete(false)
    }
  }

  return (
    <>
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <FidelityButton
            variant="secondary"
            disabled={disabled}
            busy={pending}
          >
            {libraryText.actions(count)}
            <ChevronDown />
          </FidelityButton>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            align="end"
            sideOffset={6}
            className={cn(
              'mk-glass-strong z-50 min-w-48 rounded-[var(--mk-radius-control)] border border-[var(--mk-outline)]',
              'bg-[var(--mk-surface-strong)] p-1.5 shadow-[var(--mk-shadow-panel)]',
            )}
          >
            <DropdownMenu.Item
              className={itemClasses}
              onSelect={() => void run('enable')}
            >
              <Play className="size-4" aria-hidden />
              {libraryText.enableSelected()}
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className={itemClasses}
              onSelect={() => void run('disable')}
            >
              <Square className="size-4" aria-hidden />
              {libraryText.disableSelected()}
            </DropdownMenu.Item>
            <DropdownMenu.Item
              className={cn(itemClasses, 'text-[var(--mk-danger)]')}
              onSelect={() => setConfirmDelete(true)}
            >
              <Trash2 className="size-4" aria-hidden />
              {libraryText.deleteSelected()}
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>

      <ConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={libraryText.deleteModsTitle(count)}
        description={libraryText.deleteModsDescription()}
        confirmLabel={libraryText.deleteConfirm()}
        cancelLabel={commonText.cancel()}
        destructive
        busy={pending}
        onConfirm={() => void run('delete')}
      />
    </>
  )
}
