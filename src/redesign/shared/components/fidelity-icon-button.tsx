import type { ComponentProps } from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const iconButtonVariants = cva(
  cn(
    'inline-flex shrink-0 items-center justify-center',
    'rounded-lg transition-colors',
    'outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
    'disabled:pointer-events-none disabled:opacity-50',
    '[&_svg]:shrink-0 [&_svg]:pointer-events-none',
  ),
  {
    variants: {
      variant: {
        primary: 'bg-primary text-primary-foreground hover:bg-primary/90',
        secondary:
          'border border-border bg-secondary text-foreground hover:bg-accent',
        /** The reference toolbar's tinted round tool button: primary at low alpha. */
        soft: 'bg-primary/15 text-primary hover:bg-primary/25',
        ghost:
          'bg-transparent text-muted-foreground hover:bg-accent hover:text-foreground',
        destructive: 'bg-destructive text-white hover:bg-destructive/90',
      },
      size: {
        sm: 'size-8 [&_svg]:size-4',
        md: 'size-10 [&_svg]:size-5',
        lg: 'size-12 [&_svg]:size-6',
      },
    },
    defaultVariants: {
      variant: 'ghost',
      size: 'md',
    },
  },
)

type FidelityIconButtonProps = ComponentProps<'button'> &
  VariantProps<typeof iconButtonVariants> & {
    // Icon-only controls must be labeled for assistive tech (spec §12 accessibility).
    'aria-label': string
  }

export function FidelityIconButton({
  className,
  variant,
  size,
  ...props
}: FidelityIconButtonProps) {
  return (
    <button
      data-slot="fidelity-icon-button"
      className={cn(iconButtonVariants({ variant, size }), className)}
      {...props}
    />
  )
}

export { iconButtonVariants }
