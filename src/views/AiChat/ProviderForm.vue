<template>
  <v-dialog v-model="dialogModel" max-width="480" persistent>
    <v-card class="scifi-card provider-form">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ {{ isEditing ? 'EDIT' : 'ADD' }}_PROVIDER ]</span>
        <v-spacer />
        <button class="console-btn icon-only" @click="close">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>

      <v-card-text class="console-card-text form-content">
        <!-- Provider Type Selection -->
        <div class="form-section">
          <label class="field-label">PROVIDER TYPE</label>
          <div class="provider-grid">
            <div
              v-for="p in providers"
              :key="p.id"
              class="provider-chip"
              :class="{ selected: form.type === p.id }"
              @click="selectProvider(p)"
            >
              <span class="chip-icon">{{ p.icon }}</span>
              <span class="chip-name">{{ p.name }}</span>
            </div>
          </div>
        </div>

        <!-- API Key Input -->
        <div class="form-section">
          <label class="field-label">API KEY</label>
          <div class="input-wrapper">
            <span class="input-prompt">>></span>
            <input
              v-model="form.apiKey"
              :type="showKey ? 'text' : 'password'"
              class="console-input"
              placeholder="sk-..."
            />
            <button class="console-btn icon-only small" @click="showKey = !showKey">
              <span class="btn-icon">{{ showKey ? '👁' : '👁‍🗨' }}</span>
            </button>
          </div>
        </div>

        <!-- Endpoint (optional) -->
        <div class="form-section">
          <label class="field-label">ENDPOINT <span class="optional">(OPTIONAL)</span></label>
          <div class="input-wrapper">
            <span class="input-prompt">>></span>
            <input
              v-model="form.endpoint"
              type="text"
              class="console-input"
              :placeholder="defaultEndpoint"
            />
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="console-card-actions">
        <button
          v-if="isEditing"
          class="console-btn danger"
          @click="deleteProvider"
        >
          <span class="btn-text">DELETE</span>
        </button>
        <v-spacer />
        <button class="console-btn" @click="close">
          <span class="btn-text">CANCEL</span>
        </button>
        <button
          class="console-btn primary"
          :disabled="!form.type || !form.apiKey"
          @click="save"
        >
          <span class="btn-text">{{ isEditing ? 'SAVE' : 'ADD' }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'

const props = defineProps<{
  modelValue: boolean
  providerId?: string | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const store = useAiChatStore()

const providers = [
  { id: 'openai', name: 'OpenAI', icon: '○', endpoint: 'https://api.openai.com/v1' },
  { id: 'anthropic', name: 'Anthropic', icon: '◎', endpoint: 'https://api.anthropic.com' },
  { id: 'google', name: 'Google', icon: '⊕', endpoint: 'https://generativelanguage.googleapis.com' },
  { id: 'deepseek', name: 'DeepSeek', icon: '▶', endpoint: 'https://api.deepseek.com' },
  { id: 'openrouter', name: 'OpenRouter', icon: '◎', endpoint: 'https://openrouter.ai/api/v1' },
  { id: 'ollama', name: 'Ollama', icon: '○', endpoint: 'http://localhost:11434/v1' },
]

const form = ref({
  type: '',
  apiKey: '',
  endpoint: '',
})

const showKey = ref(false)

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const isEditing = computed(() => !!props.providerId)

const defaultEndpoint = computed(() => {
  const p = providers.find(p => p.id === form.value.type)
  return p?.endpoint || ''
})

watch(() => props.modelValue, (open) => {
  if (open) {
    if (props.providerId) {
      // Load existing provider for editing
      form.value = { type: props.providerId, apiKey: '', endpoint: '' }
    } else {
      // Reset for new provider
      form.value = { type: '', apiKey: '', endpoint: '' }
    }
    showKey.value = false
  }
})

function selectProvider(p: typeof providers[0]) {
  form.value.type = p.id
  form.value.endpoint = p.endpoint
}

function close() {
  emit('update:modelValue', false)
}

async function save() {
  if (!form.value.type || !form.value.apiKey) return

  await store.saveApiKey(form.value.type, form.value.apiKey, form.value.endpoint || undefined)
  close()
}

async function deleteProvider() {
  if (!form.value.type) return
  await store.deleteApiKey(form.value.type)
  close()
}
</script>

<style scoped>
.provider-form {
  border-radius: 8px;
}

.form-content {
  padding: 20px !important;
}

.form-section {
  margin-bottom: 20px;
}

.field-label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ffff;
  margin-bottom: 8px;
}

.field-label .optional {
  color: #52525b;
  font-weight: 400;
}

.provider-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.provider-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.provider-chip:hover {
  border-color: rgba(0, 255, 255, 0.4);
  background: rgba(0, 255, 255, 0.05);
}

.provider-chip.selected {
  border-color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.2);
}

.chip-icon {
  font-size: 14px;
}

.chip-name {
  font-size: 12px;
  font-weight: 500;
  color: #e4e4e7;
}

.provider-chip.selected .chip-name {
  color: #00ffff;
}
</style>
