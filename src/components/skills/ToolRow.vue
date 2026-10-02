<template>
  <div class="tool-row" :class="{ open: expanded }">
    <div
      class="row-main"
      role="button"
      tabindex="0"
      :aria-expanded="expanded"
      @click="emit('expand')"
      @keydown.enter.prevent="emit('expand')"
      @keydown.space.prevent="emit('expand')"
    >
      <span class="row-icon">{{ tool.icon }}</span>
      <div class="row-info">
        <div class="row-name">
          <span class="row-tool-name">{{ tool.name }}</span>
          <span class="row-source" v-if="tool.source === 'mcp'">MCP</span>
          <span class="row-risk" :class="tool.risk ?? 'read'">
            {{ $t(`skills.risk.${tool.risk ?? 'read'}`) }}
          </span>
          <span v-if="statBadge" class="row-stat-badge">{{ statBadge }}</span>
        </div>
        <div class="row-desc">{{ tool.description }}</div>
        <div v-if="expanded" class="row-params">
          <div v-if="paramEntries.length === 0" class="row-param-empty">
            {{ $t('skills.noParams') }}
          </div>
          <div v-for="[name, schema] in paramEntries" :key="name" class="row-param">
            <code>{{ name }}</code>
            <span class="row-param-type">{{ schema.type }}</span>
            <span class="row-param-desc">{{ schema.description }}{{ schema.required ? ' *' : '' }}</span>
          </div>
        </div>
        <div v-if="expanded && statLines.length" class="row-stats-detail">
          <div class="row-stats-line">
            <span v-for="line in statLines" :key="line.label" class="row-stat-item">
              <b>{{ line.label }}</b> {{ line.value }}
            </span>
          </div>
          <div v-for="(err, i) in statErrors" :key="i" class="row-stat-err">
            <span class="row-err-ts">{{ new Date(err.ts).toLocaleTimeString() }}</span>
            <span class="row-err-msg">{{ err.msg }}</span>
          </div>
        </div>
      </div>
    </div>
    <div class="row-actions" @click.stop>
      <div v-if="(tool.risk ?? 'read') !== 'read'" class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.alwaysAllow') }}</span>
        <button
          class="row-toggle warn"
          :class="{ on: toolsStore.isAlwaysAllowed(tool.id) }"
          role="switch"
          :aria-checked="toolsStore.isAlwaysAllowed(tool.id)"
          :aria-label="$t('skills.alwaysAllow')"
          :title="$t('skills.alwaysAllowHint')"
          @click="toolsStore.setAlwaysAllowed(tool.id, !toolsStore.isAlwaysAllowed(tool.id))"
        >
          <span class="knob"></span>
        </button>
      </div>
      <div class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.enabled') }}</span>
        <button
          class="row-toggle"
          :class="{ on: toolsStore.isToolEnabled(tool.id) }"
          role="switch"
          :aria-checked="toolsStore.isToolEnabled(tool.id)"
          :aria-label="$t('skills.enabled')"
          :title="$t('skills.enabled')"
          @click="toolsStore.toggleTool(tool.id, !toolsStore.isToolEnabled(tool.id))"
        >
          <span class="knob"></span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Tool } from '@/agent/tools/types'
import { useToolStatsStore } from '@/stores/toolStats'
import { useToolsStore } from '@/stores/tools'

const props = defineProps<{ tool: Tool; expanded?: boolean }>()
const emit = defineEmits<{
  expand: []
}>()

const { t } = useI18n()
const toolsStore = useToolsStore()
const statsStore = useToolStatsStore()

const paramEntries = computed(() => Object.entries(props.tool.parameters?.properties ?? {}))

const statBadge = computed<string | null>(() => {
  const s = statsStore.statsFor(props.tool.id)
  if (!s || s.calls === 0) return null
  const rate = Math.round(((s.calls - s.failures) / s.calls) * 100)
  const avg = Math.round(s.totalMs / s.calls)
  return t('skills.stats.badge', { calls: s.calls, rate, avg })
})

const statLines = computed(() => {
  const s = statsStore.statsFor(props.tool.id)
  if (!s) return []
  const avg = s.calls ? Math.round(s.totalMs / s.calls) : 0
  return [
    { label: t('skills.stats.calls'), value: String(s.calls) },
    { label: t('skills.stats.failures'), value: String(s.failures) },
    { label: t('skills.stats.avgMs'), value: `${avg}ms` },
    {
      label: t('skills.stats.lastUsed'),
      value: s.lastUsedAt ? new Date(s.lastUsedAt).toLocaleString() : '—',
    },
  ]
})

const statErrors = computed(() =>
  (statsStore.statsFor(props.tool.id)?.recentErrors ?? []).slice().reverse()
)
</script>

<style scoped>
.tool-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgb(var(--text-rgb) / 0.04);
  border: 1px solid var(--border);
  transition: background 0.15s ease, border-color 0.15s ease;
}

.tool-row:hover {
  background: rgb(var(--text-rgb) / 0.07);
  border-color: var(--border-strong);
}

.row-main:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 6px;
}

.tool-row.open {
  align-items: flex-start;
}

.row-main {
  min-width: 0;
  flex: 1;
  display: flex;
  gap: 10px;
  cursor: pointer;
}

.row-icon {
  font-size: 16px;
  line-height: 1.3;
}

.row-info {
  min-width: 0;
  flex: 1;
}

.row-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
}

.row-tool-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--jedi-font-mono);
}

.row-source {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 700;
  flex-shrink: 0;
  background: rgb(var(--accent-rgb) / 0.14);
  color: var(--accent);
}

.row-risk {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 700;
  flex-shrink: 0;
}

.row-risk.read {
  background: rgb(var(--success-rgb) / 0.16);
  color: var(--success);
}

.row-risk.write {
  background: rgb(var(--warning-rgb) / 0.16);
  color: var(--warning);
}

.row-risk.system {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.row-stat-badge {
  font-size: 10.5px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgb(var(--accent-rgb) / 0.12);
  color: var(--accent);
  font-family: var(--jedi-font-mono);
  font-weight: 600;
}

.row-desc {
  margin-top: 2px;
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.row-params {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.row-param {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 11px;
}

.row-param code {
  font-family: var(--jedi-font-mono);
  color: var(--accent);
}

.row-param-type {
  color: var(--text-muted);
  font-size: 10px;
}

.row-param-desc {
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-param-empty {
  font-size: 11px;
  color: var(--text-muted);
}

.row-stats-detail {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.row-stats-line {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  font-size: 11px;
}

.row-stat-item b {
  color: var(--text-muted);
  font-weight: 600;
  margin-right: 4px;
}

.row-stat-err {
  display: flex;
  gap: 8px;
  font-size: 11px;
  align-items: baseline;
}

.row-err-ts {
  color: var(--text-muted);
  font-size: 10px;
  flex-shrink: 0;
}

.row-err-msg {
  color: var(--danger);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  flex-shrink: 0;
}

.row-toggle-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.row-toggle-label {
  font-size: 10px;
  color: var(--text-muted);
}

.row-toggle {
  width: 30px;
  height: 17px;
  border-radius: 9px;
  background: rgb(var(--text-rgb) / 0.12);
  border: none;
  padding: 0;
  position: relative;
  cursor: pointer;
  transition: background 0.15s ease;
}

.row-toggle .knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: var(--text-muted);
  transition: all 0.15s ease;
}

.row-toggle.on {
  background: rgb(var(--success-rgb) / 0.35);
}

.row-toggle.on .knob {
  left: 15px;
  background: var(--success);
}

.row-toggle.warn.on {
  background: rgb(var(--warning-rgb) / 0.35);
}

.row-toggle.warn.on .knob {
  background: var(--warning);
}
</style>
