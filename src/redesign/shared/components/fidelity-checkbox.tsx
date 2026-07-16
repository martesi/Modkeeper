import { Checkbox } from '@/components/ui/checkbox'
import { cn } from '@/lib/utils'

type FidelityCheckboxProps = {
  checked: boolean
  /** Renders the mixed state — the "some but not all selected" toolbar state. */
  indeterminate?: boolean
  onCheckedChange: (checked: boolean) => void
  disabled?: boolean
  // Checkboxes render no text of their own, so a label is mandatory (spec §12 accessibility).
  'aria-label': string
  className?: string
}

/**
 * Styled wrapper over the shadcn Checkbox (fix_plan_0.md §7 — the interactive primitive is always
 * the shadcn one). Keeps the boolean-plus-indeterminate prop surface of the old hand-rolled box.
 */
export function FidelityCheckbox({
  checked,
  indeterminate = false,
  onCheckedChange,
  className,
  ...props
}: FidelityCheckboxProps) {
  return (
    <Checkbox
      data-slot="fidelity-checkbox"
      checked={indeterminate ? 'indeterminate' : checked}
      onCheckedChange={() => onCheckedChange(!checked)}
      className={cn('size-4', className)}
      {...props}
    />
  )
}
