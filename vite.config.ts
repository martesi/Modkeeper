import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import router from '@tanstack/router-plugin/vite'
import autoImports from 'unplugin-auto-import/vite'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(() => ({
  plugins: [
    react(),
    tailwindcss(),
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
  resolve: {
    alias: {
      '@gen': fileURLToPath(new URL('.config/generated', import.meta.url)),
      '@': fileURLToPath(new URL('src', import.meta.url)),
      '@utils': fileURLToPath(new URL('src/utils', import.meta.url)),
    },
  },
}))
