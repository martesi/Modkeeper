'use client'

import * as React from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Trans } from '@lingui/react/macro'

interface RenameLibraryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentName: string
  onConfirm: (newName: string) => void
}

export function RenameLibraryDialog({
  open,
  onOpenChange,
  currentName,
  onConfirm,
}: RenameLibraryDialogProps) {
  const [value, setValue] = React.useState('')

  React.useEffect(() => {
    if (open) {
      setValue(currentName)
    }
  }, [open, currentName])

  const handleConfirm = () => {
    if (value.trim()) {
      onConfirm(value.trim())
      setValue('')
    }
  }

  const handleCancel = () => {
    onOpenChange(false)
    setValue('')
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            <Trans>Rename Library</Trans>
          </DialogTitle>
          <DialogDescription>
            <Trans>Enter a new name for this library.</Trans>
          </DialogDescription>
        </DialogHeader>
        <Input
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              handleConfirm()
            }
          }}
          placeholder={currentName || 'Library Name'}
          autoFocus
        />
        <DialogFooter>
          <Button variant="outline" onClick={handleCancel}>
            <Trans>Cancel</Trans>
          </Button>
          <Button onClick={handleConfirm} disabled={!value.trim()}>
            <Trans>Rename</Trans>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
