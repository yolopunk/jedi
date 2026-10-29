const fs=require('fs'),vm=require('vm'),ts=require('typescript'),assert=require('node:assert/strict');
function load(file,mocks={}) {const exports={};vm.runInNewContext(ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText,{exports,require:n=>mocks[n],console,setTimeout,clearTimeout});return exports}
const model=load('src/agent/hub/model.ts');
const diagnostics=load('src/agent/hub/agentError.ts');
async function testHostResume() {
 let channel,spawnArgs;
 const {ClaudeCodeDriver}=load('src/agent/hub/drivers/claudeCode.ts',{
  '../model':model,'../agentError':diagnostics,
  './claudeCodeProtocol':load('src/agent/hub/drivers/claudeCodeProtocol.ts'),
  '@tauri-apps/api/core':{Channel:class{},invoke:async(cmd,args)=>{
   if(cmd==='agent_proc_spawn'){channel=args.onEvent;spawnArgs=args.args;}
  }},
 });
 const driver=new ClaudeCodeDriver();
 for(const [host,resume] of [['windows',undefined],['wsl:Ubuntu',undefined],['windows','native-windows']]) {
  const handle=await driver.start({sessionId:'same-chat',messages:[{role:'user',content:'hi'}],engine:{id:'claude-code@'+host,agentId:'claude-code',host,installation:{spawn:{}}}},{});
  const index=spawnArgs.indexOf('--resume');
  assert.equal(index<0?undefined:spawnArgs[index+1],resume);
  channel.onmessage({type:'line',line:JSON.stringify({type:'result',subtype:'success',result:'hello',session_id:'native-'+host})});
  await handle.done;
 }
 console.log('PASS host-specific in-memory resume');
}
async function test(name,klass,agent,records,error,rpcError=false) {
 let channel;let observedModel;
 const base='src/agent/hub/drivers/';const mocks={'../agentError':diagnostics,'../model':model,'../paths':{isWslHost:()=>false},'./geminiCliProtocol':load(base+'geminiCliProtocol.ts')};
 for(const p of ['claudeCode','codex','opencode','antigravity','pi'])mocks['./'+p+'Protocol']=load(base+p+'Protocol.ts');
 mocks['@tauri-apps/api/core']={Channel:class{},invoke:async(cmd,args)=>{
  if(cmd==='agent_proc_spawn')channel=args.onEvent;
  if(cmd==='agent_proc_stdin'&&name==='codex'){
   const req=JSON.parse(args.line);if(!req.id)return;queueMicrotask(()=>channel.onmessage({type:'line',line:JSON.stringify(rpcError?{id:req.id,error:{message:'RPC failed'}}:{id:req.id,result:req.method==='thread/start'?{thread:{id:'thread-test'},model:'native-model'}:{}})}));
  }
 }};
 const driver=new (load(base+name+'.ts',mocks)[klass])();
 const promise=(async()=>{const handle=await driver.start({sessionId:'test-'+agent,messages:[{role:'user',content:'hi'}],engine:{id:agent+'@linux',name:agent,agentId:agent,host:'linux',installation:{spawn:{}}}},{onModel:m=>{observedModel=m}});for(const r of records)channel.onmessage(r.type==='exit'?r:{type:'line',line:JSON.stringify(r)});return handle.done})();
 let timer;const deadline=new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('hung driver')),1000)});
 try{if(error)await assert.rejects(Promise.race([promise,deadline]),error);else assert.equal((await Promise.race([promise,deadline])).text,'hello');}finally{clearTimeout(timer)}
 console.log('PASS',agent,error?'error':'success');
}
(async()=>{
 await testHostResume();
 await test('claudeCode','ClaudeCodeDriver','claude-code',[{type:'result',subtype:'success',result:'hello'}]);
 await test('claudeCode','ClaudeCodeDriver','claude-code',[{type:'result',subtype:'error_during_execution',is_error:true,errors:['auth failed']}],/./);
 await test('codex','CodexDriver','codex',[{method:'item/agentMessage/delta',params:{delta:'hello'}},{method:'turn/completed',params:{turn:{status:'completed'}}}]);
 await test('codex','CodexDriver','codex',[],/RPC failed/,true);
 await test('opencode','OpencodeDriver','opencode',[{type:'text',part:{text:'hello'}},{type:'exit',code:0}]);
 await test('opencode','OpencodeDriver','opencode',[{type:'exit',code:1}],e=>e.kind==='crash' && e.detail.includes('exit code: 1'));
 await test('antigravity','AntigravityDriver','antigravity',[{event:'result',result:{response:'hello',status:'OK'}}]);
 await test('antigravity','AntigravityDriver','antigravity',[{type:'exit',code:1}],e=>e.kind==='crash' && e.detail.includes('exit code: 1'));
 for(const agent of ['gemini-cli','qwen-code','cursor-cli']) {
  await test('geminiCli','GeminiCliDriver',agent,[{response:'hello',result:'hello',stats:{models:{'native-model':{}}}},{type:'exit',code:0}]);
  await test('geminiCli','GeminiCliDriver',agent,[{error:{message:'auth failed'}},{type:'exit',code:1}],e=>e.kind==='crash');
 }
 await test('codex','CodexDriver','codex',[{method:'item/agentMessage/delta',params:{delta:'hello'}},{method:'turn/completed',params:{turn:{status:'failed',error:{message:'model unavailable'}}}}],e=>e.kind==='model');
 await test('claudeCode','ClaudeCodeDriver','claude-code',[{type:'assistant',message:{content:[{type:'text',text:'hello'}]}},{type:'exit',code:1}],e=>e.kind==='crash');
 await test('opencode','OpencodeDriver','opencode',[{type:'text',part:{text:'hello'}},{type:'exit',code:1}],e=>e.kind==='crash' && e.detail.includes('exit code: 1'));
 await test('pi','PiDriver','pi',[{type:'message_end',message:{role:'assistant',content:[{text:'hello'}],errorMessage:'model unavailable'}},{type:'agent_settled'}],e=>e.kind==='model');
 assert.equal(model.nativeModel({data:{model:{id:'native-model'}}}),'native-model');
 assert.equal(model.nativeModel({model:'unknown'}),undefined);
})().catch(e=>{console.error(e);process.exitCode=1});
