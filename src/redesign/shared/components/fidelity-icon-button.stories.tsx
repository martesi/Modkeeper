import type { Meta, StoryObj } from '@storybook/react-vite'
import { Settings, Trash2 } from 'lucide-react'
import { FidelityIconButton } from './fidelity-icon-button'

const meta = {
  title: 'Shared/FidelityIconButton',
  component: FidelityIconButton,
  parameters: { layout: 'centered' },
  args: {
    'aria-label': 'Open settings',
    children: <Settings />,
  },
} satisfies Meta<typeof FidelityIconButton>

export default meta
type Story = StoryObj<typeof meta>

export const Ghost: Story = { args: { variant: 'ghost' } }
export const Primary: Story = { args: { variant: 'primary' } }
export const Secondary: Story = { args: { variant: 'secondary' } }
export const Destructive: Story = {
  args: {
    variant: 'destructive',
    'aria-label': 'Delete',
    children: <Trash2 />,
  },
}

export const Disabled: Story = { args: { disabled: true } }

export const Sizes: Story = {
  render: (args) => (
    <div className="flex items-center gap-3">
      <FidelityIconButton {...args} size="sm" />
      <FidelityIconButton {...args} size="md" />
      <FidelityIconButton {...args} size="lg" />
    </div>
  ),
}
