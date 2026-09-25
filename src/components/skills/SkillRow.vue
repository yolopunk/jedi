<template>
  <div class="skill-row" :class="{ open: expanded }">
    <div class="row-main" @click="emit('expand')">
      <span class="row-icon">{{ skill.icon }}</span>
      <div class="row-info">
        <div class="row-name">
          {{ skill.name }}
          <span class="row-risk" :class="skill.risk ?? 'read'">
            {{ $t(`skills.risk.${skill.risk ?? 'read'}`) }}
          </span>
          <span v-if="statBadge" class="row-stat-badge">{{ statBadge }}</span>
        </div>
        <div class="row-desc">{{ skill.description }}</div>
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
      <template v-if="skill.source === 'custom'">
        <button class="row-mini" :aria-label="$t('skills.editSkill')" @click="emit('edit', skill)">✎</button>
        <button
          class="row-mini danger"
          :aria-label="$t('skills.delete')"
          @click="handleDelete"
        >
          {{ armedId === skill.id ? $t('skills.confirmDelete') : $t('skills.delete') }}
        </button>
      </template>
      <div class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.autoCall') }}</span>
        <button
          class="row-toggle"
          :class="{ on: skill.autoCallable }"
          role="switch"
          :aria-checked="skill.autoCallable"
          :aria-label="$t('skills.autoCall')"
          :title="$t('skills.autoCall')"
          @click="skillsStore.toggleAutoCallable(skill.id, !skill.autoCallable)"
        >
          <span class="knob"></span>
        </button>
      </div>
      <div v-if="(skill.risk ?? 'read') !== 'read'" class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.alwaysAllow') }}</span>
        <button
          class="row-toggle warn"
          :class="{ on: skillsStore.isAlwaysAllowed(skill.id) }"
          role="switch"
          :aria-checked="skillsStore.isAlwaysAllowed(skill.id)"
          :aria-label="$t('skills.alwaysAllow')"
          :title="$t('skills.alwaysAllowHint')"
          @click="skillsStore.setAlwaysAllowed(skill.id, !skillsStore.isAlwaysAllowed(skill.id))"
        >
          <span class="knob"></span>
        </button>
      </div>
      <div class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.enabled') }}</span>
        <button
          class="row-toggle"
          :class="{ on: skill.enabled }"
          role="switch"
          :aria-checked="skill.enabled"
          :aria-label="$t('skills.enabled')"
          :title="$t('skills.enabled')"
          @click="skillsStore.toggleSkill(skill.id, !skill.enabled)"
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
import { useTwoStepConfirm } from '@/composables/useTwoStepConfirm'
import type { Skill } from '@/skills/types'
import { useSkillStatsStore } from '@/stores/skillStats'
import { useSkillsStore } from '@/stores/skills'

const props = defineProps<{ skill: Skill; expanded?: boolean }>()
const emit = defineEmits<{
  expand: []
  edit: [skill: Skill]
  remove: [skillId: string]
}>()

const { t } = useI18n()
const skillsStore = useSkillsStore()
const statsStore = useSkillStatsStore()
const { armedId, arm } = useTwoStepConfirm()

const paramEntries = computed(() => Object.entries(props.skill.parameters?.properties ?? {}))

const statBadge = computed<string | null>(() => {
  const s = statsStore.statsFor(props.skill.id)
  if (!s || s.calls === 0) return null
  const rate = Math.round(((s.calls - s.failures) / s.calls) * 100)
  const avg = Math.round(s.totalMs / s.calls)
  return t('skills.stats.badge', { calls: s.calls, rate, avg })
})

const statLines = computed(() => {
  const s = statsStore.statsFor(props.skill.id)
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
  (statsStore.statsFor(props.skill.id)?.recentErrors ?? []).slice().reverse()
)

async function handleDelete(): Promise<void> {
  // 两段式确认：第一次点击进入确认态（3 秒自动复位），再点一次才真正删除
  if (!arm(props.skill.id)) return
  emit('remove', props.skill.id)
}
</script>

<style scoped>
.skill-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgb(var(--text-rgb) / 0.04);
  border: 1px solid var(--border);
}

.skill-row.open {
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

.row-risk {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 700;
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
  opacity: 0.55;
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
  opacity: 0.5;
  font-size: 10px;
}

.row-param-desc {
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-param-empty {
  font-size: 11px;
  opacity: 0.4;
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
  opacity: 0.55;
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
  opacity: 0.45;
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
  opacity: 0.55;
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

.row-mini {
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 11px;
  background: rgb(var(--text-rgb) / 0.05);
  color: var(--text);
}

.row-mini.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}
</style>
