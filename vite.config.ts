import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import router from '@tanstack/router-plugin/vite'
import autoImports from 'unplugin-auto-import/vite'
import tailwindcss from '@tailwindcss/vite'
import tsconfigPaths from 'vite-tsconfig-paths'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(() => ({
  envDir: '.config',
  plugins: [
    react(),
    tailwindcss(),
    tsconfigPaths(),
    router({
      generatedRouteTree: '.config/generated/routes.ts',
      disableLogging: true,
    }),
    autoImports({
      include: [/\.tsx?$/],
      imports: [
        'react',
        {
          clsx: [['default', 'c']],
          '@tanstack/react-router': [
            'Link',
            'Outlet',
            'Navigate',
            'useParams',
            'useLoaderDeps',
            'useLoaderData',
            'useRouter',
            'useNavigate',
            'useBlocker',
            'useSearch',
            'useMatches',
            'redirect',
            'notFound',
          ],
        },
      ],
      dts: '.config/generated/auto-imports.d.ts',
    }),
  ],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
}))
