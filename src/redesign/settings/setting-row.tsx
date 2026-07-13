import type { ReactNode } from 'react'
import type { LucideIcon } from 'lucide-react'
import { FidelityPanel } from '../shared/components/fidelity-panel'

/**
 * One settings row (consolidated-spec.md §12.6): icon, label, description, right-aligned control.
 * Stacks on narrow widths (spec §12 accessibility).
 */
export function SettingRow({
  icon: Icon,
  label,
  description,
  children,
}: {
  icon: LucideIcon
  label: string
  description: string
  children: ReactNode
}) {
  return (
    <FidelityPanel
      radius="control"
      className="flex flex-col gap-3 p-4 sm:flex-row sm:items-center sm:justify-between"
    >
      <div className="flex min-w-0 items-center gap-3">
        <span className="inline-flex size-10 shrink-0 items-center justify-center rounded-[var(--mk-radius-control)] bg-[var(--mk-state-hover)] text-[var(--mk-primary)]">
          <Icon className="size-5" aria-hidden />
        </span>
        <div className="min-w-0">
          <p className="text-sm font-medium">{label}</p>
          <p className="text-xs text-[var(--mk-text-muted)]">{description}</p>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-2">{children}</div>
    </FidelityPanel>
  )
}
