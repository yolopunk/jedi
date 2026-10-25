// Replays REAL codex app-server captures (docs §10, M4 session on
// 2026-10-25, codex-cli 0.162.1) through the translator and asserts the
// unified-hook mapping. Run: node scripts/codex-protocol-replay.ts.
// The live probe hit the account usage limit (usageLimitExceeded), so the
// captured stream exercises the full failure path; the successful agentMessage
// / commandExecution item shapes follow the same envelope and are asserted
// synthetically alongside.

import { createCodexTranslator } from '../src/agent/hub/drivers/codexProtocol.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) console.log(`  ok  ${name}`)
  else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

const THREAD = '01a16efa-18e5-7161-bc53-ba4dad7c187d'
const TURN = '01a16efa-19dc-7683-a766-086ce525f812'

// ---------- Scenario 1: REAL captured lifecycle (usage-limit failure turn) ----------
{
  const steps: any[] = []
  const t = createCodexTranslator({
    onStepFinish: e => steps.push(e),
  })
  const records: any[] = [
    { id: 1, result: { userAgent: 'jedi-probe/0.162.1 (Arch Linux; x86_64)', codexHome: '/home/yolo/.codex' } },
    { method: 'remoteControl/status/changed', params: { status: 'disabled' }, emittedAtMs: 1792863224370 },
    { method: 'account/updated', params: { authMode: 'chatgpt', planType: 'plus' }, emittedAtMs: 1792863225641 },
    { id: 2, result: { thread: { id: THREAD, environments: [{ environmentId: 'local', cwd: '/tmp' }], sessionId: THREAD } } },
    { method: 'thread/started', params: { thread: { id: THREAD } } },
    { method: 'mcpServer/startupStatus/updated', params: { threadId: THREAD, name: 'codex_apps', status: 'starting' }, emittedAtMs: 1 },
    { id: 3, result: { turn: { id: TURN, rootTurnId: TURN, status: 'inProgress', error: null } } },
    { method: 'thread/status/changed', params: { threadId: THREAD, status: { type: 'active', activeFlags: [] } }, emittedAtMs: 2 },
    { method: 'turn/started', params: { threadId: THREAD, turn: { id: TURN, status: 'inProgress' } } },
    { method: 'item/started', params: { item: { type: 'userMessage', id: '01a16efa-2628', content: [{ type: 'text', text: 'Reply with exactly: ok' }] }, threadId: THREAD, turnId: TURN } },
    { method: 'item/completed', params: { item: { type: 'userMessage', id: '01a16efa-2628', content: [{ type: 'text', text: 'Reply with exactly: ok' }] }, threadId: THREAD, turnId: TURN } },
    { method: 'account/rateLimits/updated', params: { rateLimits: { limitId: 'premium', credits: { hasCredits: false } } }, emittedAtMs: 3 },
    { method: 'thread/status/changed', params: { threadId: THREAD, status: { type: 'systemError' } }, emittedAtMs: 4 },
    { method: 'error', params: { error: { message: 'You’ve hit your usage limit. Upgrade to Pro, visit https://chatgpt.com/settings/usage to purchase more credits or try again at 3:35 AM.', codexErrorInfo: 'usageLimitExceeded' } }, emittedAtMs: 5 },
    { method: 'turn/completed', params: { threadId: THREAD, turn: { id: TURN, status: 'failed', error: { message: 'usage limit exceeded' } } }, emittedAtMs: 6 },
    { method: 'mcpServer/startupStatus/updated', params: { threadId: THREAD, name: 'codex_apps', status: 'ready' }, emittedAtMs: 7 },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S1 threadId from response + notification', s.threadId === THREAD && s.nativeSessionId === THREAD)
  check('S1 turnId captured', s.turnId === TURN)
  check('S1 usage-limit error surfaced', !!s.lastError && s.lastError.includes('usage limit'), String(s.lastError).slice(0, 60))
  check('S1 turnComplete at turn/completed', s.turnComplete === true)
  check('S1 failed turn → finishReason error', s.finishReason === 'error')
  check('S1 empty text (driver rejects)', s.fullText === '')
  check('S1 step fired once', steps.length === 1 && steps[0].stepKind === 'initial')
}

// ---------- Scenario 2: successful agentMessage delta streaming ----------
{
  const deltas: string[] = []
  const t = createCodexTranslator({
    onTextDelta: e => deltas.push(e.delta),
  })
  const records: any[] = [
    { id: 1, result: {} },
    { id: 2, result: { thread: { id: 'th2' } } },
    { method: 'item/started', params: { item: { type: 'agentMessage', id: 'am1', content: [] } } },
    { method: 'item/updated', params: { item: { type: 'agentMessage', id: 'am1', content: [{ type: 'text', text: 'He' }] } } },
    { method: 'item/updated', params: { item: { type: 'agentMessage', id: 'am1', content: [{ type: 'text', text: 'Hello' }] } } },
    { method: 'item/completed', params: { item: { type: 'agentMessage', id: 'am1', content: [{ type: 'text', text: 'Hello' }] } } },
    { method: 'turn/completed', params: { threadId: 'th2', turn: { id: 't2', status: 'completed', tokenCount: { input: 9, output: 2 } } } },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S2 incremental deltas', deltas.join('|') === 'He|llo', deltas.join('|'))
  check('S2 fullText', s.fullText === 'Hello')
  check('S2 completed (not error)', s.finishReason !== 'error' && s.lastError === null)
  check('S2 usage from tokenCount', (s.usage as any)?.input === 9)
}

// ---------- Scenario 3: commandExecution tool lifecycle ----------
{
  const events: string[] = []
  const t = createCodexTranslator({
    onToolCall: e => events.push(`call:${e.toolName}`),
    onToolStart: e => events.push(`start:${e.toolName}`),
    onToolResult: e => events.push(`result:${e.toolName}`),
    onToolEnd: e => events.push(`end:${e.toolName}${e.error ? ':err' : ''}`),
  })
  const records: any[] = [
    { method: 'item/started', params: { item: { type: 'commandExecution', id: 'ce1', command: 'ls -la' } } },
    { method: 'item/completed', params: { item: { type: 'commandExecution', id: 'ce1', command: 'ls -la', aggregatedOutput: 'total 0', exitCode: 0 } } },
    { method: 'item/started', params: { item: { type: 'commandExecution', id: 'ce2', command: 'false' } } },
    { method: 'item/completed', params: { item: { type: 'commandExecution', id: 'ce2', command: 'false', aggregatedOutput: '', exitCode: 1 } } },
    { method: 'turn/completed', params: { threadId: 'th3', turn: { id: 't3', status: 'completed' } } },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S3 tool chains', events.join(',') === 'call:bash,start:bash,result:bash,end:bash,call:bash,start:bash,result:bash,end:bash:err', events.join(','))
  check('S3 toolCount', s.toolCount === 2)
  check('S3 stepKind tool-result', s.stepCount === 1)
}

// ---------- Scenario 4: JSON-RPC error response terminates ----------
{
  const t = createCodexTranslator({})
  t.handleRecord({ id: 5, error: { code: -32602, message: 'threadId required' } })
  const s = t.snapshot()
  check('S4 rpc error → terminal', s.turnComplete === true && s.lastError === 'threadId required' && s.finishReason === 'error')
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
