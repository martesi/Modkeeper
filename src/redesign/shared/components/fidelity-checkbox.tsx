import { Check, Minus } from 'lucide-react'
import { cn } from '@/lib/utils'

type FidelityCheckboxProps = {
  checked: boolean
  /** Renders a dash instead of a check — the "some but not all selected" toolbar state. */
  indeterminate?: boolean
  onCheckedChange: (checked: boolean) => void
  disabled?: boolean
  // Checkboxes render no text of their own, so a label is mandatory (spec §12 accessibility).
  'aria-label': string
  className?: string
}

export function FidelityCheckbox({
  checked,
  indeterminate = false,
  onCheckedChange,
  disabled = false,
  className,
  ...props
}: FidelityCheckboxProps) {
  const marked = checked || indeterminate
  return (
    <button
      type="button"
      role="checkbox"
      data-slot="fidelity-checkbox"
      aria-checked={indeterminate ? 'mixed' : checked}
      disabled={disabled}
      onClick={() => onCheckedChange(!checked)}
      className={cn(
        'mk-focus-ring inline-flex size-5 shrink-0 items-center justify-center rounded-md border transition-colors',
        'disabled:cursor-not-allowed disabled:opacity-50',
        marked
          ? 'border-[var(--mk-primary)] bg-[var(--mk-primary)] text-[var(--mk-on-primary)]'
          : 'border-[var(--mk-outline)] bg-[var(--mk-surface)]',
        className,
      )}
      {...props}
    >
      {indeterminate ? (
        <Minus className="size-3.5" aria-hidden />
      ) : (
        checked && <Check className="size-3.5" aria-hidden />
      )}
    </button>
  )
}
