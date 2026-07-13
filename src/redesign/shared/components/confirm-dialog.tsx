import type { ReactNode } from 'react'
import * as Dialog from '@radix-ui/react-dialog'
import { cn } from '@/lib/utils'
import { FidelityButton } from './fidelity-button'

type ConfirmDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: ReactNode
  description?: ReactNode
  confirmLabel: ReactNode
  cancelLabel: ReactNode
  onConfirm: () => void
  destructive?: boolean
  busy?: boolean
  // Extra body content (e.g. a delete-files checkbox) between description and footer.
  children?: ReactNode
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  cancelLabel,
  onConfirm,
  destructive = false,
  busy = false,
  children,
}: ConfirmDialogProps) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay
          className={cn(
            'fixed inset-0 z-50 bg-black/40 backdrop-blur-sm',
            'data-[state=open]:animate-in data-[state=open]:fade-in-0',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0'
          )}
        />
        <Dialog.Content
          className={cn(
            'mk-glass-strong fixed left-1/2 top-1/2 z-50 w-[min(28rem,calc(100vw-2rem))]',
            '-translate-x-1/2 -translate-y-1/2 rounded-[var(--mk-radius-dialog)] p-6',
            'border border-[var(--mk-outline)] bg-[var(--mk-surface-strong)] text-[var(--mk-text)]',
            'shadow-[var(--mk-shadow-panel)]',
            'data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95',
            'data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95'
          )}
        >
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1">
              <Dialog.Title className="text-lg font-semibold text-[var(--mk-text)]">
                {title}
              </Dialog.Title>
              {description && (
                <Dialog.Description className="text-sm text-[var(--mk-text-muted)]">
                  {description}
                </Dialog.Description>
              )}
            </div>
            {children}
            <div className="flex items-center justify-end gap-2">
              <Dialog.Close asChild>
                <FidelityButton variant="ghost" disabled={busy}>
                  {cancelLabel}
                </FidelityButton>
              </Dialog.Close>
              <FidelityButton
                variant={destructive ? 'destructive' : 'primary'}
                busy={busy}
                onClick={onConfirm}
              >
                {confirmLabel}
              </FidelityButton>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
