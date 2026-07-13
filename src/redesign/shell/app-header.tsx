/*
 * Sticky shell header (consolidated-spec.md §12): page title + subtitle only — no native window
 * controls, which stay on the Tauri title bar. Also hosts the library-busy affordance, the shell
 * side of the fire-and-track busy contract proven in 8.2.
 */
import { useAtomValue } from 'jotai'
import { Loader2 } from 'lucide-react'
import { pageTitleAtom } from './page-title'
import { libraryBusyAtom } from '../state/library-state'
import { commonText } from '../i18n/common-text'

export function AppHeader() {
  const { title, subtitle } = useAtomValue(pageTitleAtom)

  return (
    <header className="mk-glass-strong sticky top-0 z-10 border-b border-[var(--mk-outline)] bg-[var(--mk-surface-strong)]">
      <div className="mx-auto flex min-h-14 w-[min(100%,var(--mk-content-max))] items-center justify-between gap-4 px-4 py-2">
        <div className="min-w-0">
          <h1 className="truncate text-base font-semibold leading-tight text-[var(--mk-text)]">
            {title}
          </h1>
          {subtitle && (
            <p className="truncate text-xs text-[var(--mk-text-muted)]">
              {subtitle}
            </p>
          )}
        </div>
        <BusyIndicator />
      </div>
    </header>
  )
}

function BusyIndicator() {
  const busy = useAtomValue(libraryBusyAtom)
  if (!busy) return null
  return (
    <span className="inline-flex shrink-0 items-center gap-1.5 text-xs text-[var(--mk-text-muted)]">
      <Loader2 className="size-3.5 animate-spin" aria-hidden />
      {commonText.libraryBusy()}
    </span>
  )
}
