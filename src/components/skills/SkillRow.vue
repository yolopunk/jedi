<template>
  <div class="skill-row" :class="{ open: expanded }">
    <div
      class="row-main"
      role="button"
      tabindex="0"
      :aria-expanded="expanded"
      @click="emit('expand')"
      @keydown.enter.prevent="emit('expand')"
      @keydown.space.prevent="emit('expand')"
    >
      <span class="row-icon">{{ skill.icon }}</span>
      <div class="row-info">
        <div class="row-name">
          <span class="row-slash">/{{ skill.name }}</span>
          <span class="row-source" :class="skill.source">
            {{ skill.source === 'builtin' ? $t('skills.sourceBuiltin') : $t('skills.sourceUser') }}
          </span>
        </div>
        <div class="row-desc">{{ skill.description }}</div>
        <div v-if="expanded" class="row-detail">
          <div v-if="loadingBody" class="row-body-loading">{{ $t('skills.loadingBody') }}</div>
          <template v-else-if="detail">
            <pre class="row-body">{{ detail.body }}</pre>
            <div v-if="detail.files.length" class="row-files">
              <span class="row-files-label">{{ $t('skills.bundledFiles') }}</span>
              <code v-for="f in detail.files" :key="f" class="row-file">{{ f }}</code>
            </div>
          </template>
        </div>
      </div>
    </div>
    <div class="row-actions" @click.stop>
      <template v-if="skill.source === 'user'">
        <button class="row-mini" :aria-label="$t('skills.editSkill')" @click="emit('edit', skill)">✎</button>
        <button
          class="row-mini danger"
          :aria-label="$t('skills.delete')"
          @click="handleDelete"
        >
          {{ armedId === skill.name ? $t('skills.confirmDelete') : $t('skills.delete') }}
        </button>
      </template>
      <div class="row-toggle-row">
        <span class="row-toggle-label">{{ $t('skills.enabled') }}</span>
        <button
          class="row-toggle"
          :class="{ on: skillsStore.isSkillEnabled(skill.name) }"
          role="switch"
          :aria-checked="skillsStore.isSkillEnabled(skill.name)"
          :aria-label="$t('skills.enabled')"
          :title="$t('skills.enabled')"
          @click="skillsStore.toggleSkill(skill.name, !skillsStore.isSkillEnabled(skill.name))"
        >
          <span class="knob"></span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTwoStepConfirm } from '@/composables/useTwoStepConfirm'
import { loadSkill } from '@/skills/loader'
import type { SkillManifest } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

const props = defineProps<{ skill: SkillManifest; expanded?: boolean }>()
const emit = defineEmits<{
  expand: []
  edit: [skill: SkillManifest]
  remove: [name: string]
}>()

const { t } = useI18n()
const skillsStore = useSkillsStore()
const { armedId, arm } = useTwoStepConfirm()

const loadingBody = ref(false)
const detail = ref<{ body: string; files: string[] } | null>(null)

// 展开时按需加载正文（渐进披露的第三层只在需要时读取）
watch(
  () => props.expanded,
  async open => {
    if (!open || detail.value || loadingBody.value) return
    loadingBody.value = true
    try {
      detail.value = await loadSkill(props.skill.name)
    } catch (e) {
      console.error(`Failed to load skill body ${props.skill.name}:`, e)
      detail.value = { body: t('skills.loadBodyFailed'), files: [] }
    } finally {
      loadingBody.value = false
    }
  },
  { immediate: true }
)

const canDelete = computed(() => props.skill.source === 'user')

async function handleDelete(): Promise<void> {
  if (!canDelete.value) return
  // 两段式确认：第一次点击进入确认态（3 秒自动复位），再点一次才真正删除
  if (!arm(props.skill.name)) return
  emit('remove', props.skill.name)
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
  transition: background 0.15s ease, border-color 0.15s ease;
}

.skill-row:hover {
  background: rgb(var(--text-rgb) / 0.07);
  border-color: var(--border-strong);
}

.row-main:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: 6px;
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

.row-slash {
  font-family: var(--jedi-font-mono);
  color: var(--accent);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.row-source {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 700;
  flex-shrink: 0;
}

.row-source.builtin {
  background: rgb(var(--text-rgb) / 0.1);
  color: var(--text-muted);
}

.row-source.user {
  background: rgb(var(--accent-rgb) / 0.14);
  color: var(--accent);
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

.row-detail {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.row-body-loading {
  font-size: 11px;
  color: var(--text-muted);
}

.row-body {
  margin: 0;
  max-height: 220px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgb(var(--ink-rgb) / 0.12);
  border: 1px solid var(--border);
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.row-files {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 6px;
  font-size: 11px;
}

.row-files-label {
  color: var(--text-muted);
}

.row-file {
  font-family: var(--jedi-font-mono);
  font-size: 10.5px;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgb(var(--text-rgb) / 0.08);
  color: var(--text-muted);
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
