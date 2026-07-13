import { useAtom, useAtomValue } from 'jotai'
import { ArrowDownAZ, ArrowUpZA, Search } from 'lucide-react'
import { cn } from '@/lib/utils'
import { FidelityCheckbox } from '../shared/components/fidelity-checkbox'
import { FidelityIconButton } from '../shared/components/fidelity-icon-button'
import { FidelityInput } from '../shared/components/fidelity-input'
import { BulkActionsMenu } from './bulk-actions-menu'
import {
  libraryBusyAtom,
  librarySearchAtom,
  librarySortAtom,
  libraryTypeFilterAtom,
  selectedModIdsAtom,
  visibleModsAtom,
  type LibraryTypeFilter,
} from '../state/library-state'
import { libraryText } from '../i18n/library-text'

/**
 * Grid toolbar (consolidated-spec.md §12.3): Select All (visible only), sort-by-name, type filter,
 * bulk ACTIONS [count], search. Bulk-affecting controls are disabled while the library is busy so a
 * queued action reads as "library busy," not a hang; search/sort/filter stay live — they are local.
 */
export function ModGridToolbar() {
  const libraryBusy = useAtomValue(libraryBusyAtom)
  const visibleMods = useAtomValue(visibleModsAtom)
  const [selectedIds, setSelectedIds] = useAtom(selectedModIdsAtom)
  const [search, setSearch] = useAtom(librarySearchAtom)
  const [sort, setSort] = useAtom(librarySortAtom)
  const [typeFilter, setTypeFilter] = useAtom(libraryTypeFilterAtom)

  const visibleSelectedCount = visibleMods.filter((mod) =>
    selectedIds.has(mod.id),
  ).length
  const allVisibleSelected =
    visibleMods.length > 0 && visibleSelectedCount === visibleMods.length

  // Select All operates on the VISIBLE set only (§12.3); off-screen selections are untouched.
  function handleSelectAll(checked: boolean) {
    const next = new Set(selectedIds)
    for (const mod of visibleMods) {
      if (checked) next.add(mod.id)
      else next.delete(mod.id)
    }
    setSelectedIds(next)
  }

  const filterOptions: { value: LibraryTypeFilter; label: string }[] = [
    { value: 'all', label: libraryText.filterAll() },
    { value: 'client', label: libraryText.typeClient() },
    { value: 'server', label: libraryText.typeServer() },
    { value: 'both', label: libraryText.typeBoth() },
    { value: 'unknown', label: libraryText.typeUnknown() },
  ]

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="inline-flex size-10 shrink-0 items-center justify-center">
        <FidelityCheckbox
          checked={allVisibleSelected}
          indeterminate={!allVisibleSelected && visibleSelectedCount > 0}
          onCheckedChange={handleSelectAll}
          disabled={libraryBusy || visibleMods.length === 0}
          aria-label={libraryText.selectAllVisible()}
        />
      </span>

      <FidelityIconButton
        variant="secondary"
        aria-label={
          sort === 'name-asc'
            ? libraryText.sortNameAsc()
            : libraryText.sortNameDesc()
        }
        title={
          sort === 'name-asc'
            ? libraryText.sortNameAsc()
            : libraryText.sortNameDesc()
        }
        onClick={() => setSort(sort === 'name-asc' ? 'name-desc' : 'name-asc')}
      >
        {sort === 'name-asc' ? <ArrowDownAZ /> : <ArrowUpZA />}
      </FidelityIconButton>

      <select
        value={typeFilter}
        onChange={(event) =>
          setTypeFilter(event.target.value as LibraryTypeFilter)
        }
        aria-label={libraryText.filterLabel()}
        className={cn(
          'mk-focus-ring h-10 shrink-0 rounded-[var(--mk-radius-control)] px-3 text-sm',
          'border border-[var(--mk-outline)] bg-[var(--mk-surface)] text-[var(--mk-text)]',
        )}
      >
        {filterOptions.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>

      <BulkActionsMenu />

      <div className="relative min-w-40 flex-1">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-[var(--mk-text-muted)]"
          aria-hidden
        />
        <FidelityInput
          type="search"
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder={libraryText.searchPlaceholder()}
          aria-label={libraryText.searchLabel()}
          className="pl-9"
        />
      </div>
    </div>
  )
}
