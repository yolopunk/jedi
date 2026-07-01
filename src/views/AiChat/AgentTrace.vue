<template>
  <div class="agent-trace">
    <!-- 空状态 -->
    <div v-if="trace.length === 0" class="trace-empty">
      <div class="trace-empty-icon">◇</div>
      <div class="trace-empty-text">NO AGENT ACTIVITY</div>
      <div class="trace-empty-hint">
        启用工具后，Agent 的工具调用与确认过程会在此实时显示
      </div>
    </div>

    <!-- 追踪日志 -->
    <div v-else class="trace-log">
      <template v-for="(event, index) in trace" :key="index">
        <!-- 思考 -->
        <div v-if="event.type === 'thinking'" class="trace-row thinking">
          <div class="trace-line"><span class="trace-marker">»</span><span class="trace-label">THINK</span></div>
          <pre class="trace-detail">{{ truncate(event.text) }}</pre>
        </div>

        <!-- 提示（降级等） -->
        <div v-else-if="event.type === 'notice'" class="trace-row notice">
          <div class="trace-line"><span class="trace-marker">ⓘ</span><span class="trace-label">NOTICE</span></div>
          <pre class="trace-detail">{{ event.text }}</pre>
        </div>

        <!-- 工具调用 -->
        <div v-else-if="event.type === 'tool_call'" class="trace-row tool-call">
          <div class="trace-line">
            <span class="trace-marker">▶</span>
            <span class="trace-label">CALL  {{ event.server ? event.server + '.' : '' }}{{ event.name }}</span>
          </div>
          <pre class="trace-detail">{{ prettyArgs(event.arguments) }}</pre>
        </div>

        <!-- 确认卡片（交互） -->
        <div v-else-if="event.type === 'confirm_request'" class="trace-row confirm" :class="riskClass(event.risk)">
          <div class="confirm-head">
            <span class="risk-badge" :class="riskClass(event.risk)">{{ event.risk.toUpperCase() }}</span>
            <span class="confirm-title">{{ event.server ? event.server + '.' : '' }}{{ event.name }}</span>
          </div>
          <pre v-if="event.diff" class="confirm-diff">{{ event.diff }}</pre>
          <pre v-else class="confirm-diff dim">{{ prettyArgs(event.arguments) }}</pre>

          <div v-if="!decided[event.call_id]" class="confirm-actions">
            <button class="cf-btn approve" @click="approve(event.call_id)">✓ 批准</button>
            <button class="cf-btn reject" @click="reject(event.call_id)">✗ 拒绝</button>
          </div>
          <div v-else class="confirm-status" :class="decided[event.call_id]">
            {{ decided[event.call_id] === 'approved' ? '✓ 已批准' : '✗ 已拒绝' }}
          </div>
        </div>

        <!-- 工具结果 -->
        <div v-else-if="event.type === 'tool_result'" class="trace-row" :class="event.is_error ? 'error' : 'tool-result'">
          <div class="trace-line">
            <span class="trace-marker">{{ event.is_error ? '✗' : '✓' }}</span>
            <span class="trace-label">{{ event.is_error ? 'FAILED' : 'RESULT' }}  {{ event.name }}</span>
            <button
              v-if="event.undo_token && !undone[event.undo_token]"
              class="undo-btn"
              @click="undo(event.undo_token)"
            >↩ 撤销</button>
            <span v-else-if="event.undo_token" class="undo-done">已撤销</span>
          </div>
          <pre class="trace-detail">{{ truncate(event.content) }}</pre>
        </div>

        <!-- 最终回答 / 结束 / 错误 -->
        <div v-else-if="event.type === 'content'" class="trace-row content">
          <div class="trace-line"><span class="trace-marker">★</span><span class="trace-label">ANSWER READY</span></div>
        </div>
        <div v-else-if="event.type === 'done'" class="trace-row done">
          <div class="trace-line"><span class="trace-marker">─</span><span class="trace-label">TURN COMPLETE</span></div>
        </div>
        <div v-else-if="event.type === 'error'" class="trace-row error">
          <div class="trace-line"><span class="trace-marker">✗</span><span class="trace-label">ERROR</span></div>
          <pre class="trace-detail">{{ event.message }}</pre>
        </div>
      </template>

      <!-- 运行中指示 -->
      <div v-if="running" class="trace-running">
        <span class="trace-spinner"></span>
        <span class="trace-running-text">AGENT RUNNING</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import type { AgentEvent, RiskLevel } from '@/api/ai-chat'

defineProps<{
  trace: AgentEvent[]
  running?: boolean
}>()

const store = useAiChatStore()

const decided = reactive<Record<string, 'approved' | 'rejected'>>({})
const undone = reactive<Record<string, boolean>>({})

function truncate(text: string, max = 600): string {
  if (!text) return ''
  return text.length <= max ? text : text.slice(0, max) + `\n… (+${text.length - max} chars)`
}

function prettyArgs(args: unknown): string {
  try {
    return truncate(JSON.stringify(args, null, 2))
  } catch {
    return String(args)
  }
}

function riskClass(risk: RiskLevel): string {
  return `risk-${risk}`
}

async function approve(callId: string) {
  decided[callId] = 'approved'
  try {
    await store.confirmTool(callId, true)
  } catch (e) {
    console.error('confirm approve failed', e)
  }
}

async function reject(callId: string) {
  decided[callId] = 'rejected'
  try {
    await store.confirmTool(callId, false)
  } catch (e) {
    console.error('confirm reject failed', e)
  }
}

async function undo(token: string) {
  undone[token] = true
  try {
    await store.undoTool(token)
  } catch (e) {
    console.error('undo failed', e)
    undone[token] = false
  }
}
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
.trace-empty-icon { font-size: 28px; color: #3f3f46; }
.trace-empty-text { font-size: 10px; letter-spacing: 2px; color: #52525b; font-weight: 700; }
.trace-empty-hint { font-size: 10px; line-height: 1.6; color: #52525b; }

/* 日志 */
.trace-log { display: flex; flex-direction: column; gap: 8px; }

.trace-row { border-left: 2px solid #27272a; padding: 4px 0 4px 10px; }
.trace-line { display: flex; align-items: baseline; gap: 8px; }
.trace-marker { font-size: 11px; width: 12px; flex-shrink: 0; text-align: center; }
.trace-label {
  font-size: 10.5px;
  letter-spacing: 0.5px;
  font-weight: 700;
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
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
.trace-row.thinking .trace-marker, .trace-row.thinking .trace-label { color: #a78bfa; }
.trace-row.notice { border-left-color: #fbbf24; }
.trace-row.notice .trace-marker, .trace-row.notice .trace-label { color: #fbbf24; }
.trace-row.tool-call { border-left-color: #22d3ee; }
.trace-row.tool-call .trace-marker, .trace-row.tool-call .trace-label { color: #22d3ee; }
.trace-row.tool-result { border-left-color: #4ade80; }
.trace-row.tool-result .trace-marker, .trace-row.tool-result .trace-label { color: #4ade80; }
.trace-row.content { border-left-color: #60a5fa; }
.trace-row.content .trace-marker, .trace-row.content .trace-label { color: #60a5fa; }
.trace-row.done { border-left-color: #3f3f46; }
.trace-row.done .trace-marker, .trace-row.done .trace-label { color: #71717a; }
.trace-row.error { border-left-color: #f87171; }
.trace-row.error .trace-marker, .trace-row.error .trace-label { color: #f87171; }

/* 确认卡片 */
.trace-row.confirm {
  border-left-width: 3px;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 0 6px 6px 0;
  padding: 8px 10px;
}
.trace-row.confirm.risk-write { border-left-color: #fbbf24; }
.trace-row.confirm.risk-system { border-left-color: #f87171; }
.trace-row.confirm.risk-read { border-left-color: #4ade80; }

.confirm-head { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; }
.confirm-title { font-size: 11px; font-weight: 700; color: #e4e4e7; }

.risk-badge {
  font-size: 8.5px;
  font-weight: 700;
  letter-spacing: 1px;
  padding: 2px 6px;
  border-radius: 3px;
}
.risk-badge.risk-read { color: #4ade80; background: rgba(74, 222, 128, 0.12); }
.risk-badge.risk-write { color: #fbbf24; background: rgba(251, 191, 36, 0.12); }
.risk-badge.risk-system { color: #f87171; background: rgba(248, 113, 113, 0.14); }

.confirm-diff {
  margin: 0 0 8px;
  padding: 6px 8px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid #27272a;
  border-radius: 4px;
  font-size: 10px;
  line-height: 1.5;
  color: #e4e4e7;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 140px;
  overflow-y: auto;
}
.confirm-diff.dim { color: #a1a1aa; }

.confirm-actions { display: flex; gap: 8px; }
.cf-btn {
  flex: 1;
  padding: 5px 8px;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.5px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}
.cf-btn.approve { color: #4ade80; background: rgba(74, 222, 128, 0.1); border-color: rgba(74, 222, 128, 0.3); }
.cf-btn.approve:hover { background: rgba(74, 222, 128, 0.2); }
.cf-btn.reject { color: #f87171; background: rgba(248, 113, 113, 0.1); border-color: rgba(248, 113, 113, 0.3); }
.cf-btn.reject:hover { background: rgba(248, 113, 113, 0.2); }

.confirm-status { font-size: 10px; font-weight: 700; }
.confirm-status.approved { color: #4ade80; }
.confirm-status.rejected { color: #f87171; }

/* 撤销按钮 */
.undo-btn {
  font-family: inherit;
  font-size: 9px;
  font-weight: 700;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.1);
  border: 1px solid rgba(251, 191, 36, 0.3);
  border-radius: 3px;
  padding: 1px 6px;
  cursor: pointer;
}
.undo-btn:hover { background: rgba(251, 191, 36, 0.2); }
.undo-done { font-size: 9px; color: #71717a; }

/* 运行中 */
.trace-running { display: flex; align-items: center; gap: 8px; padding: 8px 2px; margin-top: 4px; }
.trace-spinner {
  width: 8px; height: 8px; border-radius: 50%;
  background: #22d3ee; box-shadow: 0 0 10px rgba(34, 211, 238, 0.7);
  animation: trace-pulse 1s ease-in-out infinite;
}
.trace-running-text { font-size: 10px; letter-spacing: 1.5px; color: #22d3ee; font-weight: 700; }
@keyframes trace-pulse {
  0%, 100% { opacity: 0.3; transform: scale(0.8); }
  50% { opacity: 1; transform: scale(1.1); }
}

/* 浅色主题 */
:global(.light-theme) .trace-empty-icon,
:global(.light-theme) .trace-empty-text,
:global(.light-theme) .trace-empty-hint { color: #9c7a4d; }
:global(.light-theme) .trace-row { border-left-color: #d8c3a0; }
:global(.light-theme) .trace-detail { background: rgba(107, 68, 35, 0.05); border-color: #d8c3a0; color: #6b4423; }
:global(.light-theme) .confirm-title { color: #3a2a15; }
:global(.light-theme) .confirm-diff { background: rgba(107, 68, 35, 0.08); border-color: #d8c3a0; color: #3a2a15; }
</style>
