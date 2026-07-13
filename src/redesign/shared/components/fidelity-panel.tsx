import type { ComponentProps } from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const panelVariants = cva('text-[var(--mk-text)] border', {
  variants: {
    variant: {
      standard:
        'mk-glass-standard bg-[var(--mk-surface-container)] border-[var(--mk-outline)] shadow-[var(--mk-shadow-panel)]',
      strong:
        'mk-glass-strong bg-[var(--mk-surface-strong)] border-[var(--mk-outline)] shadow-[var(--mk-shadow-panel)]',
      solid: 'bg-[var(--mk-surface)] border-[var(--mk-outline)]',
    },
    radius: {
      panel: 'rounded-[var(--mk-radius-panel)]',
      dialog: 'rounded-[var(--mk-radius-dialog)]',
      control: 'rounded-[var(--mk-radius-control)]',
    },
  },
  defaultVariants: {
    variant: 'standard',
    radius: 'panel',
  },
})

type FidelityPanelProps = ComponentProps<'div'> &
  VariantProps<typeof panelVariants>

export function FidelityPanel({
  className,
  variant,
  radius,
  ...props
}: FidelityPanelProps) {
  return (
    <div
      data-slot="fidelity-panel"
      className={cn(panelVariants({ variant, radius }), className)}
      {...props}
    />
  )
}

export { panelVariants }
