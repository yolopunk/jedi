// src/stores/agent.ts
//
// Trace/step bookkeeping for the chat UI. The manual step API
// (startStep/completeStep/failStep/setStatus) is driven by aiChat's runAgent
// hooks; runWithPool schedules background workers. The old AgentLoop wrapper
// was removed — runAgent (risk-based gating) is the single execution path.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { AgentEvent, AgentState, AgentStep, AgentStepType } from '@/agent/types'
import { useAgentPoolStore } from './agentPool'
import type { MessageMetadata } from './aiChat'

export const useAgentStore = defineStore('agent', () => {
  const state = ref<AgentState>({
    status: 'idle',
    currentStep: null,
    history: [],
  })
  const traceLog = ref<AgentEvent[]>([])
  const tracePanelOpen = ref(false)
  // Metadata of the message whose trace the right rail is focused on. Null means
  // "follow the latest assistant turn" (live streaming view).
  const selectedTrace = ref<MessageMetadata | null>(null)

  let manualStepCounter = 0

  const isRunning = computed(
    () => state.value.status === 'executing' || state.value.status === 'planning'
  )
  const history = computed(() => state.value.history)
  const currentStatus = computed(() => state.value.status)

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

  function reset(): void {
    traceLog.value = []
    state.value = {
      status: 'idle',
      currentStep: null,
      history: [],
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
    state,
    traceLog,
    tracePanelOpen,
    selectedTrace,
    isRunning,
    history,
    currentStatus,
    runWithPool,
    reset,
    setStatus,
    startStep,
    completeStep,
    failStep,
    toggleTracePanel,
  }
})
