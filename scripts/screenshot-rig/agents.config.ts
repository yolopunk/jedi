// Deterministic browser verification for group conversations, without native I/O.
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { defineConfig, mergeConfig } from 'vite'
import rigConfig from './vite.config'

export default mergeConfig(rigConfig, defineConfig({
  plugins: [{
    name: 'jedi-agents-verification',
    transformIndexHtml: {
      order: 'post',
      handler(html) {
        return html.replace('</head>', '<script src="/__agents-mock.js"></script></head>')
      },
    },
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url !== '/__agents-mock.js') return next()
        res.setHeader('Content-Type', 'text/javascript; charset=utf-8')
        res.setHeader('Cache-Control', 'no-store')
        res.end(readFileSync(fileURLToPath(new URL('./mock/agents-mock.js', import.meta.url)), 'utf8'))
      })
    },
  }],
  server: { port: 31423 },
}))
