<template>
  <div class="agent-trace">
    <div class="trace-header">
      <span class="trace-title">// AGENT TRACE</span>
      <span class="trace-status" :class="statusClass">{{ agentStore.currentStatus.toUpperCase() }}</span>
    </div>
    <div ref="traceList" class="trace-list">
      <div
        v-for="(event, index) in agentStore.traceLog"
        :key="`${event.timestamp}-${event.type}-${index}`"
        class="trace-entry"
        :class="event.type"
      >
        <span class="trace-time">{{ formatTime(event.timestamp) }}</span>
        <span class="trace-icon">{{ getIcon(event) }}</span>
        <span class="trace-content">{{ getContent(event) }}</span>
      </div>
      <div v-if="agentStore.traceLog.length === 0" class="trace-empty">
        <span class="empty-text">NO TRACE DATA</span>
      </div>
    </div>
    <div class="trace-footer">
      <span class="footer-text">STEPS: {{ agentStore.history.length }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { AgentEvent } from '@/agent/types'
import { useAgentStore } from '@/stores/agent'

const agentStore = useAgentStore()
const traceList = ref<HTMLElement | null>(null)

const statusClass = computed(() => ({
  idle: agentStore.currentStatus === 'idle',
  running: agentStore.currentStatus === 'executing' || agentStore.currentStatus === 'planning',
  done: agentStore.currentStatus === 'done',
  error: agentStore.currentStatus === 'error',
}))

function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString('en-US', { hour12: false })
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
    case 'confirmation_needed':
      return '?'
    default:
      return '-'
  }
}

function getContent(event: AgentEvent): string {
  if (event.step)
    return event.step.detail ? `${event.step.content}: ${event.step.detail}` : event.step.content
  if (event.status) return `Status: ${event.status}`
  return event.type
}

watch(
  () => agentStore.traceLog.length,
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
  background: rgba(0, 0, 0, 0.3);
  border-left: 1px solid rgba(0, 255, 255, 0.2);
}

.trace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
  background: rgba(0, 255, 255, 0.05);
}

.trace-title {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  font-weight: 700;
  color: #00ffff;
  letter-spacing: 1px;
}

.trace-status {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 3px;
}

.trace-status.idle { color: #52525b; }
.trace-status.running { color: #00ffff; }
.trace-status.done { color: #00ff88; }
.trace-status.error { color: #ff4444; }

.trace-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.trace-entry {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  line-height: 1.4;
}

.trace-time {
  color: #52525b;
  flex-shrink: 0;
}

.trace-icon {
  flex-shrink: 0;
  width: 12px;
  text-align: center;
}

.trace-entry.step_start .trace-icon { color: #00ffff; }
.trace-entry.step_done .trace-icon { color: #00ff88; }
.trace-entry.step_error .trace-icon { color: #ff4444; }
.trace-entry.status_change .trace-icon { color: #ffaa00; }
.trace-entry.confirmation_needed .trace-icon { color: #ff00ff; }

.trace-content {
  color: #a1a1aa;
  word-break: break-word;
}

.trace-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.empty-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: #3f3f46;
  letter-spacing: 1px;
}

.trace-footer {
  padding: 8px 16px;
  border-top: 1px solid rgba(0, 255, 255, 0.1);
  background: rgba(0, 0, 0, 0.2);
}

.footer-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: #52525b;
  letter-spacing: 1px;
}

.trace-list::-webkit-scrollbar { width: 6px; }
.trace-list::-webkit-scrollbar-track { background: transparent; }
.trace-list::-webkit-scrollbar-thumb { background: rgba(0, 255, 255, 0.2); border-radius: 3px; }
</style>
