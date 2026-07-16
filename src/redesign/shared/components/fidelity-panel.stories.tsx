import type { Meta, StoryObj } from '@storybook/react-vite'
import { FidelityPanel } from './fidelity-panel'

const meta = {
  title: 'Shared/FidelityPanel',
  component: FidelityPanel,
  parameters: { layout: 'centered' },
  args: {
    className: 'w-80 p-6',
    children: (
      <div className="flex flex-col gap-1">
        <h3 className="text-sm font-semibold">Panel surface</h3>
        <p className="text-sm text-muted-foreground">
          Warm glass container used across the redesign.
        </p>
      </div>
    ),
  },
} satisfies Meta<typeof FidelityPanel>

export default meta
type Story = StoryObj<typeof meta>

export const Standard: Story = { args: { variant: 'standard' } }
export const Strong: Story = { args: { variant: 'strong' } }
export const Solid: Story = { args: { variant: 'solid' } }
export const ControlRadius: Story = {
  args: { variant: 'standard', radius: 'control' },
}
