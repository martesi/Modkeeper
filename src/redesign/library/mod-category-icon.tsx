import { Server, MonitorSmartphone, Boxes, HelpCircle } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { ModType } from '../data/redesign-types'

/**
 * ModType-tinted category icon (consolidated-spec.md §12.3 / C6 — mods carry no icon of their own).
 * Stable dimensions to keep the card grid from shifting.
 */
const CONFIG: Record<ModType, { Icon: LucideIcon; tint: string }> = {
  client: { Icon: MonitorSmartphone, tint: 'var(--mk-tertiary)' },
  server: { Icon: Server, tint: 'var(--mk-primary)' },
  both: { Icon: Boxes, tint: 'var(--mk-primary-active)' },
  unknown: { Icon: HelpCircle, tint: 'var(--mk-text-muted)' },
}

export function ModCategoryIcon({
  type,
  className,
}: {
  type: ModType
  className?: string
}) {
  const { Icon, tint } = CONFIG[type]
  return (
    <span
      className={cn(
        'inline-flex size-9 shrink-0 items-center justify-center rounded-[var(--mk-radius-control)]',
        className
      )}
      style={{ backgroundColor: `color-mix(in srgb, ${tint} 16%, transparent)`, color: tint }}
      aria-hidden
    >
      <Icon className="size-5" />
    </span>
  )
}
