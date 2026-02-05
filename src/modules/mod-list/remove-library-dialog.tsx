'use client'

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Trans } from '@lingui/react/macro'

interface RemoveLibraryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  libraryName: string
  onConfirm: () => void
  onCancel: () => void
}

export function RemoveLibraryDialog({
  open,
  onOpenChange,
  libraryName,
  onConfirm,
  onCancel,
}: RemoveLibraryDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            <Trans>Remove Library</Trans>
          </AlertDialogTitle>
          <AlertDialogDescription>
            <Trans>
              Are you sure you want to remove &quot;{libraryName}&quot;? This
              will unlink all mods, remove it from your library list, and delete
              the library directory. This action cannot be undone.
            </Trans>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onCancel}>
            <Trans>Cancel</Trans>
          </AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} variant="destructive">
            <Trans>Remove</Trans>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
