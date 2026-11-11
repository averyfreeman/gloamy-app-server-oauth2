import process from 'node:process'
import { fileURLToPath } from 'node:url'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

// The app talks to the Gloamy daemon over same-origin paths so the browser
// never needs CORS and mixed-content rules never apply. In development Vite
// forwards those paths here; in a hosted deployment the same paths are expected
// to be forwarded by whatever reverse proxy serves `dist/`.
const DAEMON_URL = process.env.GLOAMY_DAEMON_URL || 'http://127.0.0.1:42617'

// Daemon route prefixes that must be proxied rather than served by Vite.
const DAEMON_PATHS = ['/api', '/pair', '/health']

const daemonProxy = Object.fromEntries(
  DAEMON_PATHS.map(prefix => [
    prefix,
    {
      target: DAEMON_URL,
      changeOrigin: true,
      // Server-sent events must stream through unbuffered.
      ws: false,
      configure: (proxy: { on: (event: string, cb: (...args: any[]) => void) => void }) => {
        proxy.on('proxyReq', (proxyReq: { setHeader: (k: string, v: string) => void }) => {
          proxyReq.setHeader('Accept-Encoding', 'identity')
        })
      },
    },
  ]),
)

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    proxy: daemonProxy,
  },
  preview: {
    port: 1420,
    strictPort: true,
    proxy: daemonProxy,
  },
})
