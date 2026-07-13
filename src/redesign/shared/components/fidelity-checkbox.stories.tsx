import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { FidelityCheckbox } from './fidelity-checkbox'

const meta = {
  title: 'Shared/FidelityCheckbox',
  component: FidelityCheckbox,
  parameters: { layout: 'centered' },
  args: {
    checked: false,
    onCheckedChange: () => {},
    'aria-label': 'Select mod',
  },
  render: function Render(args) {
    const [checked, setChecked] = useState(args.checked)
    return (
      <FidelityCheckbox
        {...args}
        checked={checked}
        onCheckedChange={setChecked}
      />
    )
  },
} satisfies Meta<typeof FidelityCheckbox>

export default meta
type Story = StoryObj<typeof meta>

export const Unchecked: Story = {}
export const Checked: Story = { args: { checked: true } }
export const Indeterminate: Story = { args: { indeterminate: true } }
export const Disabled: Story = { args: { disabled: true } }
export const DisabledChecked: Story = {
  args: { checked: true, disabled: true },
}
