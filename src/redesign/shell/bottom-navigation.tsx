/*
 * Bottom-center navigation dock (reference Modkeeper.dc.html): glass pill, text-only Home +
 * Settings buttons; the active one is a filled primary pill. Route-driven via TanStack Router;
 * stable dimensions so the pill never shifts layout when the active tab changes.
 */
import { Link } from '@tanstack/react-router'
import { commonText } from '../i18n/common-text'

export function BottomNavigation() {
  return (
    <nav
      aria-label="Primary"
      className="relative z-10 flex shrink-0 items-center justify-center p-2.5"
    >
      <div className="mk-glass-standard flex items-center gap-1 rounded-full border border-border bg-card p-1.5 shadow-2xl">
        <NavItem to="/library" label={commonText.home()} />
        <NavItem to="/settings" label={commonText.settings()} />
      </div>
    </nav>
  )
}

function NavItem({ to, label }: { to: string; label: string }) {
  return (
    <Link
      to={to}
      className="flex h-10 items-center rounded-full px-5 text-sm font-bold text-muted-foreground outline-none transition-colors hover:bg-accent hover:text-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50"
      activeProps={{
        className:
          'bg-primary text-primary-foreground hover:bg-primary/90 hover:text-primary-foreground',
        'aria-current': 'page',
      }}
    >
      {label}
    </Link>
  )
}
