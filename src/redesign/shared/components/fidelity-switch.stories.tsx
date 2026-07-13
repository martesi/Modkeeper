import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { FidelitySwitch } from './fidelity-switch'

const meta = {
  title: 'Shared/FidelitySwitch',
  component: FidelitySwitch,
  parameters: { layout: 'centered' },
  args: {
    checked: false,
    onCheckedChange: () => {},
    'aria-label': 'Enable mod',
  },
  render: function Render(args) {
    const [checked, setChecked] = useState(args.checked)
    return (
      <FidelitySwitch
        {...args}
        checked={checked}
        onCheckedChange={setChecked}
      />
    )
  },
} satisfies Meta<typeof FidelitySwitch>

export default meta
type Story = StoryObj<typeof meta>

export const Off: Story = {}
export const On: Story = { args: { checked: true } }
export const Disabled: Story = { args: { disabled: true } }
export const DisabledOn: Story = { args: { checked: true, disabled: true } }
export const Busy: Story = { args: { checked: true, busy: true } }
