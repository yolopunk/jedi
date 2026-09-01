<template>
  <transition name="palette-fade">
    <div v-if="modelValue" class="palette-overlay" @click.self="close">
      <div class="palette">
        <div class="palette-input-row">
          <span class="palette-prompt">&gt;</span>
          <input
            ref="inputEl"
            v-model="query"
            class="palette-input"
            placeholder="输入指令，或直接描述你想做什么…"
            @keydown.down.prevent="move(1)"
            @keydown.up.prevent="move(-1)"
            @keydown.enter.prevent="runSelected"
            @keydown.esc.prevent="close"
          />
          <span class="palette-hint">{{ shortcutLabel }}</span>
        </div>

        <div v-if="actions.length" class="palette-list">
          <div
            v-for="(a, i) in actions"
            :key="a.id"
            class="palette-item"
            :class="{ active: i === selected, primary: a.primary }"
            @mouseenter="selected = i"
            @click="run(a)"
          >
            <span class="pi-icon">{{ a.icon }}</span>
            <span class="pi-label">{{ a.label }}</span>
            <span v-if="a.hint" class="pi-hint">{{ a.hint }}</span>
          </div>
        </div>
        <div v-else class="palette-empty">无匹配项</div>

        <div class="palette-footer">
          <span><kbd>↑</kbd><kbd>↓</kbd> 选择</span>
          <span><kbd>Enter</kbd> 执行</span>
          <span><kbd>Esc</kbd> 关闭</span>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAiChatStore } from '@/stores/aiChat'

interface PaletteAction {
  id: string
  label: string
  icon: string
  hint?: string
  keywords?: string
  primary?: boolean
  run: () => void | Promise<void>
}

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'open-settings': []
}>()

const router = useRouter()
const store = useAiChatStore()

const query = ref('')
const selected = ref(0)
const inputEl = ref<HTMLInputElement | null>(null)

const isMac = /Mac|iPhone|iPad/i.test(
  (typeof navigator !== 'undefined' && (navigator.platform || navigator.userAgent)) || ''
)
const shortcutLabel = isMac ? '⌘J' : 'Ctrl+J'

const navActions: PaletteAction[] = [
  { id: 'chat', label: '打开 AI 对话', icon: '◈', keywords: 'chat ai agent duihua 对话 聊天', run: () => router.push('/chat') },
  { id: 'hosts', label: 'Hosts 管理', icon: '⛁', keywords: 'hosts dns domain 域名 主机', run: () => router.push('/hosts') },
  { id: 'wallpapers', label: '知识壁纸', icon: '▣', keywords: 'wallpaper 壁纸 背景', run: () => router.push('/wallpapers') },
  { id: 'podcast', label: '播客', icon: '◉', keywords: 'podcast 播客 小宇宙 rss', run: () => router.push('/podcast') },
  { id: 'settings', label: '设置', icon: '⚙', keywords: 'settings 设置 配置 模型 api key mcp', run: () => emit('open-settings') },
]

const actions = computed<PaletteAction[]>(() => {
  const q = query.value.trim()
  const lower = q.toLowerCase()
  const matched = q
    ? navActions.filter(
        a => a.label.toLowerCase().includes(lower) || (a.keywords || '').toLowerCase().includes(lower)
      )
    : navActions

  if (!q) return matched

  // 有输入时，"问 Agent" 始终作为首选项
  const ask: PaletteAction = {
    id: 'ask-agent',
    label: `问 Agent：${q}`,
    icon: '✦',
    hint: '交给 AI 执行',
    primary: true,
    run: () => askAgent(q),
  }
  return [ask, ...matched]
})

watch(query, () => {
  selected.value = 0
})

watch(
  () => props.modelValue,
  async (open) => {
    if (open) {
      query.value = ''
      selected.value = 0
      await nextTick()
      inputEl.value?.focus()
    }
  }
)

function close() {
  emit('update:modelValue', false)
}

function move(delta: number) {
  const n = actions.value.length
  if (!n) return
  selected.value = (selected.value + delta + n) % n
}

async function run(a: PaletteAction) {
  close()
  try {
    await a.run()
  } catch (e) {
    console.error('Command palette action failed:', e)
  }
}

function runSelected() {
  const a = actions.value[selected.value]
  if (a) run(a)
}

/** 从任意页面唤起 Agent：跳到对话页并发送，复用流式/确认/Trace 全链路 */
async function askAgent(q: string) {
  try {
    await router.push('/chat')
    await store.sendMessage(q)
  } catch (e) {
    console.error('Failed to ask agent:', e)
  }
}

function onKeydown(e: KeyboardEvent) {
  const mod = isMac ? e.metaKey : e.ctrlKey
  if (mod && e.key.toLowerCase() === 'j') {
    e.preventDefault()
    emit('update:modelValue', !props.modelValue)
  }
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onUnmounted(() => document.removeEventListener('keydown', onKeydown))
</script>

<style scoped>
.palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 4000;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 14vh;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(4px);
}

.palette {
  width: min(680px, 92vw);
  background: rgba(18, 18, 22, 0.97);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 8px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.55);
  overflow: hidden;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.palette-input-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.12);
}

.palette-prompt {
  color: #00ff88;
  font-weight: 700;
  font-size: 14px;
}

.palette-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #e4e4e7;
  font-family: inherit;
  font-size: 14px;
}

.palette-input::placeholder {
  color: #52525b;
}

.palette-hint {
  font-size: 10px;
  color: #52525b;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 3px;
  padding: 2px 6px;
}

.palette-list {
  max-height: 46vh;
  overflow-y: auto;
  padding: 6px;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 10px;
  border-radius: 5px;
  cursor: pointer;
  transition: background 0.12s ease;
}

.palette-item.active {
  background: rgba(0, 255, 255, 0.1);
}

.palette-item.primary .pi-label {
  color: #00ffff;
}

.pi-icon {
  width: 18px;
  text-align: center;
  color: #22d3ee;
  font-size: 13px;
}

.pi-label {
  flex: 1;
  font-size: 12.5px;
  color: #e4e4e7;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pi-hint {
  font-size: 10px;
  color: #71717a;
}

.palette-empty {
  padding: 20px;
  text-align: center;
  font-size: 11px;
  color: #52525b;
}

.palette-footer {
  display: flex;
  gap: 16px;
  padding: 8px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 10px;
  color: #52525b;
}

.palette-footer kbd {
  display: inline-block;
  margin-right: 3px;
  padding: 1px 4px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 3px;
  font-family: inherit;
}

.palette-fade-enter-active,
.palette-fade-leave-active {
  transition: opacity 0.14s ease;
}
.palette-fade-enter-from,
.palette-fade-leave-to {
  opacity: 0;
}

/* 浅色主题 */
:global(.light-theme) .palette {
  background: rgba(252, 247, 238, 0.98);
  border-color: #d8c3a0;
}
:global(.light-theme) .palette-input {
  color: #3a2a15;
}
:global(.light-theme) .pi-label {
  color: #3a2a15;
}
:global(.light-theme) .palette-item.active {
  background: rgba(107, 68, 35, 0.08);
}
</style>
