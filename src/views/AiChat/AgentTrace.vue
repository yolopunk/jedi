<template>
  <div class="agent-trace">
    <div class="trace-header">
      <span class="trace-title">{{ t('agentTrace.title') }}</span>
      <div class="trace-header-actions">
        <span class="trace-status" :class="statusClass">{{ statusText(agentStore.currentStatus) }}</span>
        <button class="trace-close" :aria-label="t('common.close')" @click="handleClose">
          {{ t('common.close') }}
        </button>
      </div>
    </div>
    <div v-if="latestMetadata?.run" class="trace-run-meta">
      <div class="meta-grid">
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.provider') }}</span>
          <span class="meta-value">{{ latestMetadata.run.provider }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.model') }}</span>
          <span class="meta-value">{{ latestMetadata.run.model }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.tools') }}</span>
          <span class="meta-value">{{ latestMetadata.run.toolCount }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.duration') }}</span>
          <span class="meta-value">{{ formatDuration(latestMetadata.run.totalDurationMs) }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.stepBudget') }}</span>
          <span class="meta-value">{{ latestMetadata.run.stepLimit }}</span>
        </div>
        <div class="meta-card">
          <span class="meta-label">{{ t('agentTrace.meta.finish') }}</span>
          <span class="meta-value">{{ latestMetadata.run.finishReason || t('agentTrace.pending') }}</span>
        </div>
      </div>
      <div v-if="latestMetadata.run.enabledTools.length" class="enabled-tools">
        <span class="enabled-label">{{ t('agentTrace.toolBelt') }}</span>
        <span class="enabled-value">{{ latestMetadata.run.enabledTools.join(', ') }}</span>
      </div>
      <details v-if="latestMetadata.reasoning" class="trace-details">
        <summary>{{ t('agentTrace.reasoningSummary') }}</summary>
        <pre>{{ latestMetadata.reasoning }}</pre>
      </details>
      <div v-if="usageSummary" class="usage-summary">
        <span class="usage-label">{{ t('agentTrace.usage') }}</span>
        <span class="usage-value">{{ usageSummary }}</span>
      </div>
    </div>
    <div ref="traceList" class="trace-list">
      <div
        v-for="trace in ungroupedTrace"
        :key="trace.id"
        class="trace-card"
        :class="[trace.type, trace.status]"
      >
        <div class="trace-card-row">
          <span class="trace-time">{{ formatTime(trace.timestamp) }}</span>
          <span class="trace-icon">{{ getTraceIcon(trace.type, trace.status) }}</span>
          <span class="trace-name" :title="trace.title">{{ trace.title }}</span>
          <span class="trace-kind">{{ kindText(trace.type) }}</span>
          <span class="trace-state">{{ statusText(trace.status) }}</span>
          <span v-if="trace.durationMs !== undefined" class="trace-duration">
            {{ formatDuration(trace.durationMs) }}
          </span>
        </div>
        <div
          v-if="trace.content"
          class="trace-description"
          :class="{ 'is-error': trace.status === 'error' }"
        >{{ trace.content }}</div>
        <details v-if="trace.input || trace.output" class="trace-details">
          <summary>{{ t('agentTrace.details') }}</summary>
          <pre v-if="trace.input">{{ t('agentTrace.input') }} {{ formatTraceValue(trace.input) }}</pre>
          <pre v-if="trace.output">{{ t('agentTrace.output') }} {{ formatTraceValue(trace.output) }}</pre>
        </details>
      </div>
      <details
        v-for="group in groupedTrace"
        :key="`trace-step-${group.stepIndex}`"
        class="trace-step-group"
        :class="group.status"
        :open="group.stepIndex === groupedTrace.length"
      >
        <summary class="trace-step-summary">
          <span class="trace-step-title">{{ t('agentTrace.step', { n: group.stepIndex }) }}</span>
          <span class="trace-step-kind">{{ kindText(group.kind) }}</span>
          <span class="trace-step-status">{{ statusText(group.status) }}</span>
          <span v-if="group.durationMs !== undefined" class="trace-step-duration">
            {{ formatDuration(group.durationMs) }}
          </span>
        </summary>
        <div v-if="group.summary" class="trace-step-text">{{ group.summary }}</div>
        <div
          v-for="trace in group.events"
          :key="trace.id"
          class="trace-card step-child"
          :class="[trace.type, trace.status]"
        >
          <div class="trace-card-row">
            <span class="trace-time">{{ formatTime(trace.timestamp) }}</span>
            <span class="trace-icon">{{ getTraceIcon(trace.type, trace.status) }}</span>
            <span class="trace-name" :title="trace.title">{{ trace.title }}</span>
            <span class="trace-kind">{{ kindText(trace.type) }}</span>
            <span class="trace-state">{{ statusText(trace.status) }}</span>
            <span v-if="trace.durationMs !== undefined" class="trace-duration">
              {{ formatDuration(trace.durationMs) }}
            </span>
          </div>
          <div
            v-if="trace.content"
            class="trace-description"
            :class="{ 'is-error': trace.status === 'error' }"
          >{{ trace.content }}</div>
          <details v-if="trace.input || trace.output" class="trace-details">
            <summary>{{ t('agentTrace.details') }}</summary>
            <pre v-if="trace.input">{{ t('agentTrace.input') }} {{ formatTraceValue(trace.input) }}</pre>
            <pre v-if="trace.output">{{ t('agentTrace.output') }} {{ formatTraceValue(trace.output) }}</pre>
          </details>
        </div>
      </details>
      <div class="raw-divider" v-if="agentStore.traceLog.length > 0 && latestTrace.length > 0">
        {{ t('agentTrace.rawLog') }}
      </div>
      <div
        v-for="(event, index) in agentStore.traceLog"
        :key="`${event.timestamp}-${event.type}-${index}`"
        class="trace-entry raw"
        :class="event.type"
      >
        <span class="trace-time">{{ formatTime(event.timestamp) }}</span>
        <span class="trace-icon">{{ getIcon(event) }}</span>
        <span class="trace-content">{{ getContent(event) }}</span>
      </div>
      <div v-if="agentStore.traceLog.length === 0 && latestTrace.length === 0" class="trace-empty">
        <span class="empty-text">{{ t('agentTrace.empty') }}</span>
      </div>
    </div>
    <div class="trace-footer">
      <span class="footer-text">{{ t('agentTrace.steps', { n: agentStore.history.length }) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AgentEvent } from '@/agent/types'
import { useAgentStore } from '@/stores/agent'
import { type AgentTraceDetail, useAiChatStore } from '@/stores/aiChat'

const agentStore = useAgentStore()
const aiChatStore = useAiChatStore()
const { t, te } = useI18n()
const traceList = ref<HTMLElement | null>(null)

// Unknown enum values fall back to the raw value instead of rendering a key path.
function statusText(status: string): string {
  const key = `agentTrace.status.${status.toLowerCase()}`
  return te(key) ? t(key) : status
}

function kindText(kind: string): string {
  const key = `agentTrace.kind.${kind.toLowerCase()}`
  return te(key) ? t(key) : kind
}

const latestAssistantMessage = computed(() => {
  const messages = aiChatStore.currentSession?.messages || []
  return [...messages].reverse().find(message => message.role === 'assistant') || null
})
// Focused message (clicked the trace button) wins; otherwise follow the latest turn.
const latestMetadata = computed(
  () => agentStore.selectedTrace || latestAssistantMessage.value?.metadata || null
)
const latestTrace = computed(() => latestMetadata.value?.trace || [])
const ungroupedTrace = computed(() =>
  latestTrace.value.filter(item => item.stepIndex === undefined)
)
const groupedTrace = computed(() => getTraceGroups(latestTrace.value))

const statusClass = computed(() => ({
  idle: agentStore.currentStatus === 'idle',
  running: agentStore.currentStatus === 'executing' || agentStore.currentStatus === 'planning',
  done: agentStore.currentStatus === 'done',
  error: agentStore.currentStatus === 'error',
}))
const usageSummary = computed(() => formatUsageSummary(latestMetadata.value?.usage))

function handleClose(): void {
  agentStore.tracePanelOpen = false
  agentStore.selectedTrace = null
}

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString('en-US', { hour12: false })
}

function formatDuration(ms?: number): string {
  if (ms === undefined) return '--'
  if (ms < 1000) return `${ms}ms`
  if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`
  return `${(ms / 60000).toFixed(2)}m`
}

function formatTraceValue(value: unknown): string {
  const text = typeof value === 'string' ? value : JSON.stringify(value, null, 2)
  return text.length > 2200 ? `${text.slice(0, 2200)}\n...` : text
}

function formatUsageSummary(usage: unknown): string {
  if (!usage || typeof usage !== 'object') return ''
  const usageRecord = usage as Record<string, unknown>
  const values = [
    typeof usageRecord.inputTokens === 'number' ? `in ${usageRecord.inputTokens}` : null,
    typeof usageRecord.outputTokens === 'number' ? `out ${usageRecord.outputTokens}` : null,
    typeof usageRecord.reasoningTokens === 'number'
      ? `reason ${usageRecord.reasoningTokens}`
      : null,
    typeof usageRecord.totalTokens === 'number' ? `total ${usageRecord.totalTokens}` : null,
    typeof usageRecord.promptTokens === 'number' ? `prompt ${usageRecord.promptTokens}` : null,
    typeof usageRecord.completionTokens === 'number'
      ? `completion ${usageRecord.completionTokens}`
      : null,
  ].filter(Boolean)
  return values.join(' / ')
}

function getTraceIcon(type: string, status: string): string {
  if (status === 'error') return '!'
  if (type === 'tool') return '$'
  if (type === 'step') return '#'
  if (type === 'reasoning') return '~'
  if (type === 'answer') return '>'
  if (type === 'finish') return '+'
  return '#'
}

type TraceGroup = {
  stepIndex: number
  kind: string
  status: AgentTraceDetail['status']
  durationMs?: number
  summary: string
  events: AgentTraceDetail[]
}

function getTraceGroups(trace: AgentTraceDetail[] = []): TraceGroup[] {
  const grouped = new Map<number, AgentTraceDetail[]>()
  for (const item of trace) {
    if (item.stepIndex === undefined) continue
    const items = grouped.get(item.stepIndex) || []
    items.push(item)
    grouped.set(item.stepIndex, items)
  }

  return [...grouped.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([stepIndex, events]) => {
      const stepTrace = events.find(item => item.type === 'step')
      return {
        stepIndex,
        kind:
          typeof stepTrace?.output === 'object' && stepTrace?.output
            ? String((stepTrace.output as Record<string, unknown>).type || 'step')
            : 'step',
        status:
          stepTrace?.status === 'error' || events.some(item => item.status === 'error')
            ? 'error'
            : 'done',
        durationMs: stepTrace?.durationMs,
        summary: stepTrace?.content || events[0]?.content || '',
        events: events.filter(item => item.type !== 'step'),
      }
    })
}

function getIcon(event: AgentEvent): string {
  switch (event.type) {
    case 'step_start':
      return '>'
    case 'step_done':
      return '+'
    case 'step_error':
      return '!'
    case 'status_change':
      return '~'
    default:
      return '-'
  }
}

function getContent(event: AgentEvent): string {
  if (event.step)
    return event.step.detail ? `${event.step.content}: ${event.step.detail}` : event.step.content
  if (event.status) return `${t('agentTrace.statusChange')}: ${statusText(event.status)}`
  return event.type
}

watch(
  () => [agentStore.traceLog.length, latestTrace.value.length],
  () => {
    nextTick(() => {
      if (traceList.value) {
        traceList.value.scrollTop = traceList.value.scrollHeight
      }
    })
  }
)
</script>

<style scoped>
.agent-trace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-terminal);
  border-left: 1px solid rgb(var(--accent-rgb) / 0.22);
}

.trace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgb(var(--accent-rgb) / 0.2);
  background: rgb(var(--accent-rgb) / 0.05);
}

.trace-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.trace-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--accent);
}

.trace-status {
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 4px;
}

.trace-status.idle { color: var(--text-subtle); }
.trace-status.running { color: var(--accent); }
.trace-status.done { color: var(--success); }
.trace-status.error { color: var(--danger); }

.trace-close {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  border: 1px solid rgb(var(--accent-rgb) / 0.2);
  border-radius: 6px;
  background: rgb(var(--accent-rgb) / 0.08);
  color: var(--accent-hover);
  font-size: 12px;
  padding: 0 12px;
  cursor: pointer;
}

.trace-close:hover {
  background: rgb(var(--accent-rgb) / 0.14);
}

.trace-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.trace-run-meta {
  padding: 12px;
  border-bottom: 1px solid rgb(var(--accent-rgb) / 0.12);
  background:rgb(var(--ink-rgb) / 0.18);
}

.meta-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.meta-card {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 8px;
  border: 1px solid rgb(var(--accent-rgb) / 0.12);
  border-radius: 8px;
  background:rgb(var(--ink-rgb) / 0.03);
  min-width: 0;
}

.meta-label,
.enabled-label,
.usage-label {
  font-size: 12px;
  color: var(--text-subtle);
  letter-spacing: 0.3px;
}

.meta-value,
.enabled-value,
.usage-value {
  font-size: 12px;
  color: var(--accent-hover);
  line-height: 1.5;
  word-break: break-word;
}

.enabled-tools,
.usage-summary {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.trace-card {
  margin-bottom: 8px;
  padding: 10px;
  border-left: 2px solid rgb(var(--accent-rgb) / 0.24);
  border-radius: 8px;
  background:rgb(var(--ink-rgb) / 0.03);
}

.trace-card.tool {
  border-left-color: rgb(var(--success-rgb) / 0.7);
}

.trace-card.error {
  border-left-color: rgb(var(--danger-rgb) / 0.75);
}

.trace-card.step-child {
  margin: 0 8px 8px;
}

.trace-step-group {
  margin-bottom: 10px;
  border: 1px solid rgb(var(--accent-rgb) / 0.18);
  border-radius: 8px;
  background: rgb(var(--accent-rgb) / 0.04);
}

.trace-step-summary {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 10px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent-hover);
}

.trace-step-title {
  color: var(--accent-hover);
}

.trace-step-kind {
  color: var(--text-subtle);
  font-size: 11px;
  font-weight: 400;
  font-family: var(--jedi-font-mono);
}

.trace-step-status,
.trace-step-duration {
  color: var(--text-subtle);
  font-size: 12px;
  font-weight: 400;
}

.trace-step-group.error .trace-step-status { color: var(--danger); }
.trace-step-group.done .trace-step-status { color: var(--success); }

.trace-step-text {
  padding: 0 10px 8px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.55;
  word-break: break-word;
}

.raw-divider {
  margin: 12px 0 8px;
  padding-top: 8px;
  border-top: 1px dashed rgb(var(--accent-rgb) / 0.14);
  font-size: 12px;
  color: var(--text-subtle);
  letter-spacing: 0.3px;
}

.trace-card-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.trace-name {
  flex: 1;
  min-width: 0;
  color: var(--accent-hover);
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.trace-kind {
  color: var(--text-subtle);
  font-size: 11px;
  font-family: var(--jedi-font-mono);
  flex-shrink: 0;
}

.trace-state {
  color: var(--accent-hover);
  font-size: 12px;
  flex-shrink: 0;
}

.trace-duration {
  color: var(--text-subtle);
  font-size: 12px;
  flex-shrink: 0;
}

.trace-card.running .trace-state { color: var(--accent); }
.trace-card.done .trace-state { color: var(--success); }
.trace-card.error .trace-state { color: var(--danger); }

.trace-description {
  margin-top: 6px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
}

.trace-description.is-error {
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  white-space: normal;
  color: var(--danger);
}

.trace-entry {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  font-family: var(--jedi-font-mono);
  font-size: 12px;
  line-height: 1.4;
}

.trace-time {
  color: var(--text-subtle);
  flex-shrink: 0;
  font-family: var(--jedi-font-mono);
  font-size: 11px;
}

.trace-icon {
  flex-shrink: 0;
  width: 12px;
  text-align: center;
}

.trace-entry.step_start .trace-icon { color: var(--accent); }
.trace-entry.step_done .trace-icon { color: var(--success); }
.trace-entry.step_error .trace-icon { color: var(--danger); }
.trace-entry.status_change .trace-icon { color: var(--warning); }

.trace-entry.raw {
  opacity: 0.82;
  border-top: 1px dashed rgb(var(--accent-rgb) / 0.08);
  margin-top: 10px;
  padding-top: 10px;
}

.trace-content {
  color: var(--text-muted);
  word-break: break-word;
}

.trace-details {
  margin-top: 8px;
  color: var(--accent-hover);
  font-size: 12px;
}

.trace-details summary {
  cursor: pointer;
  color: var(--accent);
}

.trace-details pre {
  max-height: 260px;
  overflow: auto;
  margin: 6px 0 0;
  padding: 8px;
  border-radius: 6px;
  background:rgb(var(--ink-rgb) / 0.26);
  color: var(--accent-hover);
  font-family: var(--jedi-font-mono);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.trace-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.empty-text {
  font-size: 12px;
  color: var(--text-subtle);
}

.trace-footer {
  padding: 8px 16px;
  border-top: 1px solid rgb(var(--accent-rgb) / 0.1);
  background:rgb(var(--ink-rgb) / 0.2);
}

.footer-text {
  font-size: 12px;
  color: var(--text-subtle);
}

.trace-list::-webkit-scrollbar { width: 6px; }
.trace-list::-webkit-scrollbar-track { background: transparent; }
.trace-list::-webkit-scrollbar-thumb { background: rgb(var(--accent-rgb) / 0.2); border-radius: 3px; }
</style>
