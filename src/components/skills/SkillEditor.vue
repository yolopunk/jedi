<template>
  <div class="skill-editor">
    <div class="editor-title">
      {{ editing ? $t('skills.editSkill') : $t('skills.newSkill') }}
    </div>
    <div class="editor-form-row">
      <input v-model="form.name" class="editor-input grow" :placeholder="$t('skills.fieldName')" />
      <input
        v-model="form.icon"
        class="editor-input icon"
        :placeholder="$t('skills.fieldIcon')"
        maxlength="4"
      />
    </div>
    <input
      v-if="!editing"
      v-model="form.id"
      class="editor-input"
      :placeholder="$t('skills.fieldId')"
    />
    <input v-model="form.description" class="editor-input" :placeholder="$t('skills.fieldDesc')" />
    <div class="editor-risk-picker">
      <button
        v-for="r in RISKS"
        :key="r"
        class="editor-tab"
        :class="{ on: form.risk === r }"
        @click="form.risk = r"
      >
        {{ $t(`skills.risk.${r}`) }}
      </button>
    </div>
    <textarea
      v-model="form.prompt"
      class="editor-input editor-textarea"
      :placeholder="$t('skills.fieldPrompt')"
      rows="6"
    ></textarea>
    <div v-if="error" class="editor-error">{{ error }}</div>
    <div class="editor-actions">
      <button class="editor-btn" @click="emit('cancel')">{{ $t('skills.cancel') }}</button>
      <button class="editor-btn primary" :disabled="!canSave || saving" @click="handleSave">
        {{ saving ? '…' : $t('skills.save') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CustomSkillDef } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'

const props = defineProps<{ editing: CustomSkillDef | null }>()
const emit = defineEmits<{
  saved: []
  cancel: []
}>()

const { t } = useI18n()
const skillsStore = useSkillsStore()

const RISKS = ['read', 'write', 'system'] as const
const ID_RE = /^[a-z0-9-_]{1,64}$/

const saving = ref(false)
const error = ref<string | null>(null)
const form = reactive({
  id: '',
  name: '',
  icon: '🧩',
  description: '',
  risk: 'read' as string,
  prompt: '',
})

watch(
  () => props.editing,
  def => {
    form.id = def?.id ?? ''
    form.name = def?.name ?? ''
    form.icon = def?.icon ?? '🧩'
    form.description = def?.description ?? ''
    form.risk = def?.risk ?? 'read'
    form.prompt = def?.prompt ?? ''
    error.value = null
  },
  { immediate: true }
)

const canSave = computed(() => {
  if (form.name.trim() === '' || form.description.trim() === '' || form.prompt.trim() === '')
    return false
  return props.editing !== null || ID_RE.test(form.id.trim())
})

async function handleSave(): Promise<void> {
  if (!canSave.value || saving.value) return
  error.value = null
  const id = props.editing?.id ?? form.id.trim()
  if (!ID_RE.test(id)) {
    error.value = t('skills.invalidId')
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
    emit('saved')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    saving.value = false
  }
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
  opacity: 0.72;
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
  border-color: rgb(var(--accent-rgb) / 0.6);
}

.editor-textarea {
  font-family: var(--jedi-font-mono);
  font-size: 12px;
  resize: vertical;
  min-height: 100px;
}

.editor-risk-picker {
  display: flex;
  gap: 6px;
}

.editor-tab {
  flex: 1;
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 6px 10px;
  font-size: 12px;
  background: rgb(var(--text-rgb) / 0.04);
  color: var(--text-muted);
}

.editor-tab.on {
  background: rgb(var(--accent-rgb) / 0.18);
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
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
