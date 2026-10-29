// Exercise the actual driver's settlement, rather than only protocol translation.
const assert = require('node:assert/strict')
const fs = require('node:fs')
const vm = require('node:vm')
const ts = require('typescript')
function load(file, mocks) {
  const exports = {}
  vm.runInNewContext(ts.transpileModule(fs.readFileSync(file, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText, { exports, require: name => mocks[name], console, setTimeout, clearTimeout })
  return exports
}
const protocol = load('src/agent/hub/drivers/piProtocol.ts', {})
async function run(records, failure) {
  let channel
  let kills = 0
  const { PiDriver } = load('src/agent/hub/drivers/pi.ts', {
    '../agentError': load('src/agent/hub/agentError.ts', {}), '../types': {}, '../model': load('src/agent/hub/model.ts', {}), './piProtocol': protocol,
    '@tauri-apps/api/core': {
      Channel: class {},
      invoke: async (cmd, args) => {
        if (cmd === 'agent_proc_spawn') channel = args.onEvent
        if (cmd === 'agent_proc_kill') kills++
      },
    },
  })
  let finishes = 0
  const handle = await new PiDriver().start({
    sessionId: 'test', messages: [{ role: 'user', content: 'hello' }],
    engine: { agentId: 'pi', host: 'linux', installation: { spawn: {} } },
  }, { onFinish: () => finishes++ })
  for (const record of records) channel.onmessage({ type: 'line', line: JSON.stringify(record) })
  let timer
  const deadline = new Promise((_, reject) => { timer = setTimeout(() => reject(Error('driver never settled')), 500) })
  try {
    if (failure) await assert.rejects(Promise.race([handle.done, deadline]), failure)
    else { assert.equal((await Promise.race([handle.done, deadline])).text, 'hello'); assert.equal(finishes, 1) }
    assert.equal(kills, 1)
  } finally { clearTimeout(timer) }
}
;(async () => {
  await run([{ type:'message_end',message:{role:'assistant',content:[{text:'hello'}]} },{type:'agent_settled'},{type:'agent_settled'}])
  await run([{type:'message_end',message:{role:'assistant',errorMessage:'bad credentials'}},{type:'agent_settled'}], /bad credentials/)
  await run([{type:'response',command:'prompt',success:false,error:'model missing'}], /model missing/)
  console.log('Pi driver settlement checks passed: success, duplicate settlement, provider error, command error.')
})().catch(e => { console.error(e); process.exitCode = 1 })
