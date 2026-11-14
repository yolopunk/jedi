/**
 * Screenshot rig config — ONLY used to retake README screenshots.
 * Run: pnpm exec vite --config scripts/screenshot-rig/vite.config.ts
 *
 * Same plugins/alias as the project config, plus:
 *  - a Tauri IPC shim injected before the app boots (mock/tauri-mock.js)
 *  - a middleware serving rig assets under /__rig/
 * Does NOT touch vite.config.ts and is not picked up by any build script.
 */
import vue from '@vitejs/plugin-vue'
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import vuetify from 'vite-plugin-vuetify'
import { defineConfig, type Plugin } from 'vite'

const rigDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(rigDir, '..', '..')
const MIME: Record<string, string> = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.webp': 'image/webp',
  '.svg': 'image/svg+xml',
}

function rigMock(): Plugin {
  return {
    name: 'jedi-screenshot-rig',
    transformIndexHtml(html) {
      // Classic script in <head>: runs before the deferred module graph,
      // so window.__TAURI_INTERNALS__ exists before @tauri-apps/api is used.
      return html.replace('<head>', '<head>\n    <script src="/__rig/tauri-mock.js"></script>')
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = (req.url || '').split('?')[0]
        if (url === '/__rig/tauri-mock.js') {
          res.setHeader('Content-Type', 'text/javascript; charset=utf-8')
          res.setHeader('Cache-Control', 'no-store')
          res.end(readFileSync(path.join(rigDir, 'mock', 'tauri-mock.js'), 'utf-8'))
          return
        }
        if (url.startsWith('/__rig/assets/')) {
          const file = path.join(rigDir, 'assets', path.basename(url))
          try {
            const body = readFileSync(file)
            res.setHeader('Content-Type', MIME[path.extname(file).toLowerCase()] ?? 'application/octet-stream')
            res.setHeader('Cache-Control', 'no-store')
            res.end(body)
          } catch {
            res.statusCode = 404
            res.end('not found')
          }
          return
        }
        next()
      })
    },
  }
}

export default defineConfig({
  base: './',
  plugins: [vue(), vuetify({ autoImport: true }), rigMock()],
  resolve: {
    alias: {
      '@': path.resolve(repoRoot, 'src'),
    },
  },
  clearScreen: false,
  server: {
    host: '127.0.0.1',
    port: 31421,
    strictPort: true,
  },
})
