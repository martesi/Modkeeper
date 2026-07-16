import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { ConfirmDialog } from './confirm-dialog'
import { FidelityButton } from './fidelity-button'

const meta = {
  title: 'Shared/ConfirmDialog',
  component: ConfirmDialog,
  parameters: { layout: 'centered' },
  // Each story drives the dialog through a stateful Harness via `render`, so these
  // are placeholder defaults only — present so the required-prop type is satisfied.
  args: {
    open: false,
    onOpenChange: () => {},
    title: 'Confirm action',
    confirmLabel: 'Confirm',
    cancelLabel: 'Cancel',
    onConfirm: () => {},
  },
} satisfies Meta<typeof ConfirmDialog>

export default meta
type Story = StoryObj<typeof meta>

function Harness({ destructive }: { destructive?: boolean }) {
  const [open, setOpen] = useState(false)
  return (
    <>
      <FidelityButton
        variant={destructive ? 'destructive' : 'primary'}
        onClick={() => setOpen(true)}
      >
        {destructive ? 'Delete library' : 'Confirm action'}
      </FidelityButton>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        destructive={destructive}
        title={destructive ? 'Delete this library?' : 'Rebuild cache?'}
        description={
          destructive
            ? 'This removes the library entry. Files on disk are kept unless you choose otherwise.'
            : 'Reconcile recorded mods against what is on disk.'
        }
        confirmLabel={destructive ? 'Delete' : 'Rebuild'}
        cancelLabel="Cancel"
        onConfirm={() => setOpen(false)}
      />
    </>
  )
}

export const Default: Story = {
  render: () => <Harness />,
}

export const Destructive: Story = {
  render: () => <Harness destructive />,
}

function ExtraContentHarness() {
  const [open, setOpen] = useState(false)
  const [deleteFiles, setDeleteFiles] = useState(false)
  return (
    <>
      <FidelityButton variant="destructive" onClick={() => setOpen(true)}>
        Delete library
      </FidelityButton>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        destructive
        title="Delete this library?"
        description="Choose whether the files on disk are also removed."
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onConfirm={() => setOpen(false)}
      >
        <label className="flex items-center gap-2.5 rounded-lg border border-border bg-secondary px-3.5 py-2.5 text-sm text-foreground">
          <input
            type="checkbox"
            checked={deleteFiles}
            onChange={(e) => setDeleteFiles(e.target.checked)}
            className="size-4"
            style={{ accentColor: 'var(--primary)' }}
          />
          Also delete files on disk
        </label>
      </ConfirmDialog>
    </>
  )
}

export const WithExtraContent: Story = {
  render: () => <ExtraContentHarness />,
}
