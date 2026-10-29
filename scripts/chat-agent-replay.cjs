// Exercise the chat store with real Vue/Pinia and a controlled external driver.
const assert = require('node:assert/strict')
const fs = require('node:fs')
const vm = require('node:vm')
const ts = require('typescript')
const { createPinia, setActivePinia } = require('pinia')

function load(file, mocks) {
  const exports = {}
  vm.runInNewContext(ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText, {
    exports, require: name => mocks[name], console: { ...console, error() {} },
    AbortController, setInterval, clearInterval,
  })
  return exports
}

async function run(mode) {
  setActivePinia(createPinia())
  const persisted = []
  let store
  let receivedSession
  const engine = { id: 'codex@windows', tier: 'A', name: 'Codex' }
  const diagnostics = load('src/agent/hub/agentError.ts', {})
  const { agentError } = diagnostics
  const driver = {
    capabilities: { usesJediKey: false },
    async start(spec, hooks) {
      receivedSession = spec.nativeSessionId
      hooks.onTextDelta({ fullContent: 'partial answer', delta: 'partial answer' })
      if (mode === 'stop') store.stopGeneration()
      return {
        capabilities: {},
        done: mode === 'error'
          ? Promise.reject(agentError('network', 'Codex', 'connection reset'))
          : Promise.resolve({ text: 'partial answer', nativeSessionId: 'new-thread' }),
      }
    },
  }
  const noop = () => {}
  const { useAiChatStore } = load('src/stores/aiChat.ts', {
    pinia: require('pinia'), vue: require('vue'),
    '@tauri-apps/api/core': { invoke: async (cmd, args) => {
      if (cmd === 'append_message') persisted.push(args.request)
    } },
    '@/agent/hub/agentError': diagnostics,
    '@/agent/hub/registry': { driverFor: () => driver },
    '@/agent/runAgent': {
      DEFAULT_STEP_LIMIT: 10, summarizeValue: value => String(value),
    },
    '@/agent/statsHooks': { withStatsHooks: hooks => hooks },
    '@/agent/tools/registry': { toolRegistry: { listEnabled: () => [] } },
    './agent': { useAgentStore: () => ({
      reset: noop, setStatus: noop, startStep: () => ({}), completeStep: noop,
    }) },
    './agentHub': { useAgentHubStore: () => ({ selectedEngine: engine }) },
    './modelsDev': { useModelsDevStore: () => ({}) },
    './providerConfig': { useProviderConfigStore: () => ({}) },
  })
  store = useAiChatStore()
  const messages = [
    { role: 'assistant', content: 'older', metadata: { run: {
      engine: 'codex@windows', nativeSessionId: 'windows-thread',
    } } },
    { role: 'assistant', content: 'newer', metadata: { run: {
      engine: 'codex@wsl:Ubuntu', nativeSessionId: 'wsl-thread',
    } } },
    { role: 'assistant', content: 'latest', metadata: { run: {
      engine: 'claude-code@windows', nativeSessionId: 'claude-session',
    } } },
  ]
  if (mode === 'fresh') messages.shift()
  store.sessions = [{ id: 'chat', title: 'Existing chat', messages }]
  store.currentSessionId = 'chat'
  const send = store.sendMessage('hello')
  if (mode === 'error') await assert.rejects(send)
  else await send
  assert.equal(receivedSession, mode === 'fresh' ? undefined : 'windows-thread')
  const answer = store.currentSession.messages.at(-1)
  assert.equal(answer.metadata.run.phase, mode === 'stop' ? 'stopped' : mode === 'error' ? 'failed' : 'done')
  assert.ok(answer.content.startsWith('partial answer'))
  assert.equal(answer.isStreaming, false)
  assert.equal(store.isLoading, false)
  const saved = persisted.at(-1)
  assert.equal(saved.content, answer.content)
  assert.equal(JSON.parse(saved.metadata).run.phase, answer.metadata.run.phase)
  if (mode === 'error') {
    assert.ok(answer.error)
    assert.equal(answer.metadata.run.errorKind, 'network')
    assert.ok(answer.content.includes('\n\n'))
  }
  console.log('PASS chat agent', mode)
}

;(async () => {
  for (const mode of ['resume', 'fresh', 'stop', 'error']) await run(mode)
})().catch(error => { console.error(error); process.exitCode = 1 })
