const fs=require('fs'),vm=require('vm'),ts=require('typescript'),assert=require('node:assert/strict');
function load(file,mocks={}) {const exports={};vm.runInNewContext(ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText,{exports,require:n=>mocks[n],console,setTimeout,clearTimeout});return exports}
const model=load('src/agent/hub/model.ts');
async function test(name,klass,agent,records,error,rpcError=false) {
 let channel;let observedModel;
 const base='src/agent/hub/drivers/';const mocks={'../model':model,'../paths':{isWslHost:()=>false},'./geminiCliProtocol':load(base+'geminiCliProtocol.ts')};
 for(const p of ['claudeCode','codex','opencode','antigravity'])mocks['./'+p+'Protocol']=load(base+p+'Protocol.ts');
 mocks['@tauri-apps/api/core']={Channel:class{},invoke:async(cmd,args)=>{
  if(cmd==='agent_proc_spawn')channel=args.onEvent;
  if(cmd==='agent_proc_stdin'&&name==='codex'){
   const req=JSON.parse(args.line);if(!req.id)return;queueMicrotask(()=>channel.onmessage({type:'line',line:JSON.stringify(rpcError?{id:req.id,error:{message:'RPC failed'}}:{id:req.id,result:req.method==='thread/start'?{thread:{id:'thread-test'},model:'native-model'}:{}})}));
  }
 }};
 const driver=new (load(base+name+'.ts',mocks)[klass])();
 const promise=(async()=>{const handle=await driver.start({sessionId:'test-'+agent,messages:[{role:'user',content:'hi'}],engine:{agentId:agent,host:'linux',installation:{spawn:{}}}},{onModel:m=>{observedModel=m}});for(const r of records)channel.onmessage(r.type==='exit'?r:{type:'line',line:JSON.stringify(r)});return handle.done})();
 let timer;const deadline=new Promise((_,reject)=>{timer=setTimeout(()=>reject(Error('hung driver')),1000)});
 try{if(error)await assert.rejects(Promise.race([promise,deadline]),error);else assert.equal((await Promise.race([promise,deadline])).text,'hello');}finally{clearTimeout(timer)}
 console.log('PASS',agent,error?'error':'success');
}
(async()=>{
 await test('claudeCode','ClaudeCodeDriver','claude-code',[{type:'result',subtype:'success',result:'hello'}]);
 await test('claudeCode','ClaudeCodeDriver','claude-code',[{type:'result',subtype:'error_during_execution',is_error:true,errors:['auth failed']}],/./);
 await test('codex','CodexDriver','codex',[{method:'item/agentMessage/delta',params:{delta:'hello'}},{method:'turn/completed',params:{turn:{status:'completed'}}}]);
 await test('codex','CodexDriver','codex',[],/RPC failed/,true);
 await test('opencode','OpencodeDriver','opencode',[{type:'text',part:{text:'hello'}},{type:'exit',code:0}]);
 await test('opencode','OpencodeDriver','opencode',[{type:'exit',code:1}],/code 1/);
 await test('antigravity','AntigravityDriver','antigravity',[{event:'result',result:{response:'hello',status:'OK'}}]);
 await test('antigravity','AntigravityDriver','antigravity',[{type:'exit',code:1}],/exited/);
 for(const agent of ['gemini-cli','qwen-code','cursor-cli']) {
  await test('geminiCli','GeminiCliDriver',agent,[{response:'hello',result:'hello',stats:{models:{'native-model':{}}}},{type:'exit',code:0}]);
  await test('geminiCli','GeminiCliDriver',agent,[{error:{message:'auth failed'}},{type:'exit',code:1}],/auth failed/);
 }
 assert.equal(model.nativeModel({data:{model:{id:'native-model'}}}),'native-model');
 assert.equal(model.nativeModel({model:'unknown'}),undefined);
})().catch(e=>{console.error(e);process.exitCode=1});
