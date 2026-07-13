import type { ComponentProps } from 'react'
import { cva, type VariantProps } from 'class-variance-authority'
import { cn } from '@/lib/utils'

const iconButtonVariants = cva(
  cn(
    'mk-focus-ring inline-flex shrink-0 items-center justify-center',
    'rounded-[var(--mk-radius-control)] transition-colors',
    'disabled:pointer-events-none disabled:opacity-50',
    '[&_svg]:shrink-0 [&_svg]:pointer-events-none'
  ),
  {
    variants: {
      variant: {
        primary:
          'bg-[var(--mk-primary)] text-[var(--mk-on-primary)] hover:bg-[var(--mk-primary-hover)] active:bg-[var(--mk-primary-active)]',
        secondary:
          'border border-[var(--mk-outline)] bg-[var(--mk-surface-container)] text-[var(--mk-text)] hover:bg-[var(--mk-surface-strong)]',
        ghost:
          'bg-transparent text-[var(--mk-text)] hover:bg-[var(--mk-surface-container)]',
        destructive:
          'bg-[var(--mk-danger)] text-[var(--mk-on-danger)] hover:bg-[var(--mk-danger-hover)]',
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
  }
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
