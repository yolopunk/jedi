<template>
  <div class="thread-wrapper">
    <div
      ref="scrollElement"
      class="message-scroll"
      :aria-label="t('agents.thread.label')"
      role="log"
      aria-live="polite"
      aria-relevant="additions text"
      tabindex="0"
      data-testid="agents-message-list"
      @scroll="onScroll"
    >
      <div v-if="!messages.length" class="thread-empty" data-testid="agents-thread-empty">
        <div class="thread-empty-icon" aria-hidden="true"><v-icon :icon="mdiForumOutline" size="28" /></div>
        <h3>{{ t('agents.thread.emptyTitle') }}</h3>
        <p>{{ t('agents.thread.emptyHint') }}</p>
      </div>
      <ol v-else class="message-list">
        <li v-for="row in messages" :key="row.message.id" class="message-row" :class="{ 'from-user': row.message.role === 'user' }">
          <article :data-message-id="row.message.id" :data-message-seq="row.message.seq" data-testid="agents-message">
            <div class="message-avatar" aria-hidden="true">
              <v-icon :icon="row.message.role === 'user' ? mdiAccountOutline : mdiRobotOutline" size="19" />
            </div>
            <div class="message-main">
              <div class="message-meta">
                <strong>{{ row.message.role === 'user' ? t('agents.thread.you') : row.message.authorName }}</strong>
                <span v-if="row.member?.role" class="message-role">{{ row.member.role }}</span>
                <time v-if="row.timestamp" :datetime="row.message.createdAt" :title="row.timestamp.full">{{ row.timestamp.short }}</time>
              </div>
              <p v-if="row.message.role === 'user'" class="message-recipients">{{ recipientLabel(row.message) }}</p>
              <div v-if="row.message.content" class="agents-markdown" v-html="row.html"></div>
              <p v-else-if="row.active" class="message-pending">{{ t('agents.thread.pending') }}</p>
              <div v-if="row.run" class="message-run-meta">
                <span class="agents-badge" :class="runStatusClass(row.run)" data-testid="agents-message-run-status">
                  <v-progress-circular v-if="row.active && row.run.status !== 'awaiting_approval'" indeterminate size="11" width="1.5" aria-hidden="true" />
                  {{ t(`agents.status.${row.run.status}`) }}
                </span>
                <span v-if="row.run.model" class="message-model">{{ row.run.model }}</span>
              </div>
              <p v-if="row.run?.error" class="message-error" data-testid="agents-message-error">{{ row.run.error }}</p>
              <details v-if="row.run?.tools.length" class="message-tools" data-testid="agents-message-tools">
                <summary>{{ t('agents.thread.toolActivity', { count: row.run.tools.length }) }}</summary>
                <ul>
                  <li v-for="tool in row.run.tools" :key="tool.id">
                    <div><strong>{{ tool.name }}</strong><span>{{ t(`agents.toolStatus.${tool.status}`) }}</span></div>
                    <pre v-if="tool.detail">{{ tool.detail }}</pre>
                  </li>
                </ul>
              </details>
              <div v-if="row.message.content || row.retryable" class="message-actions">
                <button
                  v-if="row.message.content"
                  type="button"
                  class="agents-button quiet"
                  :disabled="copyingId === row.message.id"
                  data-testid="agents-message-copy"
                  @click="copyMessage(row.message)"
                >
                  <v-icon :icon="copiedId === row.message.id ? mdiCheck : mdiContentCopy" size="14" aria-hidden="true" />
                  {{ t(copiedId === row.message.id ? 'agents.thread.copied' : 'agents.thread.copy') }}
                </button>
                <button
                  v-if="row.retryable && row.run"
                  type="button"
                  class="agents-button quiet"
                  :disabled="!row.canRetry || retryingRunIds.length > 0 || saving || scanning"
                  data-testid="agents-message-retry"
                  @click="emit('retryRun', group.id, row.run.id)"
                >
                  <v-icon :icon="mdiRefresh" size="15" aria-hidden="true" />
                  {{ t('agents.thread.retry') }}
                </button>
              </div>
              <p v-if="copyError?.id === row.message.id" class="message-error" role="alert">{{ copyError.text }}</p>
            </div>
          </article>
        </li>
      </ol>
    </div>
    <button
      v-if="!nearBottom && messages.length"
      type="button"
      class="agents-button latest-button"
      data-testid="agents-jump-latest"
      @click="scrollToLatest"
    >
      <v-icon :icon="mdiArrowDown" size="15" aria-hidden="true" />
      {{ t('agents.thread.latest') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import {
  mdiAccountOutline,
  mdiArrowDown,
  mdiCheck,
  mdiContentCopy,
  mdiForumOutline,
  mdiRefresh,
  mdiRobotOutline,
} from '@mdi/js'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AgentEngineOption, AgentGroup, AgentGroupMessage } from '@/types/agents'
import { renderSafe, sharedMd } from '@/utils/markdown'
import { engineSupportsMode, isActiveRun, runStatusClass } from './agent-ui'

const props = defineProps<{
  group: AgentGroup
  engines: AgentEngineOption[]
  saving: boolean
  scanning: boolean
  retryingRunIds: string[]
}>()
const emit = defineEmits<{ retryRun: [groupId: string, runId: string] }>()
const { t, locale } = useI18n()
const scrollElement = ref<HTMLElement | null>(null)
const nearBottom = ref(true)
const copiedId = ref<string | null>(null)
const copyingId = ref<string | null>(null)
const copyError = ref<{ id: string; text: string } | null>(null)
let copyTimer: ReturnType<typeof setTimeout> | undefined
const renderCache = new Map<string, { content: string; html: string }>()

function formatTimestamp(value: string): { short: string; full: string } | null {
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return null
  return {
    short: date.toLocaleTimeString(locale.value, { hour: '2-digit', minute: '2-digit' }),
    full: date.toLocaleString(locale.value),
  }
}

const messages = computed(() => {
  const runs = new Map(props.group.runs.map(run => [run.id, run]))
  const members = new Map(props.group.members.map(member => [member.id, member]))
  const engines = new Map(props.engines.map(engine => [engine.id, engine]))
  const busyMembers = new Set(props.group.runs.filter(isActiveRun).map(run => run.memberId))
  return [...props.group.messages]
    .sort((a, b) => a.seq - b.seq)
    .map(message => {
      let cached = renderCache.get(message.id)
      if (!cached || cached.content !== message.content) {
        cached = { content: message.content, html: renderSafe(sharedMd, message.content) }
        renderCache.set(message.id, cached)
      }
      const run = message.runId ? runs.get(message.runId) : undefined
      const member = message.authorId ? members.get(message.authorId) : undefined
      const retryable =
        !!run &&
        ['failed', 'stopped', 'interrupted'].includes(run.status) &&
        props.group.messages.some(item => item.id === run.triggerMessageId && item.role === 'user')
      return {
        retryable,
        canRetry:
          retryable &&
          !!member?.enabled &&
          !busyMembers.has(member.id) &&
          engineSupportsMode(engines.get(member.engineId), props.group.executionMode),
        message,
        run,
        active: isActiveRun(run),
        member: message.authorId ? members.get(message.authorId) : undefined,
        html: cached.html,
        timestamp: formatTimestamp(message.createdAt),
      }
    })
})

async function copyMessage(message: AgentGroupMessage): Promise<void> {
  copyingId.value = message.id
  copyError.value = null
  try {
    await navigator.clipboard.writeText(message.content)
    copiedId.value = message.id
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => {
      copiedId.value = null
    }, 2000)
  } catch {
    copyError.value = { id: message.id, text: t('agents.thread.copyFailed') }
  } finally {
    copyingId.value = null
  }
}

function recipientLabel(message: AgentGroupMessage): string {
  if (!message.recipients.length) return t('agents.thread.allRecipients')
  const names = message.recipients.map(
    id => props.group.members.find(member => member.id === id)?.name || id
  )
  return t('agents.thread.recipients', {
    names: names.join(locale.value.startsWith('zh') ? '、' : ', '),
  })
}

function onScroll(): void {
  const element = scrollElement.value
  if (!element) return
  nearBottom.value = element.scrollHeight - element.scrollTop - element.clientHeight < 90
}

function scrollToLatest(): void {
  const element = scrollElement.value
  if (!element) return
  element.scrollTop = element.scrollHeight
  nearBottom.value = true
}

watch(
  () => props.group.id,
  () => {
    renderCache.clear()
    nearBottom.value = true
    void nextTick(scrollToLatest)
  }
)
watch(messages, () => {
  if (nearBottom.value) void nextTick(scrollToLatest)
})
onMounted(() => {
  void nextTick(scrollToLatest)
})
onUnmounted(() => {
  if (copyTimer) clearTimeout(copyTimer)
})
</script>

<style scoped>
.thread-wrapper { position: relative; display: flex; flex: 1; min-height: 0; min-width: 0; }
.message-scroll { flex: 1; min-height: 0; min-width: 0; overflow-y: auto; overflow-x: hidden; padding: 25px 26px 34px; scroll-padding: 20px; }
.message-scroll:focus-visible { outline-offset: -3px; }
.message-list { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: 26px; }
.message-row article { display: flex; align-items: flex-start; gap: 11px; }
.message-avatar { flex-shrink: 0; display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; border: 1px solid var(--border); border-radius: 9px; background: var(--bg-elevated); color: var(--text-muted); }
.from-user .message-avatar { border-color: rgb(var(--accent-rgb) / 0.25); background: rgb(var(--accent-rgb) / 0.08); color: var(--accent); }
.message-main { min-width: 0; flex: 1; }
.message-meta { display: flex; flex-wrap: wrap; align-items: baseline; gap: 4px 9px; min-height: 24px; }
.message-meta strong { font-weight: 650; font-size: 12px; overflow-wrap: anywhere; color: var(--text); }
.message-role { font-size: 11px; color: var(--text-muted); overflow-wrap: anywhere; }
.message-meta time { margin-left: auto; font-size: 10px; color: var(--text-subtle); white-space: nowrap; }
.message-recipients { font-size: 10px; color: var(--text-muted); margin: 0 0 8px; overflow-wrap: anywhere; }
.from-user .agents-markdown { padding: 11px 13px; background: rgb(var(--accent-rgb) / 0.06); border: 1px solid rgb(var(--accent-rgb) / 0.13); border-radius: 4px 12px 12px; }
.message-run-meta { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; margin-top: 10px; }
.message-model { font-size: 10px; color: var(--text-muted); overflow-wrap: anywhere; }
.message-actions { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 8px; }
.message-actions .agents-button { min-height: 30px; padding: 4px 7px; font-size: 11px; }
.message-tools { margin-top: 10px; color: var(--text-muted); font-size: 11px; }
.message-tools summary { cursor: pointer; }
.message-tools ul { list-style: none; margin: 8px 0 0; padding: 0; }
.message-tools li { padding: 8px 0; border-top: 1px solid var(--border); }
.message-tools li > div { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 6px; }
.message-tools strong { color: var(--text); font-weight: 550; }
.message-tools pre { margin: 6px 0 0; white-space: pre-wrap; overflow-wrap: anywhere; font: inherit; max-height: 160px; overflow-y: auto; }
.message-pending { margin: 4px 0; color: var(--text-muted); font-size: 12px; }
.message-error { padding: 9px 11px; margin: 9px 0 0; border-left: 2px solid var(--danger); background: rgb(var(--danger-rgb) / 0.05); color: var(--danger); font-size: 12px; line-height: 1.7; white-space: pre-wrap; overflow-wrap: anywhere; }
.thread-empty { min-height: 100%; max-width: 430px; margin: auto; display: flex; flex-direction: column; align-items: flex-start; justify-content: center; padding: 24px 0; }
.thread-empty-icon { display: flex; align-items: center; justify-content: center; width: 50px; height: 50px; color: var(--accent); background: rgb(var(--accent-rgb) / 0.07); border: 1px solid rgb(var(--accent-rgb) / 0.16); border-radius: 12px; margin-bottom: 17px; }
.thread-empty h3 { margin: 0; font-weight: 600; font-size: 18px; line-height: 1.5; }
.thread-empty p { margin: 9px 0 0; color: var(--text-muted); font-size: 13px; line-height: 1.8; }
.latest-button { position: absolute; bottom: 12px; left: 50%; transform: translateX(-50%); box-shadow: var(--shadow-md); max-width: calc(100% - 24px); border-radius: 18px; white-space: nowrap; }
.agents-markdown { color: var(--text); font-size: 13px; line-height: 1.8; overflow-wrap: anywhere; min-width: 0; }
.agents-markdown :deep(> :first-child) { margin-top: 0; }
.agents-markdown :deep(> :last-child) { margin-bottom: 0; }
.agents-markdown :deep(p) { margin: 0 0 11px; }
.agents-markdown :deep(h1), .agents-markdown :deep(h2), .agents-markdown :deep(h3), .agents-markdown :deep(h4) { line-height: 1.45; margin: 20px 0 9px; color: var(--text); font-weight: 650; }
.agents-markdown :deep(h1) { font-size: 20px; }
.agents-markdown :deep(h2) { font-size: 18px; }
.agents-markdown :deep(h3) { font-size: 16px; }
.agents-markdown :deep(h4) { font-size: 14px; }
.agents-markdown :deep(ul), .agents-markdown :deep(ol) { padding-left: 21px; margin: 8px 0 13px; }
.agents-markdown :deep(li + li) { margin-top: 4px; }
.agents-markdown :deep(a) { color: var(--accent); text-decoration: underline; text-underline-offset: 3px; }
.agents-markdown :deep(code) { font-family: var(--jedi-font-mono); font-size: .9em; padding: 2px 4px; background: var(--bg-elevated-2); border-radius: 4px; }
.agents-markdown :deep(pre) { margin: 12px 0; padding: 13px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg-elevated); color: var(--text); max-width: 100%; overflow-x: auto; white-space: pre; line-height: 1.65; }
.agents-markdown :deep(pre code) { background: transparent; padding: 0; border-radius: 0; font-size: 12px; }
.agents-markdown :deep(blockquote) { margin: 12px 0; padding: 3px 0 3px 13px; color: var(--text-muted); border-left: 3px solid var(--border-strong); }
.agents-markdown :deep(blockquote p) { margin-bottom: 0; }
.agents-markdown :deep(table) { display: block; width: max-content; max-width: 100%; overflow-x: auto; border-collapse: collapse; margin: 12px 0; font-size: 12px; }
.agents-markdown :deep(th), .agents-markdown :deep(td) { padding: 7px 10px; border: 1px solid var(--border); text-align: left; }
.agents-markdown :deep(th) { font-weight: 600; background: var(--bg-elevated); }
.agents-markdown :deep(img) { max-width: 100%; height: auto; border-radius: 6px; }
.agents-markdown :deep(hr) { border: 0; border-top: 1px solid var(--border); margin: 18px 0; }
.agents-markdown :deep(.hljs-comment), .agents-markdown :deep(.hljs-quote) { color: var(--text-subtle); }
.agents-markdown :deep(.hljs-keyword), .agents-markdown :deep(.hljs-selector-tag), .agents-markdown :deep(.hljs-literal) { color: var(--accent-2); }
.agents-markdown :deep(.hljs-string), .agents-markdown :deep(.hljs-addition) { color: var(--success); }
.agents-markdown :deep(.hljs-title), .agents-markdown :deep(.hljs-built_in), .agents-markdown :deep(.hljs-attr) { color: var(--accent); }
.agents-markdown :deep(.hljs-number), .agents-markdown :deep(.hljs-symbol) { color: var(--warning); }
.agents-markdown :deep(.hljs-deletion) { color: var(--danger); }
@media (max-width: 700px) {
  .message-scroll { padding: 20px 16px 32px; }
  .message-avatar { width: 28px; height: 28px; border-radius: 7px; }
  .message-row article { gap: 8px; }
  .message-list { gap: 22px; }
}
</style>
