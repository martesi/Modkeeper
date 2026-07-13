import type { Preview } from '@storybook/react-vite'
import '../src/assets/style.css'
import '../src/redesign/styles/fidelity.css'

// Warm Fidelity backdrop for every story so glass surfaces and focus rings read correctly in
// isolation. The design system's own tokens drive the component colors; this only frames them.
const preview: Preview = {
  parameters: {
    layout: 'centered',
    controls: { expanded: true },
  },
  decorators: [
    (Story) => (
      <div
        style={{
          minWidth: 320,
          padding: '2.5rem',
          borderRadius: '1.5rem',
          background:
            'radial-gradient(120% 120% at 0% 0%, #ffe9e8 0%, #fff8f7 55%, #ffeef2 100%)',
        }}
      >
        <Story />
      </div>
    ),
  ],
}

export default preview
