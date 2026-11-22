import { afterEach, describe, expect, test } from 'bun:test'
import type { Channel } from '@tauri-apps/api/core'
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks'
import { ClaudeCodeDriver } from '../src/agent/hub/drivers/claudeCode'
import { CodexDriver } from '../src/agent/hub/drivers/codex'
import { codexApprovalDecision, codexMcpConfig } from '../src/agent/hub/drivers/groupPolicy'
import type { DriverStartOpts, EngineDescriptor } from '../src/agent/hub/types'

const cwd = String.raw`E:\workspace\project with spaces\it's here`
const wslCwd = "/custom/drives/e/workspace/project with spaces/it's here"

type Agent = 'claude-code' | 'codex'
type Wire = Record<string, any>
type Launch = { id: string; args: string[]; cwd: string | null; onEvent: Channel<Wire> }

function options(agentId: Agent, overrides: Partial<DriverStartOpts> = {}): DriverStartOpts {
  const engine: EngineDescriptor = {
    id: `${agentId}@wsl:ArchLinux`,
    agentId,
    name: agentId,
    location: 'ArchLinux',
    host: 'wsl:ArchLinux',
    tier: 'A',
    installation: {
      agentId,
      host: 'wsl:ArchLinux',
      exe: `/usr/bin/${agentId}`,
      spawn: { kind: 'wsl', distro: 'ArchLinux', exe: `/usr/bin/${agentId}` },
    },
  }
  return {
    provider: '',
    apiKey: '',
    model: '',
    sessionId: crypto.randomUUID(),
    messages: [{ role: 'user', content: 'offline replay' }],
    engine,
    ...overrides,
  }
}

function line(launch: Launch, record: unknown) {
  launch.onEvent.onmessage({ type: 'line', line: JSON.stringify(record) })
}

function fakeIpc(handler: (command: string, args: Wire) => unknown) {
  Object.defineProperty(globalThis, 'window', {
    configurable: true,
    value: { crypto },
  })
  mockIPC((command, args) => handler(command, args as Wire))
}

afterEach(() => {
  if (typeof window !== 'undefined') clearMocks()
  Reflect.deleteProperty(globalThis, 'window')
})

const restrictedInit = {
  type: 'system',
  subtype: 'init',
  session_id: 'claude-scoped',
  permissionMode: 'dontAsk',
  tools: ['Read', 'Glob', 'Grep'],
}
const claudeResult = { type: 'result', subtype: 'success', result: 'done', session_id: 'claude-scoped' }
const commandApproval = 'item/commandExecution/requestApproval'
const fileApproval = 'item/fileChange/requestApproval'
const approvalParams = { threadId: 'thread-test', turnId: 'turn-test', itemId: 'change-1' }

function codexIpc(policyOverrides: Wire = {}) {
  const launches: Launch[] = []
  const writes: Wire[] = []
  const kills: string[] = []
  let app: Launch
  fakeIpc((command, args) => {
    if (command === 'agent_proc_spawn') {
      const launch = args as Launch
      launches.push(launch)
      if (launch.args[0] === 'app-server') {
        app = launch
      } else {
        expect(launch.args.slice(-3)).toEqual(['mcp', 'list', '--json'])
        line(launch, [{ name: 'user_mcp', enabled: true }])
        launch.onEvent.onmessage({ type: 'exit', code: 0 })
      }
      return launch.cwd === null ? null : wslCwd
    }
    if (command === 'agent_proc_kill') {
      kills.push(args.id)
      return
    }
    if (command === 'agent_proc_stdin') {
      const request = JSON.parse(args.line)
      writes.push(request)
      if (request.id === undefined || !request.method) return
      let result = {}
      if (request.method === 'thread/start' || request.method === 'thread/resume') {
        result = {
          thread: { id: 'thread-test' },
          approvalPolicy: 'never',
          approvalsReviewer: 'user',
          sandbox: { type: 'readOnly', networkAccess: false },
          cwd: wslCwd,
          ...policyOverrides,
        }
      }
      if (request.method === 'turn/start') result = { turn: { id: 'turn-test' } }
      line(app, { id: request.id, result })
    }
  })
  return { launches, writes, kills, record: (record: Wire) => line(app, record) }
}

describe('Claude Code driver', () => {
  test('opts in to explicit read-only tools and only resumes a matching group ID', async () => {
    const launches: Launch[] = []
    const nativeIds: string[] = []
    const kills: string[] = []
    fakeIpc((command, args) => {
      if (command === 'agent_proc_spawn') launches.push(args as Launch)
      if (command === 'agent_proc_kill') kills.push(args.id)
      return cwd
    })
    const driver = new ClaudeCodeDriver()
    const opts = options('claude-code')
    const legacy = await driver.start({ ...opts, nativeSessionId: 'legacy' }, {})
    expect(launches[0].cwd).toBeNull()
    expect(launches[0].args.slice(-4)).toEqual(['--permission-mode', 'default', '--resume', 'legacy'])
    expect(launches[0].args).not.toContain('--tools')
    legacy.abort?.()
    await legacy.done

    const group = { ...opts, cwd, executionMode: 'discussion' } as const
    const first = await driver.start(
      { ...group, nativeSessionId: 'legacy', processId: 'jedi-agents-stable' },
      { onNativeSessionId: id => nativeIds.push(id) }
    )
    expect(launches[1].id).toBe('jedi-agents-stable')
    expect(launches[1].cwd).toBe(cwd)
    expect(launches[1].args).not.toContain('--resume')
    for (const [flag, value] of [
      ['--tools', 'Read,Glob,Grep'],
      ['--allowedTools', 'Read,Glob,Grep'],
      ['--permission-mode', 'dontAsk'],
      ['--permission-prompts', 'none'],
      ['--mcp-config', '{"mcpServers":{}}'],
    ]) {
      const index = launches[1].args.indexOf(flag)
      expect(index).toBeGreaterThan(-1)
      expect(launches[1].args[index + 1]).toBe(value)
    }
    expect(launches[1].args).toContain('--strict-mcp-config')
    expect(launches[1].args).toContain('--safe-mode')
    expect(launches[1].args).toContain('--restricted')
    line(launches[1], restrictedInit)
    expect(nativeIds).toEqual(['claude-scoped'])
    line(launches[1], claudeResult)
    await expect(first.done).resolves.toMatchObject({ nativeSessionId: 'claude-scoped' })
    expect(nativeIds).toEqual(['claude-scoped'])
    expect(kills).toContain('jedi-agents-stable')

    const resumed = await driver.start({ ...group, nativeSessionId: 'claude-scoped' }, {})
    expect(launches[2].args.slice(-2)).toEqual(['--resume', 'claude-scoped'])
    resumed.abort?.()
    await resumed.done
    const clean = await driver.start(group, {})
    expect(launches[3].args).not.toContain('--resume')
    expect(launches[3].id).not.toBe(launches[2].id)
    clean.abort?.()
    await clean.done
    const moved = await driver.start({ ...group, cwd: '/other', nativeSessionId: 'claude-scoped' }, {})
    expect(launches[4].args).not.toContain('--resume')
    moved.abort?.()
    await moved.done
  })

  test('cancel during spawn kills the spawned process without sending a prompt', async () => {
    const controller = new AbortController()
    let resolveSpawn!: (value: string) => void
    let created!: Launch
    const calls: string[] = []
    fakeIpc((command, args) => {
      calls.push(command)
      if (command === 'agent_proc_spawn') {
        created = args as Launch
        return new Promise<string>(resolve => { resolveSpawn = resolve })
      }
    })
    let hookCount = 0
    const starting = new ClaudeCodeDriver().start(
      options('claude-code', { signal: controller.signal }),
      { onTextDelta: () => { hookCount++ }, onNativeSessionId: () => { hookCount++ } }
    )
    controller.abort()
    line(created, claudeResult)
    resolveSpawn(cwd)
    const handle = await starting
    await expect(handle.done).resolves.toMatchObject({ text: '', finishReason: 'aborted' })
    expect(calls).toEqual(['agent_proc_spawn', 'agent_proc_kill'])
    expect(hookCount).toBe(0)
  })

  test('discussion fails closed if CLI init exposes a forbidden tool', async () => {
    let launch!: Launch
    fakeIpc((command, args) => {
      if (command === 'agent_proc_spawn') launch = args as Launch
      return wslCwd
    })
    const nativeIds: string[] = []
    const handle = await new ClaudeCodeDriver().start(
      options('claude-code', { cwd, executionMode: 'discussion' }),
      { onNativeSessionId: id => nativeIds.push(id) }
    )
    line(launch, { ...restrictedInit, tools: ['Read', 'Bash'] })
    await expect(handle.done).rejects.toThrow('did not confirm discussion tool restrictions')
    expect(nativeIds).toEqual([])
  })
})

describe('Codex driver', () => {
  test('applies the canonical WSL cwd and read-only/never policy on start, resume, and turn', async () => {
    const ipc = codexIpc()
    const nativeIds: string[] = []
    const driver = new CodexDriver()
    const opts = options('codex', { cwd, executionMode: 'discussion', processId: 'jedi-agents-codex' })
    const handle = await driver.start({ ...opts, nativeSessionId: 'legacy' }, {
      onNativeSessionId: id => nativeIds.push(id),
    })
    expect(nativeIds).toEqual(['thread-test'])
    expect(ipc.launches[0].id).not.toBe(opts.processId)
    expect(ipc.launches[1]).toMatchObject({ id: opts.processId, cwd })
    expect(ipc.launches[1].args).toContain('approval_policy="never"')
    expect(ipc.launches[1].args).toContain('sandbox_mode="read-only"')
    expect(ipc.launches[1].args).toContain('mcp_servers.user_mcp.enabled=false')
    expect(ipc.writes.find(r => r.method === 'thread/start')?.params).toMatchObject({
      cwd: wslCwd, sandbox: 'read-only', approvalPolicy: 'never', approvalsReviewer: 'user',
    })
    expect(ipc.writes.find(r => r.method === 'turn/start')?.params).toMatchObject({
      cwd: wslCwd, sandboxPolicy: { type: 'readOnly', networkAccess: false }, approvalPolicy: 'never',
    })
    ipc.record({ method: 'turn/completed', params: { turn: { status: 'completed' } } })
    await expect(handle.done).resolves.toMatchObject({ nativeSessionId: 'thread-test' })
    const resumed = await driver.start({ ...opts, nativeSessionId: 'thread-test' }, {})
    expect(ipc.writes.find(r => r.method === 'thread/resume')?.params).toMatchObject({
      threadId: 'thread-test', cwd: wslCwd, sandbox: 'read-only', approvalPolicy: 'never',
    })
    resumed.abort?.()
    await resumed.done
    const fresh = await driver.start(opts, {})
    expect(ipc.writes.filter(r => r.method === 'thread/start')).toHaveLength(2)
    fresh.abort?.()
    await fresh.done
  })

  test('rejects an unsupported mode and pre-aborted signal before any IPC', async () => {
    fakeIpc(command => { throw new Error(`unexpected IPC: ${command}`) })
    const driver = new CodexDriver()
    expect(driver.capabilities.groupChatModes).toEqual(['discussion'])
    await expect(driver.start(options('codex', { cwd, executionMode: 'approval' }), {})).rejects.toThrow('Unsupported group execution mode')
    await expect(driver.start(options('codex', { executionMode: 'discussion' }), {})).rejects.toThrow('explicit working directory')
    await expect(driver.start(options('codex', { signal: AbortSignal.abort() }), {})).rejects.toThrow('aborted')
  })

  test('never starts a model turn after a mismatched native sandbox response', async () => {
    const ipc = codexIpc({ sandbox: { type: 'dangerFullAccess' } })
    await expect(new CodexDriver().start(options('codex', { cwd, executionMode: 'discussion' }), {})).rejects.toThrow('did not confirm')
    expect(ipc.writes.some(r => r.method === 'turn/start')).toBe(false)
    expect(ipc.kills).toContain(ipc.launches[1].id)
  })

  test('denies server requests in discussion without invoking human approval', async () => {
    const ipc = codexIpc()
    const handle = await new CodexDriver().start(options('codex', {
      cwd, executionMode: 'discussion', confirmTool: async () => { throw new Error('must stay dormant') },
    }), {})
    ipc.record({ id: 1, method: commandApproval, params: { ...approvalParams, command: 'test' } })
    ipc.record({ id: 'extension', method: 'item/permissions/requestApproval', params: approvalParams })
    await Promise.resolve()
    expect(ipc.writes).toContainEqual({ id: 1, result: { decision: 'decline' } })
    expect(ipc.writes.find(r => r.id === 'extension')?.error.code).toBe(-32601)
    handle.abort?.()
    await handle.done
  })

  test('legacy approval is a single-use response and an abort discards a pending decision', async () => {
    const ipc = codexIpc()
    let confirmations = 0
    let release!: (approved: boolean) => void
    const handle = await new CodexDriver().start(options('codex', {
      confirmTool: () => {
        confirmations++
        return confirmations === 1 ? Promise.resolve(true) : new Promise(resolve => { release = resolve })
      },
    }), {})
    expect(ipc.launches[0].args).toEqual(['app-server'])
    expect(ipc.launches[0].cwd).toBeNull()
    expect(ipc.writes.find(r => r.method === 'thread/start')?.params).toEqual({})
    const request = { id: 'approval-1', method: commandApproval, params: { ...approvalParams, command: 'test' } }
    ipc.record(request)
    ipc.record(request)
    await Promise.resolve()
    await Promise.resolve()
    expect(confirmations).toBe(1)
    expect(ipc.writes.filter(r => r.id === 'approval-1')).toEqual([{ id: 'approval-1', result: { decision: 'accept' } }])
    ipc.record({ ...request, id: 'approval-2' })
    expect(confirmations).toBe(2)
    handle.abort?.()
    release(true)
    await expect(handle.done).resolves.toMatchObject({ finishReason: 'aborted' })
    ipc.record({ method: 'item/agentMessage/delta', params: { delta: 'late' } })
    expect(ipc.writes.some(r => r.id === 'approval-2')).toBe(false)
  })
})

test('approval decisions fail closed for exceptions, missing changes, and root grants', async () => {
  const req = { id: 'patch', method: fileApproval, params: approvalParams }
  const context = { ...approvalParams, active: () => true, item: { id: 'change-1', type: 'fileChange' } }
  expect(await codexApprovalDecision(req, { confirmTool: async () => true }, context)).toBe('accept')
  expect(await codexApprovalDecision(req, {}, context)).toBe('decline')
  expect(await codexApprovalDecision(req, { confirmTool: async () => { throw new Error('closed') } }, context)).toBe('decline')
  expect(await codexApprovalDecision(req, { confirmTool: async () => true }, { ...context, item: undefined })).toBe('decline')
  expect(await codexApprovalDecision({ ...req, params: { ...approvalParams, grantRoot: cwd } }, { confirmTool: async () => true }, context)).toBe('decline')
  expect(() => codexMcpConfig([{ name: 'invalid.name' }])).toThrow('invalid MCP server name')
})
