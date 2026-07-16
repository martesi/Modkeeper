import type { ComponentProps } from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const panelVariants = cva('text-foreground border', {
  variants: {
    variant: {
      standard: 'mk-glass-standard bg-card border-border shadow-sm',
      strong: 'mk-glass-strong bg-popover border-border shadow-xl',
      solid: 'bg-background border-border',
    },
    radius: {
      panel: 'rounded-[1.25rem]',
      dialog: 'rounded-[2rem]',
      control: 'rounded-lg',
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
