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

interface CloseLibraryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  libraryName: string
  onConfirm: () => void
  onCancel: () => void
}

export function CloseLibraryDialog({
  open,
  onOpenChange,
  libraryName,
  onConfirm,
  onCancel,
}: CloseLibraryDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            <Trans>Close Library</Trans>
          </AlertDialogTitle>
          <AlertDialogDescription>
            <Trans>
              Are you sure you want to close &quot;{libraryName}&quot;? It will
              be removed from your library list but files will remain on disk.
            </Trans>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onCancel}>
            <Trans>Cancel</Trans>
          </AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>
            <Trans>Close</Trans>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
