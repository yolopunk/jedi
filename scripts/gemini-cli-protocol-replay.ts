// Replays REAL gemini-family CLI captures (docs §10, M4 Tier B session on
// 2026-10-25) through the translator and asserts the unified-hook mapping.
// Captured live: gemini-cli 0.65.0 auth-error document and qwen-code 0.26.1
// error record array; success shapes follow the same envelopes.
// Run: node scripts/gemini-cli-protocol-replay.ts

import {
  buildConversationPrompt,
  emitGeminiFamilyResult,
  parseGeminiFamilyOutput,
} from '../src/agent/hub/drivers/geminiCliProtocol.ts'
import { isWslHost, toWslPath } from '../src/agent/hub/paths.ts'

let failures = 0
function check(name: string, cond: boolean, detail = '') {
  if (cond) console.log(`  ok  ${name}`)
  else {
    failures++
    console.error(`FAIL  ${name} ${detail}`)
  }
}

// ---------- Scenario 1: REAL gemini-cli auth-error document ----------
{
  const raw = `{
  "session_id": "b37ada03-3e41-4cb9-ae4c-f24c38bb3ab4",
  "error": {
    "type": "Error",
    "message": "Please set an Auth method in your /home/yolo/.gemini/settings.json or specify one of the following environment variables before running: GEMINI_API_KEY, GOOGLE_GENAI_USE_VERTEXAI, GOOGLE_GENAI_USE_GCA",
    "code": 41
  }
}`.trim()
  const r = parseGeminiFamilyOutput(raw)
  check('S1 gemini error surfaced', !!r.error && r.error.includes('GEMINI_API_KEY'))
  check('S1 sessionId kept', r.sessionId === 'b37ada03-3e41-4cb9-ae4c-f24c38bb3ab4')
  check('S1 finishReason error + empty text (driver rejects)', r.finishReason === 'error' && r.text === '')
}

// ---------- Scenario 2: REAL qwen-code error record array ----------
{
  const raw =
    '[{"type":"result","subtype":"error_during_execution","uuid":"c343ae2b-f047-426f-8c83-f10a38a78615","session_id":"9d58f1bf-cd30-47e7-b760-d613729aaba5","is_error":true,"duration_ms":0,"duration_api_ms":0,"num_turns":0,"usage":{"input_tokens":0,"output_tokens":0},"permission_denials":[],"error":{"message":"No auth type is selected. Please configure an auth type (e.g. via settings or `--auth-type`) before running in non-interactive mode."}}]'
  const r = parseGeminiFamilyOutput(raw)
  check('S2 qwen error surfaced', !!r.error && r.error.includes('auth type'))
  check('S2 usage normalized', (r.usage as any)?.inputTokens === 0)
  check('S2 finishReason error', r.finishReason === 'error')
}

// ---------- Scenario 3: success shapes (both variants) ----------
{
  const gemini = parseGeminiFamilyOutput(
    JSON.stringify({
      session_id: 's-g1',
      response: 'ok',
      stats: { models: { 'gemini-2.5-pro': { 'models.totalTokenCount': 42 } } },
    })
  )
  check('S3 gemini success text', gemini.text === 'ok')
  check('S3 gemini usage total', (gemini.usage as any)?.totalTokens === 42)
  check('S3 gemini no error', gemini.error === null && gemini.finishReason === 'end_turn')

  const qwen = parseGeminiFamilyOutput(
    JSON.stringify([
      {
        type: 'result',
        subtype: 'success',
        session_id: 's-q1',
        is_error: false,
        result: '你好',
        usage: { input_tokens: 7, output_tokens: 3 },
      },
    ])
  )
  check('S3 qwen success text', qwen.text === '你好')
  check('S3 qwen usage normalized', (qwen.usage as any)?.outputTokens === 3)
  check('S3 qwen end_turn', qwen.finishReason === 'end_turn' && qwen.error === null)
}

// ---------- Scenario 4: hooks emission for a final result ----------
{
  const fired: string[] = []
  emitGeminiFamilyResult(
    { text: 'final answer', finishReason: 'end_turn', error: null, usage: { totalTokens: 5 } },
    {
      onTextDelta: e => fired.push(`delta:${e.delta}`),
      onStepFinish: e => fired.push(`step:${e.stepKind}:${e.finishReason}`),
      onFinish: e => fired.push(`finish:${e.finishReason}`),
    }
  )
  check('S4 single-shot delta', fired[0] === 'delta:final answer')
  check('S4 step + finish fired', fired[1] === 'step:initial:end_turn' && fired[2] === 'finish:end_turn')
}

// ---------- Scenario 5: conversation prompt builder ----------
{
  const prompt = buildConversationPrompt([
    { role: 'user', content: '第一个问题' },
    { role: 'assistant', content: '第一个回答' },
    { role: 'user', content: '第二个问题' },
  ])
  check('S5 includes prior assistant context', prompt.includes('Assistant: 第一个回答'))
  check('S5 ends with latest user message', prompt.includes('User: 第二个问题') && prompt.endsWith('第二个问题'))

  const single = buildConversationPrompt([{ role: 'user', content: 'only question' }])
  check('S5 single turn passes through bare', single === 'only question')
}

// ---------- Scenario 6: noisy / garbage output NEVER leaks into the bubble ----------
{
  // Pure garbage: no JSON at all → error, empty text (driver rejects).
  const garbage = parseGeminiFamilyOutput('plain text crash output')
  check('S6 garbage → error not text', garbage.text === '' && !!garbage.error)

  // Banner noise wrapped around a valid qwen error record → JSON extracted.
  const noisy = parseGeminiFamilyOutput(
    '\u001b[?25lWelcome to qwen 0.26.1\n[{"type":"result","subtype":"error_during_execution","session_id":"s9","is_error":true,"usage":{"input_tokens":0,"output_tokens":0},"error":{"message":"No auth type is selected."}}]\nTip: run qwen login'
  )
  check('S6 noise+ANSI stripped, JSON extracted', !!noisy.error && noisy.error.includes('auth type'), JSON.stringify(noisy.error))
  check('S6 noisy path yields no raw text', noisy.text === '')

  // Unparseable JSON body → error with preview, never the raw wall.
  const broken = parseGeminiFamilyOutput('log line\n{"type":"result", broken')
  check('S6 broken JSON → error preview', broken.text === '' && !!broken.error)
}

// ---------- Scenario 7: WSL path translation (docs §5.3 W5) ----------
{
  check('S7 drive C', toWslPath('C:\\Users\\epoch\\shot.png') === '/mnt/c/Users/epoch/shot.png')
  check('S7 drive D lowercase', toWslPath('D:\\a b\\c.png') === '/mnt/d/a b/c.png')
  check('S7 forward slashes', toWslPath('E:/x/y.png') === '/mnt/e/x/y.png')
  check('S7 non-windows null', toWslPath('/home/yolo/x.png') === null)
  check('S7 UNC null', toWslPath('\\\\wsl$\\Arch\\home') === null)
  check('S7 host detect', isWslHost('wsl:ArchLinux') === true && isWslHost('windows') === false)
}

console.log(failures === 0 ? '\nALL REPLAY CHECKS PASSED' : `\n${failures} CHECKS FAILED`)
if (failures > 0) {
  throw new Error(`${failures} replay checks failed`)
}
