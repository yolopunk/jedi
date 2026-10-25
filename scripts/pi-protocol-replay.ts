// Replays REAL pi RPC event captures (from docs §10, M2 session on
// 2026-10-25) through the pi translator and asserts the unified-hook
// mapping. Run: node scripts/pi-protocol-replay.ts (Node >= 23 type stripping).

import { createPiTranslator } from '../src/agent/hub/drivers/piProtocol.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) {
    console.log(`  ok  ${name}`)
  } else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

// ---------- Scenario 1: successful token streaming ----------
{
  const events: any[] = []
  const deltas: string[] = []
  const t = createPiTranslator({
    onTextDelta: e => {
      events.push(e)
      deltas.push(e.delta)
    },
    onStepFinish: e => events.push({ step: e }),
    onFinish: e => events.push({ finish: e }),
  })
  const records = [
    { id: '2', type: 'response', command: 'prompt', success: true, data: { disposition: 'started' } },
    { id: '9', type: 'response', command: 'get_state', success: true, data: { sessionId: '01a16ee9-6f43-70e3-9748-5a0cdacff487', messageCount: 3 } },
    { type: 'agent_start' },
    { type: 'turn_start' },
    { type: 'message_update', assistantMessageEvent: { type: 'text_delta', delta: 'Repl' } },
    { type: 'message_update', assistantMessageEvent: { type: 'text_delta', delta: 'y with e' } },
    { type: 'message_update', assistantMessageEvent: { type: 'reasoning_delta', delta: 'thinking…' } },
    { type: 'message_update', assistantMessageEvent: { type: 'text_delta', delta: 'xactly: ok' } },
    { type: 'message_end', message: { role: 'assistant', model: 'claude-sonnet-4-5', content: [], usage: { input: 12, output: 4, cost: { total: 0.001 } }, stopReason: 'stop' } },
    { type: 'turn_end', message: { role: 'assistant', stopReason: 'stop', usage: { input: 12, output: 4 } } },
    { type: 'agent_end', messages: [] },
    { type: 'agent_settled', aborted: false },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S1 fullText joined', s.fullText === 'Reply with exactly: ok', `got ${JSON.stringify(s.fullText)}`)
  check('S1 3 text deltas (reasoning excluded)', deltas.length === 3, `got ${deltas.length}`)
  check('S1 usage captured', (s.usage as any)?.input === 12)
  check('S1 finishReason stop', s.finishReason === 'stop')
  check('S1 sessionId from get_state', s.piSessionId === '01a16ee9-6f43-70e3-9748-5a0cdacff487')
  check('S1 step fired', events.some(e => e.step && e.step.stepNumber === 1 && e.step.stepKind === 'initial'))
  check('S1 reasoning captured', s.reasoning === 'thinking…')
}

// ---------- Scenario 2: REAL 400-error capture (out-of-credits account) ----------
{
  const t = createPiTranslator({})
  const records = [
    { id: '2', type: 'response', command: 'prompt', success: true, data: { disposition: 'started' } },
    { type: 'agent_start' },
    { type: 'turn_start' },
    { type: 'message_start', message: { role: 'system', content: '', sections: { preamble: 'You are…' } } },
    { type: 'message_end', message: { role: 'system', content: '' } },
    { type: 'message_start', message: { role: 'user', content: [{ type: 'text', text: 'Reply with exactly: ok' }], timestamp: 1792862040943 } },
    { type: 'message_end', message: { role: 'user', content: [{ type: 'text', text: 'Reply with exactly: ok' }] } },
    { type: 'message_start', message: { role: 'assistant', content: [], api: 'anthropic-messages', provider: 'anthropic', model: 'claude-sonnet-4-5', usage: { input: 0, output: 0, totalTokens: 0 }, stopReason: 'error', errorMessage: '400 {"type":"error","error":{"type":"invalid_request_error","message":"Third-party apps now draw from your extra usage…"}' } },
    { type: 'message_end', message: { role: 'assistant', content: [], stopReason: 'error', errorMessage: '400 {"type":"error","error":{"type":"invalid_request_error","message":"Third-party apps now draw from your extra usage…"}' } },
    { type: 'turn_end', message: { role: 'assistant', stopReason: 'error' } },
    { type: 'agent_end', messages: [] },
    { type: 'agent_settled', aborted: false },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S2 error captured', !!s.lastError && s.lastError.startsWith('400'))
  check('S2 finishReason error', s.finishReason === 'error')
  check('S2 empty text (driver rejects)', s.fullText === '')
}

// ---------- Scenario 3: REAL 429 auto-retry capture ----------
{
  const t = createPiTranslator({})
  const records = [
    { type: 'agent_start' },
    { type: 'turn_start' },
    { type: 'message_end', message: { role: 'assistant', content: [], stopReason: 'error', errorMessage: '429 rate_limit_error' } },
    { type: 'turn_end', message: { role: 'assistant', stopReason: 'error' } },
    { type: 'agent_end', messages: [] },
    { type: 'auto_retry_start', attempt: 1, maxAttempts: 3, delayMs: 2000, errorMessage: '429 …' },
    { type: 'entry_appended', entry: { type: 'context_edit', id: '5dc600db', parentId: '0854d206' } },
    { type: 'agent_settled', aborted: false },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S3 survives retry records', s.stepCount === 1 && !!s.lastError)
  check('S3 finishReason stays error', s.finishReason === 'error')
}

// ---------- Scenario 4: tool execution + late text provider ----------
{
  const calls: any[] = []
  const t = createPiTranslator({
    onToolCall: e => calls.push(['call', e.toolName, e.input]),
    onToolStart: e => calls.push(['start', e.toolName]),
    onToolEnd: e => calls.push(['end', e.toolName, e.output]),
    onToolResult: e => calls.push(['result', e.toolName, e.output]),
  })
  const records = [
    { type: 'tool_execution_start', toolCallId: 'tc1', toolName: 'bash', args: { command: 'ls' } },
    { type: 'tool_execution_end', toolCallId: 'tc1', toolName: 'bash', result: { content: 'file a\nfile b' } },
    { type: 'message_end', message: { role: 'assistant', content: [{ type: 'text', text: 'late text' }], usage: { input: 5, output: 2 }, stopReason: 'tool_use' } },
    { type: 'turn_end', message: { role: 'assistant', stopReason: 'tool_use' } },
    { type: 'agent_settled', aborted: false },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S4 tool call chain', calls.length === 4 && calls[0][1] === 'bash' && calls[3][1] === 'bash')
  check('S4 toolCount', s.toolCount === 1)
  check('S4 late text fallback', s.fullText === 'late text')
  check('S4 stepKind tool-result', s.stepCount === 1)
}

// ---------- Scenario 5: aborted settle ----------
{
  const t = createPiTranslator({})
  t.handleRecord({ type: 'agent_settled', aborted: true })
  check('S5 aborted finishReason', t.snapshot().finishReason === 'aborted')
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
