// Exercise the group coordinator with real Vue reactivity and controlled I/O.
// No native processes, credentials, model requests, or application data are used.
const assert = require('node:assert/strict')
const crypto = require('node:crypto').webcrypto
const fs = require('node:fs')
const vm = require('node:vm')
const ts = require('typescript')
const vue = require('vue')

function load(file, mocks = {}, extra = {}) {
  const exports = {}
  vm.runInNewContext(ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText, {
    exports, require: name => {
      if (!(name in mocks)) throw Error(`Missing mock: ${name}`)
      return mocks[name]
    }, console, AbortController, setTimeout, clearTimeout, setInterval, clearInterval,
    TextEncoder, crypto, ...extra,
  }, { filename: file })
  return exports
}

const copy = value => JSON.parse(JSON.stringify(value))
const types = load('src/types/agents.ts')
const paths = load('src/utils/path.ts')
const english = load('src/i18n/locales/agents-en.ts').default
const t = (key, params = {}) => {
  let value = key.split('.').slice(1).reduce((node, part) => node?.[part], english)
  assert.equal(typeof value, 'string', `Missing locale: ${key}`)
  for (const [name, argument] of Object.entries(params)) value = value.replaceAll(`{${name}}`, String(argument))
  return value
}
const tick = () => new Promise(resolve => setImmediate(resolve))
async function until(predicate) {
  for (let tries = 0; tries < 80; tries++) {
    if (predicate()) return
    await tick()
  }
  assert.fail('Coordinator did not reach expected state')
}
function deferred() {
  let resolve, reject
  const promise = new Promise((ok, fail) => { resolve = ok; reject = fail })
  return { promise, resolve, reject }
}
function result(text = 'done', nativeSessionId) {
  return { text, reasoning: '', toolCount: 0, enabledToolNames: [], nativeSessionId }
}

function fixture({ saved = [], driverStart, timerOverrides = {} } = {}) {
  const database = new Map(saved.map(group => [group.id, copy(group)]))
  const events = []
  const starts = []
  const stoppedProcesses = []
  const writeState = { fail: false, block: null, removeBlock: null }
  const engines = vue.reactive(['a', 'b'].map((id, index) => ({
    id: `${id}@windows`, name: `Agent ${id.toUpperCase()}`, location: 'Windows', tier: 'A',
    agentId: index ? 'codex' : 'claude-code', host: 'windows',
    installation: { status: 'ready' },
  })))
  const hub = vue.reactive({ engines, scanning: false, error: null, async detect() {} })
  const driver = {
    capabilities: { groupChatModes: ['discussion'], usesJediKey: false },
    async start(options, hooks) {
      const done = deferred()
      const handle = {
        done: done.promise, capabilities: this.capabilities, engineId: options.engine.id,
        abort() { done.resolve(result()) }, dispose() {},
      }
      const start = { options, hooks, done, handle }
      starts.push(start)
      return driverStart ? driverStart(start) : handle
    },
  }
  const api = {
    async listAgentGroups() { return copy([...database.values()]) },
    async saveAgentGroup(group) {
      events.push('save')
      if (writeState.block) await writeState.block.promise
      if (writeState.fail) throw Error('Disk unavailable')
      const savedGroup = { ...copy(group), revision: (database.get(group.id)?.revision ?? 0) + 1, updatedAt: new Date().toISOString() }
      database.set(group.id, savedGroup)
      return copy(savedGroup)
    },
    async deleteAgentGroup(id) {
      events.push('delete')
      database.delete(id)
      if (writeState.removeBlock) await writeState.removeBlock.promise
    },
    async stopAgentGroupProcess(id) { stoppedProcesses.push(id) },
  }
  const source = load('src/stores/agents.ts', {
    vue, '@/utils/path': paths, '@/api/agents': api,
    '@/agent/hub/registry': { driverFor: () => driver },
    '@/agent/runAgent': { summarizeValue: (value, limit) => JSON.stringify(value)?.slice(0, limit) ?? '' },
    '@/agent/statsHooks': { withStatsHooks: hooks => hooks },
    '@/i18n': { default: { global: { t } } },
    '@/stores/agentHub': { useAgentHubStore: () => hub },
    '@/types/agents': types,
  }, timerOverrides)
  const store = source.useAgentsStore()
  return { store, database, events, starts, stoppedProcesses, writeState, engines, hub }
}
async function create(f, ids = ['a', 'b']) {
  await f.store.init()
  return f.store.createGroup({
    name: 'Test project', projectPath: 'E:\\projects\\test', executionMode: 'discussion',
    members: ids.map(id => ({ engineId: `${id}@windows`, role: 'Reviewer' })),
  })
}
async function settleAll(f) {
  for (const start of f.starts) start.done.resolve(result())
  await until(() => f.store.groups.every(group => group.runs.every(run => !types.ACTIVE_AGENT_RUN_STATUSES.has(run.status))))
  await until(() => !f.store.saving)
}
async function test(name, run) {
  await run()
  console.log('PASS agents group', name)
}

;(async () => {
  await test('lazy singleton and typed readiness', async () => {
    const f = fixture()
    await create(f, ['a'])
    assert.equal(f.store.groups.length, 1)
    assert.equal(f.store.currentGroup.name, 'Test project')
    f.engines[0].installation.status = 'needs_auth'
    assert.equal(f.store.engineOptions[0].available, false)
    await assert.rejects(f.store.sendMessage('hello'), /unavailable/)
    assert.equal(f.starts.length, 0)
    assert.equal(f.store.currentGroup.messages.length, 0)
  })

  await test('broadcast, targeted context, and independent stop', async () => {
    const f = fixture()
    const group = await create(f)
    await f.store.sendMessage('Remember ORCHID')
    await until(() => f.starts.length === 2)
    assert.equal(new Set(f.starts.map(start => start.options.processId)).size, 2)
    assert.ok(f.starts.every(start => start.options.cwd === group.projectPath && start.options.executionMode === 'discussion'))
    const a = f.starts[0], b = f.starts[1]
    a.hooks.onTextDelta({ fullContent: 'A partial', delta: 'A partial' })
    b.hooks.onNativeSessionId('b-session')
    b.done.resolve(result('B complete', 'b-session'))
    await until(() => group.runs[1].status === 'completed')
    await f.store.sendMessage('B follow-up while A is running', [group.members[1].id])
    await until(() => f.starts.length === 3)
    const secondB = f.starts[2]
    assert.ok(!secondB.options.messages[0].content.includes('A partial'))
    assert.equal(secondB.options.nativeSessionId, 'b-session')
    a.hooks.onTextDelta({ fullContent: 'A final important answer', delta: ' final' })
    a.done.resolve(result('A final important answer', 'a-session'))
    secondB.done.resolve(result('B follow-up done', 'b-session'))
    await until(() => group.runs.every(run => run.status === 'completed'))
    await f.store.sendMessage('B read final shared reply', [group.members[1].id])
    await until(() => f.starts.length === 4)
    assert.ok(f.starts[3].options.messages[0].content.includes('A final important answer'))
    assert.ok(f.starts[3].options.messages[0].content.includes('Remember ORCHID'))
    f.store.stopMember(group.members[1].id)
    assert.equal(f.starts[3].options.signal.aborted, true)
    const before = group.messages.at(-1).content
    f.starts[3].hooks.onTextDelta({ fullContent: 'LATE DATA', delta: 'LATE DATA' })
    f.starts[3].hooks.onNativeSessionId('late-session')
    assert.equal(group.messages.at(-1).content, before)
    await settleAll(f)
    assert.equal(group.runs.at(-1).status, 'stopped')
    assert.equal(group.members[1].nativeSessionId, undefined)
  })

  await test('failed dispatch rolls back and retries normally', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    f.writeState.fail = true
    await assert.rejects(f.store.sendMessage('keep this draft'), /Disk unavailable/)
    assert.equal(group.messages.length, 0)
    assert.equal(group.runs.length, 0)
    assert.equal(f.starts.length, 0)
    assert.ok(f.store.error)
    f.writeState.fail = false
    await f.store.retrySave()
    await f.store.sendMessage('keep this draft')
    await until(() => f.starts.length === 1)
    await settleAll(f)
    assert.equal(group.runs[0].status, 'completed')
  })

  await test('history cap rejects before mutating state', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    group.runs.push(...Array.from({ length: 500 }, (_, i) => ({ id: `old-${i}`, status: 'completed', tools: [] })))
    await assert.rejects(f.store.sendMessage('too many'), /history limit/)
    assert.equal(group.messages.length, 0)
    assert.equal(group.runs.length, 500)
    assert.equal(f.starts.length, 0)
  })

  await test('pause preserves active run and excludes new broadcast', async () => {
    const f = fixture()
    const group = await create(f)
    await f.store.sendMessage('only A', [group.members[0].id])
    await until(() => f.starts.length === 1)
    await f.store.toggleMember(group.members[0].id)
    assert.equal(group.members[0].enabled, false)
    assert.equal(f.starts[0].options.signal.aborted, false)
    await f.store.sendMessage('enabled only')
    await until(() => f.starts.length === 2)
    assert.equal(f.starts[1].options.engine.id, 'b@windows')
    f.store.stopMember(group.members[0].id)
    assert.equal(f.starts[1].options.signal.aborted, false)
    await settleAll(f)
  })

  await test('delete cannot be undone by final or late checkpoints', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    await f.store.sendMessage('start')
    await until(() => f.starts.length === 1 && !f.store.saving)
    f.writeState.removeBlock = deferred()
    const deletion = f.store.deleteGroup(group.id)
    await until(() => f.events.includes('delete'))
    f.starts[0].hooks.onTextDelta({ fullContent: 'late', delta: 'late' })
    f.starts[0].done.resolve(result('late', 'late-session'))
    await tick()
    f.writeState.removeBlock.resolve()
    await deletion
    await tick()
    assert.equal(f.store.groups.length, 0)
    assert.equal(f.database.has(group.id), false)
    assert.equal(f.events.at(-1), 'delete')
  })

  await test('stop during driver startup and queued stop', async () => {
    const gate = deferred()
    const f = fixture({ driverStart: start => gate.promise.then(() => start.handle) })
    const group = await create(f, ['a'])
    await f.store.sendMessage('starting')
    await until(() => f.starts.length === 1)
    f.store.stopMember(group.members[0].id)
    await until(() => !f.store.saving)
    assert.equal(group.runs[0].status, 'stopped')
    gate.resolve()
    await tick()
    await settleAll(f)
    const g = fixture()
    const queued = await create(g, ['a'])
    const sending = g.store.sendMessage('queued')
    g.store.stopMember(queued.members[0].id)
    await sending
    await until(() => !g.store.saving)
    assert.equal(queued.runs[0].status, 'stopped')
    assert.equal(g.starts.length, 0)
  })

  await test('full current request and failed-session replay', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    const request = 'x'.repeat(19_960) + ' TAIL_CONSTRAINT'
    await f.store.sendMessage(request)
    await until(() => f.starts.length === 1)
    assert.ok(f.starts[0].options.messages[0].content.includes('TAIL_CONSTRAINT'))
    f.starts[0].done.reject(Error('startup failed'))
    await until(() => group.runs[0].status === 'failed' && !f.store.saving)
    await f.store.sendMessage('follow up')
    await until(() => f.starts.length === 2)
    assert.ok(f.starts[1].options.messages[0].content.includes('TAIL_CONSTRAINT'))
    assert.equal(f.starts[1].options.nativeSessionId, undefined)
    await settleAll(f)
  })

  await test('restart recovery preserves partials without restarting work', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    await f.store.sendMessage('recovery task')
    await until(() => f.starts.length === 1)
    f.starts[0].hooks.onTextDelta({ fullContent: 'partial durable answer', delta: 'partial durable answer' })
    f.starts[0].hooks.onNativeSessionId('old-session')
    await f.store.retrySave()
    const saved = copy([...f.database.values()])
    f.store.stopGroup()
    await settleAll(f)
    const restarted = fixture({ saved })
    await restarted.store.init()
    await until(() => !restarted.store.saving)
    const restored = restarted.store.currentGroup
    assert.equal(restored.runs[0].status, 'interrupted')
    assert.equal(restored.messages[1].content, 'partial durable answer')
    assert.equal(restored.members[0].nativeSessionId, undefined)
    assert.equal(restarted.stoppedProcesses[0], restored.runs[0].id)
    assert.equal(restarted.starts.length, 0)
  })

  await test('retry targets only the failed member and preserves history', async () => {
    const f = fixture()
    const group = await create(f)
    await f.store.sendMessage('Original request')
    await until(() => f.starts.length === 2)
    f.starts[0].hooks.onTextDelta({ fullContent: 'Partial reply', delta: 'Partial reply' })
    f.starts[0].done.reject(Error('Connection lost'))
    f.starts[1].done.resolve(result('B reply'))
    await until(() => !f.store.saving && group.runs.every(run => !types.ACTIVE_AGENT_RUN_STATUSES.has(run.status)))
    const failed = group.runs[0]
    const trigger = group.messages[0]
    await f.store.sendMessage('Later shared context', [group.members[1].id])
    await until(() => f.starts.length === 3)
    f.starts[2].done.resolve(result('Later B reply'))
    await until(() => !f.store.saving && group.runs[2].status === 'completed')
    const count = group.messages.length
    const other = await f.store.createGroup({ name: 'Other', projectPath: 'E:\\other', executionMode: 'discussion', members: [{ engineId: 'a@windows', role: 'Reviewer' }] })
    await f.store.retryRun(group.id, failed.id)
    await until(() => f.starts.length === 4)
    assert.equal(f.store.currentGroup.id, other.id)
    assert.equal(group.messages.length, count + 1)
    assert.equal(group.messages.filter(message => message.id === trigger.id).length, 1)
    assert.equal(group.messages[1].content, 'Partial reply')
    assert.equal(failed.status, 'failed')
    assert.equal(group.runs.at(-1).triggerMessageId, trigger.id)
    assert.equal(f.starts[3].options.engine.id, 'a@windows')
    assert.ok(f.starts[3].options.messages[0].content.includes('Later B reply'))
    await assert.rejects(f.store.retryRun(group.id, failed.id), /still working/)
    await settleAll(f)
    await assert.rejects(f.store.retryRun(group.id, group.runs[1].id), /cannot be retried/)
    const restored = fixture({ saved: copy([...f.database.values()]) })
    await restored.store.init()
    const history = restored.store.groups.find(item => item.id === group.id)
    assert.equal(history.runs.at(-1).status, 'completed')
    assert.equal(history.runs.at(-1).triggerMessageId, trigger.id)
    assert.equal(history.messages[1].content, 'Partial reply')
    assert.equal(restored.starts.length, 0)
  })

  await test('retry keeps an old triggering request beyond the context window', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    await f.store.sendMessage('Old request TAIL_CONSTRAINT')
    await until(() => f.starts.length === 1)
    f.starts[0].done.reject(Error('Connection lost'))
    await until(() => group.runs[0].status === 'failed' && !f.store.saving)
    for (let index = 0; index < 100; index++) {
      group.messages.push({ id: `recent-${index}`, seq: index + 3, role: 'user', authorName: 'user', recipients: [group.members[0].id], content: `Recent context ${index}`, createdAt: group.createdAt })
    }
    await f.store.retryRun(group.id, group.runs[0].id)
    await until(() => f.starts.length === 2)
    const prompt = f.starts[1].options.messages[0].content
    const snapshot = JSON.parse(prompt.split('\n\n').find(part => part.startsWith('[{')))
    assert.ok(snapshot.length <= 80)
    assert.equal(snapshot[0].content, 'Old request TAIL_CONSTRAINT')
    assert.equal(snapshot.at(-1).content, 'Recent context 99')
    await settleAll(f)
  })

  await test('retry rollback and native terminal status', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    await f.store.sendMessage('Fail without throwing')
    await until(() => f.starts.length === 1)
    f.starts[0].done.resolve({ ...result('partial'), finishReason: 'error' })
    await until(() => group.runs[0].status === 'failed' && !f.store.saving)
    const old = copy(group.messages)
    f.writeState.fail = true
    await assert.rejects(f.store.retryRun(group.id, group.runs[0].id), /Disk unavailable/)
    assert.equal(JSON.stringify(group.messages), JSON.stringify(old))
    assert.equal(f.starts.length, 1)
    assert.ok(f.store.currentSaveError)
    f.writeState.fail = false
    await f.store.retrySave()
    assert.equal(f.store.currentSaveError, null)
    await f.store.retryRun(group.id, group.runs[0].id)
    await until(() => f.starts.length === 2)
    f.starts[1].done.resolve({ ...result('interrupted'), finishReason: 'aborted' })
    await until(() => group.runs.at(-1).status === 'interrupted' && !f.store.saving)
    assert.equal(group.members[0].nativeSessionId, undefined)
  })

  await test('bounded output and tool settlement', async () => {
    const f = fixture()
    const group = await create(f, ['a'])
    await f.store.sendMessage('large answer')
    await until(() => f.starts.length === 1)
    const start = f.starts[0]
    start.hooks.onToolStart({ toolId: 'read-1', toolName: 'Read', toolDescription: 'Read file', startedAt: 1 })
    start.hooks.onToolEnd({ toolId: 'read-1', toolName: 'Read', startedAt: 1, output: { found: true } })
    start.hooks.onToolStart({ toolId: 'read-2', toolName: 'Read', toolDescription: 'More file', startedAt: 2 })
    start.hooks.onTextDelta({ fullContent: 'A'.repeat(100_000), delta: 'A' })
    start.done.resolve(result('A'.repeat(100_000)))
    await until(() => group.runs[0].status === 'completed' && !f.store.saving)
    assert.equal(group.messages[1].content.length, 32_000)
    assert.match(group.runs[0].error, /truncated/)
    assert.equal(group.runs[0].tools.length, 2)
    assert.equal(group.runs[0].tools[0].status, 'completed')
    assert.equal(group.runs[0].tools[1].status, 'failed')
    assert.match(group.runs[0].tools[0].detail, /found/)
    assert.ok(new TextEncoder().encode(JSON.stringify(f.database.get(group.id))).byteLength < 4 * 1024 * 1024)
  })
})().catch(error => { console.error(error); process.exitCode = 1 })
