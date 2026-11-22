import { computed, reactive, ref, watch } from 'vue'
import { driverFor } from '@/agent/hub/registry'
import type { DriverHandle, DriverStartOpts, EngineDescriptor } from '@/agent/hub/types'
import { type RunAgentHooks, summarizeValue } from '@/agent/runAgent'
import { withStatsHooks } from '@/agent/statsHooks'
import {
  deleteAgentGroup,
  listAgentGroups,
  saveAgentGroup,
  stopAgentGroupProcess,
} from '@/api/agents'
import i18n from '@/i18n'
import { useAgentHubStore } from '@/stores/agentHub'
import type {
  AgentApproval,
  AgentEngineOption,
  AgentGroup,
  AgentGroupMember,
  AgentGroupMessage,
  AgentGroupRun,
  AgentRunTool,
  CreateAgentGroupInput,
  PendingAgentApproval,
} from '@/types/agents'
import { ACTIVE_AGENT_RUN_STATUSES } from '@/types/agents'
import { isAbsoluteProjectPath } from '@/utils/path'

const MAX_GROUPS = 100
const MAX_MEMBERS = 16
const MAX_MESSAGES = 2000
const MAX_RUNS = 500
const MAX_CONTEXT_MESSAGES = 80
const MAX_CONTEXT_CHARS = 80_000
const MAX_REPLY_CHARS = 32_000
// Leave room below SQLite's 4 MiB limit for server fields and final run metadata.
const MAX_GROUP_BYTES = 3_670_016
const RUN_METADATA_RESERVE = 65_536
const CHECKPOINT_INTERVAL_MS = 500
const IDLE_TIMEOUT_MS = 5 * 60 * 1000

function uuid(): string {
  return crypto.randomUUID()
}

function errorText(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause)
}

function cloneGroup(group: AgentGroup): AgentGroup {
  return JSON.parse(JSON.stringify(group)) as AgentGroup
}

function groupBytes(group: AgentGroup): number {
  return new TextEncoder().encode(JSON.stringify(group)).byteLength
}

function cancelled(): Error {
  return new Error('aborted')
}

function withCancellation<T>(promise: Promise<T>, signal: AbortSignal): Promise<T> {
  if (signal.aborted) return Promise.reject(cancelled())
  return new Promise<T>((resolve, reject) => {
    const abort = () => reject(cancelled())
    signal.addEventListener('abort', abort, { once: true })
    void promise.then(resolve, reject).finally(() => signal.removeEventListener('abort', abort))
  })
}

function createAgentGroupState() {
  const agentHub = useAgentHubStore()
  const groups = ref<AgentGroup[]>([])
  const currentGroupId = ref<string | null>(null)
  const loading = ref(false)
  const initialized = ref(false)
  const saveCount = ref(0)
  const scanPending = ref(false)
  const operationError = ref<string | null>(null)
  const saveErrors = reactive(new Map<string, string>())
  const approvals = ref<AgentApproval[]>([])
  const currentGroup = computed(
    () => groups.value.find(group => group.id === currentGroupId.value) ?? null
  )
  const error = computed(
    () =>
      saveErrors.get(currentGroupId.value ?? '') ??
      operationError.value ??
      [...saveErrors.values()][0] ??
      null
  )
  const currentSaveError = computed(() => saveErrors.get(currentGroupId.value ?? '') ?? null)
  const saving = computed(() => saveCount.value > 0)
  const scanning = computed(() => scanPending.value || agentHub.scanning)
  const activeRuns = computed(() =>
    (currentGroup.value?.runs ?? []).filter(run => ACTIVE_AGENT_RUN_STATUSES.has(run.status))
  )
  const engineOptions = computed<AgentEngineOption[]>(() =>
    agentHub.engines
      .filter(engine => engine.tier !== 'builtin')
      .map(engine => {
        const modes =
          driverFor(engine)?.capabilities.groupChatModes?.filter(mode => mode === 'discussion') ??
          []
        const status = engine.installation?.status ?? 'unknown'
        return {
          id: engine.id,
          name: engine.name,
          location: engine.location,
          status,
          modes,
          available: modes.length > 0 && status !== 'needs_auth' && status !== 'invalid_path',
        }
      })
      .sort((a, b) => a.name.localeCompare(b.name) || a.location.localeCompare(b.location))
  )

  const controllers = new Map<string, AbortController>()
  const handles = new Map<string, DriverHandle>()
  const runQueues = new Map<string, Promise<void>>()
  const saveQueues = new Map<string, Promise<void>>()
  const saveGenerations = new Map<string, number>()
  const checkpointTimers = new Map<string, ReturnType<typeof setTimeout>>()
  const pendingApprovals = new Map<string, PendingAgentApproval>()
  const outputLimits = new Map<string, number>()
  const dispatching = new Set<string>()
  const deleting = new Set<string>()

  function getGroup(groupId: string): AgentGroup | undefined {
    return groups.value.find(group => group.id === groupId)
  }

  function setError(cause: unknown): void {
    operationError.value = errorText(cause)
  }

  function fail(key: string, params: Record<string, string | number> = {}): never {
    const cause = new Error(i18n.global.t(`agents.errors.${key}`, params))
    setError(cause)
    throw cause
  }

  function cancelCheckpoint(groupId: string): void {
    const timer = checkpointTimers.get(groupId)
    if (timer) clearTimeout(timer)
    checkpointTimers.delete(groupId)
  }

  // Coalesce changes arriving during a write into one subsequent snapshot.
  // Every caller waits until its changes have been included in a durable save.
  function persist(groupId: string): Promise<void> {
    cancelCheckpoint(groupId)
    if (deleting.has(groupId) || !getGroup(groupId)) return Promise.resolve()
    saveGenerations.set(groupId, (saveGenerations.get(groupId) ?? 0) + 1)
    const existing = saveQueues.get(groupId)
    if (existing) return existing
    const task = Promise.resolve().then(async () => {
      try {
        while (!deleting.has(groupId)) {
          const group = getGroup(groupId)
          if (!group) return
          const generation = saveGenerations.get(groupId)
          if (groupBytes(group) > MAX_GROUP_BYTES) fail('historyBytes')
          const saved = await saveAgentGroup(cloneGroup(group))
          const current = getGroup(groupId)
          if (current) {
            current.revision = saved.revision
            current.updatedAt = saved.updatedAt
          }
          saveErrors.delete(groupId)
          if (saveGenerations.get(groupId) === generation) return
        }
      } catch (cause) {
        saveErrors.set(groupId, errorText(cause))
        throw cause
      } finally {
        if (saveQueues.get(groupId) === task) saveQueues.delete(groupId)
        saveCount.value = saveQueues.size
      }
    })
    saveQueues.set(groupId, task)
    saveCount.value = saveQueues.size
    return task
  }

  function persistInBackground(groupId: string): void {
    void persist(groupId).catch(() => undefined)
  }

  function scheduleCheckpoint(groupId: string): void {
    if (deleting.has(groupId) || checkpointTimers.has(groupId)) return
    checkpointTimers.set(
      groupId,
      setTimeout(() => {
        checkpointTimers.delete(groupId)
        persistInBackground(groupId)
      }, CHECKPOINT_INTERVAL_MS)
    )
  }

  async function init(): Promise<void> {
    if (initialized.value || loading.value) return
    loading.value = true
    operationError.value = null
    try {
      const saved = await listAgentGroups()
      groups.value = saved.map(group => ({
        ...group,
        messages: [...group.messages].sort((a, b) => a.seq - b.seq),
        runs: group.runs.map(run => ({ ...run, tools: run.tools ?? [] })),
      }))
      currentGroupId.value = groups.value[0]?.id ?? null
      for (const group of groups.value) {
        let recovered = false
        for (const member of group.members) {
          if (member.nativeSessionId || member.lastDeliveredSeq) recovered = true
          // Policy-bound native sessions are trusted only within this renderer.
          member.nativeSessionId = undefined
          member.lastDeliveredSeq = 0
        }
        for (const run of group.runs) {
          if (!ACTIVE_AGENT_RUN_STATUSES.has(run.status)) continue
          // A renderer reload may leave a process in the still-running backend.
          await stopAgentGroupProcess(run.id).catch(() => undefined)
          run.status = 'interrupted'
          run.finishedAt = new Date().toISOString()
          for (const tool of run.tools) if (tool.status === 'running') tool.status = 'failed'
          recovered = true
        }
        if (recovered) persistInBackground(group.id)
      }
      initialized.value = true
    } catch (cause) {
      setError(cause)
    } finally {
      loading.value = false
    }
    // History is usable while CLI discovery runs, including slow WSL startups.
    if (initialized.value) void scan(false).catch(() => undefined)
  }

  async function scan(force = true): Promise<void> {
    if (scanPending.value) return
    scanPending.value = true
    try {
      if (agentHub.scanning) {
        await new Promise<void>(resolve => {
          const stop = watch(
            () => agentHub.scanning,
            value => {
              if (!value) {
                stop()
                resolve()
              }
            }
          )
        })
      } else {
        await agentHub.detect(force)
      }
      if (agentHub.error) throw new Error(agentHub.error)
    } catch (cause) {
      setError(cause)
      throw cause
    } finally {
      scanPending.value = false
    }
  }

  function selectGroup(groupId: string): void {
    if (getGroup(groupId) && !deleting.has(groupId)) currentGroupId.value = groupId
  }

  function eligibleEngine(
    engineId: string,
    group: Pick<AgentGroup, 'executionMode'>
  ): EngineDescriptor {
    const engine = agentHub.engines.find(candidate => candidate.id === engineId)
    const option = engineOptions.value.find(candidate => candidate.id === engineId)
    if (!engine || !option?.available) fail('engineUnavailable', { name: engine?.name ?? engineId })
    if (!option.modes.includes(group.executionMode)) fail('modeUnavailable', { name: engine.name })
    return engine
  }

  async function createGroup(input: CreateAgentGroupInput): Promise<AgentGroup> {
    operationError.value = null
    const name = input.name.trim()
    const projectPath = input.projectPath.trim()
    if (!name) fail('nameRequired')
    if (name.length > 80) fail('nameTooLong')
    if (projectPath.length > 2048 || !isAbsoluteProjectPath(projectPath)) fail('pathAbsolute')
    if (!input.members.length || input.members.length > MAX_MEMBERS) fail('membersRequired')
    if (groups.value.length >= MAX_GROUPS) fail('tooManyGroups')
    if (new Set(input.members.map(member => member.engineId)).size !== input.members.length)
      fail('duplicateEngine')
    for (const member of input.members) eligibleEngine(member.engineId, input)
    const now = new Date().toISOString()
    const group: AgentGroup = {
      id: uuid(),
      name,
      projectPath,
      executionMode: input.executionMode,
      members: input.members.map(({ engineId, role }) => ({
        id: uuid(),
        engineId,
        name: agentHub.engines.find(engine => engine.id === engineId)?.name ?? engineId,
        role: role.trim().slice(0, 160),
        enabled: true,
        lastDeliveredSeq: 0,
      })),
      messages: [],
      runs: [],
      createdAt: now,
      updatedAt: now,
      revision: 0,
    }
    const previousSelection = currentGroupId.value
    groups.value.unshift(group)
    currentGroupId.value = group.id
    try {
      await persist(group.id)
      return getGroup(group.id)!
    } catch (cause) {
      groups.value = groups.value.filter(item => item.id !== group.id)
      saveErrors.delete(group.id)
      currentGroupId.value = previousSelection
      setError(cause)
      throw cause
    }
  }

  async function deleteGroup(groupId: string): Promise<void> {
    if (!getGroup(groupId) || deleting.has(groupId)) return
    // Tombstone before aborting: late hooks, checkpoints and run cleanup cannot
    // enqueue an upsert after DELETE. An already-started save is drained first.
    deleting.add(groupId)
    cancelCheckpoint(groupId)
    stopGroup(groupId)
    try {
      await saveQueues.get(groupId)?.catch(() => undefined)
      await deleteAgentGroup(groupId)
      groups.value = groups.value.filter(group => group.id !== groupId)
      saveErrors.delete(groupId)
      saveGenerations.delete(groupId)
      if (currentGroupId.value === groupId) currentGroupId.value = groups.value[0]?.id ?? null
    } catch (cause) {
      deleting.delete(groupId)
      setError(cause)
      persistInBackground(groupId)
      throw cause
    }
  }

  function settleApproval(approvalId: string, allow: boolean): void {
    const pending = pendingApprovals.get(approvalId)
    if (!pending) return
    pendingApprovals.delete(approvalId)
    approvals.value = approvals.value.filter(item => item.id !== approvalId)
    const request = pending.request
    const run = getGroup(request.groupId)?.runs.find(item => item.id === request.runId)
    if (
      run?.status === 'awaiting_approval' &&
      !approvals.value.some(item => item.runId === run.id)
    ) {
      run.status = 'running'
    }
    pending.resolve(allow)
  }

  function clearRunApprovals(runId: string): void {
    for (const approval of approvals.value.filter(item => item.runId === runId))
      settleApproval(approval.id, false)
  }

  function requestApproval(
    group: AgentGroup,
    run: AgentGroupRun,
    member: AgentGroupMember,
    req: { toolName: string; args: unknown },
    signal: AbortSignal
  ): Promise<boolean> {
    if (group.executionMode !== 'approval' || signal.aborted || deleting.has(group.id))
      return Promise.resolve(false)
    const request: AgentApproval = {
      id: uuid(),
      groupId: group.id,
      runId: run.id,
      memberId: member.id,
      memberName: member.name,
      toolName: req.toolName,
      args: req.args,
    }
    run.status = 'awaiting_approval'
    return new Promise(resolve => {
      const abort = () => settleApproval(request.id, false)
      pendingApprovals.set(request.id, {
        request,
        resolve: allow => {
          signal.removeEventListener('abort', abort)
          resolve(allow)
        },
      })
      approvals.value.push(request)
      signal.addEventListener('abort', abort, { once: true })
      void persist(group.id).catch(() => settleApproval(request.id, false))
    })
  }

  async function decideApproval(approvalId: string, allow: boolean): Promise<void> {
    const approval = approvals.value.find(item => item.id === approvalId)
    if (!approval) return
    // Commit the decision's state before releasing the native tool request.
    const group = getGroup(approval.groupId)
    const run = group?.runs.find(item => item.id === approval.runId)
    if (!group || !run) {
      settleApproval(approvalId, false)
      return
    }
    try {
      await persist(group.id)
      settleApproval(approvalId, allow)
      persistInBackground(group.id)
    } catch (cause) {
      settleApproval(approvalId, false)
      throw cause
    }
  }

  function updateTool(run: AgentGroupRun, tool: AgentRunTool): void {
    const bounded = {
      ...tool,
      id: tool.id.slice(0, 128),
      name: tool.name.slice(0, 96),
      detail: tool.detail?.slice(0, 200),
    }
    const index = run.tools.findIndex(item => item.id === bounded.id)
    if (index === -1) run.tools.push(bounded)
    else run.tools[index] = { ...run.tools[index], ...bounded }
    if (run.tools.length > 16) run.tools.splice(0, run.tools.length - 16)
  }

  function transcriptFor(group: AgentGroup, member: AgentGroupMember, run: AgentGroupRun): string {
    const runById = new Map(group.runs.map(item => [item.id, item]))
    const prior = group.messages.filter(message => message.seq <= run.contextSeq)
    const completed = prior.filter(
      message =>
        !message.runId ||
        !ACTIVE_AGENT_RUN_STATUSES.has(runById.get(message.runId)?.status ?? 'completed')
    )
    const selected: object[] = []
    let remaining = MAX_CONTEXT_CHARS
    let includesTrigger = false
    for (const message of completed.slice().reverse()) {
      const isTrigger = message.id === run.triggerMessageId
      // A retry may target an old request. Reserve a slot even when its
      // trigger sits outside the most recent context window.
      if (!isTrigger && selected.length >= MAX_CONTEXT_MESSAGES - (includesTrigger ? 0 : 1))
        continue
      const entry = {
        id: message.id,
        seq: message.seq,
        role: message.role,
        author: message.role === 'user' ? 'user' : message.authorName,
        memberId: message.authorId,
        recipients: message.recipients,
        content:
          message.id === run.triggerMessageId ? message.content : message.content.slice(-16_000),
        status: message.runId ? runById.get(message.runId)?.status : undefined,
      }
      const length = JSON.stringify(entry).length
      // Never truncate or omit the triggering request, even when JSON escaping
      // expands its accepted 20,000 characters beyond the history budget.
      if (length > remaining && message.id !== run.triggerMessageId) continue
      selected.unshift(entry)
      if (isTrigger) includesTrigger = true
      remaining -= length
    }
    return [
      i18n.global.t('agents.prompt.introduction', {
        group: group.name,
        name: member.name,
        engine: member.engineId,
        role: member.role || i18n.global.t('agents.prompt.defaultRole'),
        path: group.projectPath,
      }),
      i18n.global.t('agents.prompt.snapshotHint'),
      i18n.global.t('agents.prompt.omitted', { count: completed.length - selected.length }),
      i18n.global.t('agents.prompt.pendingReplies', { count: prior.length - completed.length }),
      JSON.stringify(selected),
      i18n.global.t('agents.prompt.currentRequest', { id: run.triggerMessageId }),
      i18n.global.t('agents.prompt.replyInstruction'),
    ].join('\n\n')
  }

  async function runMember(groupId: string, runId: string): Promise<void> {
    const group = getGroup(groupId)
    const run = group?.runs.find(item => item.id === runId)
    const member = group?.members.find(item => item.id === run?.memberId)
    if (!group || !run || !member || deleting.has(groupId) || run.status !== 'queued') return
    const controller = new AbortController()
    controllers.set(runId, controller)
    const reply = group.messages.find(item => item.id === run.messageId)
    let settled = false
    let lastActivityAt = Date.now()
    let timedOut = false
    const live = () =>
      !settled &&
      !controller.signal.aborted &&
      !deleting.has(groupId) &&
      getGroup(groupId) === group
    const activity = () => {
      if (!live()) return false
      lastActivityAt = Date.now()
      return true
    }
    const writeReply = (text: string) => {
      if (!reply) return
      const limit = outputLimits.get(runId) ?? MAX_REPLY_CHARS
      reply.content = text.slice(0, limit)
      if (text.length > limit) run.error = i18n.global.t('agents.errors.outputTruncated')
    }
    let watchdog: ReturnType<typeof setInterval> | undefined
    try {
      const engine = eligibleEngine(member.engineId, group)
      const driver = driverFor(engine)!
      run.status = 'running'
      run.startedAt = new Date().toISOString()
      await persist(groupId)
      if (!live()) return
      const base: RunAgentHooks = {
        onModel: model => {
          if (activity() && model !== run.model) {
            run.model = model.slice(0, 128)
            scheduleCheckpoint(groupId)
          }
        },
        onNativeSessionId: id => {
          if (!activity() || !id) return
          member.nativeSessionId = id.slice(0, 512)
          scheduleCheckpoint(groupId)
        },
        onTextDelta: event => {
          if (activity()) {
            writeReply(event.fullContent)
            scheduleCheckpoint(groupId)
          }
        },
        onReasoningDelta: () => {
          activity()
        },
        onToolCall: () => {
          activity()
        },
        onToolResult: () => {
          activity()
        },
        onStepFinish: () => {
          activity()
        },
        onFinish: () => {
          activity()
        },
        onToolStart: event => {
          if (!activity()) return
          updateTool(run, {
            id: event.toolId,
            name: event.toolName,
            status: 'running',
            detail: event.toolDescription,
          })
          scheduleCheckpoint(groupId)
        },
        onToolEnd: event => {
          updateTool(run, {
            id: event.toolId,
            name: event.toolName,
            status: event.error ? 'failed' : 'completed',
            detail: event.error ?? summarizeValue(event.output, 200),
          })
          scheduleCheckpoint(groupId)
        },
      }
      const stats = withStatsHooks(base)
      const hooks = {
        ...stats,
        onToolEnd: (event: Parameters<NonNullable<RunAgentHooks['onToolEnd']>>[0]) => {
          if (activity()) stats.onToolEnd?.(event)
        },
      }
      const options: DriverStartOpts = {
        provider: '',
        model: '',
        apiKey: '',
        messages: [{ role: 'user', content: transcriptFor(group, member, run) }],
        sessionId: `agents:${group.id}:${member.id}`,
        processId: `jedi-agents-${run.id}`,
        signal: controller.signal,
        engine,
        nativeSessionId: member.nativeSessionId,
        cwd: group.projectPath,
        executionMode: group.executionMode,
        confirmTool: async request => {
          if (!activity()) return false
          try {
            return await requestApproval(group, run, member, request, controller.signal)
          } finally {
            activity()
          }
        },
      }
      const stalled = new Promise<never>((_, reject) => {
        watchdog = setInterval(() => {
          if (run.status === 'awaiting_approval' || Date.now() - lastActivityAt < IDLE_TIMEOUT_MS)
            return
          timedOut = true
          reject(new Error(i18n.global.t('agents.errors.timeout')))
          controller.abort()
        }, 15_000)
      })
      const starting = driver.start(options, hooks).then(handle => {
        // A stopped startup can return after the caller has already unwound.
        if (!live()) {
          handle.abort?.()
          handle.dispose?.()
          void handle.done.catch(() => undefined)
        }
        return handle
      })
      const handle = await withCancellation(Promise.race([starting, stalled]), controller.signal)
      if (!live()) return
      handles.set(runId, handle)
      const result = await withCancellation(Promise.race([handle.done, stalled]), controller.signal)
      if (!live()) return
      if (result.text && !reply?.content) writeReply(result.text)
      if (result.nativeSessionId) member.nativeSessionId = result.nativeSessionId.slice(0, 512)
      member.lastDeliveredSeq = Math.max(member.lastDeliveredSeq, run.contextSeq)
      if (result.finishReason === 'error') throw new Error(i18n.global.t('agents.errors.runFailed'))
      run.status = result.finishReason === 'aborted' ? 'interrupted' : 'completed'
    } catch (cause) {
      run.status = controller.signal.aborted && !timedOut ? 'stopped' : 'failed'
      if (run.status === 'failed') {
        run.error = timedOut
          ? i18n.global.t('agents.errors.timeout')
          : errorText(cause).slice(0, 1024)
        setError(run.error)
      }
    } finally {
      settled = true
      if (watchdog) clearInterval(watchdog)
      clearRunApprovals(runId)
      if (run.status !== 'completed') {
        member.nativeSessionId = undefined
        member.lastDeliveredSeq = 0
      }
      for (const tool of run.tools) if (tool.status === 'running') tool.status = 'failed'
      handles.get(runId)?.dispose?.()
      handles.delete(runId)
      controllers.delete(runId)
      run.finishedAt = new Date().toISOString()
      outputLimits.delete(runId)
      await persist(groupId).catch(() => undefined)
    }
  }

  async function sendMessage(content: string, recipients: string[] = []): Promise<void> {
    const group = currentGroup.value
    if (!group) fail('groupMissing')
    await dispatchMessage(group, content, recipients)
  }

  async function retryRun(groupId: string, runId: string): Promise<void> {
    const group = getGroup(groupId)
    const run = group?.runs.find(item => item.id === runId)
    const trigger = group?.messages.find(item => item.id === run?.triggerMessageId)
    if (!group || !run || !trigger || trigger.role !== 'user') fail('retryUnavailable')
    if (!['failed', 'stopped', 'interrupted'].includes(run.status)) fail('retryUnavailable')
    await dispatchMessage(group, trigger.content, [run.memberId], trigger)
  }

  async function dispatchMessage(
    group: AgentGroup,
    content: string,
    recipients: string[],
    retryTrigger?: AgentGroupMessage
  ): Promise<void> {
    operationError.value = null
    if (deleting.has(group.id)) fail('groupDeleting')
    if (dispatching.has(group.id)) fail('dispatchBusy')
    const text = content.trim()
    if (!text) fail('messageRequired')
    if (text.length > 20_000) fail('messageTooLong')
    if (recipients.some(id => !group.members.some(member => member.id === id)))
      fail('invalidRecipient')
    const selected = group.members.filter(member =>
      recipients.length ? recipients.includes(member.id) : member.enabled
    )
    if (!selected.length) fail('recipientsRequired')
    if (selected.some(member => !member.enabled)) fail('recipientPaused')
    if (
      selected.some(member =>
        group.runs.some(
          run => run.memberId === member.id && ACTIVE_AGENT_RUN_STATUSES.has(run.status)
        )
      )
    )
      fail('recipientsBusy')
    for (const member of selected) eligibleEngine(member.engineId, group)
    if (
      group.messages.length + (retryTrigger ? 0 : 1) + selected.length > MAX_MESSAGES ||
      group.runs.length + selected.length > MAX_RUNS
    )
      fail('historyLimit')

    const now = new Date().toISOString()
    let seq = group.messages.reduce((latest, message) => Math.max(latest, message.seq), 0)
    const userMessage: AgentGroupMessage = retryTrigger ?? {
      id: uuid(),
      seq: ++seq,
      role: 'user',
      authorName: 'user',
      recipients: selected.map(member => member.id),
      content: text,
      createdAt: now,
    }
    const contextSeq = seq
    const newMessages: AgentGroupMessage[] = retryTrigger ? [] : [userMessage]
    const newRuns: AgentGroupRun[] = selected.map(member => {
      const id = uuid()
      const messageId = uuid()
      newMessages.push({
        id: messageId,
        seq: ++seq,
        role: 'assistant',
        authorId: member.id,
        authorName: member.name,
        recipients: [member.id],
        content: '',
        createdAt: now,
        runId: id,
      })
      return {
        id,
        memberId: member.id,
        triggerMessageId: userMessage.id,
        messageId,
        status: 'queued',
        contextSeq,
        tools: [],
      }
    })
    const candidate = {
      ...group,
      messages: [...group.messages, ...newMessages],
      runs: [...group.runs, ...newRuns],
    }
    const reserved = group.runs.reduce(
      (bytes, run) =>
        bytes +
        (outputLimits.has(run.id) ? outputLimits.get(run.id)! * 6 + RUN_METADATA_RESERVE : 0),
      0
    )
    // JSON uses at most six UTF-8 bytes per UTF-16 code unit, including escapes.
    const limit = Math.min(
      MAX_REPLY_CHARS,
      Math.floor(
        (MAX_GROUP_BYTES -
          groupBytes(candidate) -
          reserved -
          selected.length * RUN_METADATA_RESERVE) /
          (selected.length * 6)
      )
    )
    if (limit < 2000) fail('historyBytes')
    dispatching.add(group.id)
    group.messages.push(...newMessages)
    group.runs.push(...newRuns)
    for (const run of newRuns) outputLimits.set(run.id, limit)
    try {
      await persist(group.id)
      if (deleting.has(group.id)) fail('groupDeleting')
      for (const run of newRuns) {
        const memberKey = `${group.id}:${run.memberId}`
        const previous = runQueues.get(memberKey) ?? Promise.resolve()
        const task = previous.catch(() => undefined).then(() => runMember(group.id, run.id))
        runQueues.set(memberKey, task)
        void task
          .finally(() => {
            if (runQueues.get(memberKey) === task) runQueues.delete(memberKey)
          })
          .catch(() => undefined)
      }
    } catch (cause) {
      const messageIds = new Set(newMessages.map(message => message.id))
      const runIds = new Set(newRuns.map(run => run.id))
      group.messages = group.messages.filter(message => !messageIds.has(message.id))
      group.runs = group.runs.filter(run => !runIds.has(run.id))
      for (const run of newRuns) outputLimits.delete(run.id)
      setError(cause)
      // Another member's checkpoint may have observed the tentative records.
      // Retry Save always writes the rolled-back, dispatchable state.
      if (!deleting.has(group.id)) scheduleCheckpoint(group.id)
      throw cause
    } finally {
      dispatching.delete(group.id)
    }
  }

  async function toggleMember(memberId: string): Promise<void> {
    const group = currentGroup.value
    const member = group?.members.find(item => item.id === memberId)
    if (!group || !member || deleting.has(group.id)) return
    const previous = member.enabled
    member.enabled = !previous
    try {
      await persist(group.id)
    } catch (cause) {
      member.enabled = previous
      throw cause
    }
  }

  function stopRun(group: AgentGroup, run: AgentGroupRun): void {
    if (!ACTIVE_AGENT_RUN_STATUSES.has(run.status)) return
    run.status = 'stopped'
    run.finishedAt = new Date().toISOString()
    clearRunApprovals(run.id)
    controllers.get(run.id)?.abort()
    handles.get(run.id)?.abort?.()
    if (!controllers.has(run.id)) outputLimits.delete(run.id)
    const member = group.members.find(item => item.id === run.memberId)
    if (member) {
      member.nativeSessionId = undefined
      member.lastDeliveredSeq = 0
    }
    for (const tool of run.tools) if (tool.status === 'running') tool.status = 'failed'
    persistInBackground(group.id)
  }

  function stopMember(memberId: string): void {
    const group = currentGroup.value
    if (group)
      for (const run of group.runs.filter(item => item.memberId === memberId)) stopRun(group, run)
  }

  function stopGroup(groupId = currentGroupId.value): void {
    const group = groupId ? getGroup(groupId) : undefined
    if (group) for (const run of group.runs) stopRun(group, run)
  }

  async function retrySave(): Promise<void> {
    if (!currentGroup.value) return
    await persist(currentGroup.value.id)
    operationError.value = null
  }

  return reactive({
    groups,
    currentGroupId,
    currentGroup,
    activeRuns,
    engineOptions,
    approvals,
    loading,
    initialized,
    saving,
    scanning,
    error,
    currentSaveError,
    init,
    scan,
    selectGroup,
    createGroup,
    deleteGroup,
    sendMessage,
    retryRun,
    toggleMember,
    stopMember,
    stopGroup,
    decideApproval,
    retrySave,
  })
}

// Lazy initialization: importing a route must not access Pinia before app.use().
let agentGroupState: ReturnType<typeof createAgentGroupState> | undefined
export function useAgentsStore() {
  agentGroupState ??= createAgentGroupState()
  return agentGroupState
}
