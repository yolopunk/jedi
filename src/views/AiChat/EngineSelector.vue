<script setup lang="ts">
// Agent engine selector (docs §3.1): shows every detected "agent@host"
// engine plus the builtin fallback, grouped by detected / not detected.
// Tier-A engines are selectable; Tier-B stay visible with an "adapter
// pending" badge until their driver lands (M2-M4).

import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { EngineDescriptor } from '@/agent/hub/types'
import { useAgentHubStore } from '@/stores/agentHub'

const { t } = useI18n()
const hub = useAgentHubStore()

const open = ref(false)
const rootRef = ref<HTMLElement | null>(null)

const installedEngines = computed(() => hub.engines)
const notInstalledAgents = computed(() => hub.reports.filter(r => r.installations.length === 0))

const label = computed(() => {
  const e = hub.selectedEngine
  if (e.tier === 'builtin') return t('chatWorkspace.engineBuiltin')
  return e.location ? `${e.name} · ${e.location}` : e.name
})

function toggle() {
  open.value = !open.value
}

function pick(engine: EngineDescriptor) {
  hub.selectEngine(engine.id)
  open.value = false
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
  <div ref="rootRef" class="model-selector engine-selector" @keydown.esc.stop="open = false">
    <button
      class="model-dropdown-btn"
      :title="t('chatWorkspace.engine')"
      :aria-expanded="open"
      aria-haspopup="listbox"
      @click="toggle"
    >
      <span class="engine-dot" :class="{ scanning: hub.scanning }" aria-hidden="true"></span>
      <span class="model-dropdown-name">{{ label }}</span>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
        <polyline points="6 9 12 15 18 9" stroke="currentColor" stroke-width="2" />
      </svg>
    </button>
    <div v-if="open" class="model-dropdown-menu engine-menu" role="listbox">
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
        <span class="engine-item-name">
          {{ engine.name }}
          <span v-if="engine.location" class="engine-item-loc">{{ engine.location }}</span>
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
          {{ hub.scanning ? t('chatWorkspace.engineScanning') : t('chatWorkspace.engineRefresh') }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.engine-selector {
  flex-shrink: 0;
}

.engine-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--success, #34d399);
  flex-shrink: 0;
}

.engine-dot.scanning {
  background: var(--warning, #fbbf24);
  animation: engine-pulse 1s ease-in-out infinite;
}

@keyframes engine-pulse {
  50% {
    opacity: 0.35;
  }
}

.engine-menu {
  min-width: 230px;
  max-height: 340px;
}

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
  padding: 7px 12px;
  border: none;
  background: transparent;
  color: inherit;
  font-size: 12.5px;
  cursor: pointer;
  text-align: left;
}

.engine-item:hover:not(:disabled) {
  background: rgba(127, 127, 127, 0.12);
}

.engine-item.selected {
  background: rgba(127, 127, 127, 0.18);
}

.engine-item:disabled {
  cursor: default;
  opacity: 0.55;
}

.engine-item.missing {
  cursor: default;
  opacity: 0.4;
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
  padding: 6px 12px;
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
  border: none;
  background: transparent;
  color: inherit;
  font-size: 11.5px;
  opacity: 0.75;
  cursor: pointer;
  padding: 2px 4px;
  flex-shrink: 0;
}

.engine-refresh:hover:not(:disabled) {
  opacity: 1;
}

.engine-refresh:disabled {
  cursor: default;
}
</style>
