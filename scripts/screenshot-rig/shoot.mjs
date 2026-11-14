/**
 * CDP-based screenshot driver for the Jedi README rig.
 *
 * More reliable than `msedge --headless --screenshot` on Edge 132+:
 * real-time waits, explicit readiness predicates per route, and no
 * virtual-time starvation hangs. Primary entry; shoot.ps1 kept as fallback.
 *
 * Usage: node scripts/screenshot-rig/shoot.mjs [--base URL] [--out DIR]
 */
import { spawn } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const rigDir = path.dirname(fileURLToPath(import.meta.url))
const OUT = resolveArg('--out') ?? path.join(rigDir, 'output')
const BASE = resolveArg('--base') ?? 'http://127.0.0.1:31421'
const DEBUG_PORT = 9223

function resolveArg(name) {
  const i = process.argv.indexOf(name)
  return i !== -1 ? process.argv[i + 1] : null
}

const EDGE = [
  'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
  'C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe',
].find(p => fs.existsSync(p))
if (!EDGE) throw new Error('msedge.exe not found')

const PROFILE = path.join(os.tmpdir(), 'jedi-rig-cdp-profile')
fs.rmSync(PROFILE, { recursive: true, force: true })
fs.mkdirSync(OUT, { recursive: true })

const SHOTS = [
  {
    name: 'ai-chat',
    url: '/chat?rigAction=chat-hero',
    ready: `!!document.querySelector('.messages-list .console-message') && !!document.querySelector('.session-item') && !document.querySelector('.app-splash')`,
  },
  { name: 'skills', url: '/skills', ready: `!!document.querySelector('.skills-page') && !document.querySelector('.app-splash')`, post: `(() => { const el = [...document.querySelectorAll('.skills-page *')].find(e => e.childElementCount === 0 && /SKILL\\.md/.test(e.textContent || '')); if (el) el.scrollIntoView({ block: 'start' }); return !!el })()` },
  { name: 'hosts', url: '/hosts', ready: `!!document.querySelector('.hosts-container') && !!document.querySelector('.hosts-container .v-list, .hosts-container table, .hosts-container .hosts-header') && !document.querySelector('.app-splash')` },
  { name: 'wallpapers', url: '/wallpapers', ready: `!!document.querySelector('.wallpaper-manager img') && !document.querySelector('.app-splash')` },
  { name: 'podcast', url: '/podcast', ready: `!!document.querySelector('.podcast-manager') && !!document.querySelector('.podcast-manager img') && !document.querySelector('.app-splash')` },
  { name: 'screenshot-tool', url: '/screenshot', ready: `!!document.querySelector('.screenshot-page .gallery-collections img') && !document.querySelector('.app-splash')` },
]

const child = spawn(EDGE, [
  '--headless',
  '--disable-gpu',
  `--remote-debugging-port=${DEBUG_PORT}`,
  '--window-size=1700,1100',
  '--no-first-run',
  '--disable-extensions',
  '--disable-sync',
  '--mute-audio',
  `--user-data-dir=${PROFILE}`,
  'about:blank',
], { stdio: 'ignore' })

const sleep = ms => new Promise(r => setTimeout(r, ms))

async function waitDevtools(timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/list`)
      const targets = await res.json()
      const page = targets.find(t => t.type === 'page' && t.webSocketDebuggerUrl)
      if (page) return page.webSocketDebuggerUrl
    } catch {
      /* devtools not up yet */
    }
    await sleep(300)
  }
  throw new Error('devtools endpoint never came up')
}

class CDP {
  constructor(ws) {
    this.ws = ws
    this.seq = 0
    this.pending = new Map()
    this.listeners = new Map()
    ws.addEventListener('message', ev => {
      const msg = JSON.parse(ev.data)
      if (msg.id !== undefined && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id)
        this.pending.delete(msg.id)
        if (msg.error) reject(new Error(`${msg.error.message} (${msg.error.data ?? ''})`))
        else resolve(msg.result)
      } else if (msg.method) {
        const subs = this.listeners.get(msg.method) ?? []
        for (const fn of subs) fn(msg.params)
      }
    })
  }
  on(method, fn) {
    if (!this.listeners.has(method)) this.listeners.set(method, [])
    this.listeners.get(method).push(fn)
  }
  send(method, params = {}) {
    const id = ++this.seq
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject })
      this.ws.send(JSON.stringify({ id, method, params }))
    })
  }
}

function waitEvent(cdp, method, timeoutMs = 20000) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`event timeout: ${method}`)), timeoutMs)
    cdp.on(method, params => {
      clearTimeout(timer)
      resolve(params)
    })
  })
}

async function evalJs(cdp, expression, awaitPromise = false) {
  const res = await cdp.send('Runtime.evaluate', {
    expression,
    awaitPromise,
    returnByValue: true,
  })
  if (res.exceptionDetails) {
    throw new Error(`page eval failed: ${res.exceptionDetails.exception?.description ?? res.exceptionDetails.text}`)
  }
  return res.result?.value
}

async function main() {
  const wsUrl = await waitDevtools()
  const ws = new WebSocket(wsUrl)
  await new Promise((resolve, reject) => {
    ws.addEventListener('open', resolve)
    ws.addEventListener('error', reject)
  })
  const cdp = new CDP(ws)
  cdp.on('Runtime.exceptionThrown', p => {
    const d = p.exceptionDetails
    console.warn('[page error]', d.exception?.description ?? d.text)
  })

  await cdp.send('Page.enable')
  await cdp.send('Runtime.enable')
  await cdp.send('Emulation.setDeviceMetricsOverride', {
    width: 1600,
    height: 1000,
    deviceScaleFactor: 2,
    mobile: false,
  })

  let failures = 0
  for (const shot of SHOTS) {
    const target = path.join(OUT, `${shot.name}.png`)
    console.log(`-> ${shot.url}`)
    try {
      const loaded = waitEvent(cdp, 'Page.loadEventFired', 25000)
      await cdp.send('Page.navigate', { url: BASE + shot.url })
      await loaded
      // readiness: route predicate, then fonts + every image settled
      const readyAt = Date.now() + 30000
      for (;;) {
        const ok = await evalJs(cdp, shot.ready).catch(() => false)
        if (ok) break
        if (Date.now() > readyAt) throw new Error('readiness timeout')
        await sleep(250)
      }
      await evalJs(
        cdp,
        `Promise.all([document.fonts.ready, ...Array.from(document.images).map(img => img.complete ? 0 : new Promise(r => { img.onload = img.onerror = r }))]).then(() => true)`,
        true
      )
      if (shot.post) {
        await evalJs(cdp, shot.post)
        await sleep(500)
      }
      await sleep(900) // let transitions/hover states settle
      const shotData = await cdp.send('Page.captureScreenshot', {
        format: 'png',
        captureBeyondViewport: false,
      })
      fs.writeFileSync(target, Buffer.from(shotData.data, 'base64'))
      console.log(`   ok  ${shot.name}.png  (${fs.statSync(target).size} bytes)`)
    } catch (err) {
      failures++
      console.warn(`   FAIL ${shot.name}: ${err.message}`)
    }
  }

  try {
    await cdp.send('Browser.close')
  } catch {
    /* already gone */
  }
  child.kill()
  if (failures) process.exitCode = 1
}

main().catch(err => {
  console.error(err)
  child.kill()
  process.exitCode = 1
})
