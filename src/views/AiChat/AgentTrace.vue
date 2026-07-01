<template>
  <div class="agent-trace">
    <!-- 空状态 -->
    <div v-if="rows.length === 0" class="trace-empty">
      <div class="trace-empty-icon">◇</div>
      <div class="trace-empty-text">NO AGENT ACTIVITY</div>
      <div class="trace-empty-hint">
        启用 MCP 服务后，Agent 的工具调用过程会在此实时显示
      </div>
    </div>

    <!-- 追踪日志 -->
    <div v-else class="trace-log">
      <div
        v-for="(row, index) in rows"
        :key="index"
        class="trace-row"
        :class="row.kind"
      >
        <div class="trace-line">
          <span class="trace-marker">{{ row.marker }}</span>
          <span class="trace-label">{{ row.label }}</span>
        </div>
        <pre v-if="row.detail" class="trace-detail">{{ row.detail }}</pre>
      </div>

      <!-- 运行中指示 -->
      <div v-if="running" class="trace-running">
        <span class="trace-spinner"></span>
        <span class="trace-running-text">AGENT RUNNING</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { AgentEvent } from '@/api/ai-chat'

const props = defineProps<{
  trace: AgentEvent[]
  running?: boolean
}>()

interface TraceRow {
  kind: string
  marker: string
  label: string
  detail?: string
}

function truncate(text: string, max = 600): string {
  if (text.length <= max) return text
  return text.slice(0, max) + `\n… (+${text.length - max} chars)`
}

const rows = computed<TraceRow[]>(() => {
  const result: TraceRow[] = []
  for (const event of props.trace) {
    switch (event.type) {
      case 'thinking':
        result.push({ kind: 'thinking', marker: '»', label: 'THINK', detail: truncate(event.text) })
        break
      case 'tool_call': {
        const target = event.server ? `${event.server}.${event.name}` : event.name
        let args = ''
        try {
          args = JSON.stringify(event.arguments, null, 2)
        } catch {
          args = String(event.arguments)
        }
        result.push({ kind: 'tool-call', marker: '▶', label: `CALL  ${target}`, detail: truncate(args) })
        break
      }
      case 'tool_result':
        result.push({
          kind: event.is_error ? 'error' : 'tool-result',
          marker: event.is_error ? '✗' : '✓',
          label: `${event.is_error ? 'FAILED' : 'RESULT'}  ${event.name}`,
          detail: truncate(event.content),
        })
        break
      case 'content':
        result.push({ kind: 'content', marker: '★', label: 'ANSWER READY' })
        break
      case 'done':
        result.push({ kind: 'done', marker: '─', label: 'TURN COMPLETE' })
        break
      case 'error':
        result.push({ kind: 'error', marker: '✗', label: 'ERROR', detail: event.message })
        break
    }
  }
  return result
})
</script>

<style scoped>
.agent-trace {
  flex: 1;
  overflow-y: auto;
  padding: 10px 12px;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

/* 空状态 */
.trace-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  text-align: center;
  padding: 24px 16px;
  gap: 10px;
}

.trace-empty-icon {
  font-size: 28px;
  color: #3f3f46;
}

.trace-empty-text {
  font-size: 10px;
  letter-spacing: 2px;
  color: #52525b;
  font-weight: 700;
}

.trace-empty-hint {
  font-size: 10px;
  line-height: 1.6;
  color: #52525b;
}

/* 日志 */
.trace-log {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.trace-row {
  border-left: 2px solid #27272a;
  padding: 4px 0 4px 10px;
}

.trace-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.trace-marker {
  font-size: 11px;
  width: 12px;
  flex-shrink: 0;
  text-align: center;
}

.trace-label {
  font-size: 10.5px;
  letter-spacing: 0.5px;
  font-weight: 700;
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
}

.trace-detail {
  margin: 6px 0 0;
  padding: 6px 8px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid #1f1f28;
  border-radius: 4px;
  font-size: 10px;
  line-height: 1.5;
  color: #a1a1aa;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 160px;
  overflow-y: auto;
}

/* 各事件类型配色 */
.trace-row.thinking { border-left-color: #a78bfa; }
.trace-row.thinking .trace-marker,
.trace-row.thinking .trace-label { color: #a78bfa; }

.trace-row.tool-call { border-left-color: #22d3ee; }
.trace-row.tool-call .trace-marker,
.trace-row.tool-call .trace-label { color: #22d3ee; text-shadow: 0 0 8px rgba(34, 211, 238, 0.35); }

.trace-row.tool-result { border-left-color: #4ade80; }
.trace-row.tool-result .trace-marker,
.trace-row.tool-result .trace-label { color: #4ade80; }

.trace-row.content { border-left-color: #60a5fa; }
.trace-row.content .trace-marker,
.trace-row.content .trace-label { color: #60a5fa; }

.trace-row.done { border-left-color: #3f3f46; }
.trace-row.done .trace-marker,
.trace-row.done .trace-label { color: #71717a; }

.trace-row.error { border-left-color: #f87171; }
.trace-row.error .trace-marker,
.trace-row.error .trace-label { color: #f87171; }

/* 运行中 */
.trace-running {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 2px;
  margin-top: 4px;
}

.trace-spinner {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #22d3ee;
  box-shadow: 0 0 10px rgba(34, 211, 238, 0.7);
  animation: trace-pulse 1s ease-in-out infinite;
}

.trace-running-text {
  font-size: 10px;
  letter-spacing: 1.5px;
  color: #22d3ee;
  font-weight: 700;
}

@keyframes trace-pulse {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1.1); }
}

/* 浅色主题 */
:global(.light-theme) .trace-empty-icon,
:global(.light-theme) .trace-empty-text,
:global(.light-theme) .trace-empty-hint { color: #9c7a4d; }

:global(.light-theme) .trace-row { border-left-color: #d8c3a0; }

:global(.light-theme) .trace-detail {
  background: rgba(107, 68, 35, 0.05);
  border-color: #d8c3a0;
  color: #6b4423;
}
</style>
