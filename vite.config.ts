import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import router from '@tanstack/router-plugin/vite'
import autoImports from 'unplugin-auto-import/vite'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'

const host = process.env.TAURI_DEV_HOST

// https://vite.dev/config/
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

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
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
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
  resolve: {
    alias: {
      '@gen': fileURLToPath(new URL('.config/generated', import.meta.url)),
    },
  },
}))
