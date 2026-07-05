// src/stores/agent.ts

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { AgentLoop } from '@/agent/loop'
import type { AgentConfig, AgentEvent, AgentState, AgentStep, AgentStepType } from '@/agent/types'
import { useAgentPoolStore } from './agentPool'
import type { MessageMetadata } from './aiChat'

const defaultConfig: AgentConfig = {
  model: '',
  provider: '',
  confirmationMode: 'dangerous',
  maxIterations: 10,
  temperature: 0.7,
}

export const useAgentStore = defineStore('agent', () => {
  const config = ref<AgentConfig>({ ...defaultConfig })
  const state = ref<AgentState>({
    status: 'idle',
    currentStep: null,
    history: [],
    confirmationRequired: false,
  })
  const traceLog = ref<AgentEvent[]>([])
  const tracePanelOpen = ref(false)
  // Metadata of the message whose trace the right rail is focused on. Null means
  // "follow the latest assistant turn" (live streaming view).
  const selectedTrace = ref<MessageMetadata | null>(null)

  let loop: AgentLoop | null = null
  let manualStepCounter = 0

  const isRunning = computed(
    () => state.value.status === 'executing' || state.value.status === 'planning'
  )
  const history = computed(() => state.value.history)
  const currentStatus = computed(() => state.value.status)

  function initLoop(): AgentLoop {
    loop = new AgentLoop(config.value)
    loop.onEvent((event: AgentEvent) => {
      traceLog.value.push(event)
      state.value = loop?.getState() ?? state.value
    })
    return loop
  }

  async function run(message: string): Promise<void> {
    if (!loop) initLoop()
    await loop?.run(message)
  }

  function runWithPool(prompt: string, description: string): string {
    const pool = useAgentPoolStore()
    return pool.schedule({
      id: `task-${Date.now()}`,
      description,
      prompt,
      tools: ['read', 'edit', 'bash', 'search'],
      abort_on_error: true,
    })
  }

  async function executeSkill(skillId: string, args: any): Promise<any> {
    if (!loop) initLoop()
    return await loop?.executeSkill(skillId, args)
  }

  function abort(): void {
    loop?.abort()
  }

  function reset(): void {
    loop?.reset()
    traceLog.value = []
    state.value = {
      status: 'idle',
      currentStep: null,
      history: [],
      confirmationRequired: false,
    }
  }

  function setStatus(status: AgentState['status']): void {
    state.value.status = status
    traceLog.value.push({
      type: 'status_change',
      status,
      timestamp: Date.now(),
    })
  }

  function startStep(type: AgentStepType, content: string, detail?: string): AgentStep {
    const step: AgentStep = {
      id: `manual-step-${++manualStepCounter}`,
      type,
      status: 'running',
      content,
      detail,
      timestamp: Date.now(),
    }
    state.value.currentStep = step
    state.value.history.push(step)
    traceLog.value.push({
      type: 'step_start',
      step,
      timestamp: Date.now(),
    })
    return step
  }

  function completeStep(step: AgentStep, result?: unknown): void {
    step.status = 'done'
    step.result = result
    if (state.value.currentStep?.id === step.id) {
      state.value.currentStep = null
    }
    traceLog.value.push({
      type: 'step_done',
      step,
      timestamp: Date.now(),
    })
  }

  function failStep(step: AgentStep, error: string): void {
    step.status = 'error'
    step.error = error
    if (state.value.currentStep?.id === step.id) {
      state.value.currentStep = null
    }
    traceLog.value.push({
      type: 'step_error',
      step,
      timestamp: Date.now(),
    })
  }

  function toggleTracePanel(): void {
    tracePanelOpen.value = !tracePanelOpen.value
  }

  return {
    config,
    state,
    traceLog,
    tracePanelOpen,
    selectedTrace,
    isRunning,
    history,
    currentStatus,
    initLoop,
    run,
    runWithPool,
    executeSkill,
    abort,
    reset,
    setStatus,
    startStep,
    completeStep,
    failStep,
    toggleTracePanel,
  }
})
