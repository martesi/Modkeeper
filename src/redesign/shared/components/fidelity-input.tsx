import type { ComponentProps } from 'react'
import { cn } from '@/lib/utils'

export function FidelityInput({ className, ...props }: ComponentProps<'input'>) {
  return (
    <input
      data-slot="fidelity-input"
      className={cn(
        'mk-focus-ring h-10 w-full min-w-0 rounded-[var(--mk-radius-control)] px-3.5 text-sm',
        'border border-[var(--mk-outline)] bg-[var(--mk-surface)] text-[var(--mk-text)]',
        'transition-colors placeholder:text-[var(--mk-text-muted)]',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'aria-invalid:border-[var(--mk-danger)] aria-invalid:text-[var(--mk-danger)]',
        className
      )}
      {...props}
    />
  )
}
