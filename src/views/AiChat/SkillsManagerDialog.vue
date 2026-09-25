<template>
  <Transition name="skl-fade">
    <div v-if="modelValue" class="skl-overlay" @click.self="close">
      <div class="skl-dialog">
        <div class="skl-head">
          <h3>{{ $t('skills.title') }}</h3>
          <div class="skl-head-right">
            <span class="skl-count">{{ $t('skills.enabledCount', { count: skillsStore.enabledCount }) }}</span>
            <button class="skl-close" @click="close">✕</button>
          </div>
        </div>

        <p class="skl-hint">{{ $t('skills.hint') }}</p>

        <div v-if="skillsStore.error" class="skl-error">{{ skillsStore.error }}</div>

        <!-- 自定义技能编辑器（新建/编辑） -->
        <div v-if="editorOpen" class="skl-editor">
          <div class="skl-add-title">{{ editingId ? $t('skills.editSkill') : $t('skills.newSkill') }}</div>
          <div class="skl-form-row">
            <input v-model="form.name" class="skl-input grow" :placeholder="$t('skills.fieldName')" />
            <input v-model="form.icon" class="skl-input icon" :placeholder="$t('skills.fieldIcon')" maxlength="4" />
          </div>
          <input v-if="!editingId" v-model="form.id" class="skl-input" :placeholder="$t('skills.fieldId')" />
          <input v-model="form.description" class="skl-input" :placeholder="$t('skills.fieldDesc')" />
          <div class="skl-risk-picker">
            <button
              v-for="r in RISKS"
              :key="r"
              class="skl-tab"
              :class="{ on: form.risk === r }"
              @click="form.risk = r"
            >
              {{ $t(`skills.risk.${r}`) }}
            </button>
          </div>
          <textarea
            v-model="form.prompt"
            class="skl-input skl-textarea"
            :placeholder="$t('skills.fieldPrompt')"
            rows="6"
          ></textarea>
          <div v-if="formError" class="skl-error">{{ formError }}</div>
          <div class="skl-form-actions">
            <button class="skl-btn" @click="closeEditor">{{ $t('skills.cancel') }}</button>
            <button class="skl-btn primary" :disabled="!canSave || saving" @click="handleSave">
              {{ saving ? '…' : $t('skills.save') }}
            </button>
          </div>
        </div>

        <div class="skl-list">
          <div v-for="group in groups" :key="group.key" class="skl-group">
            <div class="skl-group-head">
              <span class="skl-group-label">{{ group.label }}</span>
              <span class="skl-group-count">{{ group.skills.length }}</span>
              <button
                v-if="group.key === 'custom' && !editorOpen"
                class="skl-btn primary slim"
                @click="startCreate"
              >
                {{ $t('skills.newSkill') }}
              </button>
            </div>
            <div v-if="group.skills.length === 0" class="skl-empty">
              {{ group.key === 'custom' ? $t('skills.emptyCustom') : $t('skills.emptyMcp') }}
            </div>
            <div
              v-for="skill in group.skills"
              :key="skill.id"
              class="skl-item"
              :class="{ open: expandedId === skill.id }"
            >
              <div class="skl-item-main" @click="toggleExpand(skill.id)">
                <span class="skl-icon">{{ skill.icon }}</span>
                <div class="skl-item-info">
                  <div class="skl-item-name">
                    {{ skill.name }}
                    <span class="skl-risk" :class="skill.risk ?? 'read'">{{ $t(`skills.risk.${skill.risk ?? 'read'}`) }}</span>
                  </div>
                  <div class="skl-item-desc">{{ skill.description }}</div>
                  <div v-if="expandedId === skill.id" class="skl-params">
                    <div v-if="paramEntries(skill).length === 0" class="skl-param-empty">
                      {{ $t('skills.noParams') }}
                    </div>
                    <div v-for="[name, schema] in paramEntries(skill)" :key="name" class="skl-param">
                      <code>{{ name }}</code>
                      <span class="skl-param-type">{{ schema.type }}</span>
                      <span class="skl-param-desc">{{ schema.description }}{{ schema.required ? ' *' : '' }}</span>
                    </div>
                  </div>
                </div>
              </div>
              <div class="skl-item-actions" @click.stop>
                <template v-if="skill.source === 'custom'">
                  <button class="skl-mini" :title="$t('skills.editSkill')" @click="startEdit(skill)">✎</button>
                  <button class="skl-mini danger" @click="handleDelete(skill.id)">
                    {{ pendingDeleteId === skill.id ? $t('skills.confirmDelete') : $t('skills.delete') }}
                  </button>
                </template>
                <div class="skl-toggle-row">
                  <span class="skl-toggle-label">{{ $t('skills.autoCall') }}</span>
                  <button
                    class="skl-toggle"
                    :class="{ on: skill.autoCallable }"
                    :title="$t('skills.autoCall')"
                    @click="skillsStore.toggleAutoCallable(skill.id, !skill.autoCallable)"
                  >
                    <span class="knob"></span>
                  </button>
                </div>
                <div v-if="(skill.risk ?? 'read') !== 'read'" class="skl-toggle-row">
                  <span class="skl-toggle-label">{{ $t('skills.alwaysAllow') }}</span>
                  <button
                    class="skl-toggle warn"
                    :class="{ on: skillsStore.isAlwaysAllowed(skill.id) }"
                    :title="$t('skills.alwaysAllowHint')"
                    @click="skillsStore.setAlwaysAllowed(skill.id, !skillsStore.isAlwaysAllowed(skill.id))"
                  >
                    <span class="knob"></span>
                  </button>
                </div>
                <div class="skl-toggle-row">
                  <span class="skl-toggle-label">{{ $t('skills.enabled') }}</span>
                  <button
                    class="skl-toggle"
                    :class="{ on: skill.enabled }"
                    :title="$t('skills.enabled')"
                    @click="skillsStore.toggleSkill(skill.id, !skill.enabled)"
                  >
                    <span class="knob"></span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Skill } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const { t } = useI18n()
const skillsStore = useSkillsStore()

const RISKS = ['read', 'write', 'system'] as const
const ID_RE = /^[a-z0-9-_]{1,64}$/

const expandedId = ref<string | null>(null)
const pendingDeleteId = ref<string | null>(null)
const editorOpen = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
const formError = ref<string | null>(null)
const form = reactive({
  id: '',
  name: '',
  icon: '🧩',
  description: '',
  risk: 'read' as string,
  prompt: '',
})

const groups = computed(() => [
  {
    key: 'builtin',
    label: t('skills.groupBuiltin'),
    skills: skillsStore.skillsBySource('builtin'),
  },
  { key: 'custom', label: t('skills.groupCustom'), skills: skillsStore.skillsBySource('custom') },
  { key: 'mcp', label: t('skills.groupMcp'), skills: skillsStore.skillsBySource('mcp') },
])

const canSave = computed(() => {
  if (form.name.trim() === '' || form.description.trim() === '' || form.prompt.trim() === '')
    return false
  return editingId.value !== null || ID_RE.test(form.id.trim())
})

function close(): void {
  emit('update:modelValue', false)
}

function toggleExpand(id: string): void {
  expandedId.value = expandedId.value === id ? null : id
}

function paramEntries(
  skill: Skill
): [string, { type: string; description: string; required?: boolean }][] {
  return Object.entries(skill.parameters?.properties ?? {})
}

function startCreate(): void {
  editingId.value = null
  form.id = ''
  form.name = ''
  form.icon = '🧩'
  form.description = ''
  form.risk = 'read'
  form.prompt = ''
  formError.value = null
  editorOpen.value = true
}

function startEdit(skill: Skill): void {
  const def = skillsStore.customDefs.find(d => d.id === skill.id)
  if (!def) return
  editingId.value = def.id
  form.name = def.name
  form.icon = def.icon
  form.description = def.description
  form.risk = def.risk
  form.prompt = def.prompt
  formError.value = null
  editorOpen.value = true
}

function closeEditor(): void {
  editorOpen.value = false
  editingId.value = null
  formError.value = null
}

async function handleSave(): Promise<void> {
  if (!canSave.value || saving.value) return
  formError.value = null
  const id = editingId.value ?? form.id.trim()
  if (!ID_RE.test(id)) {
    formError.value = t('skills.invalidId')
    return
  }
  saving.value = true
  try {
    await skillsStore.saveCustom({
      id,
      name: form.name.trim(),
      icon: form.icon.trim() || '🧩',
      description: form.description.trim(),
      risk: form.risk,
      prompt: form.prompt,
    })
    closeEditor()
  } catch (e) {
    formError.value = e instanceof Error ? e.message : String(e)
  } finally {
    saving.value = false
  }
}

async function handleDelete(id: string): Promise<void> {
  // 两段式确认：第一次点击进入确认态，再点一次才真正删除
  if (pendingDeleteId.value !== id) {
    pendingDeleteId.value = id
    return
  }
  pendingDeleteId.value = null
  try {
    await skillsStore.removeCustom(id)
    if (editingId.value === id) closeEditor()
  } catch (e) {
    formError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<style scoped>
.skl-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(2px);
}

.skl-dialog {
  width: min(640px, calc(100vw - 32px));
  max-height: 82vh;
  overflow: auto;
  border-radius: 14px;
  padding: 20px 22px;
  background: rgba(24, 26, 32, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.45);
  color: #e8e8ec;
}

.skl-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.skl-head h3 {
  margin: 0;
  font-size: 16px;
}

.skl-head-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.skl-count {
  font-size: 11px;
  opacity: 0.66;
}

.skl-close {
  background: none;
  border: none;
  color: #aaa;
  cursor: pointer;
  font-size: 15px;
}

.skl-hint {
  margin: 0 0 12px;
  font-size: 12px;
  opacity: 0.66;
  line-height: 1.5;
}

.skl-error {
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgba(255, 86, 86, 0.14);
  color: #ff7a7a;
}

.skl-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.skl-group-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.skl-group-label {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.72;
}

.skl-group-count {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.08);
  opacity: 0.7;
}

.skl-group-head .skl-btn {
  margin-left: auto;
}

.skl-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  opacity: 0.5;
}

.skl-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.skl-item.open {
  align-items: flex-start;
}

.skl-item-main {
  min-width: 0;
  flex: 1;
  display: flex;
  gap: 10px;
  cursor: pointer;
}

.skl-icon {
  font-size: 16px;
  line-height: 1.3;
}

.skl-item-info {
  min-width: 0;
  flex: 1;
}

.skl-item-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
}

.skl-risk {
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 4px;
  font-weight: 700;
}

.skl-risk.read {
  background: rgba(74, 222, 128, 0.16);
  color: #4ade80;
}

.skl-risk.write {
  background: rgba(251, 191, 36, 0.16);
  color: #fbbf24;
}

.skl-risk.system {
  background: rgba(255, 86, 86, 0.16);
  color: #ff7a7a;
}

.skl-item-desc {
  margin-top: 2px;
  font-size: 11px;
  opacity: 0.55;
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.skl-params {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.skl-param {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 11px;
}

.skl-param code {
  font-family: var(--mono-font, monospace);
  color: #9ecbff;
}

.skl-param-type {
  opacity: 0.5;
  font-size: 10px;
}

.skl-param-desc {
  opacity: 0.6;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skl-param-empty {
  font-size: 11px;
  opacity: 0.4;
}

.skl-item-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  flex-shrink: 0;
}

.skl-toggle-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.skl-toggle-label {
  font-size: 10px;
  opacity: 0.55;
}

.skl-toggle {
  width: 30px;
  height: 17px;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.12);
  border: none;
  padding: 0;
  position: relative;
  cursor: pointer;
  transition: background 0.15s ease;
}

.skl-toggle .knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 13px;
  height: 13px;
  border-radius: 50%;
  background: #9a9aa2;
  transition: all 0.15s ease;
}

.skl-toggle.on {
  background: rgba(74, 222, 128, 0.35);
}

.skl-toggle.on .knob {
  left: 15px;
  background: #4ade80;
}

.skl-toggle.warn.on {
  background: rgba(251, 191, 36, 0.35);
}

.skl-toggle.warn.on .knob {
  background: #fbbf24;
}

.skl-mini {
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 11px;
  background: rgba(255, 255, 255, 0.05);
  color: #c9c9cf;
}

.skl-mini.danger {
  background: rgba(255, 86, 86, 0.16);
  color: #ff7a7a;
}

.skl-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
  padding: 14px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.skl-add-title {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.72;
}

.skl-form-row {
  display: flex;
  gap: 8px;
}

.skl-form-row .grow {
  flex: 1;
}

.skl-input {
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e8e8ec;
  font-size: 13px;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.skl-input.icon {
  width: 64px;
  text-align: center;
}

.skl-input:focus {
  border-color: rgba(91, 140, 255, 0.6);
}

.skl-textarea {
  font-family: var(--mono-font, monospace);
  font-size: 12px;
  resize: vertical;
  min-height: 100px;
}

.skl-risk-picker {
  display: flex;
  gap: 6px;
}

.skl-tab {
  flex: 1;
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  padding: 6px 10px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.04);
  color: #bdbdc2;
}

.skl-tab.on {
  background: rgba(91, 140, 255, 0.18);
  border-color: rgba(91, 140, 255, 0.5);
  color: #fff;
}

.skl-form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.skl-btn {
  cursor: pointer;
  border: none;
  border-radius: 7px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.08);
  color: #d6d6da;
  transition: filter 0.15s ease;
}

.skl-btn:hover:not(:disabled) {
  filter: brightness(1.15);
}

.skl-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.skl-btn.primary {
  background: linear-gradient(135deg, #5b8cff, #6a5bff);
  color: #fff;
}

.skl-btn.slim {
  padding: 3px 10px;
}

.skl-fade-enter-active,
.skl-fade-leave-active {
  transition: opacity 0.18s ease;
}

.skl-fade-enter-from,
.skl-fade-leave-to {
  opacity: 0;
}
</style>
