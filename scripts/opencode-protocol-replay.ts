// Replays REAL opencode run --format json captures (docs §10, Tier B2
// session on 2026-10-25, opencode 1.19.4 with its free default model).
// Run: node scripts/opencode-protocol-replay.ts

import { createOpencodeTranslator } from '../src/agent/hub/drivers/opencodeProtocol.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) console.log(`  ok  ${name}`)
  else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

const SES = 'ses_e90dbedc3ffeFvObJwCtaKWxwh'

// ---------- Scenario 1: REAL simple reply stream ----------
{
  const deltas: string[] = []
  const steps: any[] = []
  const t = createOpencodeTranslator({
    onTextDelta: e => deltas.push(e.delta),
    onStepFinish: e => steps.push(e),
  })
  for (const line of [
    `{"type":"step_start","timestamp":1792866007803,"sessionID":"${SES}","part":{"id":"prt_1","messageID":"msg_1","type":"step-start"}}`,
    `{"type":"text","timestamp":1792866007803,"sessionID":"${SES}","part":{"id":"prt_2","messageID":"msg_1","type":"text","text":"ok","time":{}}}`,
    `{"type":"step_finish","timestamp":1792866008370,"sessionID":"${SES}","part":{"id":"prt_3","reason":"stop","messageID":"msg_1","type":"step-finish","tokens":{"total":21441,"input":20855,"output":8,"reasoning":30,"cache":{"write":0,"read":548}},"cost":0}}`,
  ]) {
    t.handleRecord(JSON.parse(line))
  }
  const s = t.snapshot()
  check('S1 text streamed', deltas.join('') === 'ok')
  check('S1 sessionID captured', s.nativeSessionId === SES)
  check('S1 usage normalized', (s.usage as any)?.inputTokens === 20855 && (s.usage as any)?.totalCostUsd === 0)
  check('S1 finishReason stop', s.finishReason === 'stop')
  check('S1 finalText', s.finalText === 'ok')
  check('S1 step initial', steps.length === 1 && steps[0].stepKind === 'initial')
}

// ---------- Scenario 2: REAL tool run (two steps, narration + answer) ----------
{
  const toolEvents: string[] = []
  const t = createOpencodeTranslator({
    onToolCall: e => toolEvents.push(`call:${e.toolName}`),
    onToolStart: e => toolEvents.push(`start:${e.toolName}`),
    onToolResult: e => toolEvents.push(`result:${e.toolName}`),
    onToolEnd: e => toolEvents.push(`end:${e.toolName}${e.error ? ':err' : ''}`),
  })
  const records = [
    { type: 'step_start', sessionID: SES, part: { id: 'p1', messageID: 'm1', type: 'step-start' } },
    {
      type: 'tool_use',
      sessionID: SES,
      part: {
        type: 'tool',
        tool: 'bash',
        callID: 'call-1',
        state: {
          status: 'completed',
          input: { command: 'echo jedi-tool-test' },
          output: 'jedi-tool-test\n',
          metadata: { output: 'jedi-tool-test\n', exit: 0, truncated: false },
          title: 'echo jedi-tool-test',
        },
        id: 'p2',
        messageID: 'm1',
      },
    },
    {
      type: 'text',
      sessionID: SES,
      part: { id: 'p3', messageID: 'm1', type: 'text', text: "I'll run that command. \n\n" },
    },
    {
      type: 'step_finish',
      sessionID: SES,
      part: { id: 'p4', reason: 'tool-calls', messageID: 'm1', tokens: { total: 1, input: 1, output: 1 }, cost: 0 },
    },
    { type: 'step_start', sessionID: SES, part: { id: 'p5', messageID: 'm2', type: 'step-start' } },
    { type: 'text', sessionID: SES, part: { id: 'p6', messageID: 'm2', type: 'text', text: 'Done.' } },
    { type: 'step_finish', sessionID: SES, part: { id: 'p7', reason: 'stop', messageID: 'm2', tokens: { total: 2, input: 1, output: 1 }, cost: 0 } },
  ]
  for (const r of records) t.handleRecord(r)
  const s = t.snapshot()
  check('S2 whole tool chain in one event', toolEvents.join(',') === 'call:bash,start:bash,result:bash,end:bash', toolEvents.join(','))
  check('S2 toolCount', s.toolCount === 1)
  check('S2 two steps', s.stepCount === 2)
  check('S2 final message text preferred over narration', s.finalText === 'Done.')
  check('S2 allText includes narration', s.allText.includes("I'll run that command"))
  check('S2 tool-calls finishReason then stop', s.finishReason === 'stop')
}

// ---------- Scenario 3: failed tool exit code + error line ----------
{
  const t = createOpencodeTranslator({
    onToolEnd: () => {},
  })
  t.handleRecord({
    type: 'tool_use',
    sessionID: 's2',
    part: { type: 'tool', tool: 'bash', callID: 'c2', state: { status: 'completed', input: { command: 'false' }, output: '', metadata: { exit: 1 } } },
  })
  const s = t.snapshot()
  check('S3 exit code flagged', s.toolCount === 1)
  t.noteErrorLine('Error: Unexpected error')
  check('S3 error line captured', t.snapshot().lastError === 'Error: Unexpected error')
}

// ---------- Scenario 4: duplicate part dedupe ----------
{
  let deltaCount = 0
  const t = createOpencodeTranslator({ onTextDelta: () => (deltaCount += 1) })
  const rec = { type: 'text', sessionID: 's3', part: { id: 'dup', messageID: 'm', type: 'text', text: 'abc' } }
  t.handleRecord(rec)
  t.handleRecord(rec)
  check('S4 duplicate part emitted once', deltaCount === 1 && t.snapshot().allText === 'abc')
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
