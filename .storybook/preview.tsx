import type { Preview } from '@storybook/react-vite'
import '../src/assets/style.css'
import '../src/redesign/styles/fidelity.css'

// Fidelity page-gradient backdrop for every story so glass surfaces and focus rings read
// correctly in isolation. The design system's own tokens (style.css) drive the component colors;
// this only frames them.
const preview: Preview = {
  parameters: {
    layout: 'centered',
    controls: { expanded: true },
  },
  decorators: [
    (Story) => (
      <div
        className="mk-page-gradient"
        style={{
          minWidth: 320,
          padding: '2.5rem',
          borderRadius: '1.5rem',
        }}
      >
        <Story />
      </div>
    ),
  ],
}

export default preview
