<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="600"
    scrollable
  >
    <v-card class="provider-form-dialog">
      <!-- Header -->
      <v-card-title class="d-flex align-center pa-4">
        <v-icon :icon="isEditing ? 'mdi-pencil' : 'mdi-plus'" start />
        {{ isEditing ? 'Edit Provider' : 'Add Provider' }}
        <v-spacer />
        <v-btn
          icon
          variant="text"
          @click="$emit('update:modelValue', false)"
        >
          <v-icon icon="mdi-close" />
        </v-btn>
      </v-card-title>

      <v-divider />

      <!-- Content -->
      <v-card-text class="pa-0">
        <v-stepper v-model="currentStep" class="provider-stepper" hide-actions>
          <v-stepper-header>
            <v-stepper-item
              :value="1"
              :complete="currentStep > 1"
              title="Provider Type"
            />
            <v-divider />
            <v-stepper-item
              :value="2"
              :complete="currentStep > 2"
              title="Configuration"
            />
            <v-divider />
            <v-stepper-item
              :value="3"
              :complete="currentStep > 3"
              title="Models"
            />
          </v-stepper-header>

          <v-stepper-window>
            <!-- Step 1: Provider Type -->
            <v-stepper-window-item :value="1">
              <div class="step-content">
                <h3 class="text-h6 mb-4">Select Provider Type</h3>
                
                <div class="provider-types">
                  <v-card
                    v-for="type in providerTypes"
                    :key="type.value"
                    :class="['provider-type-card', { selected: form.type === type.value }]"
                    variant="outlined"
                    @click="form.type = type.value"
                  >
                    <div class="provider-type-content">
                      <v-icon :icon="type.icon" size="32" color="primary" />
                      <h4>{{ type.label }}</h4>
                      <p>{{ type.description }}</p>
                    </div>
                  </v-card>
                </div>
              </div>
            </v-stepper-window-item>

            <!-- Step 2: Configuration -->
            <v-stepper-window-item :value="2">
              <div class="step-content">
                <h3 class="text-h6 mb-4">Configure {{ selectedProviderType?.label }}</h3>

                <v-form ref="configForm" v-model="configValid">
                  <v-text-field
                    v-model="form.name"
                    label="Display Name"
                    variant="outlined"
                    :rules="[rules.required]"
                    class="mb-4"
                  >
                    <template v-slot:prepend-inner>
                      <v-icon icon="mdi-tag" />
                    </template>
                  </v-text-field>

                  <v-text-field
                    v-model="form.baseUrl"
                    label="API Base URL"
                    variant="outlined"
                    :rules="[rules.required, rules.url]"
                    :placeholder="defaultBaseUrl"
                    class="mb-4"
                  >
                    <template v-slot:prepend-inner>
                      <v-icon icon="mdi-link" />
                    </template>
                  </v-text-field>

                  <v-text-field
                    v-model="form.apiKey"
                    label="API Key"
                    variant="outlined"
                    :type="showApiKey ? 'text' : 'password'"
                    :rules="form.type !== 'ollama' ? [rules.required] : []"
                    class="mb-4"
                  >
                    <template v-slot:prepend-inner>
                      <v-icon icon="mdi-key" />
                    </template>
                    <template v-slot:append-inner>
                      <v-btn
                        icon
                        variant="text"
                        size="small"
                        @click="showApiKey = !showApiKey"
                      >
                        <v-icon :icon="showApiKey ? 'mdi-eye-off' : 'mdi-eye'" />
                      </v-btn>
                    </template>
                  </v-text-field>

                  <v-alert
                    v-if="form.type === 'ollama'"
                    type="info"
                    variant="tonal"
                    class="mb-4"
                  >
                    Ollama runs locally and doesn't require an API key.
                    Make sure Ollama is running on your machine.
                  </v-alert>

                  <v-switch
                    v-model="form.isActive"
                    label="Enable this provider"
                    color="primary"
                    hide-details
                  />
                </v-form>
              </div>
            </v-stepper-window-item>

            <!-- Step 3: Models -->
            <v-stepper-window-item :value="3">
              <div class="step-content">
                <h3 class="text-h6 mb-2">Available Models</h3>
                <p class="text-body-2 text-medium-emphasis mb-4">
                  Select the models you want to use with this provider
                </p>

                <div class="d-flex align-center mb-4">
                  <v-btn
                    variant="outlined"
                    prepend-icon="mdi-refresh"
                    :loading="loadingModels"
                    @click="fetchModels"
                  >
                    Fetch Models
                  </v-btn>
                  
                  <v-btn
                    variant="text"
                    prepend-icon="mdi-plus"
                    class="ml-2"
                    @click="addCustomModel"
                  >
                    Add Custom
                  </v-btn>
                </div>

                <div v-if="form.models.length === 0" class="empty-models">
                  <v-icon icon="mdi-robot-off" size="48" color="surface-variant" />
                  <p>No models added yet</p>
                  <p class="text-caption">
                    Fetch models from the provider or add custom models
                  </p>
                </div>

                <div v-else class="models-list">
                  <v-card
                    v-for="(model, index) in form.models"
                    :key="model.id"
                    variant="outlined"
                    class="model-card mb-2"
                  >
                    <div class="d-flex align-center pa-3">
                      <v-checkbox
                        v-model="model.enabled"
                        hide-details
                        class="mr-2"
                      />
                      
                      <div class="flex-grow-1">
                        <v-text-field
                          v-model="model.id"
                          label="Model ID"
                          variant="outlined"
                          density="compact"
                          hide-details
                          class="mb-2"
                        />
                        <v-text-field
                          v-model="model.name"
                          label="Display Name"
                          variant="outlined"
                          density="compact"
                          hide-details
                        />
                      </div>

                      <v-btn
                        icon
                        variant="text"
                        size="small"
                        color="error"
                        class="ml-2"
                        @click="removeModel(index)"
                      >
                        <v-icon icon="mdi-delete" />
                      </v-btn>
                    </div>
                  </v-card>
                </div>
              </div>
            </v-stepper-window-item>
          </v-stepper-window>
        </v-stepper>
      </v-card-text>

      <v-divider />

      <!-- Footer -->
      <v-card-actions class="pa-4">
        <v-btn
          v-if="currentStep > 1"
          variant="text"
          @click="currentStep--"
        >
          Back
        </v-btn>
        <v-spacer />
        <v-btn variant="text" @click="$emit('update:modelValue', false)">
          Cancel
        </v-btn>
        <v-btn
          v-if="currentStep < 3"
          color="primary"
          @click="nextStep"
        >
          Continue
        </v-btn>
        <v-btn
          v-else
          color="primary"
          @click="saveProvider"
        >
          {{ isEditing ? 'Save Changes' : 'Add Provider' }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { Provider } from '@/stores/aiChat'

interface ProviderFormModel {
  id: string
  name: string
  providerId: string
  enabled: boolean
}

const props = defineProps<{
  modelValue: boolean
  provider?: Provider | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', provider: Partial<Provider>): void
  (e: 'close'): void
}>()

// Form state
const currentStep = ref(1)
const configForm = ref<{ validate: () => void } | null>(null)
const configValid = ref(false)
const showApiKey = ref(false)
const loadingModels = ref(false)

// Form data
const form = ref({
  id: '',
  name: '',
  type: 'openai',
  baseUrl: '',
  apiKey: '',
  isActive: true,
  models: [] as ProviderFormModel[],
})

// Provider types
const providerTypes = [
  {
    value: 'openai',
    label: 'OpenAI',
    icon: 'mdi-robot',
    description: 'GPT-4, GPT-3.5, and more',
    defaultBaseUrl: 'https://api.openai.com/v1',
  },
  {
    value: 'anthropic',
    label: 'Anthropic',
    icon: 'mdi-brain',
    description: 'Claude models',
    defaultBaseUrl: 'https://api.anthropic.com/v1',
  },
  {
    value: 'google',
    label: 'Google AI',
    icon: 'mdi-google',
    description: 'Gemini models',
    defaultBaseUrl: 'https://generativelanguage.googleapis.com/v1',
  },
  {
    value: 'ollama',
    label: 'Ollama',
    icon: 'mdi-llama',
    description: 'Local LLMs',
    defaultBaseUrl: 'http://localhost:11434/v1',
  },
  {
    value: 'openrouter',
    label: 'OpenRouter',
    icon: 'mdi-router-wireless',
    description: 'Multiple providers',
    defaultBaseUrl: 'https://openrouter.ai/api/v1',
  },
  {
    value: 'custom',
    label: 'Custom',
    icon: 'mdi-api',
    description: 'OpenAI-compatible API',
    defaultBaseUrl: '',
  },
]

// Validation rules
const rules = {
  required: (v: string) => !!v || 'This field is required',
  url: (v: string) => {
    try {
      new URL(v)
      return true
    } catch {
      return 'Please enter a valid URL'
    }
  },
}

// Computed
const isEditing = computed(() => !!props.provider)
const selectedProviderType = computed(() => 
  providerTypes.find(t => t.value === form.value.type)
)
const defaultBaseUrl = computed(() => 
  selectedProviderType.value?.defaultBaseUrl || ''
)

// Watch for provider changes
watch(() => props.provider, (provider) => {
  if (provider) {
    form.value = {
      id: provider.id,
      name: provider.name,
      type: provider.type || 'openai',
      baseUrl: provider.baseUrl || '',
      apiKey: provider.apiKey || '',
      isActive: provider.isActive ?? true,
      models: (provider.models || []).map(m => ({ ...m, enabled: true })),
    }
  } else {
    resetForm()
  }
}, { immediate: true })

// Reset form when dialog opens
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
    isActive: true,
    models: [],
  }
}

function nextStep() {
  if (currentStep.value === 2) {
    // Validate config form
    if (!configValid.value) {
      configForm.value?.validate()
      return
    }
    // Set default base URL if empty
    if (!form.value.baseUrl && defaultBaseUrl.value) {
      form.value.baseUrl = defaultBaseUrl.value
    }
  }
  currentStep.value++
}

async function fetchModels() {
  loadingModels.value = true
  try {
    // TODO: Implement actual model fetching
    // For now, add some default models based on provider type
    const defaultModels: Record<string, string[]> = {
      openai: ['gpt-4o', 'gpt-4-turbo', 'gpt-3.5-turbo'],
      anthropic: ['claude-3-opus', 'claude-3-sonnet', 'claude-3-haiku'],
      google: ['gemini-pro', 'gemini-pro-vision'],
      ollama: ['llama2', 'codellama', 'mistral'],
      openrouter: ['openai/gpt-4', 'anthropic/claude-3-opus'],
      custom: [],
    }
    
    const models = defaultModels[form.value.type] || []
    form.value.models = models.map(id => ({
      id,
      name: id,
      providerId: form.value.id || 'new',
      enabled: true,
    }))
  } catch (e) {
    console.error('Failed to fetch models:', e)
  } finally {
    loadingModels.value = false
  }
}

function addCustomModel() {
  form.value.models.push({
    id: '',
    name: '',
    providerId: form.value.id || 'new',
    enabled: true,
  })
}

function removeModel(index: number) {
  form.value.models.splice(index, 1)
}

function saveProvider() {
  const provider: Partial<Provider> = {
    id: form.value.id || undefined,
    name: form.value.name,
    type: form.value.type,
    baseUrl: form.value.baseUrl,
    apiKey: form.value.apiKey || undefined,
    isActive: form.value.isActive,
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
  border-radius: 16px;
  overflow: hidden;
}

.provider-stepper {
  background: transparent;
}

.provider-stepper :deep(.v-stepper-header) {
  box-shadow: none;
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.1);
}

.step-content {
  padding: 24px;
  min-height: 300px;
}

.provider-types {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.provider-type-card {
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 2px solid transparent;
}

.provider-type-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.3);
}

.provider-type-card.selected {
  border-color: rgb(var(--v-theme-primary));
  background: rgba(var(--v-theme-primary), 0.1);
}

.provider-type-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 16px;
  text-align: center;
}

.provider-type-content h4 {
  margin-top: 12px;
  font-size: 0.9375rem;
  font-weight: 600;
}

.provider-type-content p {
  margin-top: 4px;
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.6);
}

.empty-models {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px;
  text-align: center;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.empty-models p {
  margin: 16px 0 0;
}

.models-list {
  max-height: 300px;
  overflow-y: auto;
}

.model-card {
  border-radius: 8px;
}

/* Scrollbar styling */
.models-list::-webkit-scrollbar {
  width: 6px;
}

.models-list::-webkit-scrollbar-track {
  background: transparent;
}

.models-list::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 3px;
}

/* Mobile responsive */
@media (max-width: 600px) {
  .provider-types {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>