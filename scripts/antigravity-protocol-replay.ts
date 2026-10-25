// Replays REAL Antigravity CLI (agy) captures (docs §10, Tier B3 session on
// 2026-10-25, agy 1.4.7). The live capture is the unauthenticated ERROR
// result; the OK result and tool events follow the same envelope.
// Run: node scripts/antigravity-protocol-replay.ts

import { createAntigravityTranslator } from '../src/agent/hub/drivers/antigravityProtocol.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) console.log(`  ok  ${name}`)
  else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

// ---------- Scenario 1: REAL unauthenticated ERROR result ----------
{
  const t = createAntigravityTranslator({})
  t.handleRecord(
    JSON.parse(
      '{"event":"result","result":{"conversation_id":"","status":"ERROR","response":"","error":"authentication failed or timed out","duration_seconds":0,"num_turns":0,"usage":{"input_tokens":0,"output_tokens":0,"thinking_tokens":0,"cache_read_tokens":0,"total_tokens":0}}}'
    )
  )
  const s = t.snapshot()
  check('S1 auth error surfaced', s.lastError === 'authentication failed or timed out')
  check('S1 turnComplete', s.turnComplete === true)
  check('S1 finishReason error + empty text (driver rejects)', s.finishReason === 'error' && s.fullText === '')
  check('S1 empty conversation_id not stored', s.nativeSessionId === undefined)
}

// ---------- Scenario 2: successful result ----------
{
  const deltas: string[] = []
  const steps: any[] = []
  const t = createAntigravityTranslator({
    onTextDelta: e => deltas.push(e.delta),
    onStepFinish: e => steps.push(e),
  })
  t.handleRecord({
    event: 'result',
    result: {
      conversation_id: 'conv_abc123',
      status: 'OK',
      response: 'ok',
      duration_seconds: 2,
      num_turns: 1,
      usage: { input_tokens: 10, output_tokens: 2, thinking_tokens: 3, cache_read_tokens: 5, total_tokens: 15 },
    },
  })
  const s = t.snapshot()
  check('S2 response text', s.fullText === 'ok' && deltas[0] === 'ok')
  check('S2 conversation id', s.nativeSessionId === 'conv_abc123')
  check('S2 usage normalized', (s.usage as any)?.inputTokens === 10 && (s.usage as any)?.reasoningTokens === 3)
  check('S2 finishReason stop', s.finishReason === 'stop' && s.lastError === null)
  check('S2 step fired', steps.length === 1 && steps[0].stepKind === 'initial')
}

// ---------- Scenario 3: tool events (defensive shapes) ----------
{
  const events: string[] = []
  const t = createAntigravityTranslator({
    onToolCall: () => events.push('call'),
    onToolEnd: e => events.push(`end${e.error ? ':err' : ''}`),
  })
  t.handleRecord({ event: 'tool_use', id: 't1', tool: 'terminal', input: { command: 'ls' } })
  t.handleRecord({ event: 'tool_result', id: 't1', tool: 'terminal', output: 'a\nb', error: 'exit 1' })
  t.handleRecord({ event: 'result', result: { conversation_id: 'c1', status: 'OK', response: 'done' } })
  const s = t.snapshot()
  check('S3 tool chain', events.join(',') === 'call,end:err', events.join(','))
  check('S3 stepKind tool-result', s.toolCount === 1 && s.finishReason === 'stop')
}

// ---------- Scenario 4: pre-result text passthrough ----------
{
  const t = createAntigravityTranslator({ onTextDelta: () => {} })
  t.handleRecord({ event: 'text', text: 'partial narration' })
  t.handleRecord({ event: 'result', result: { conversation_id: 'c2', status: 'OK', response: 'final answer' } })
  check('S4 result text overrides streaming text', t.snapshot().fullText === 'final answer')
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
