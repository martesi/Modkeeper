import { cn } from '@/lib/utils'

type FidelitySwitchProps = {
  checked: boolean
  onCheckedChange: (checked: boolean) => void
  disabled?: boolean
  /** Announced while an awaited change is settling (local per-click pending). */
  busy?: boolean
  // Switches render no text of their own, so a label is mandatory (spec §12 accessibility).
  'aria-label': string
  className?: string
}

/** Stable-dimension toggle switch (spec §11 guardrails — no layout shift between states). */
export function FidelitySwitch({
  checked,
  onCheckedChange,
  disabled = false,
  busy = false,
  className,
  ...props
}: FidelitySwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      data-slot="fidelity-switch"
      aria-checked={checked}
      aria-busy={busy || undefined}
      disabled={disabled}
      onClick={() => onCheckedChange(!checked)}
      className={cn(
        'mk-focus-ring relative h-6 w-11 shrink-0 rounded-full transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        checked ? 'bg-[var(--mk-primary)]' : 'bg-[var(--mk-outline)]',
        className,
      )}
      {...props}
    >
      <span
        className={cn(
          'absolute top-0.5 size-5 rounded-full bg-white shadow-sm transition-transform',
          checked ? 'translate-x-[1.375rem]' : 'translate-x-0.5',
        )}
      />
    </button>
  )
}
