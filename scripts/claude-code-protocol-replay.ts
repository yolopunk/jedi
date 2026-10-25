// Replays REAL Claude Code stream-json captures (docs §10, M3 session on
// 2026-10-25, Claude Code 2.1.299) through the translator and asserts the
// unified-hook mapping. Run: node scripts/claude-code-protocol-replay.ts.

import {
  controlResponseFor,
  createClaudeTranslator,
} from '../src/agent/hub/drivers/claudeCodeProtocol.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) console.log(`  ok  ${name}`)
  else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

// ---------- Scenario 1: REAL streaming capture (curl weather turn) ----------
{
  const deltas: string[] = []
  const steps: any[] = []
  const t = createClaudeTranslator({
    hooks: {
      onTextDelta: e => deltas.push(e.delta),
      onStepFinish: e => steps.push(e),
    },
  })
  const records = [
    { type: 'system', subtype: 'hook_started', hook_id: '97ed7758', session_id: 'fa2a891d-ac4d-43d2-9b22-92cfdc0adcaf' },
    {
      type: 'system',
      subtype: 'init',
      session_id: 'fa2a891d-ac4d-43d2-9b22-92cfdc0adcaf',
      model: 'claude-sonnet-5-5',
      permissionMode: 'default',
      tools: ['Bash', 'Read', 'Write'],
    },
    { type: 'stream_event', event: { type: 'message_start' } },
    { type: 'stream_event', event: { type: 'content_block_start' } },
    { type: 'stream_event', event: { type: 'content_block_delta', delta: { type: 'text_delta', text: 'ok' } } },
    {
      type: 'assistant',
      message: { model: 'claude-sonnet-5-5', content: [{ type: 'text', text: 'ok' }], usage: { input_tokens: 2, output_tokens: 4 }, stop_reason: 'end_turn' },
      session_id: 'fa2a891d-ac4d-43d2-9b22-92cfdc0adcaf',
    },
    { type: 'stream_event', event: { type: 'content_block_stop' } },
    { type: 'stream_event', event: { type: 'message_delta', delta: { stop_reason: 'end_turn' } } },
    { type: 'stream_event', event: { type: 'message_stop' } },
    { type: 'rate_limit_event', rate_limit_info: { status: 'allowed' } },
    {
      type: 'result',
      subtype: 'success',
      is_error: false,
      result: 'ok',
      session_id: 'fa2a891d-ac4d-43d2-9b22-92cfdc0adcaf',
      total_cost_usd: 0.012,
      usage: { input_tokens: 2, output_tokens: 4 },
      duration_ms: 3200,
    },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S1 text from delta + result sync', s.fullText === 'ok', JSON.stringify(s.fullText))
  check('S1 sessionId from init/result', s.nativeSessionId === 'fa2a891d-ac4d-43d2-9b22-92cfdc0adcaf')
  check('S1 usage normalized', (s.usage as any)?.inputTokens === 2 && (s.usage as any)?.totalCostUsd === 0.012)
  check('S1 turnComplete', s.turnComplete === true)
  check('S1 finishReason success->end_turn', s.finishReason === 'end_turn', String(s.finishReason))
  check('S1 step fired initial', steps.length === 1 && steps[0].stepKind === 'initial')
  check('S1 delta stream fired', deltas.length >= 1)
}

// ---------- Scenario 2: REAL tool_use / tool_result turn ----------
{
  const toolEvents: string[] = []
  const t = createClaudeTranslator({
    hooks: {
      onToolCall: e => toolEvents.push(`call:${e.toolName}`),
      onToolStart: e => toolEvents.push(`start:${e.toolName}`),
      onToolResult: e => toolEvents.push(`result:${e.toolName}`),
      onToolEnd: e => toolEvents.push(`end:${e.toolName}${e.error ? ':err' : ''}`),
    },
  })
  const records = [
    {
      type: 'assistant',
      message: { content: [{ type: 'tool_use', id: 'toolu_01A', name: 'Write', input: { file_path: '/tmp/jedi-cc-test.txt', content: 'hello-from-jedi' } }], usage: { input_tokens: 10, output_tokens: 6 }, stop_reason: 'tool_use' },
    },
    {
      type: 'user',
      message: { content: [{ type: 'tool_result', tool_use_id: 'toolu_01A', content: 'File created successfully at: /tmp/jedi-cc-test.txt' }] },
    },
    {
      type: 'result',
      subtype: 'success',
      is_error: false,
      result: 'I created `/tmp/jedi-cc-test.txt` with the content `hello-from-jedi`.',
      session_id: '1386be72-6146-4ce9-a83e-869efbf532a0',
      usage: { input_tokens: 10, output_tokens: 8 },
    },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S2 tool chain order', toolEvents.join(',') === 'call:Write,start:Write,result:Write,end:Write', toolEvents.join(','))
  check('S2 toolCount', s.toolCount === 1)
  check('S2 result text wins over empty delta stream', s.fullText.startsWith('I created'))
  check('S2 stepKind tool-result', s.stepCount === 1)
}

// ---------- Scenario 3: error result + no-delta text fallback ----------
{
  const t = createClaudeTranslator({ hooks: {} })
  t.handleRecord({
    type: 'result',
    subtype: 'error_during_execution',
    is_error: true,
    result: 'API Error: 500',
    session_id: 'abc',
    usage: { input_tokens: 1, output_tokens: 1 },
  })
  const s = t.snapshot()
  check('S3 terminal error flagged', s.isTerminalError === true && s.lastError === 'API Error: 500')
  check('S3 finishReason error subtype', s.finishReason === 'error_during_execution')
}

// ---------- Scenario 4: control_request approval (dormant path, SDK shapes) ----------
{
  const approvals: any[] = []
  const t = createClaudeTranslator({
    hooks: {},
    onApprovalRequest: req => approvals.push(req),
  })
  // Shape A: Agent SDK (sdk-go) documented form.
  t.handleRecord({
    type: 'sdk_control_request',
    request: { subtype: 'permission', request_id: 'perm_1', tool_name: 'Bash', tool_input: { command: 'rm -rf /tmp/test' } },
  })
  // Shape B: plain control_request with payload.
  t.handleRecord({
    type: 'control_request',
    request: { id: 'req_9', type: 'tool_permission_request', payload: { tool_name: 'Write', tool_input: { file_path: '/x' } } },
  })
  check('S4 both approval shapes captured', approvals.length === 2 && approvals[0].requestId === 'perm_1' && approvals[1].requestId === 'req_9', JSON.stringify(approvals.map(a => a.requestId)))
  check('S4 tool input mapped', (approvals[0].toolInput as any)?.command === 'rm -rf /tmp/test')
  const allow = controlResponseFor('perm_1', true)
  const deny = controlResponseFor('perm_1', false, 'nope')
  check('S4 control_response allow shape', (allow as any).response.behavior === 'allow' && (allow as any).request_id === 'perm_1')
  check('S4 control_response deny shape', (deny as any).response.behavior === 'deny' && (deny as any).response.message === 'nope')
}

// ---------- Scenario 5: resume continuity (verified live: codeword recall) ----------
{
  const t = createClaudeTranslator({ hooks: {} })
  t.handleRecord({ type: 'system', subtype: 'init', session_id: 'df53383f-929f-4732-bbad-f00261b0cdaa', model: 'claude-sonnet-5-5' })
  t.handleRecord({ type: 'result', subtype: 'success', is_error: false, result: 'jedi-m3-test', session_id: 'df53383f-929f-4732-bbad-f00261b0cdaa', usage: { input_tokens: 3, output_tokens: 2 } })
  check('S5 nativeSessionId stable across init/result', t.snapshot().nativeSessionId === 'df53383f-929f-4732-bbad-f00261b0cdaa')
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
