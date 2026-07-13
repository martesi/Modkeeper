import type { ReactNode } from 'react'
import { CloudUpload } from 'lucide-react'
import { cn } from '@/lib/utils'

/**
 * The shared 16:9 dashed-boundary empty-state card (consolidated-spec.md §12.1/§12.2): cloud-upload
 * icon, caller-supplied prompt/action content, optional corner badge. When `onClick` is given the
 * whole card is one big button (the §12.2 drop state).
 */
export function LibraryEmptyCard({
  onClick,
  badge,
  children,
}: {
  onClick?: () => void
  badge?: ReactNode
  children: ReactNode
}) {
  const Tag = onClick ? 'button' : 'div'
  return (
    <Tag
      type={onClick ? 'button' : undefined}
      onClick={onClick}
      className={cn(
        'relative mx-auto flex aspect-video w-full max-w-xl flex-col items-center justify-center gap-4',
        'rounded-[var(--mk-radius-panel)] border-2 border-dashed border-[var(--mk-outline)]',
        'bg-[var(--mk-surface-container)] p-8 text-center text-[var(--mk-text)]',
        onClick &&
          'mk-focus-ring cursor-pointer transition-colors hover:border-[var(--mk-primary)] hover:bg-[var(--mk-state-hover)]',
      )}
    >
      <CloudUpload
        className="size-12 text-[var(--mk-text-muted)]"
        aria-hidden
      />
      {children}
      {badge && (
        <span className="absolute bottom-4 right-5 rounded-full border border-[var(--mk-outline)] bg-[var(--mk-surface)] px-2.5 py-0.5 text-xs font-medium text-[var(--mk-text-muted)]">
          {badge}
        </span>
      )}
    </Tag>
  )
}
