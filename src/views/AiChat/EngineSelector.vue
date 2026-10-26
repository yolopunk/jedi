<script setup lang="ts">
// Agent engine selector (docs §3.1): shows every detected "agent@host"
// engine plus the builtin fallback, grouped by detected / not detected.
// Tier-A engines are selectable; Tier-B stay visible with an "adapter
// pending" badge until their driver lands (M2-M4).

import { mdiChevronDown, mdiConsoleLine, mdiRefresh } from '@mdi/js'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { EngineDescriptor } from '@/agent/hub/types'
import { useAgentHubStore } from '@/stores/agentHub'

const { t } = useI18n()
const hub = useAgentHubStore()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)
const triggerRef = ref<HTMLButtonElement | null>(null)
const menuRef = ref<HTMLDivElement | null>(null)

const installedEngines = computed(() => hub.engines)
const notInstalledAgents = computed(() => hub.reports.filter(r => r.installations.length === 0))

const label = computed(() => {
  const e = hub.selectedEngine
  if (e.tier === 'builtin') return t('chatWorkspace.jediBuiltin')
  return e.name
})
const meta = computed(() => {
  const e = hub.selectedEngine
  if (e.tier === 'builtin') return t('chatWorkspace.builtinRuntime')
  return [e.location, e.installation?.version].filter(Boolean).join(' · ')
})

function toggle() {
  open.value = !open.value
}

function pick(engine: EngineDescriptor) {
  hub.selectEngine(engine.id)
  open.value = false
  triggerRef.value?.focus()
}

// 可选项 = .engine-item 按钮且非 disabled（tier-B 与 "未检测到" 占位均被跳过）
function listSelectableItems(): HTMLButtonElement[] {
  return Array.from(
    menuRef.value?.querySelectorAll<HTMLButtonElement>('button.engine-item:not(:disabled)') ?? []
  )
}

function focusItem(items: HTMLButtonElement[], index: number) {
  const item = items[index]
  if (!item) return
  item.focus()
  item.scrollIntoView({ block: 'nearest' })
}

// 菜单打开时聚焦当前选中项；无选中则聚焦第一个可选项
watch(open, async value => {
  if (!value) return
  await nextTick()
  const items = listSelectableItems()
  if (!items.length) return
  const selected = items.findIndex(item => item.classList.contains('selected'))
  focusItem(items, selected >= 0 ? selected : 0)
})

// 焦点在选项间到头即停（与模型下拉一致）；Escape 关闭并把焦点还给触发器
function handleMenuKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.stopPropagation()
    if (open.value) {
      open.value = false
      triggerRef.value?.focus()
    }
    return
  }
  const items = listSelectableItems()
  if (!items.length) return
  const current = items.indexOf(document.activeElement as HTMLButtonElement)
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    focusItem(items, Math.min(current + 1, items.length - 1))
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    focusItem(items, Math.max(current - 1, 0))
  } else if (event.key === 'Home') {
    event.preventDefault()
    focusItem(items, 0)
  } else if (event.key === 'End') {
    event.preventDefault()
    focusItem(items, items.length - 1)
  }
}

function onDocClick(e: MouseEvent) {
  if (rootRef.value && !rootRef.value.contains(e.target as Node)) open.value = false
}

onMounted(() => {
  document.addEventListener('mousedown', onDocClick)
  void hub.loadPersisted()
  if (!hub.detectedAt) void hub.detect()
})
onBeforeUnmount(() => document.removeEventListener('mousedown', onDocClick))
</script>

<template>
  <div ref="rootRef" class="engine-selector" @keydown="handleMenuKeydown">
    <button
      ref="triggerRef"
      class="engine-trigger"
      :aria-label="t('chatWorkspace.changeAgent', { agent: label })"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="engine-icon"><v-icon :icon="mdiConsoleLine" size="19" aria-hidden="true" /></span>
      <span class="engine-trigger-copy">
        <span class="engine-trigger-name">{{ label }}</span>
        <span class="engine-trigger-meta">{{ meta }}</span>
      </span>
      <span class="engine-state" :class="{ scanning: hub.scanning, danger: !!hub.error }" aria-hidden="true"></span>
      <v-icon :icon="mdiChevronDown" size="16" aria-hidden="true" />
    </button>
    <div v-if="open" ref="menuRef" class="engine-menu" role="listbox">
      <div class="engine-menu-header">
        <div>
          <strong>{{ t('chatWorkspace.chooseAgent') }}</strong>
          <span>{{ t('chatWorkspace.chooseAgentHint') }}</span>
        </div>
        <span>{{ String(hub.installedCount).padStart(2, '0') }}</span>
      </div>
      <div class="engine-group">{{ t('chatWorkspace.engineInstalled') }}</div>
      <button
        v-for="engine in installedEngines"
        :key="engine.id"
        class="engine-item"
        :class="{ selected: engine.id === hub.selectedEngineId }"
        :disabled="engine.tier === 'B'"
        role="option"
        :aria-selected="engine.id === hub.selectedEngineId"
        @click="engine.tier !== 'B' && pick(engine)"
      >
        <span class="engine-item-main">
          <span
            class="engine-item-dot"
            :class="{ active: engine.id === hub.selectedEngineId }"
            aria-hidden="true"
          ></span>
          <span class="engine-item-name">
            {{ engine.name }}
            <span v-if="engine.location" class="engine-item-loc">{{ engine.location }}</span>
          </span>
        </span>
        <span class="engine-item-side">
          <span v-if="engine.tier === 'B'" class="engine-badge soon">{{ t('chatWorkspace.engineComingSoon') }}</span>
          <span v-else-if="engine.installation?.version" class="engine-version">{{ engine.installation.version }}</span>
        </span>
      </button>

      <template v-if="notInstalledAgents.length">
        <div class="engine-group">{{ t('chatWorkspace.engineNotDetected') }}</div>
        <div v-for="agent in notInstalledAgents" :key="agent.id" class="engine-item missing">
          <span class="engine-item-name">{{ agent.name }}</span>
        </div>
      </template>

      <div class="engine-footer">
        <span v-if="hub.error" class="engine-error">{{ t('chatWorkspace.engineDetectFailed') }}</span>
        <button class="engine-refresh" :disabled="hub.scanning" @click="hub.detect(true)">
          <v-icon :icon="mdiRefresh" size="14" aria-hidden="true" />
          {{ hub.scanning ? t('chatWorkspace.engineScanning') : t('chatWorkspace.engineRefresh') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.engine-selector {
  position: relative;
  min-width: 0;
  width: min(260px, 36vw);
}

.engine-trigger {
  width: 100%;
  min-height: 48px;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px 6px 7px;
  border: 1px solid rgb(var(--text-rgb) / 0.09);
  border-radius: 12px;
  background: rgb(var(--text-rgb) / 0.025);
  color: var(--text);
  text-align: left;
  cursor: pointer;
  transition: border-color var(--motion-fast), background var(--motion-fast), box-shadow var(--motion-fast);
}

.engine-trigger:hover {
  border-color: rgb(var(--accent-rgb) / 0.32);
  background: rgb(var(--accent-rgb) / 0.055);
}

.engine-trigger:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.engine-icon {
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  display: grid;
  place-items: center;
  border: 1px solid rgb(var(--accent-rgb) / 0.22);
  border-radius: 9px;
  background: linear-gradient(145deg, rgb(var(--accent-rgb) / 0.16), rgb(var(--accent-rgb) / 0.055));
  color: var(--accent);
}

.engine-trigger-copy {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.engine-trigger-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 650;
}

.engine-trigger-meta {
  overflow: hidden;
  color: var(--text-muted);
  font: 9px var(--jedi-font-mono);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.engine-state {
  width: 7px;
  height: 7px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: var(--success);
  box-shadow: 0 0 0 4px rgb(var(--success-rgb) / 0.1);
}

.engine-state.danger {
  background: var(--danger);
  box-shadow: 0 0 0 4px rgb(var(--danger-rgb) / 0.1);
}

.engine-state.scanning {
  background: var(--warning, #fbbf24);
  box-shadow: none;
  animation: engine-pulse 1s ease-in-out infinite;
}

@keyframes engine-pulse {
  50% {
    opacity: 0.35;
  }
}

/* 引擎下拉菜单 - 自带完整浮层样式，不依赖 index.vue 的 scoped CSS */
.engine-menu {
  position: absolute;
  top: calc(100% + 8px);
  left: 0;
  width: 340px;
  max-width: min(88vw, 340px);
  max-height: min(440px, 68vh);
  overflow-y: auto;
  background: var(--surface-raised);
  border: 1px solid var(--border-strong);
  border-radius: 12px;
  padding: 6px;
  box-shadow: var(--shadow-lg);
  z-index: 100;
}

.engine-menu-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin: 2px 2px 6px;
  padding: 12px;
  border-radius: 9px;
  background: rgb(var(--accent-rgb) / 0.07);
}

.engine-menu-header > div {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.engine-menu-header strong { font-size: 12px; }
.engine-menu-header div span { color: var(--text-muted); font-size: 10px; line-height: 1.45; }
.engine-menu-header > span { color: var(--accent); font: 600 11px var(--jedi-font-mono); }

.engine-group {
  padding: 6px 12px 4px;
  font-size: 11px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.55;
}

.engine-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 9px 10px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: inherit;
  font-size: 12.5px;
  cursor: pointer;
  text-align: left;
}

.engine-item:hover:not(:disabled),
.engine-item:focus-visible:not(:disabled) {
  background: rgb(var(--text-rgb) / 0.055);
  outline: none;
}

.engine-item.selected {
  border-color: rgb(var(--accent-rgb) / 0.22);
  background: rgb(var(--accent-rgb) / 0.09);
}

.engine-item:disabled {
  cursor: default;
  opacity: 0.55;
}

.engine-item.missing {
  cursor: default;
  opacity: 0.4;
}

.engine-item-main {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

/* 菜单项状态点：与触发器点同源 —— accent = 选中项，中性 = 可选未选中 */
.engine-item-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--text-subtle);
  flex-shrink: 0;
}

.engine-item-dot.active {
  background: var(--accent);
}

.engine-item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.engine-item-loc {
  opacity: 0.6;
  margin-left: 6px;
  font-size: 11px;
}

.engine-item-side {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.engine-version {
  opacity: 0.5;
  font-size: 10.5px;
  max-width: 90px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.engine-badge.soon {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: rgba(127, 127, 127, 0.2);
  opacity: 0.8;
}

.engine-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 5px;
  padding: 9px 10px 4px;
  border-top: 1px solid rgba(127, 127, 127, 0.18);
}

.engine-error {
  font-size: 11px;
  color: var(--error, #f87171);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.engine-refresh {
  display: flex;
  align-items: center;
  gap: 5px;
  border: none;
  background: transparent;
  color: inherit;
  font-size: 11.5px;
  opacity: 0.75;
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
}

@media (max-width: 720px) {
  .engine-selector { width: min(100%, 320px); }
  .engine-menu { width: min(88vw, 340px); }
}

.engine-refresh:hover:not(:disabled) {
  opacity: 1;
}

.engine-refresh:disabled {
  cursor: default;
}
</style>
