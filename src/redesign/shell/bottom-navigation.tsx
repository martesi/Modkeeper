/*
 * Bottom-center navigation dock (consolidated-spec.md §12): always centered, Home + Settings only.
 * Active state is route-driven via TanStack Router; stable dimensions per §11's guardrails so the
 * pill never shifts layout when the active tab changes.
 */
import { Link } from '@tanstack/react-router'
import { House, Settings } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { commonText } from '../i18n/common-text'

export function BottomNavigation() {
  return (
    <nav
      aria-label="Primary"
      className="fixed bottom-4 left-1/2 z-20 -translate-x-1/2"
    >
      <div className="mk-glass-strong flex items-center gap-1 rounded-full border border-[var(--mk-outline)] bg-[var(--mk-surface-strong)] p-1.5 shadow-[var(--mk-shadow-panel)]">
        <NavItem to="/library" icon={House} label={commonText.home()} />
        <NavItem to="/settings" icon={Settings} label={commonText.settings()} />
      </div>
    </nav>
  )
}

function NavItem({
  to,
  icon: Icon,
  label,
}: {
  to: string
  icon: LucideIcon
  label: string
}) {
  return (
    <Link
      to={to}
      className="mk-focus-ring flex h-10 items-center gap-2 rounded-full px-4 text-sm font-medium text-[var(--mk-text-muted)] transition-colors hover:bg-[var(--mk-state-hover)] hover:text-[var(--mk-text)]"
      activeProps={{
        className:
          'bg-[var(--mk-primary)] text-[var(--mk-on-primary)] hover:bg-[var(--mk-primary-hover)] hover:text-[var(--mk-on-primary)]',
        'aria-current': 'page',
      }}
    >
      <Icon className="size-4" aria-hidden />
      {label}
    </Link>
  )
}
