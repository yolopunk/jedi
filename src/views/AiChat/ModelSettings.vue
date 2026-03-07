<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="800"
    scrollable
  >
    <v-card class="settings-dialog">
      <!-- Header -->
      <v-card-title class="d-flex align-center pa-4">
        <v-icon icon="mdi-cog" start />
        Settings
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
        <v-tabs v-model="activeTab" class="settings-tabs">
          <v-tab value="providers">
            <v-icon icon="mdi-cloud" start />
            Providers
          </v-tab>
          <v-tab value="parameters">
            <v-icon icon="mdi-tune-vertical" start />
            Parameters
          </v-tab>
          <v-tab value="appearance">
            <v-icon icon="mdi-palette" start />
            Appearance
          </v-tab>
        </v-tabs>

        <v-window v-model="activeTab" class="settings-content">
          <!-- Providers Tab -->
          <v-window-item value="providers">
            <div class="tab-content">
              <!-- Add Provider Button -->
              <div class="d-flex justify-end mb-4">
                <v-btn
                  color="primary"
                  prepend-icon="mdi-plus"
                  @click="showProviderForm = true"
                >
                  Add Provider
                </v-btn>
              </div>

              <!-- Providers List -->
              <div v-if="providers.length === 0" class="empty-state">
                <v-icon icon="mdi-cloud-off" size="48" color="surface-variant" />
                <p>No providers configured</p>
                <p class="text-caption">
                  Add an AI provider to start chatting
                </p>
              </div>

              <div v-else class="providers-list">
                <v-card
                  v-for="provider in providers"
                  :key="provider.id"
                  variant="outlined"
                  class="provider-card mb-3"
                >
                  <div class="d-flex align-center pa-4">
                    <div class="provider-icon mr-4">
                      <v-avatar :color="provider.isActive ? 'primary' : 'surface-variant'" size="48">
                        <v-icon :icon="getProviderIcon(provider.type)" />
                      </v-avatar>
                    </div>

                    <div class="provider-info flex-grow-1">
                      <div class="d-flex align-center">
                        <h3 class="text-h6">{{ provider.name }}</h3>
                        <v-chip
                          v-if="provider.isActive"
                          color="success"
                          size="small"
                          class="ml-2"
                        >
                          Active
                        </v-chip>
                      </div>
                      <p class="text-body-2 text-medium-emphasis mb-0">
                        {{ provider.type }} · {{ provider.models.length }} models
                      </p>
                    </div>

                    <div class="provider-actions">
                      <v-btn
                        icon
                        variant="text"
                        size="small"
                        @click="editProvider(provider)"
                      >
                        <v-icon icon="mdi-pencil" />
                      </v-btn>
                      <v-btn
                        icon
                        variant="text"
                        size="small"
                        color="error"
                        @click="confirmDeleteProvider(provider)"
                      >
                        <v-icon icon="mdi-delete" />
                      </v-btn>
                    </div>
                  </div>
                </v-card>
              </div>
            </div>
          </v-window-item>

          <!-- Parameters Tab -->
          <v-window-item value="parameters">
            <div class="tab-content">
              <div class="parameter-group mb-6">
                <h3 class="text-subtitle-1 font-weight-medium mb-3">
                  Generation Parameters
                </h3>

                <v-slider
                  v-model="localSettings.temperature"
                  label="Temperature"
                  :min="0"
                  :max="2"
                  :step="0.1"
                  thumb-label
                  class="mb-4"
                >
                  <template v-slot:append>
                    <v-chip size="small">{{ localSettings.temperature }}</v-chip>
                  </template>
                </v-slider>

                <v-slider
                  v-model="localSettings.maxTokens"
                  label="Max Tokens"
                  :min="256"
                  :max="32768"
                  :step="256"
                  thumb-label
                  class="mb-4"
                >
                  <template v-slot:append>
                    <v-chip size="small">{{ localSettings.maxTokens }}</v-chip>
                  </template>
                </v-slider>

                <v-slider
                  v-model="localSettings.topP"
                  label="Top P"
                  :min="0"
                  :max="1"
                  :step="0.05"
                  thumb-label
                  class="mb-4"
                >
                  <template v-slot:append>
                    <v-chip size="small">{{ localSettings.topP }}</v-chip>
                  </template>
                </v-slider>

                <v-switch
                  v-model="localSettings.streamEnabled"
                  label="Enable streaming responses"
                  color="primary"
                  hide-details
                />
              </div>

              <v-alert type="info" variant="tonal" class="mb-4">
                These parameters control how the AI generates responses.
                Higher temperature = more creative, lower = more focused.
              </v-alert>
            </div>
          </v-window-item>

          <!-- Appearance Tab -->
          <v-window-item value="appearance">
            <div class="tab-content">
              <div class="parameter-group mb-6">
                <h3 class="text-subtitle-1 font-weight-medium mb-3">
                  Theme
                </h3>

                <v-radio-group v-model="localTheme" inline hide-details>
                  <v-radio label="System" value="system" />
                  <v-radio label="Light" value="light" />
                  <v-radio label="Dark" value="dark" />
                </v-radio-group>
              </div>

              <div class="parameter-group mb-6">
                <h3 class="text-subtitle-1 font-weight-medium mb-3">
                  Font Size
                </h3>

                <v-slider
                  v-model="localFontSize"
                  label="Font Size"
                  :min="12"
                  :max="20"
                  :step="1"
                  thumb-label
                >
                  <template v-slot:append>
                    <v-chip size="small">{{ localFontSize }}px</v-chip>
                  </template>
                </v-slider>
              </div>

              <div class="parameter-group">
                <v-switch
                  v-model="localShowTimestamp"
                  label="Show message timestamps"
                  color="primary"
                  hide-details
                  class="mb-2"
                />

                <v-switch
                  v-model="localCompactMode"
                  label="Compact message layout"
                  color="primary"
                  hide-details
                />
              </div>
            </div>
          </v-window-item>
        </v-window>
      </v-card-text>

      <v-divider />

      <!-- Footer -->
      <v-card-actions class="pa-4">
        <v-spacer />
        <v-btn variant="text" @click="$emit('update:modelValue', false)">
          Cancel
        </v-btn>
        <v-btn color="primary" @click="saveSettings">
          Save Changes
        </v-btn>
      </v-card-actions>
    </v-card>

    <!-- Provider Form Dialog -->
    <ProviderForm
      v-model="showProviderForm"
      :provider="editingProvider"
      @save="handleSaveProvider"
      @close="closeProviderForm"
    />

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="showDeleteConfirm" max-width="400">
      <v-card>
        <v-card-title class="text-h6">
          Delete Provider?
        </v-card-title>
        <v-card-text>
          Are you sure you want to delete <strong>{{ deletingProvider?.name }}</strong>?
          This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="showDeleteConfirm = false">
            Cancel
          </v-btn>
          <v-btn color="error" @click="deleteProvider">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Provider } from '@/stores/aiChat'
import ProviderForm from './ProviderForm.vue'

const props = defineProps<{
  modelValue: boolean
  providers: Provider[]
  settings: {
    temperature: number
    maxTokens: number
    topP: number
    streamEnabled: boolean
  }
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save-provider', provider: Partial<Provider>): void
  (e: 'delete-provider', providerId: string): void
  (e: 'update-settings', settings: any): void
}>()

// Tab state
const activeTab = ref('providers')

// Provider form state
const showProviderForm = ref(false)
const editingProvider = ref<Provider | null>(null)

// Delete confirmation
const showDeleteConfirm = ref(false)
const deletingProvider = ref<Provider | null>(null)

// Local settings
const localSettings = ref({ ...props.settings })
const localTheme = ref('system')
const localFontSize = ref(14)
const localShowTimestamp = ref(true)
const localCompactMode = ref(false)

// Watch for settings changes
watch(() => props.settings, (newSettings) => {
  localSettings.value = { ...newSettings }
}, { deep: true })

// Provider helpers
function getProviderIcon(type: string): string {
  const icons: Record<string, string> = {
    openai: 'mdi-robot',
    anthropic: 'mdi-brain',
    google: 'mdi-google',
    azure: 'mdi-microsoft-azure',
    ollama: 'mdi-llama',
    openrouter: 'mdi-router-wireless',
    custom: 'mdi-api',
  }
  return icons[type] || 'mdi-cloud'
}

function editProvider(provider: Provider) {
  editingProvider.value = provider
  showProviderForm.value = true
}

function confirmDeleteProvider(provider: Provider) {
  deletingProvider.value = provider
  showDeleteConfirm.value = true
}

function deleteProvider() {
  if (deletingProvider.value) {
    emit('delete-provider', deletingProvider.value.id)
    showDeleteConfirm.value = false
    deletingProvider.value = null
  }
}

function handleSaveProvider(provider: Partial<Provider>) {
  emit('save-provider', provider)
  closeProviderForm()
}

function closeProviderForm() {
  showProviderForm.value = false
  editingProvider.value = null
}

function saveSettings() {
  emit('update-settings', {
    ...localSettings.value,
    theme: localTheme.value,
    fontSize: localFontSize.value,
    showTimestamp: localShowTimestamp.value,
    compactMode: localCompactMode.value,
  })
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-dialog {
  border-radius: 16px;
  overflow: hidden;
}

.settings-tabs {
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.1);
}

.settings-content {
  height: 400px;
  overflow-y: auto;
}

.tab-content {
  padding: 24px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  text-align: center;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.empty-state p {
  margin: 16px 0 0;
}

.provider-card {
  border-radius: 12px;
  transition: all 0.2s ease;
}

.provider-card:hover {
  border-color: rgba(var(--v-theme-primary), 0.3);
}

.parameter-group {
  padding: 16px;
  background: rgba(var(--v-theme-surface-variant), 0.3);
  border-radius: 12px;
}

/* Scrollbar styling */
.settings-content::-webkit-scrollbar {
  width: 8px;
}

.settings-content::-webkit-scrollbar-track {
  background: transparent;
}

.settings-content::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 4px;
}
</style>