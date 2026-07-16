import { Switch } from '@/components/ui/switch'
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

/**
 * Styled wrapper over the shadcn Switch (fix_plan_0.md §1/§7 — the interactive primitive is always
 * the shadcn one; thumb geometry is Radix's). Sized to the reference's 44×26 track / 20px knob.
 */
export function FidelitySwitch({
  busy = false,
  className,
  ...props
}: FidelitySwitchProps) {
  return (
    <Switch
      data-slot="fidelity-switch"
      aria-busy={busy || undefined}
      className={cn(
        'h-[1.625rem] w-11',
        '[&_[data-slot=switch-thumb]]:size-5',
        '[&_[data-slot=switch-thumb][data-state=unchecked]]:translate-x-[2px]',
        '[&_[data-slot=switch-thumb][data-state=checked]]:translate-x-[20px]',
        className,
      )}
      {...props}
    />
  )
}
