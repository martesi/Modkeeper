import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { ConfirmDialog } from './confirm-dialog'
import { FidelityButton } from './fidelity-button'

const meta = {
  title: 'Shared/ConfirmDialog',
  component: ConfirmDialog,
  parameters: { layout: 'centered' },
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

export const WithExtraContent: Story = {
  render: () => {
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
          <label className="flex items-center gap-2 text-sm text-[var(--mk-text)]">
            <input
              type="checkbox"
              checked={deleteFiles}
              onChange={(e) => setDeleteFiles(e.target.checked)}
            />
            Also delete files on disk
          </label>
        </ConfirmDialog>
      </>
    )
  },
}
