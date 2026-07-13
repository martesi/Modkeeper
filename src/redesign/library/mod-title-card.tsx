import { useState } from 'react'
import { useAtomValue } from 'jotai'
import { cn } from '@/lib/utils'
import { FidelityPanel } from '../shared/components/fidelity-panel'
import { ModCategoryIcon } from './mod-category-icon'
import { libraryBusyAtom } from '../state/library-state'
import { toggleModStatus } from '../data/library-repository'
import { libraryText } from '../i18n/library-text'
import type { ModSummary } from '../data/redesign-types'

/**
 * Title-only mod card (consolidated-spec.md §12.3), walking-skeleton slice.
 *
 * Proves the PLAIN repository shape against a real consumer: the toggle calls `toggleModStatus`
 * (awaited) with LOCAL per-click pending — set on click, cleared when the promise settles — so it
 * gives instant feedback even queued behind a heavy op. The card is also disabled while the library
 * is busy (a fire-and-track op in flight), the consumer side of the "library busy" contract.
 */
export function ModTitleCard({ mod }: { mod: ModSummary }) {
  const [pending, setPending] = useState(false)
  const libraryBusy = useAtomValue(libraryBusyAtom)
  const disabled = pending || libraryBusy

  async function handleToggle() {
    setPending(true)
    try {
      await toggleModStatus({
        libraryId: mod.libraryId,
        modId: mod.id,
        enabled: !mod.isEnabled,
      })
    } finally {
      setPending(false)
    }
  }

  return (
    <FidelityPanel
      radius="control"
      className={cn(
        'flex items-center gap-3 p-3 transition-opacity',
        mod.isEnabled
          ? 'border-[var(--mk-primary)] bg-[var(--mk-state-hover)]'
          : 'opacity-70'
      )}
    >
      <ModCategoryIcon type={mod.type} />
      <span className="min-w-0 flex-1 truncate text-sm font-medium" title={mod.name}>
        {mod.name}
      </span>
      <ModToggle
        checked={mod.isEnabled}
        disabled={disabled}
        pending={pending}
        label={mod.name}
        onToggle={handleToggle}
      />
    </FidelityPanel>
  )
}

function ModToggle({
  checked,
  disabled,
  pending,
  label,
  onToggle,
}: {
  checked: boolean
  disabled: boolean
  pending: boolean
  label: string
  onToggle: () => void
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={libraryText.enableMod(label)}
      aria-busy={pending || undefined}
      disabled={disabled}
      onClick={onToggle}
      className={cn(
        'mk-focus-ring relative h-6 w-11 shrink-0 rounded-full transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        checked ? 'bg-[var(--mk-primary)]' : 'bg-[var(--mk-outline)]'
      )}
    >
      <span
        className={cn(
          'absolute top-0.5 size-5 rounded-full bg-white shadow-sm transition-transform',
          checked ? 'translate-x-[1.375rem]' : 'translate-x-0.5'
        )}
      />
    </button>
  )
}
