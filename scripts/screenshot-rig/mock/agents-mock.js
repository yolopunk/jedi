// Browser-only group fixture. Loaded only by the agents verification config.
// Wrap the screenshot rig IPC; never spawn a CLI or read application data.
(() => {
  // Tauri v2 event cleanup is called during HMR as well as page unmount.
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ ||= { unregisterListener() {} }
  const internals = window.__TAURI_INTERNALS__
  const originalInvoke = internals.invoke.bind(internals)
  const key = 'jedi-agents-preview-groups'
  const now = new Date().toISOString()
  const seed = [{
    id: 'preview-group', name: '本地智能体协作', projectPath: 'E:\\projects\\demo',
    executionMode: 'discussion', createdAt: now, updatedAt: now, revision: 1,
    members: [{ id: 'preview-member', engineId: 'claude-code@windows', name: 'Claude Code', role: '架构审查', enabled: true, lastDeliveredSeq: 0 }],
    messages: [
      { id: 'preview-request', seq: 1, role: 'user', authorName: 'user', recipients: ['preview-member'], content: '请检查这个项目的会话恢复方案。', createdAt: now },
      { id: 'preview-reply', seq: 2, role: 'assistant', authorId: 'preview-member', authorName: 'Claude Code', recipients: ['preview-member'], content: '已检查持久化入口，接下来核对异常恢复。', createdAt: now, runId: 'preview-run' },
    ],
    runs: [{ id: 'preview-run', memberId: 'preview-member', triggerMessageId: 'preview-request', messageId: 'preview-reply', contextSeq: 1, status: 'failed', error: '模拟连接中断，可重试本次回复。', tools: [{ id: 'preview-tool', name: 'Read', status: 'completed', detail: '已读取会话配置。' }], startedAt: now, finishedAt: now }],
  }]
  let groups = JSON.parse(localStorage.getItem(key) || 'null') || seed
  const processes = new Map()
  const save = () => localStorage.setItem(key, JSON.stringify(groups))
  internals.invoke = async (cmd, args = {}) => {
    if (cmd === 'list_agent_groups') return structuredClone(groups)
    if (cmd === 'save_agent_group') {
      const group = structuredClone(args.group)
      const previous = groups.find(item => item.id === group.id)
      group.revision = (previous?.revision || 0) + 1
      group.updatedAt = new Date().toISOString()
      groups = [group, ...groups.filter(item => item.id !== group.id)]
      save()
      return structuredClone(group)
    }
    if (cmd === 'delete_agent_group') {
      groups = groups.filter(item => item.id !== args.groupId)
      save()
      return
    }
    if (cmd === 'agent_proc_kill') {
      clearTimeout(processes.get(args.id)?.timer)
      processes.delete(args.id)
      return
    }
    if (cmd === 'agent_proc_spawn') {
      if (!args.id.startsWith('jedi-agents-')) throw Error('Only group fixtures are supported')
      processes.set(args.id, { channel: args.onEvent })
      return
    }
    if (cmd === 'agent_proc_stdin') {
      const process = processes.get(args.id)
      if (!process) return
      const line = record => process.channel.onmessage({ type: 'line', line: JSON.stringify(record) })
      line({ type: 'system', subtype: 'init', permissionMode: 'dontAsk', tools: ['Read', 'Glob', 'Grep'], session_id: 'preview-session' })
      process.timer = setTimeout(() => {
        line({ type: 'assistant', message: { role: 'assistant', content: [{ type: 'text', text: '### 会话恢复检查\n\n- 历史回复与工具记录已保留。\n- 中断后可只重试指定成员。\n- 重试使用最新共享上下文。\n\n这是离线验证数据。' }] } })
        line({ type: 'result', subtype: 'success', result: '### 会话恢复检查\n\n- 历史回复与工具记录已保留。\n- 中断后可只重试指定成员。\n- 重试使用最新共享上下文。\n\n这是离线验证数据。', session_id: 'preview-session', is_error: false })
      }, 600)
      return
    }
    return originalInvoke(cmd, args)
  }
})()
