<template>
  <div class="skill-editor">
    <div class="editor-title">
      {{ editing ? $t('skills.editSkill') : $t('skills.newSkill') }}
    </div>
    <div class="editor-form-row">
      <input
        v-model="form.name"
        class="editor-input grow"
        :placeholder="$t('skills.fieldName')"
        :aria-label="$t('skills.fieldName')"
        :disabled="!!editing"
      />
      <input
        v-model="form.icon"
        class="editor-input icon"
        :placeholder="$t('skills.fieldIcon')"
        :aria-label="$t('skills.fieldIcon')"
        maxlength="4"
      />
    </div>
    <input
      v-model="form.description"
      class="editor-input"
      :placeholder="$t('skills.fieldDesc')"
      :aria-label="$t('skills.fieldDesc')"
    />
    <textarea
      v-model="form.body"
      class="editor-input editor-textarea"
      :placeholder="$t('skills.fieldBody')"
      :aria-label="$t('skills.fieldBody')"
      rows="10"
    ></textarea>
    <p class="editor-note">{{ $t('skills.editorNote') }}</p>
    <div v-if="error" class="editor-error" role="alert">{{ error }}</div>
    <div class="editor-actions">
      <button class="editor-btn" @click="emit('cancel')">{{ $t('skills.cancel') }}</button>
      <button
        class="editor-btn primary"
        :disabled="!canSave || saving"
        :aria-busy="saving"
        @click="handleSave"
      >
        {{ saving ? $t('skills.saving') : $t('skills.save') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { BUILTIN_SKILLS } from '@/skills/builtin'
import type { SkillDetail } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

const props = defineProps<{ editing: SkillDetail | null }>()
const emit = defineEmits<{
  saved: []
  cancel: []
}>()

const { t } = useI18n()
const skillsStore = useSkillsStore()

const NAME_RE = /^[a-z0-9]+(-[a-z0-9]+)*$/

const saving = ref(false)
const error = ref<string | null>(null)
const form = reactive({
  name: '',
  icon: '🧩',
  description: '',
  body: '',
})

watch(
  () => props.editing,
  def => {
    form.name = def?.name ?? ''
    form.icon = def?.icon ?? '🧩'
    form.description = def?.description ?? ''
    form.body = def?.body ?? ''
    error.value = null
  },
  { immediate: true }
)

const canSave = computed(
  () => form.name.trim() !== '' && form.description.trim() !== '' && form.body.trim() !== ''
)

async function handleSave(): Promise<void> {
  if (!canSave.value || saving.value) return
  error.value = null
  const name = form.name.trim()
  if (!NAME_RE.test(name)) {
    error.value = t('skills.invalidName')
    return
  }
  if (!props.editing && (skillExists(name) || BUILTIN_SKILLS.some(s => s.name === name))) {
    error.value = t('skills.duplicateName', { name })
    return
  }
  saving.value = true
  try {
    await skillsStore.saveUserSkill({
      name,
      icon: form.icon.trim() || '🧩',
      description: form.description.trim(),
      body: form.body,
    })
    emit('saved')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    saving.value = false
  }
}

/** 已存在（含旧版迁移遗留）的用户技能名视为占用 */
function skillExists(name: string): boolean {
  return skillsStore.allSkills.some(s => s.name === name && s.source === 'user')
}
</script>

<style scoped>
.skill-editor {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px;
  border-radius: 12px;
  background: rgb(var(--text-rgb) / 0.03);
  border: 1px solid var(--border);
}

.editor-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.editor-form-row {
  display: flex;
  gap: 8px;
}

.editor-form-row .grow {
  flex: 1;
}

.editor-input {
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--bg-terminal);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 13px;
  outline: none;
  width: 100%;
  box-sizing: border-box;
}

.editor-input.icon {
  width: 64px;
  text-align: center;
}

.editor-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px rgb(var(--accent-rgb) / 0.3);
}

.editor-input:disabled {
  opacity: 0.6;
}

.editor-textarea {
  font-family: var(--jedi-font-mono);
  font-size: 12px;
  resize: vertical;
  min-height: 160px;
  line-height: 1.5;
}

.editor-note {
  margin: 0;
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.5;
}

.editor-error {
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgb(var(--danger-rgb) / 0.14);
  color: var(--danger);
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.editor-btn {
  cursor: pointer;
  border: none;
  border-radius: 7px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  background: rgb(var(--text-rgb) / 0.08);
  color: var(--text);
  transition: filter 0.15s ease;
}

.editor-btn:hover:not(:disabled) {
  filter: brightness(1.15);
}

.editor-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.editor-btn.primary {
  background: var(--accent);
  color: var(--on-accent);
}
</style>
