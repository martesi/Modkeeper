import type { ComponentProps } from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import { Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils'

const buttonVariants = cva(
  cn(
    'mk-focus-ring relative inline-flex shrink-0 items-center justify-center gap-2',
    'whitespace-nowrap rounded-[var(--mk-radius-control)] font-medium transition-colors',
    'disabled:pointer-events-none disabled:opacity-50',
    '[&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:pointer-events-none'
  ),
  {
    variants: {
      variant: {
        primary:
          'bg-[var(--mk-primary)] text-[var(--mk-on-primary)] hover:bg-[var(--mk-primary-hover)] active:bg-[var(--mk-primary-active)]',
        secondary:
          'border border-[var(--mk-outline)] bg-[var(--mk-surface-container)] text-[var(--mk-text)] hover:bg-[var(--mk-surface-container-hover)]',
        outline:
          'border border-[var(--mk-outline)] bg-transparent text-[var(--mk-text)] hover:bg-[var(--mk-state-hover)] active:bg-[var(--mk-state-active)]',
        ghost:
          'bg-transparent text-[var(--mk-text)] hover:bg-[var(--mk-state-hover)] active:bg-[var(--mk-state-active)]',
        destructive:
          'bg-[var(--mk-danger)] text-[var(--mk-on-danger)] hover:bg-[var(--mk-danger-hover)]',
      },
      size: {
        sm: 'h-8 px-3 text-sm',
        md: 'h-10 px-4 text-sm',
        lg: 'h-12 px-6 text-base',
      },
    },
    defaultVariants: {
      variant: 'primary',
      size: 'md',
    },
  }
)

type FidelityButtonProps = ComponentProps<'button'> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean
    busy?: boolean
  }

export function FidelityButton({
  className,
  variant,
  size,
  asChild = false,
  busy = false,
  disabled,
  children,
  ...props
}: FidelityButtonProps) {
  const classes = cn(buttonVariants({ variant, size }), className)

  // Slot accepts a single child, so the busy spinner is a plain-button affordance only.
  if (asChild) {
    return (
      <Slot
        data-slot="fidelity-button"
        data-variant={variant ?? 'primary'}
        className={classes}
        {...props}
      >
        {children}
      </Slot>
    )
  }

  return (
    <button
      data-slot="fidelity-button"
      data-variant={variant ?? 'primary'}
      aria-busy={busy || undefined}
      disabled={disabled || busy}
      className={classes}
      {...props}
    >
      {busy && <Loader2 className="animate-spin" aria-hidden />}
      {children}
    </button>
  )
}

export { buttonVariants }
