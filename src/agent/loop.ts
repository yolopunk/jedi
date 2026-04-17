// src/agent/loop.ts

import { skillRegistry } from '@/skills/registry'
import { useModelsDevStore } from '@/stores/modelsDev'
import type { AgentConfig, AgentEvent, AgentState, AgentStep } from './types'

type EventHandler = (event: AgentEvent) => void

let stepCounter = 0
function nextStepId(): string {
  return `step-${++stepCounter}`
}

function emitEvent(handlers: EventHandler[], event: AgentEvent) {
  handlers.forEach(h => {
    h(event)
  })
}

export class AgentLoop {
  private state: AgentState = {
    status: 'idle',
    currentStep: null,
    history: [],
    confirmationRequired: false,
  }
  private config: AgentConfig
  private eventHandlers: EventHandler[] = []
  private aborted = false

  constructor(config: AgentConfig) {
    this.config = config
  }

  /**
   * Get the currently selected provider and model from modelsDevStore.
   * Use this when sending chat messages or creating completions.
   */
  protected getModelInfo(): { providerId: string | null; modelId: string | null } {
    const modelsDevStore = useModelsDevStore()
    return {
      providerId: modelsDevStore.selectedProviderId,
      modelId: modelsDevStore.selectedModelId,
    }
  }

  getState(): AgentState {
    return { ...this.state }
  }

  onEvent(handler: EventHandler): () => void {
    this.eventHandlers.push(handler)
    return () => {
      this.eventHandlers = this.eventHandlers.filter(h => h !== handler)
    }
  }

  abort(): void {
    this.aborted = true
    this.setStatus('paused')
  }

  reset(): void {
    this.aborted = false
    this.state = {
      status: 'idle',
      currentStep: null,
      history: [],
      confirmationRequired: false,
    }
  }

  private setStatus(status: AgentState['status']): void {
    this.state.status = status
    emitEvent(this.eventHandlers, {
      type: 'status_change',
      status,
      timestamp: Date.now(),
    })
  }

  private addStep(type: AgentStep['type'], content: string): AgentStep {
    const step: AgentStep = {
      id: nextStepId(),
      type,
      status: 'running',
      content,
      timestamp: Date.now(),
    }
    this.state.currentStep = step
    this.state.history.push(step)
    emitEvent(this.eventHandlers, {
      type: 'step_start',
      step,
      timestamp: Date.now(),
    })
    return step
  }

  private completeStep(step: AgentStep, result?: any): void {
    step.status = 'done'
    step.result = result
    this.state.currentStep = null
    emitEvent(this.eventHandlers, {
      type: 'step_done',
      step,
      timestamp: Date.now(),
    })
  }

  private failStep(step: AgentStep, error: string): void {
    step.status = 'error'
    step.error = error
    this.state.currentStep = null
    emitEvent(this.eventHandlers, {
      type: 'step_error',
      step,
      timestamp: Date.now(),
    })
  }

  async executeSkill(skillId: string, args: any): Promise<any> {
    const skill = skillRegistry.get(skillId)
    if (!skill) throw new Error(`Skill not found: ${skillId}`)
    if (!skill.enabled) throw new Error(`Skill disabled: ${skillId}`)

    const step = this.addStep('skill', `Executing skill: ${skill.name}`)

    if (this.config.confirmationMode === 'always') {
      this.state.confirmationRequired = true
      emitEvent(this.eventHandlers, {
        type: 'confirmation_needed',
        step,
        timestamp: Date.now(),
      })
      return
    }

    try {
      this.setStatus('executing')
      const result = await skill.execute(args, { sessionId: '' })
      this.completeStep(step, result)
      return result
    } catch (e: any) {
      this.failStep(step, e.message)
      throw e
    }
  }

  async run(userMessage: string): Promise<void> {
    if (this.state.status === 'executing') return
    this.aborted = false

    // Planning phase
    this.setStatus('planning')
    const planStep = this.addStep('think', `Analyzing: ${userMessage}`)
    this.completeStep(planStep, 'Plan created')

    // Execution phase
    this.setStatus('executing')

    let iterations = 0
    while (!this.aborted && iterations < this.config.maxIterations) {
      iterations++

      // This is a simplified loop - in production, the LLM would
      // determine what steps to take and when to finish
      const finishStep = this.addStep('finish', 'Task complete')
      this.completeStep(finishStep)
      break
    }

    if (this.aborted) {
      this.setStatus('paused')
    } else {
      this.setStatus('done')
    }
  }
}
