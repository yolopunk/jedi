<template>
  <v-dialog v-model="dialogModel" max-width="700" persistent>
    <v-card class="scifi-card provider-form-dialog">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ {{ isEditing ? 'EDIT_PROVIDER' : 'ADD_PROVIDER' }} ]</span>
        <v-spacer />
        <button class="console-btn icon-only" @click="$emit('update:modelValue', false)">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>

      <v-card-text class="console-card-text provider-form-content">
        <!-- Step Indicator -->
        <div class="step-indicator">
          <div
            v-for="step in 3"
            :key="step"
            class="step-dot"
            :class="{
              active: currentStep === step,
              completed: currentStep > step
            }"
          >
            <span class="step-num">{{ step }}</span>
            <span class="step-label">{{ stepLabels[step - 1] }}</span>
          </div>
        </div>

        <div class="divider-line"></div>

        <!-- Step 1: Provider Type -->
        <div v-if="currentStep === 1" class="step-panel">
          <div class="panel-header">
            <span class="step-title">SELECT PROVIDER TYPE</span>
          </div>
          <div class="provider-grid">
            <div
              v-for="type in providerTypes"
              :key="type.value"
              class="provider-type-card"
              :class="{ selected: form.type === type.value }"
              @click="form.type = type.value; form.name = type.label"
            >
              <div class="provider-icon">{{ type.icon }}</div>
              <div class="provider-info">
                <span class="provider-name">{{ type.label }}</span>
                <span class="provider-desc">{{ type.description }}</span>
              </div>
              <div v-if="form.type === type.value" class="selected-check">✓</div>
            </div>
          </div>
        </div>

        <!-- Step 2: Configuration -->
        <div v-if="currentStep === 2" class="step-panel">
          <div class="panel-header">
            <span class="step-title">CONFIGURE {{ selectedProviderType?.label?.toUpperCase() }}</span>
          </div>
          <div class="config-form">
            <div class="form-field">
              <label class="field-label">DISPLAY NAME</label>
              <div class="input-wrapper">
                <span class="input-prompt">>></span>
                <input
                  v-model="form.name"
                  type="text"
                  class="console-input"
                  placeholder="My OpenAI"
                />
              </div>
            </div>

            <div class="form-field">
              <label class="field-label">API BASE URL</label>
              <div class="input-wrapper">
                <span class="input-prompt">>></span>
                <input
                  v-model="form.baseUrl"
                  type="text"
                  class="console-input"
                  :placeholder="defaultBaseUrl"
                />
              </div>
            </div>

            <div class="form-field">
              <label class="field-label">API KEY</label>
              <div class="input-wrapper">
                <span class="input-prompt">>></span>
                <input
                  v-model="form.apiKey"
                  :type="showApiKey ? 'text' : 'password'"
                  class="console-input"
                  placeholder="sk-..."
                />
                <button class="console-btn icon-only small" @click="showApiKey = !showApiKey">
                  <span class="btn-icon">{{ showApiKey ? '👁' : '👁‍🗨' }}</span>
                </button>
              </div>
            </div>

            <div v-if="form.type === 'ollama'" class="info-alert">
              <span class="alert-icon">ℹ</span>
              <span class="alert-text">Ollama runs locally and doesn't require an API key.</span>
            </div>
          </div>
        </div>

        <!-- Step 3: Models -->
        <div v-if="currentStep === 3" class="step-panel">
          <div class="panel-header">
            <span class="step-title">SELECT MODELS</span>
          </div>
          <div class="models-actions">
            <button class="console-btn" @click="fetchModels" :disabled="loadingModels">
              <span class="btn-icon" :class="{ spinning: loadingModels }">↻</span>
              <span class="btn-text">FETCH MODELS</span>
            </button>
            <button class="console-btn" @click="addCustomModel">
              <span class="btn-icon">+</span>
              <span class="btn-text">ADD CUSTOM</span>
            </button>
          </div>

          <div v-if="form.models.length === 0" class="empty-models">
            <span class="empty-icon">🤖</span>
            <span class="empty-text">No models added yet</span>
          </div>

          <div v-else class="models-list">
            <div
              v-for="(model, index) in form.models"
              :key="index"
              class="model-item"
            >
              <label class="model-toggle">
                <input type="checkbox" v-model="model.enabled" />
                <span class="toggle-box"></span>
              </label>
              <div class="model-fields">
                <input
                  v-model="model.id"
                  type="text"
                  class="console-input model-input"
                  placeholder="Model ID (e.g., gpt-4o)"
                />
                <input
                  v-model="model.name"
                  type="text"
                  class="console-input model-input"
                  placeholder="Display Name"
                />
              </div>
              <button class="console-btn icon-only small danger" @click="removeModel(index)">
                <span class="btn-icon">×</span>
              </button>
            </div>
          </div>
        </div>
      </v-card-text>

      <v-card-actions class="console-card-actions">
        <button v-if="currentStep > 1" class="console-btn" @click="currentStep--">
          <span class="btn-text">BACK</span>
        </button>
        <v-spacer />
        <button class="console-btn" @click="$emit('update:modelValue', false)">
          <span class="btn-text">CANCEL</span>
        </button>
        <button v-if="currentStep < 3" class="console-btn primary" @click="nextStep">
          <span class="btn-text">NEXT →</span>
        </button>
        <button v-else class="console-btn primary" @click="saveProvider">
          <span class="btn-text">{{ isEditing ? 'SAVE CHANGES' : 'ADD PROVIDER' }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Provider } from '@/stores/aiChat'
import { getModelsForProvider } from '@/api/ai-chat'

const props = defineProps<{
  modelValue: boolean
  provider?: Provider | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', provider: Partial<Provider>): void
}>()

const currentStep = ref(1)
const showApiKey = ref(false)
const loadingModels = ref(false)

const form = ref({
  id: '',
  name: '',
  type: 'openai',
  baseUrl: '',
  apiKey: '',
  models: [] as { id: string; name: string; enabled: boolean }[],
})

const stepLabels = ['TYPE', 'CONFIG', 'MODELS']

const providerTypes = [
  { value: 'openai', label: 'OpenAI', icon: '○', description: 'GPT-4, GPT-3.5 models', defaultBaseUrl: 'https://api.openai.com/v1' },
  { value: 'anthropic', label: 'Anthropic', icon: '◎', description: 'Claude models', defaultBaseUrl: 'https://api.anthropic.com' },
  { value: 'google', label: 'Google', icon: '⊕', description: 'Gemini models', defaultBaseUrl: 'https://generativelanguage.googleapis.com' },
  { value: 'ollama', label: 'Ollama', icon: '○', description: 'Local LLMs', defaultBaseUrl: 'http://localhost:11434/v1' },
  { value: 'openrouter', label: 'OpenRouter', icon: '◎', description: 'Multiple providers', defaultBaseUrl: 'https://openrouter.ai/api/v1' },
  { value: 'deepseek', label: 'DeepSeek', icon: '▶', description: 'DeepSeek models', defaultBaseUrl: 'https://api.deepseek.com' },
]

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const isEditing = computed(() => !!props.provider)

const selectedProviderType = computed(() =>
  providerTypes.find(t => t.value === form.value.type)
)

const defaultBaseUrl = computed(() =>
  selectedProviderType.value?.defaultBaseUrl || ''
)

watch(() => props.provider, (provider) => {
  if (provider) {
    form.value = {
      id: provider.id,
      name: provider.name,
      type: provider.type || 'openai',
      baseUrl: provider.baseUrl || '',
      apiKey: provider.apiKey || '',
      models: (provider.models || []).map(m => ({ ...m, enabled: true })),
    }
  } else {
    resetForm()
  }
}, { immediate: true })

watch(() => props.modelValue, (value) => {
  if (value && !props.provider) {
    resetForm()
  }
})

function resetForm() {
  currentStep.value = 1
  form.value = {
    id: '',
    name: '',
    type: 'openai',
    baseUrl: '',
    apiKey: '',
    models: [],
  }
}

function nextStep() {
  if (currentStep.value === 2 && !form.value.baseUrl && defaultBaseUrl.value) {
    form.value.baseUrl = defaultBaseUrl.value
  }
  currentStep.value++
}

async function fetchModels() {
  loadingModels.value = true
  try {
    const modelsFromDev = await getModelsForProvider(form.value.type)
    if (modelsFromDev && modelsFromDev.length > 0) {
      form.value.models = modelsFromDev.map((m: any) => ({
        id: m.id,
        name: m.name || m.id,
        enabled: true,
      }))
    } else {
      const defaults: Record<string, string[]> = {
        openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo'],
        anthropic: ['claude-sonnet-4-20250514', 'claude-3-5-sonnet-20241022'],
        google: ['gemini-2.0-flash', 'gemini-1.5-pro'],
        ollama: ['llama2', 'mistral'],
        openrouter: ['openai/gpt-4', 'anthropic/claude-3-opus'],
        deepseek: ['deepseek-chat', 'deepseek-coder'],
      }
      const ids = defaults[form.value.type] || []
      form.value.models = ids.map(id => ({ id, name: id, enabled: true }))
    }
  } catch (e) {
    console.error('Failed to fetch models:', e)
  } finally {
    loadingModels.value = false
  }
}

function addCustomModel() {
  form.value.models.push({ id: '', name: '', enabled: true })
}

function removeModel(index: number) {
  form.value.models.splice(index, 1)
}

function saveProvider() {
  const provider: Partial<Provider> = {
    id: form.value.id || undefined,
    name: form.value.name,
    type: form.value.type,
    baseUrl: form.value.baseUrl || defaultBaseUrl.value,
    apiKey: form.value.apiKey || undefined,
    models: form.value.models
      .filter(m => m.enabled && m.id)
      .map(({ enabled, ...model }) => model),
  }
  emit('save', provider)
  emit('update:modelValue', false)
}
</script>

<style scoped>
.provider-form-dialog {
  border-radius: 8px;
}

.provider-form-content {
  padding: 20px 24px !important;
}

.step-indicator {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 16px;
}

.step-dot {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 4px;
  transition: all 0.2s;
}

.step-dot.active {
  border-color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
}

.step-dot.completed {
  border-color: #00ff88;
  background: rgba(0, 255, 136, 0.1);
}

.step-num {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(0, 255, 255, 0.1);
  font-size: 11px;
  font-weight: 700;
  color: #00ffff;
}

.step-dot.completed .step-num {
  background: rgba(0, 255, 136, 0.2);
  color: #00ff88;
}

.step-label {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #a1a1aa;
}

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.2), transparent);
  margin-bottom: 20px;
}

.step-panel {
  min-height: 300px;
}

.panel-header {
  margin-bottom: 16px;
}

.step-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.provider-type-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  background: rgba(0, 0, 0, 0.2);
  position: relative;
}

.provider-type-card:hover {
  border-color: rgba(0, 255, 255, 0.4);
  background: rgba(0, 255, 255, 0.05);
}

.provider-type-card.selected {
  border-color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.15);
}

.provider-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 6px;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 13px;
  font-weight: 600;
  color: #e4e4e7;
}

.provider-desc {
  font-size: 10px;
  color: #71717a;
}

.selected-check {
  color: #00ffff;
  font-size: 16px;
  font-weight: 700;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ffff;
}

.info-alert {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: rgba(0, 255, 255, 0.05);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 4px;
}

.alert-icon {
  color: #00ffff;
}

.alert-text {
  font-size: 11px;
  color: #a1a1aa;
}

.models-actions {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.empty-models {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  gap: 8px;
}

.empty-icon {
  font-size: 32px;
}

.empty-text {
  font-size: 11px;
  color: #52525b;
  letter-spacing: 1px;
}

.models-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 240px;
  overflow-y: auto;
}

.model-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 4px;
}

.model-toggle {
  cursor: pointer;
}

.model-toggle input {
  display: none;
}

.toggle-box {
  display: block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(0, 255, 255, 0.3);
  border-radius: 3px;
  transition: all 0.15s;
}

.model-toggle input:checked + .toggle-box {
  background: #00ffff;
  border-color: #00ffff;
}

.model-fields {
  flex: 1;
  display: flex;
  gap: 8px;
}

.model-input {
  flex: 1;
  padding: 6px 10px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 4px;
  font-size: 11px;
}

.models-list::-webkit-scrollbar {
  width: 6px;
}

.models-list::-webkit-scrollbar-track {
  background: transparent;
}

.models-list::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}

@media (max-width: 600px) {
  .provider-grid {
    grid-template-columns: 1fr;
  }
}
</style>
