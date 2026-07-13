import type { StorybookConfig } from '@storybook/react-vite'

// Scoped to the shared design-system primitives only (consolidated-spec.md §8.1) — not screens,
// state, or business logic.
const config: StorybookConfig = {
  stories: ['../src/redesign/shared/components/**/*.stories.tsx'],
  addons: [],
  framework: {
    name: '@storybook/react-vite',
    options: {},
  },
}

export default config
