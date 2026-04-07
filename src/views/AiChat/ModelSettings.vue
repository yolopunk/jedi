<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="700"
  >
    <v-card class="scifi-card model-settings-dialog">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ MODEL_SETTINGS ]</span>
        <v-spacer />
        <button class="console-btn icon-only" @click="$emit('update:modelValue', false)">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>

      <v-card-text class="console-card-text settings-content">
        <!-- Loading State -->
        <div v-if="loading" class="state-container">
          <v-progress-circular indeterminate color="primary" />
          <span class="state-text">Loading providers...</span>
        </div>

        <!-- Error State -->
        <div v-else-if="error" class="state-container">
          <v-icon icon="mdi-alert-circle-outline" size="48" color="error" />
          <span class="state-text error">{{ error }}</span>
          <button class="console-btn" @click="retryLoad">
            <span class="btn-text">RETRY</span>
          </button>
        </div>

        <!-- Custom Provider Form -->
        <div v-else-if="showCustomForm">
          <div class="section-header">
            <span class="section-title">ADD CUSTOM PROVIDER</span>
          </div>
          <CustomProviderForm
            @save="handleCustomProviderSave"
            @cancel="showCustomForm = false"
          />
        </div>

        <!-- Provider Selector + Model List -->
        <div v-else class="stepped-content">
          <!-- Provider Selector -->
          <div class="section-header">
            <span class="section-title">SELECT PROVIDER</span>
          </div>
          <ProviderSelector @configure="openConfigDialog" />

          <!-- Model List (shown when a non-custom provider is selected) -->
          <div v-if="modelsDevStore.selectedProvider && !isSelectedProviderCustom" class="model-list-wrapper">
            <div class="section-header">
              <span class="section-title">SELECT MODEL</span>
            </div>
            <ModelList
              :provider-name="modelsDevStore.selectedProvider.name"
              :models="modelsDevStore.selectedProviderModels"
              :selected-model-id="modelsDevStore.selectedModelId"
              @back="modelsDevStore.selectProvider('')"
              @select="handleModelSelect"
            />
          </div>
        </div>

        <!-- API Key Config Dialog -->
        <v-dialog v-model="showConfigDialog" max-width="400">
          <v-card class="scifi-card">
            <v-card-title class="console-title-bar">
              <span class="dialog-title">[ {{ currentProviderName }}_CONFIG ]</span>
            </v-card-title>
            <v-card-text class="console-card-text">
              <div class="form-field">
                <label class="field-label">API KEY</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configApiKey"
                    :type="showKey ? 'text' : 'password'"
                    class="console-input"
                    placeholder="sk-..."
                  />
                  <button class="console-btn icon-only small" @click="showKey = !showKey">
                    <span class="btn-icon">{{ showKey ? '👁' : '👁‍🗨' }}</span>
                  </button>
                </div>
              </div>
              <div class="form-field mt-3">
                <label class="field-label">API ENDPOINT (OPTIONAL)</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configEndpoint"
                    type="text"
                    class="console-input"
                    :placeholder="currentProviderEndpoint || 'https://api.example.com'"
                  />
                </div>
              </div>
            </v-card-text>
            <v-card-actions class="console-card-actions">
              <button
                v-if="currentProviderConfigured"
                class="console-btn danger"
                @click="deleteKey"
              >
                <span class="btn-text">DELETE</span>
              </button>
              <v-spacer />
              <button class="console-btn" @click="showConfigDialog = false">
                <span class="btn-text">CANCEL</span>
              </button>
              <button class="console-btn primary" @click="saveKey">
                <span class="btn-text">SAVE</span>
              </button>
            </v-card-actions>
          </v-card>
        </v-dialog>

      </v-card-text>

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button class="console-btn" @click="saveAndClose">
          <span class="btn-text">SAVE</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import { useProviderConfigStore } from '@/stores/providerConfig'
import ProviderSelector from './components/ProviderSelector.vue'
import ModelList from './components/ModelList.vue'
import CustomProviderForm from './components/CustomProviderForm.vue'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const modelsDevStore = useModelsDevStore()
const providerConfigStore = useProviderConfigStore()

const showConfigDialog = ref(false)
const showCustomForm = ref(false)
const configApiKey = ref('')
const configEndpoint = ref('')
const showKey = ref(false)

const loading = computed(() => modelsDevStore.loading || providerConfigStore.loading)
const error = computed(() => modelsDevStore.error || providerConfigStore.error)

const currentProviderName = computed(() => {
  if (!modelsDevStore.selectedProvider) return ''
  return modelsDevStore.selectedProvider.name
})

const currentProviderEndpoint = computed(() => {
  return modelsDevStore.selectedProvider?.api || ''
})

const currentProviderConfigured = computed(() => {
  if (!modelsDevStore.selectedProviderId) return false
  return providerConfigStore.isProviderConfigured(modelsDevStore.selectedProviderId)
})

const isSelectedProviderCustom = computed(() => {
  return modelsDevStore.selectedProvider?.isCustom === true
})

watch(() => props.modelValue, async (open) => {
  if (open) {
    // Load both configured providers and models.dev data
    await Promise.all([
      providerConfigStore.loadConfiguredProviders(),
      modelsDevStore.fetchProviders()
    ])
  }
})

async function retryLoad() {
  await Promise.all([
    providerConfigStore.loadConfiguredProviders(),
    modelsDevStore.fetchProviders(true)
  ])
}

function openConfigDialog(providerId: string) {
  // Ensure the provider is selected in the store
  if (providerId) {
    modelsDevStore.selectProvider(providerId)
  }
  configApiKey.value = ''
  configEndpoint.value = modelsDevStore.selectedProvider?.api || ''
  showConfigDialog.value = true
}

async function saveKey() {
  const providerId = modelsDevStore.selectedProviderId
  if (!providerId) return

  await providerConfigStore.saveApiKey(
    providerId,
    configApiKey.value,
    configEndpoint.value || undefined
  )
  showConfigDialog.value = false
}

async function deleteKey() {
  const providerId = modelsDevStore.selectedProviderId
  if (!providerId) return

  await providerConfigStore.deleteApiKey(providerId)
  showConfigDialog.value = false
}

function handleModelSelect(model: any) {
  modelsDevStore.selectModel(model.id)
}

function handleCustomProviderSave(data: { name: string; baseUrl: string; apiKey: string; modelId?: string }) {
  // TODO: Implement custom provider save logic
  console.log('Custom provider save:', data)
  showCustomForm.value = false
}

function saveAndClose() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-content {
  padding: 20px !important;
}

.state-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  gap: 16px;
}

.state-text {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.state-text.error {
  color: #ff6b6b;
}

.stepped-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-header {
  margin-bottom: 8px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
}

.model-list-wrapper {
  margin-top: 8px;
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

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  border-radius: 4px;
  padding: 8px 12px;
}

.input-prompt {
  color: rgba(0, 255, 255, 0.5);
  font-size: 12px;
  margin-right: 8px;
  user-select: none;
}

.console-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #00ffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
}

.console-input::placeholder {
  color: #52525b;
}

.console-card-actions {
  padding: 12px 16px;
  display: flex;
  gap: 8px;
}

/* Console button styles */
.console-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid #00ff88;
  color: #00ff88;
  padding: 6px 12px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.console-btn:hover:not(:disabled) {
  background: rgba(0, 255, 136, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 136, 0.27), inset 0 0 15px rgba(0, 255, 136, 0.07);
}

.console-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.console-btn.primary {
  border-color: #00ffff;
  color: #00ffff;
}

.console-btn.primary:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.27), inset 0 0 15px rgba(0, 255, 255, 0.07);
}

.console-btn.danger {
  border-color: #ff6b6b;
  color: #ff6b6b;
}

.console-btn.danger:hover:not(:disabled) {
  background: rgba(255, 107, 107, 0.07);
  box-shadow: 0 0 15px rgba(255, 107, 107, 0.27), inset 0 0 15px rgba(255, 107, 107, 0.07);
}

.console-btn.small {
  padding: 4px 10px;
  font-size: 9px;
}

.console-btn.icon-only {
  padding: 4px 8px;
}

.btn-text {
  font-size: 10px;
}

.btn-icon {
  font-size: 14px;
}

.console-title-bar {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(0, 255, 255, 0.1);
}

.dialog-title {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ffff;
}
</style>
